// DEFLATE compression, written to answer byte for byte what zlib answers at
// its default settings. A program that records compressed output and reads it
// back elsewhere depends on the exact bytes, so the choices zlib makes are
// reproduced rather than approximated: the same hash chains, the same lazy
// matching, the same heap order when code lengths are handed out, and the same
// comparison between a stored, a fixed, and a dynamic block.
//
// The names follow zlib's own (`good_length`, `nice_match`, `bl_tree`) so the
// two can be read side by side.

/// The shortest run of bytes a match may name.
const MIN_MATCH: usize = 3;
/// The longest run a single match can name.
const MAX_MATCH: usize = 258;
/// The window a match may reach back into, at the default 15 window bits.
const W_SIZE: usize = 1 << 15;
const W_MASK: usize = W_SIZE - 1;
/// The bytes zlib keeps ahead of the cursor before it will slide the window,
/// which is also what holds matches short of the whole window.
const MIN_LOOKAHEAD: usize = MAX_MATCH + MIN_MATCH + 1;
/// The furthest back a match may name, which is the window less the lookahead.
const MAX_DIST: usize = W_SIZE - MIN_LOOKAHEAD;

const HASH_BITS: usize = 15;
const HASH_SIZE: usize = 1 << HASH_BITS;
const HASH_MASK: usize = HASH_SIZE - 1;
const HASH_SHIFT: usize = HASH_BITS.div_ceil(MIN_MATCH);

/// No position, which is also position zero. zlib cannot tell the two apart,
/// and a match at position zero is passed over for that reason.
const NIL: u32 = 0;

/// How many symbols a block holds before it is written out, at the default
/// memory level.
const LIT_BUFSIZE: usize = 1 << 14;

// The settings the default compression level runs with.
const GOOD_LENGTH: usize = 8;
const MAX_LAZY: usize = 16;
const NICE_LENGTH: usize = 128;
const MAX_CHAIN: usize = 128;

const LITERALS: usize = 256;
const LENGTH_CODES: usize = 29;
const L_CODES: usize = LITERALS + 1 + LENGTH_CODES;
const D_CODES: usize = 30;
const BL_CODES: usize = 19;
const HEAP_SIZE: usize = 2 * L_CODES + 1;
const MAX_BITS: usize = 15;
const MAX_BL_BITS: usize = 7;
const END_BLOCK: usize = 256;
const REP_3_6: usize = 16;
const REPZ_3_10: usize = 17;
const REPZ_11_138: usize = 18;

/// The order the code lengths of the code-length alphabet are written in,
/// which puts the ones most often left out at the end.
const BL_ORDER: [usize; BL_CODES] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

const EXTRA_LBITS: [u8; LENGTH_CODES] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const EXTRA_DBITS: [u8; D_CODES] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const EXTRA_BLBITS: [u8; BL_CODES] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 7];

const BASE_LENGTH: [u16; LENGTH_CODES] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 24, 28, 32, 40, 48, 56, 64, 80, 96, 112, 128,
    160, 192, 224, 255,
];
const BASE_DIST: [u16; D_CODES] = [
    0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536,
    2048, 3072, 4096, 6144, 8192, 12288, 16384, 24576,
];

/// One node of a Huffman code: the count while the code is being built, then
/// the code itself, beside the number of bits it is written in.
#[derive(Clone, Copy, Default)]
struct Node {
    /// The count while the code is built, and the code once it is.
    code: u16,
    /// The parent while the code is built, and the bit length once it is.
    len: u16,
}

/// What a code is built from: the extra bits its symbols carry, where those
/// begin, how many symbols it has, and how long a code it allows.
struct TreeDesc {
    extra_bits: &'static [u8],
    extra_base: usize,
    elems: usize,
    max_length: usize,
}

const L_DESC: TreeDesc = TreeDesc {
    extra_bits: &EXTRA_LBITS,
    extra_base: LITERALS + 1,
    elems: L_CODES,
    max_length: MAX_BITS,
};
const D_DESC: TreeDesc = TreeDesc {
    extra_bits: &EXTRA_DBITS,
    extra_base: 0,
    elems: D_CODES,
    max_length: MAX_BITS,
};
const BL_DESC: TreeDesc = TreeDesc {
    extra_bits: &EXTRA_BLBITS,
    extra_base: 0,
    elems: BL_CODES,
    max_length: MAX_BL_BITS,
};

/// The length code a run of the given length is named by.
fn length_code(length: usize) -> usize {
    let mut code = LENGTH_CODES - 1;
    while code > 0 && (BASE_LENGTH[code] as usize) + MIN_MATCH > length {
        code -= 1;
    }
    code
}

/// The distance code a distance is named by.
fn dist_code(dist: usize) -> usize {
    let mut code = D_CODES - 1;
    while code > 0 && (BASE_DIST[code] as usize) + 1 > dist {
        code -= 1;
    }
    code
}

/// The bit lengths of the fixed literal code every decoder already knows.
fn static_l_lengths() -> Vec<u16> {
    (0..L_CODES + 2)
        .map(|symbol| match symbol {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        })
        .collect()
}

/// The fixed literal code every decoder already knows.
fn static_ltree() -> Vec<Node> {
    codes_from_lengths(&static_l_lengths(), MAX_BITS)
}

/// The fixed distance code, which writes every distance in five bits.
fn static_dtree() -> Vec<Node> {
    codes_from_lengths(&[5u16; D_CODES], MAX_BITS)
}

/// Hand out the codes a set of lengths calls for, shortest length first and
/// in symbol order within a length, which is what the format requires.
fn codes_from_lengths(lengths: &[u16], max_bits: usize) -> Vec<Node> {
    let mut bl_count = vec![0u16; max_bits + 2];
    for &len in lengths {
        if len != 0 {
            bl_count[len as usize] += 1;
        }
    }
    let mut next_code = vec![0u16; max_bits + 2];
    let mut code = 0u16;
    for bits in 1..=max_bits {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }
    let mut tree = vec![Node::default(); lengths.len()];
    for (symbol, &len) in lengths.iter().enumerate() {
        tree[symbol].len = len;
        if len == 0 {
            continue;
        }
        tree[symbol].code = reverse_bits(next_code[len as usize], len as usize);
        next_code[len as usize] += 1;
    }
    tree
}

/// A code is written out with its most significant bit first, while the bit
/// writer sends the least significant bit first, so each code is turned round
/// once when it is handed out.
fn reverse_bits(code: u16, len: usize) -> u16 {
    let mut held = code;
    let mut out = 0u16;
    for _ in 0..len {
        out = (out << 1) | (held & 1);
        held >>= 1;
    }
    out
}

/// Bits written out least significant first, which is the order DEFLATE
/// names.
struct BitWriter {
    out: Vec<u8>,
    bits: u32,
    count: u32,
}

impl BitWriter {
    fn new() -> Self {
        BitWriter {
            out: Vec::new(),
            bits: 0,
            count: 0,
        }
    }

    fn send(&mut self, value: u16, length: usize) {
        self.bits |= (value as u32) << self.count;
        self.count += length as u32;
        while self.count >= 8 {
            self.out.push((self.bits & 0xff) as u8);
            self.bits >>= 8;
            self.count -= 8;
        }
    }

    /// Fill out the byte being written, which is what a stored block has to
    /// start on.
    fn align(&mut self) {
        if self.count > 0 {
            self.out.push((self.bits & 0xff) as u8);
            self.bits = 0;
            self.count = 0;
        }
    }
}

/// One symbol of a block: a literal, or a match naming how far back and how
/// long.
#[derive(Clone, Copy)]
struct Symbol {
    dist: u16,
    literal_or_length: u16,
}

/// The symbols a block holds, with the counts they were tallied into.
struct Block {
    symbols: Vec<Symbol>,
    dyn_ltree: Vec<Node>,
    dyn_dtree: Vec<Node>,
    /// Where in the window this block's bytes start, so it can be written out
    /// as a stored block when that is smaller.
    start: usize,
}

impl Block {
    fn new(start: usize) -> Self {
        Block {
            symbols: Vec::new(),
            dyn_ltree: vec![Node::default(); HEAP_SIZE],
            dyn_dtree: vec![Node::default(); 2 * D_CODES + 1],
            start,
        }
    }

    fn tally_literal(&mut self, byte: u8) {
        self.symbols.push(Symbol {
            dist: 0,
            literal_or_length: byte as u16,
        });
        self.dyn_ltree[byte as usize].code += 1;
    }

    fn tally_match(&mut self, dist: usize, length: usize) {
        let held = length - MIN_MATCH;
        self.symbols.push(Symbol {
            dist: dist as u16,
            literal_or_length: held as u16,
        });
        self.dyn_ltree[LITERALS + 1 + length_code(length)].code += 1;
        self.dyn_dtree[dist_code(dist)].code += 1;
    }

    fn is_full(&self) -> bool {
        self.symbols.len() + 1 >= LIT_BUFSIZE
    }
}

/// What building a code answered: how far its symbols run, and what a block
/// written with it and with the fixed code would cost in bits.
struct BuiltTree {
    max_code: usize,
    opt_len: i64,
    static_len: i64,
}

/// Build the Huffman code a set of counts calls for, the way zlib builds it:
/// a heap ordered by count, with the depth of a node breaking a tie, so two
/// codes of equal weight are always handed out the same way round.
struct TreeBuilder<'a> {
    tree: &'a mut [Node],
    desc: &'a TreeDesc,
    heap: Vec<usize>,
    heap_len: usize,
    heap_max: usize,
    depth: Vec<u8>,
    bl_count: Vec<u16>,
    max_code: usize,
    has_symbols: bool,
    opt_len: i64,
    static_len: i64,
}

impl<'a> TreeBuilder<'a> {
    fn build(
        tree: &'a mut [Node],
        desc: &'a TreeDesc,
        static_lengths: Option<&[u16]>,
    ) -> BuiltTree {
        let mut builder = TreeBuilder {
            tree,
            desc,
            heap: vec![0; HEAP_SIZE + 1],
            heap_len: 0,
            heap_max: HEAP_SIZE,
            depth: vec![0; HEAP_SIZE],
            bl_count: vec![0; MAX_BITS + 1],
            max_code: 0,
            has_symbols: false,
            opt_len: 0,
            static_len: 0,
        };
        builder.run(static_lengths);
        BuiltTree {
            max_code: builder.max_code,
            opt_len: builder.opt_len,
            static_len: builder.static_len,
        }
    }

    fn smaller(&self, left: usize, right: usize) -> bool {
        let left_freq = self.tree[left].code;
        let right_freq = self.tree[right].code;
        left_freq < right_freq || (left_freq == right_freq && self.depth[left] <= self.depth[right])
    }

    fn down_heap(&mut self, mut at: usize) {
        let held = self.heap[at];
        let mut child = at << 1;
        while child <= self.heap_len {
            if child < self.heap_len && self.smaller(self.heap[child + 1], self.heap[child]) {
                child += 1;
            }
            if self.smaller(held, self.heap[child]) {
                break;
            }
            self.heap[at] = self.heap[child];
            at = child;
            child <<= 1;
        }
        self.heap[at] = held;
    }

    fn run(&mut self, static_lengths: Option<&[u16]>) {
        // zlib counts from -1 so that the first symbol seen leaves max_code
        // at its own place, and so that a code with no symbols at all can be
        // told apart.
        let mut max_code: i64 = -1;
        for symbol in 0..self.desc.elems {
            if self.tree[symbol].code != 0 {
                self.heap_len += 1;
                self.heap[self.heap_len] = symbol;
                max_code = symbol as i64;
                self.depth[symbol] = 0;
            } else {
                self.tree[symbol].len = 0;
            }
        }
        self.has_symbols = max_code >= 0;

        // A code has to name two symbols at the least, so a block using one
        // or none is given a made-up symbol to stand beside it.
        while self.heap_len < 2 {
            self.heap_len += 1;
            let node = if max_code < 2 {
                max_code += 1;
                max_code as usize
            } else {
                0
            };
            self.heap[self.heap_len] = node;
            self.tree[node].code = 1;
            self.depth[node] = 0;
            self.opt_len -= 1;
            if let Some(static_lengths) = static_lengths {
                self.static_len -= static_lengths[node] as i64;
            }
        }
        self.max_code = max_code as usize;

        for at in (1..=self.heap_len / 2).rev() {
            self.down_heap(at);
        }

        // Join the two smallest nodes over and over, which is the Huffman
        // construction itself.
        let mut node = self.desc.elems;
        while self.heap_len >= 2 {
            let smallest = self.heap[1];
            self.heap[1] = self.heap[self.heap_len];
            self.heap_len -= 1;
            self.down_heap(1);
            let second = self.heap[1];

            self.heap_max -= 1;
            self.heap[self.heap_max] = smallest;
            self.heap_max -= 1;
            self.heap[self.heap_max] = second;

            self.tree[node].code = self.tree[smallest].code + self.tree[second].code;
            self.depth[node] = self.depth[smallest].max(self.depth[second]) + 1;
            self.tree[smallest].len = node as u16;
            self.tree[second].len = node as u16;

            self.heap[1] = node;
            node += 1;
            self.down_heap(1);
        }
        self.heap_max -= 1;
        self.heap[self.heap_max] = self.heap[1];

        self.gen_bitlen(node - 1, static_lengths);

        // Only the symbols themselves take a code. The nodes the tree was
        // joined through sit past them in the same array, still holding the
        // parents they were given while it was built, and counting those
        // would hand out the wrong codes.
        let mut lengths = vec![0u16; self.max_code + 1];
        lengths.copy_from_slice(
            &self.tree.iter().map(|node| node.len).collect::<Vec<u16>>()[..=self.max_code],
        );
        let coded = codes_from_lengths(&lengths, self.desc.max_length);
        for node in self.tree.iter_mut() {
            *node = Node::default();
        }
        self.tree[..=self.max_code].copy_from_slice(&coded);
    }

    /// Hand each symbol the number of bits its place in the tree calls for,
    /// then pull any code longer than the format allows back into range.
    fn gen_bitlen(&mut self, top: usize, static_lengths: Option<&[u16]>) {
        for count in self.bl_count.iter_mut() {
            *count = 0;
        }
        self.tree[self.heap[self.heap_max]].len = 0;

        let mut overflow: i64 = 0;
        for at in (self.heap_max + 1)..HEAP_SIZE {
            let symbol = self.heap[at];
            if symbol > top {
                continue;
            }
            let parent = self.tree[symbol].len as usize;
            let mut bits = self.tree[parent].len as usize + 1;
            if bits > self.desc.max_length {
                bits = self.desc.max_length;
                overflow += 1;
            }
            self.tree[symbol].len = bits as u16;
            if symbol > self.max_code {
                continue;
            }
            self.bl_count[bits] += 1;
            let extra = if symbol >= self.desc.extra_base {
                self.desc.extra_bits[symbol - self.desc.extra_base] as i64
            } else {
                0
            };
            let freq = self.tree[symbol].code as i64;
            self.opt_len += freq * (bits as i64 + extra);
            if let Some(static_lengths) = static_lengths {
                self.static_len += freq * (static_lengths[symbol] as i64 + extra);
            }
        }
        if overflow == 0 {
            return;
        }

        // A code longer than the format allows is shortened by moving a node
        // up, which lengthens a shorter one to pay for it.
        loop {
            let mut bits = self.desc.max_length - 1;
            while self.bl_count[bits] == 0 {
                bits -= 1;
            }
            self.bl_count[bits] -= 1;
            self.bl_count[bits + 1] += 2;
            self.bl_count[self.desc.max_length] -= 1;
            overflow -= 2;
            if overflow <= 0 {
                break;
            }
        }

        let mut at = HEAP_SIZE;
        for bits in (1..=self.desc.max_length).rev() {
            let mut count = self.bl_count[bits];
            while count != 0 {
                at -= 1;
                let symbol = self.heap[at];
                if symbol > self.max_code {
                    continue;
                }
                if self.tree[symbol].len as usize != bits {
                    let freq = self.tree[symbol].code as i64;
                    self.opt_len += (bits as i64 - self.tree[symbol].len as i64) * freq;
                    self.tree[symbol].len = bits as u16;
                }
                count -= 1;
            }
        }
    }
}

/// Compress `bytes` the way zlib does at its default level, with `dictionary`
/// standing in front of the data where one was given.
pub(crate) fn deflate(bytes: &[u8], dictionary: &[u8]) -> Vec<u8> {
    // The dictionary sits in the window ahead of the data, so a match may
    // reach back into it. None of it is written out.
    let mut window: Vec<u8> = Vec::with_capacity(dictionary.len() + bytes.len());
    window.extend_from_slice(dictionary);
    window.extend_from_slice(bytes);
    Deflater::new(window, dictionary.len()).run()
}

struct Deflater {
    window: Vec<u8>,
    head: Vec<u32>,
    prev: Vec<u32>,
    ins_h: usize,
    strstart: usize,
    match_start: usize,
    match_length: usize,
    prev_length: usize,
    prev_match: usize,
    match_available: bool,
    writer: BitWriter,
    block: Block,
}

impl Deflater {
    fn new(window: Vec<u8>, data_start: usize) -> Self {
        let mut state = Deflater {
            window,
            head: vec![NIL; HASH_SIZE],
            prev: vec![NIL; W_SIZE],
            ins_h: 0,
            strstart: 0,
            match_start: 0,
            match_length: MIN_MATCH - 1,
            prev_length: MIN_MATCH - 1,
            prev_match: 0,
            match_available: false,
            writer: BitWriter::new(),
            block: Block::new(data_start),
        };
        // The rolling hash carries the two bytes before the one it is being
        // folded with, so it is started on the first two bytes of the window.
        if state.window.len() >= 2 {
            state.ins_h = state.window[0] as usize;
            state.ins_h = ((state.ins_h << HASH_SHIFT) ^ state.window[1] as usize) & HASH_MASK;
        }
        // A dictionary sits in the window ahead of the data and is fed
        // through the hash there, so a match may name it.
        for at in 0..data_start {
            if at + MIN_MATCH <= state.window.len() {
                state.update_hash(at);
                state.insert_string(at);
            }
        }
        state.strstart = data_start;
        state
    }

    /// Fold the byte that closes the run starting here into the rolling hash.
    fn update_hash(&mut self, at: usize) {
        let byte = self.window.get(at + MIN_MATCH - 1).copied().unwrap_or(0);
        self.ins_h = ((self.ins_h << HASH_SHIFT) ^ byte as usize) & HASH_MASK;
    }

    /// Record the run of three bytes starting here, and answer where the last
    /// run with the same hash sat.
    fn insert_string(&mut self, at: usize) -> u32 {
        let held = self.head[self.ins_h];
        self.prev[at & W_MASK] = held;
        self.head[self.ins_h] = at as u32;
        held
    }

    /// The longest match at or after `cur_match`, looking no further back
    /// than the window allows and no longer than the chain permits.
    fn longest_match(&mut self, mut cur_match: usize) -> usize {
        let mut chain_length = MAX_CHAIN;
        let mut best_len = self.prev_length;
        let limit = self.strstart.saturating_sub(MAX_DIST);
        if best_len >= GOOD_LENGTH {
            chain_length >>= 2;
        }
        let max_len = MAX_MATCH.min(self.window.len() - self.strstart);
        if max_len < MIN_MATCH {
            return MIN_MATCH - 1;
        }
        let nice_match = NICE_LENGTH.min(max_len);

        loop {
            let mut len = 0;
            while len < max_len && self.window[cur_match + len] == self.window[self.strstart + len]
            {
                len += 1;
            }
            if len > best_len {
                self.match_start = cur_match;
                best_len = len;
                if len >= nice_match {
                    break;
                }
            }
            chain_length -= 1;
            if chain_length == 0 {
                break;
            }
            let next = self.prev[cur_match & W_MASK] as usize;
            if next <= limit || next == 0 {
                break;
            }
            cur_match = next;
        }
        best_len.min(max_len)
    }

    /// The lazy-matching loop zlib runs at every level above three: a match
    /// found here is held back while the next position is tried, and the
    /// longer of the two is the one written.
    fn run(mut self) -> Vec<u8> {
        let end = self.window.len();
        while self.strstart < end {
            let mut hash_head = NIL;
            if self.strstart + MIN_MATCH <= end {
                self.update_hash(self.strstart);
                hash_head = self.insert_string(self.strstart);
            }

            self.prev_length = self.match_length;
            self.prev_match = self.match_start;
            self.match_length = MIN_MATCH - 1;

            if hash_head != NIL
                && self.prev_length < MAX_LAZY
                && self.strstart - hash_head as usize <= MAX_DIST
            {
                self.match_length = self.longest_match(hash_head as usize);
                // A match of the shortest length reaching a long way back
                // costs more than the bytes it stands for, so zlib passes it
                // over.
                if self.match_length == MIN_MATCH && self.strstart - self.match_start > 4096 {
                    self.match_length = MIN_MATCH - 1;
                }
            }

            if self.prev_length >= MIN_MATCH && self.match_length <= self.prev_length {
                let max_insert = end.saturating_sub(MIN_MATCH);
                self.block
                    .tally_match(self.strstart - 1 - self.prev_match, self.prev_length);
                // The bytes the match covers are still fed through the hash,
                // so a later match may name them.
                let mut left = self.prev_length - 2;
                while left > 0 {
                    self.strstart += 1;
                    if self.strstart <= max_insert {
                        self.update_hash(self.strstart);
                        self.insert_string(self.strstart);
                    }
                    left -= 1;
                }
                self.match_available = false;
                self.match_length = MIN_MATCH - 1;
                self.strstart += 1;
            } else if self.match_available {
                self.block.tally_literal(self.window[self.strstart - 1]);
                self.strstart += 1;
            } else {
                self.match_available = true;
                self.strstart += 1;
            }

            if self.block.is_full() {
                self.flush_block(false);
            }
        }
        if self.match_available {
            self.block.tally_literal(self.window[end - 1]);
        }
        self.flush_block(true);
        self.writer.align();
        self.writer.out
    }

    /// Write the block gathered so far, as whichever of a stored, a fixed, or
    /// a dynamic block is smallest, which is the comparison zlib makes.
    fn flush_block(&mut self, last: bool) {
        let start = self.block.start;
        let stored_len = self.strstart.min(self.window.len()) - start;
        let mut block = std::mem::replace(&mut self.block, Block::new(self.strstart));

        block.dyn_ltree[END_BLOCK].code += 1;
        let static_l = static_l_lengths();
        let static_d = vec![5u16; D_CODES];
        let built_l = TreeBuilder::build(&mut block.dyn_ltree, &L_DESC, Some(&static_l));
        let built_d = TreeBuilder::build(&mut block.dyn_dtree, &D_DESC, Some(&static_d));

        let mut opt_len = built_l.opt_len + built_d.opt_len;
        let static_len = built_l.static_len + built_d.static_len;

        // The code lengths of the two codes are themselves written with a
        // code, whose own cost is part of what a dynamic block costs.
        let (mut bl_tree, bl_counts) = scan_both_trees(&block, built_l.max_code, built_d.max_code);
        TreeBuilder::build(&mut bl_tree, &BL_DESC, None);
        let mut max_blindex = 3;
        for at in (4..BL_CODES).rev() {
            if bl_tree[BL_ORDER[at]].len != 0 {
                max_blindex = at;
                break;
            }
        }
        for (symbol, count) in bl_counts.iter().enumerate() {
            opt_len += *count as i64
                * (bl_tree[symbol].len as i64
                    + if symbol >= 16 {
                        EXTRA_BLBITS[symbol] as i64
                    } else {
                        0
                    });
        }
        opt_len += 3 * (max_blindex as i64 + 1) + 5 + 5 + 4;

        let opt_lenb = ((opt_len + 3 + 7) >> 3) as usize;
        let static_lenb = ((static_len + 3 + 7) >> 3) as usize;
        let opt_lenb = opt_lenb.min(static_lenb);

        if stored_len + 4 <= opt_lenb {
            self.stored_block(start, stored_len, last);
            return;
        }
        if static_lenb == opt_lenb {
            self.writer.send((1 << 1) | last as u16, 3);
            let ltree = static_ltree();
            let dtree = static_dtree();
            self.compress_block(&block, &ltree, &dtree);
            return;
        }
        self.writer.send((2 << 1) | last as u16, 3);
        self.send_all_trees(
            &block,
            &bl_tree,
            built_l.max_code + 1,
            built_d.max_code + 1,
            max_blindex + 1,
        );
        let ltree = block.dyn_ltree.clone();
        let dtree = block.dyn_dtree.clone();
        self.compress_block(&block, &ltree, &dtree);
    }

    fn stored_block(&mut self, start: usize, len: usize, last: bool) {
        self.writer.send(last as u16, 3);
        self.writer.align();
        self.writer
            .out
            .extend_from_slice(&(len as u16).to_le_bytes());
        self.writer
            .out
            .extend_from_slice(&(!(len as u16)).to_le_bytes());
        self.writer
            .out
            .extend_from_slice(&self.window[start..start + len]);
    }

    /// Write the lengths of both codes, themselves written with the
    /// code-length code.
    fn send_all_trees(
        &mut self,
        block: &Block,
        bl_tree: &[Node],
        lcodes: usize,
        dcodes: usize,
        blcodes: usize,
    ) {
        self.writer.send((lcodes - 257) as u16, 5);
        self.writer.send((dcodes - 1) as u16, 5);
        self.writer.send((blcodes - 4) as u16, 4);
        for at in 0..blcodes {
            self.writer.send(bl_tree[BL_ORDER[at]].len, 3);
        }
        self.send_tree(bl_tree, &block.dyn_ltree, lcodes - 1);
        self.send_tree(bl_tree, &block.dyn_dtree, dcodes - 1);
    }

    /// Write one code's lengths, with a run of equal lengths named by the
    /// repeat codes rather than written out one by one.
    fn send_tree(&mut self, bl_tree: &[Node], tree: &[Node], max_code: usize) {
        walk_lengths(tree, max_code, |step| match step {
            LengthStep::Single { len, times } => {
                for _ in 0..times {
                    self.send_code(bl_tree, len);
                }
            }
            LengthStep::Repeat { len, extra, times } => {
                if let Some(len) = len {
                    self.send_code(bl_tree, len);
                }
                self.send_code(bl_tree, extra.symbol());
                self.writer.send(times as u16, extra.bits());
            }
        });
    }

    fn send_code(&mut self, tree: &[Node], symbol: usize) {
        let node = tree[symbol];
        self.writer.send(node.code, node.len as usize);
    }

    /// Write the symbols of a block with the two codes it was measured
    /// against.
    fn compress_block(&mut self, block: &Block, ltree: &[Node], dtree: &[Node]) {
        for symbol in &block.symbols {
            if symbol.dist == 0 {
                self.send_code(ltree, symbol.literal_or_length as usize);
                continue;
            }
            let length = symbol.literal_or_length as usize + MIN_MATCH;
            let code = length_code(length);
            self.send_code(ltree, LITERALS + 1 + code);
            let extra = EXTRA_LBITS[code] as usize;
            if extra != 0 {
                let held = length - MIN_MATCH - BASE_LENGTH[code] as usize;
                self.writer.send(held as u16, extra);
            }
            let dist = symbol.dist as usize;
            let code = dist_code(dist);
            self.send_code(dtree, code);
            let extra = EXTRA_DBITS[code] as usize;
            if extra != 0 {
                let held = dist - 1 - BASE_DIST[code] as usize;
                self.writer.send(held as u16, extra);
            }
        }
        self.send_code(ltree, END_BLOCK);
    }
}

/// Which repeat code a run of code lengths is named by.
#[derive(Clone, Copy)]
enum Repeat {
    /// Three to six of the length just written.
    Same,
    /// Three to ten zeros.
    ShortRunOfZeros,
    /// Eleven to a hundred and thirty-eight zeros.
    LongRunOfZeros,
}

impl Repeat {
    fn symbol(self) -> usize {
        match self {
            Repeat::Same => REP_3_6,
            Repeat::ShortRunOfZeros => REPZ_3_10,
            Repeat::LongRunOfZeros => REPZ_11_138,
        }
    }

    fn bits(self) -> usize {
        match self {
            Repeat::Same => 2,
            Repeat::ShortRunOfZeros => 3,
            Repeat::LongRunOfZeros => 7,
        }
    }
}

/// One step of writing out a code's lengths.
enum LengthStep {
    /// A length written on its own, some number of times over.
    Single { len: usize, times: usize },
    /// A run named by a repeat code, with the length written once first where
    /// the run is of a length not written already.
    Repeat {
        len: Option<usize>,
        extra: Repeat,
        times: usize,
    },
}

/// Walk a code's lengths the way the format writes them, handing each step to
/// the caller. Counting the steps and writing them out read the lengths the
/// same way, so both go through here.
fn walk_lengths(tree: &[Node], max_code: usize, mut step: impl FnMut(LengthStep)) {
    let mut prevlen: i64 = -1;
    let mut nextlen = tree[0].len as usize;
    let mut count = 0usize;
    let (mut max_count, mut min_count) = if nextlen == 0 { (138, 3) } else { (7, 4) };

    for at in 0..=max_code {
        let curlen = nextlen;
        nextlen = if at < max_code {
            tree[at + 1].len as usize
        } else {
            usize::MAX
        };
        count += 1;
        if count < max_count && curlen == nextlen {
            continue;
        }
        if count < min_count {
            step(LengthStep::Single {
                len: curlen,
                times: count,
            });
        } else if curlen != 0 {
            let names_itself = curlen as i64 != prevlen;
            let times = if names_itself { count - 1 } else { count };
            step(LengthStep::Repeat {
                len: names_itself.then_some(curlen),
                extra: Repeat::Same,
                times: times - 3,
            });
        } else if count <= 10 {
            step(LengthStep::Repeat {
                len: None,
                extra: Repeat::ShortRunOfZeros,
                times: count - 3,
            });
        } else {
            step(LengthStep::Repeat {
                len: None,
                extra: Repeat::LongRunOfZeros,
                times: count - 11,
            });
        }
        count = 0;
        prevlen = curlen as i64;
        if nextlen == 0 {
            max_count = 138;
            min_count = 3;
        } else if curlen == nextlen {
            max_count = 6;
            min_count = 3;
        } else {
            max_count = 7;
            min_count = 4;
        }
    }
}

/// Count how often each code-length symbol would be written across both
/// codes, which is what the code-length code is built from.
fn scan_both_trees(block: &Block, max_l_code: usize, max_d_code: usize) -> (Vec<Node>, Vec<u16>) {
    let mut bl_tree = vec![Node::default(); 2 * BL_CODES + 1];
    let mut counts = vec![0u16; BL_CODES];
    for (tree, max_code) in [
        (&block.dyn_ltree, max_l_code),
        (&block.dyn_dtree, max_d_code),
    ] {
        walk_lengths(tree, max_code, |step| match step {
            LengthStep::Single { len, times } => {
                bl_tree[len].code += times as u16;
                counts[len] += times as u16;
            }
            LengthStep::Repeat { len, extra, .. } => {
                if let Some(len) = len {
                    bl_tree[len].code += 1;
                    counts[len] += 1;
                }
                bl_tree[extra.symbol()].code += 1;
                counts[extra.symbol()] += 1;
            }
        });
    }
    (bl_tree, counts)
}
