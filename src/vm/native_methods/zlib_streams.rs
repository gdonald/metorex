// The compressed stream formats zlib reads and writes: the two checksums it
// keeps, the DEFLATE encoding underneath, and the wrappers that carry it.

/// The table CRC-32 is computed from, one entry per byte value.
fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for (index, slot) in table.iter_mut().enumerate() {
        let mut held = index as u32;
        for _ in 0..8 {
            held = if held & 1 == 1 {
                0xedb8_8320 ^ (held >> 1)
            } else {
                held >> 1
            };
        }
        *slot = held;
    }
    table
}

/// The CRC-32 of `bytes`, continuing from `running`.
pub(crate) fn crc32(running: u32, bytes: &[u8]) -> u32 {
    let table = crc_table();
    let mut held = !running;
    for byte in bytes {
        held = table[((held ^ u32::from(*byte)) & 0xff) as usize] ^ (held >> 8);
    }
    !held
}

/// The whole CRC-32 table, which `Zlib.crc_table` hands out.
pub(crate) fn crc_table_values() -> Vec<u32> {
    crc_table().to_vec()
}

/// The Adler-32 of `bytes`, continuing from `running`.
pub(crate) fn adler32(running: u32, bytes: &[u8]) -> u32 {
    const MODULUS: u32 = 65521;
    let mut low = running & 0xffff;
    let mut high = (running >> 16) & 0xffff;
    for byte in bytes {
        low = (low + u32::from(*byte)) % MODULUS;
        high = (high + low) % MODULUS;
    }
    (high << 16) | low
}

/// Reads a DEFLATE stream one bit at a time, least significant bit first.
struct BitReader<'a> {
    bytes: &'a [u8],
    at: usize,
    bit: u32,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            at: 0,
            bit: 0,
        }
    }

    fn read_bit(&mut self) -> Option<u32> {
        let byte = *self.bytes.get(self.at)?;
        let held = (u32::from(byte) >> self.bit) & 1;
        self.bit += 1;
        if self.bit == 8 {
            self.bit = 0;
            self.at += 1;
        }
        Some(held)
    }

    fn read_bits(&mut self, count: u32) -> Option<u32> {
        let mut held = 0u32;
        for place in 0..count {
            held |= self.read_bit()? << place;
        }
        Some(held)
    }

    /// Move to the start of the next byte, which a stored block begins on.
    fn align(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.at += 1;
        }
    }
}

/// A canonical Huffman code, held as the counts and symbols the decoder walks.
struct Huffman {
    counts: Vec<u16>,
    symbols: Vec<u16>,
}

impl Huffman {
    /// Build the code the given lengths describe, which is how DEFLATE names
    /// both of its alphabets.
    fn from_lengths(lengths: &[u8]) -> Self {
        const MAX_BITS: usize = 15;
        let mut counts = vec![0u16; MAX_BITS + 1];
        for length in lengths {
            counts[*length as usize] += 1;
        }
        counts[0] = 0;
        let mut offsets = [0u16; MAX_BITS + 2];
        for bits in 1..=MAX_BITS {
            offsets[bits + 1] = offsets[bits] + counts[bits];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (symbol, length) in lengths.iter().enumerate() {
            if *length != 0 {
                symbols[offsets[*length as usize] as usize] = symbol as u16;
                offsets[*length as usize] += 1;
            }
        }
        Self { counts, symbols }
    }

    fn decode(&self, reader: &mut BitReader<'_>) -> Option<u16> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for bits in 1..self.counts.len() {
            code |= reader.read_bit()? as i32;
            let count = i32::from(self.counts[bits]);
            if code - first < count {
                return self.symbols.get((index + (code - first)) as usize).copied();
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        None
    }
}

/// How much each length symbol stands for, and how many extra bits follow it.
const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// The fixed code DEFLATE uses when a block names no code of its own.
fn fixed_codes() -> (Huffman, Huffman) {
    let mut lengths = vec![8u8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    let distances = vec![5u8; 30];
    (
        Huffman::from_lengths(&lengths),
        Huffman::from_lengths(&distances),
    )
}

/// The code lengths a dynamic block names for itself.
fn dynamic_codes(reader: &mut BitReader<'_>) -> Option<(Huffman, Huffman)> {
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let literal_count = reader.read_bits(5)? as usize + 257;
    let distance_count = reader.read_bits(5)? as usize + 1;
    let code_count = reader.read_bits(4)? as usize + 4;
    let mut code_lengths = [0u8; 19];
    for place in ORDER.iter().take(code_count) {
        code_lengths[*place] = reader.read_bits(3)? as u8;
    }
    let code_code = Huffman::from_lengths(&code_lengths);
    let mut lengths = Vec::with_capacity(literal_count + distance_count);
    while lengths.len() < literal_count + distance_count {
        let symbol = code_code.decode(reader)?;
        match symbol {
            0..=15 => lengths.push(symbol as u8),
            16 => {
                let previous = *lengths.last()?;
                let repeat = reader.read_bits(2)? + 3;
                for _ in 0..repeat {
                    lengths.push(previous);
                }
            }
            17 => {
                let repeat = reader.read_bits(3)? as usize + 3;
                lengths.extend(std::iter::repeat_n(0u8, repeat));
            }
            18 => {
                let repeat = reader.read_bits(7)? as usize + 11;
                lengths.extend(std::iter::repeat_n(0u8, repeat));
            }
            _ => return None,
        }
    }
    if lengths.len() > literal_count + distance_count {
        return None;
    }
    Some((
        Huffman::from_lengths(&lengths[..literal_count]),
        Huffman::from_lengths(&lengths[literal_count..]),
    ))
}

/// What a DEFLATE stream stands for, or None when it is not one.
pub(crate) fn inflate(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut reader = BitReader::new(bytes);
    let mut out: Vec<u8> = Vec::new();
    loop {
        let last = reader.read_bit()?;
        let kind = reader.read_bits(2)?;
        match kind {
            0 => {
                reader.align();
                let at = reader.at;
                let length = u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]) as usize;
                reader.at = at + 4;
                let piece = bytes.get(reader.at..reader.at + length)?;
                out.extend_from_slice(piece);
                reader.at += length;
            }
            1 | 2 => {
                let (literals, distances) = if kind == 1 {
                    fixed_codes()
                } else {
                    dynamic_codes(&mut reader)?
                };
                loop {
                    let symbol = literals.decode(&mut reader)?;
                    match symbol {
                        0..=255 => out.push(symbol as u8),
                        256 => break,
                        257..=285 => {
                            let slot = symbol as usize - 257;
                            let length = LENGTH_BASE[slot] as usize
                                + reader.read_bits(LENGTH_EXTRA[slot])? as usize;
                            let distance_symbol = distances.decode(&mut reader)? as usize;
                            if distance_symbol >= DISTANCE_BASE.len() {
                                return None;
                            }
                            let distance = DISTANCE_BASE[distance_symbol] as usize
                                + reader.read_bits(DISTANCE_EXTRA[distance_symbol])? as usize;
                            if distance > out.len() {
                                return None;
                            }
                            let start = out.len() - distance;
                            for step in 0..length {
                                let byte = out[start + step];
                                out.push(byte);
                            }
                        }
                        _ => return None,
                    }
                }
            }
            _ => return None,
        }
        if last == 1 {
            break;
        }
    }
    Some(out)
}

/// Write `bytes` as DEFLATE stored blocks, which name no code and so read
/// back through any decoder.
pub(crate) fn deflate_stored(bytes: &[u8]) -> Vec<u8> {
    const BLOCK: usize = 65535;
    let mut out = Vec::new();
    let mut pieces = bytes.chunks(BLOCK).peekable();
    if bytes.is_empty() {
        out.push(1);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&u16::MAX.to_le_bytes());
        return out;
    }
    while let Some(piece) = pieces.next() {
        out.push(u8::from(pieces.peek().is_none()));
        let length = piece.len() as u16;
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&(!length).to_le_bytes());
        out.extend_from_slice(piece);
    }
    out
}

/// Wrap a DEFLATE stream in the two-byte header and Adler-32 that name it a
/// zlib stream.
pub(crate) fn zlib_wrap(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x9c];
    out.extend_from_slice(&deflate_stored(bytes));
    out.extend_from_slice(&adler32(1, bytes).to_be_bytes());
    out
}

/// What a zlib stream stands for, with its header and checksum taken off.
pub(crate) fn zlib_unwrap(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 2 {
        return None;
    }
    let body = bytes.get(2..bytes.len().saturating_sub(4))?;
    inflate(body)
}

/// What a gzip member stands for, with its header and trailer taken off.
pub(crate) fn gzip_unwrap(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 18 || bytes[0] != 0x1f || bytes[1] != 0x8b {
        return None;
    }
    let flags = bytes[3];
    let mut at = 10;
    if flags & 0x04 != 0 {
        let extra = u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]) as usize;
        at += 2 + extra;
    }
    for mask in [0x08u8, 0x10] {
        if flags & mask != 0 {
            while *bytes.get(at)? != 0 {
                at += 1;
            }
            at += 1;
        }
    }
    if flags & 0x02 != 0 {
        at += 2;
    }
    let body = bytes.get(at..bytes.len().saturating_sub(8))?;
    inflate(body)
}

/// Wrap a DEFLATE stream in the gzip header and trailer.
pub(crate) fn gzip_wrap(bytes: &[u8], name: Option<&str>, stamp: u32) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 0x08];
    out.push(if name.is_some() { 0x08 } else { 0x00 });
    out.extend_from_slice(&stamp.to_le_bytes());
    out.push(0);
    out.push(0x03);
    if let Some(name) = name {
        out.extend_from_slice(name.as_bytes());
        out.push(0);
    }
    out.extend_from_slice(&deflate_stored(bytes));
    out.extend_from_slice(&crc32(0, bytes).to_le_bytes());
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out
}

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// `Zlib.__stream__(action, text, running)` — the one place the library
    /// reaches the encodings above, with bytes held one to a character.
    pub(crate) fn zlib_stream(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(Object::String(action)) = arguments.first() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                arguments.len(),
                position,
            ));
        };
        let text = match arguments.get(1) {
            Some(Object::String(held)) => held.as_str().to_string(),
            _ => String::new(),
        };
        let bytes = super::pack_format::string_to_bytes(&text);
        let running = match arguments.get(2) {
            Some(Object::Int(held)) => *held as u32,
            _ => 0,
        };
        let refuse = |what: &str| {
            crate::vm::errors::simple_exception(
                "Zlib::DataError",
                &format!("{what} is not a stream this can read"),
                position,
            )
        };
        match &*action.as_str() {
            "crc32" => Ok(Object::Int(i64::from(crc32(running, &bytes)))),
            "adler32" => Ok(Object::Int(i64::from(adler32(running, &bytes)))),
            "crc_table" => Ok(Object::array(
                crc_table_values()
                    .into_iter()
                    .map(|held| Object::Int(i64::from(held)))
                    .collect(),
            )),
            "deflate" => Ok(super::pack_format::bytes_to_string(&zlib_wrap(&bytes))),
            "inflate" => match zlib_unwrap(&bytes) {
                Some(held) => Ok(super::pack_format::bytes_to_string(&held)),
                None => Err(refuse("the text")),
            },
            "raw_inflate" => match inflate(&bytes) {
                Some(held) => Ok(super::pack_format::bytes_to_string(&held)),
                None => Err(refuse("the text")),
            },
            "gzip" => {
                let name = match arguments.get(3) {
                    Some(Object::String(held)) => Some(held.as_str().to_string()),
                    _ => None,
                };
                Ok(super::pack_format::bytes_to_string(&gzip_wrap(
                    &bytes,
                    name.as_deref(),
                    running,
                )))
            }
            "gunzip" => match gzip_unwrap(&bytes) {
                Some(held) => Ok(super::pack_format::bytes_to_string(&held)),
                None => Err(refuse("the text")),
            },
            other => Err(MetorexError::runtime_error(
                format!("unknown stream action {other}"),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}
