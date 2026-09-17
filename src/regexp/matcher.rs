//! Walking a pattern over a subject, giving back what it read when a piece
//! further on cannot carry on.

use super::classes::in_class;
use super::node::{Greed, Node, Target};

/// What is left to match once the piece in hand has matched. Each frame sits
/// on the stack of the call that built it and points at the one built before
/// it, so giving back what a piece read is returning from that call.
///
/// `'a` is how long the pattern lives and `'r` how long the chain of frames
/// does, which is only as long as the call that is matching.
enum Rest<'a, 'r> {
    /// Nothing is left, so the match ends here.
    Done,
    /// These pieces in order, then whatever follows them.
    Seq {
        pieces: &'a [Node],
        next: &'r Rest<'a, 'r>,
    },
    /// A group closes here, taking what was read since `from`.
    Close {
        index: usize,
        from: usize,
        next: &'r Rest<'a, 'r>,
    },
    /// One turn of a repetition ended here.
    Again {
        node: &'a Node,
        least: u32,
        most: Option<u32>,
        greed: Greed,
        clears: Option<usize>,
        done: u32,
        from: usize,
        /// What the groups held when the turn began, which says whether a
        /// turn that read nothing still left the match somewhere new.
        writes: u64,
        next: &'r Rest<'a, 'r>,
    },
    /// The match has to end at this position and nowhere else, which is how a
    /// lookbehind pins what it reads to where it was written.
    EndsAt(usize),
    /// The match has to stop no later than this position, which bounds what a
    /// piece read inside an absent group may cover.
    NoFurtherThan(usize),
}

/// Where a group's text sits in the subject, counted in characters.
pub type Slot = Option<(usize, usize)>;

/// What a whole match read.
pub struct Found {
    pub start: usize,
    pub end: usize,
    pub slots: Vec<Slot>,
}

/// How many pieces one match may try before the engine gives up.
const STEP_BUDGET: u64 = 20_000_000;

pub struct Matcher<'a> {
    letters: &'a [char],
    /// Where the search began, which `\G` stands for.
    search_start: usize,
    slots: Vec<Slot>,
    /// The body of each group by number, so `\g<name>` can match it again.
    bodies: Vec<Option<&'a Node>>,
    names: &'a [Option<String>],
    /// Where `\K` said the reported match begins.
    keep: Option<usize>,
    steps: u64,
    depth: u32,

    /// Whether the pattern captures anything, which decides whether a branch
    /// has to copy the slots before trying it.
    captures: bool,
}

impl<'a> Matcher<'a> {
    pub fn new(letters: &'a [char], names: &'a [Option<String>], root: &'a Node) -> Self {
        let mut bodies = vec![None; names.len()];
        collect_bodies(root, &mut bodies);
        Self {
            letters,
            search_start: 0,
            slots: vec![None; names.len()],
            bodies,
            names,
            keep: None,
            steps: 0,
            depth: 0,
            captures: names.len() > 1,
        }
    }

    /// The leftmost match at or after `from`, or None when there is none.
    pub fn search(&mut self, root: &'a Node, from: usize) -> Option<Found> {
        self.search_start = from;
        let mut at = from;
        loop {
            self.reset();
            if let Some(end) = self.run(root, at, &Rest::Done) {
                return Some(Found {
                    start: self.keep.unwrap_or(at),
                    end,
                    slots: std::mem::take(&mut self.slots),
                });
            }
            if at >= self.letters.len() {
                return None;
            }
            at += 1;
        }
    }

    /// Whether a match begins at `at` exactly.
    pub fn matches_at(&mut self, root: &'a Node, at: usize) -> bool {
        self.search_start = at;
        self.reset();
        self.run(root, at, &Rest::Done).is_some()
    }

    fn reset(&mut self) {
        self.slots.iter_mut().for_each(|slot| *slot = None);
        self.keep = None;
        self.steps = 0;
        self.depth = 0;
    }

    /// What every group holds right now, as one number. A turn of a
    /// repetition that reads nothing and leaves this unchanged has left the
    /// match where it already was, so the loop ends there.
    fn state_mark(&self) -> u64 {
        let mut mark: u64 = 0xcbf2_9ce4_8422_2325;
        for slot in &self.slots {
            let held = match slot {
                Some((from, to)) => (*from as u64) << 32 | (*to as u64) | 1 << 63,
                None => 0,
            };
            mark = (mark ^ held).wrapping_mul(0x100_0000_01b3);
        }
        mark
    }

    /// Forget what the groups written inside a repetition read, which is what
    /// keeps a backreference in one turn from reaching the turn before.
    fn forget(&mut self, clears: Option<usize>) {
        if let Some(index) = clears
            && let Some(slot) = self.slots.get_mut(index)
        {
            *slot = None;
        }
    }

    fn saved(&self) -> Vec<Slot> {
        if self.captures {
            return self.slots.clone();
        }
        Vec::new()
    }

    fn restore(&mut self, saved: Vec<Slot>) {
        if self.captures {
            self.slots = saved;
        }
    }

    fn word_at(&self, at: usize) -> bool {
        self.letters
            .get(at)
            .is_some_and(|held| held.is_alphanumeric() || *held == '_')
    }

    /// Match `node` at `at`, then whatever `rest` says follows.
    fn run<'r>(&mut self, node: &'a Node, at: usize, rest: &'r Rest<'a, 'r>) -> Option<usize> {
        self.steps += 1;
        if self.steps > STEP_BUDGET {
            return None;
        }
        match node {
            Node::Empty => self.carry_on(rest, at),
            Node::Letter(letter) => {
                if self.letters.get(at) == Some(letter) {
                    return self.carry_on(rest, at + 1);
                }
                None
            }
            Node::Letters(run) => {
                let stop = at + run.len();
                if stop > self.letters.len() || self.letters[at..stop] != run[..] {
                    return None;
                }
                self.carry_on(rest, stop)
            }
            Node::AnyChar { newline } => match self.letters.get(at) {
                Some('\n') if !newline => None,
                Some(_) => self.carry_on(rest, at + 1),
                None => None,
            },
            Node::Class(class) => match self.letters.get(at) {
                Some(letter) if in_class(class, *letter) => self.carry_on(rest, at + 1),
                _ => None,
            },
            Node::LineStart => {
                // A line starts at the beginning of the subject and after
                // every newline but one closing it, since there is no line
                // after that one.
                let opens =
                    at == 0 || (self.letters.get(at - 1) == Some(&'\n') && at < self.letters.len());
                if opens {
                    return self.carry_on(rest, at);
                }
                None
            }
            Node::LineEnd => {
                if at == self.letters.len() || self.letters.get(at) == Some(&'\n') {
                    return self.carry_on(rest, at);
                }
                None
            }
            Node::SubjectStart => {
                if at == 0 {
                    return self.carry_on(rest, at);
                }
                None
            }
            Node::SubjectEnd => {
                if at == self.letters.len() {
                    return self.carry_on(rest, at);
                }
                None
            }
            Node::SubjectEndBeforeNewline => {
                let length = self.letters.len();
                if at == length || (at + 1 == length && self.letters[at] == '\n') {
                    return self.carry_on(rest, at);
                }
                None
            }
            Node::SearchStart => {
                if at == self.search_start {
                    return self.carry_on(rest, at);
                }
                None
            }
            Node::WordBoundary { negated } => {
                let before = at > 0 && self.word_at(at - 1);
                let after = self.word_at(at);
                if (before != after) != *negated {
                    return self.carry_on(rest, at);
                }
                None
            }
            Node::KeepOut => {
                let held = self.keep;
                self.keep = Some(at);
                match self.carry_on(rest, at) {
                    Some(end) => Some(end),
                    None => {
                        self.keep = held;
                        None
                    }
                }
            }
            Node::Grapheme => match self.grapheme_end(at) {
                Some(end) => self.carry_on(rest, end),
                None => None,
            },
            Node::Concat(pieces) => self.sequence(pieces, at, rest),
            Node::Alternate(branches) => {
                for branch in branches {
                    let saved = self.saved();
                    if let Some(end) = self.run(branch, at, rest) {
                        return Some(end);
                    }
                    self.restore(saved);
                }
                None
            }
            Node::Repeat {
                node,
                least,
                most,
                greed,
                clears,
            } => self.repeat(node, *least, *most, *greed, *clears, 0, at, rest),
            Node::Capture { index, node } => {
                let saved = self.slots[*index];
                let frame = Rest::Close {
                    index: *index,
                    from: at,
                    next: rest,
                };
                match self.run(node, at, &frame) {
                    Some(end) => Some(end),
                    None => {
                        self.slots[*index] = saved;
                        None
                    }
                }
            }
            Node::Group(node) => self.run(node, at, rest),
            Node::Atomic(node) => {
                let saved = self.saved();
                match self.run(node, at, &Rest::Done) {
                    Some(end) => match self.carry_on(rest, end) {
                        Some(done) => Some(done),
                        None => {
                            self.restore(saved);
                            None
                        }
                    },
                    None => {
                        self.restore(saved);
                        None
                    }
                }
            }
            Node::Look {
                behind,
                negated,
                node,
            } => self.look(*behind, *negated, node, at, rest),
            Node::Backreference { target, folded } => self.backreference(target, *folded, at, rest),
            Node::Call(target) => {
                let index = self.slot_index(target)?;
                let body = self.bodies.get(index).copied().flatten()?;
                if self.depth > 48 {
                    return None;
                }
                self.depth += 1;
                let outcome = self.run(body, at, rest);
                self.depth -= 1;
                outcome
            }
            Node::Absent(node) => self.absent(node, at, rest),
            Node::Conditional {
                target,
                then_node,
                else_node,
            } => {
                let taken = self
                    .slot_index(target)
                    .and_then(|index| self.slots.get(index).copied().flatten())
                    .is_some();
                let branch = if taken { then_node } else { else_node };
                self.run(branch, at, rest)
            }
        }
    }

    /// Where the character a reader sees at `at` ends.
    fn grapheme_end(&self, at: usize) -> Option<usize> {
        let mut cursor = self.carried_end(at)?;
        // A run joined by a zero-width joiner reads as one character.
        while self.letters.get(cursor) == Some(&'\u{200d}') {
            match self.carried_end(cursor + 1) {
                Some(end) => cursor = end,
                None => return Some(cursor),
            }
        }
        Some(cursor)
    }

    /// Where one base together with everything carried on it ends.
    fn carried_end(&self, at: usize) -> Option<usize> {
        let base = *self.letters.get(at)?;
        let mut cursor = at + 1;
        // A carriage return and the newline after it are one character.
        if base == '\r' && self.letters.get(cursor) == Some(&'\n') {
            return Some(cursor + 1);
        }
        while self
            .letters
            .get(cursor)
            .is_some_and(|held| super::classes::is_carried(*held))
        {
            cursor += 1;
        }
        Some(cursor)
    }

    /// Match each piece of a sequence in turn, the frame for the remainder
    /// built as each piece is reached.
    fn sequence<'r>(
        &mut self,
        pieces: &'a [Node],
        at: usize,
        rest: &'r Rest<'a, 'r>,
    ) -> Option<usize> {
        let Some((first, others)) = pieces.split_first() else {
            return self.carry_on(rest, at);
        };
        if others.is_empty() {
            return self.run(first, at, rest);
        }
        let frame = Rest::Seq {
            pieces: others,
            next: rest,
        };
        self.run(first, at, &frame)
    }

    /// Carry on with whatever the frames say follows.
    fn carry_on<'r>(&mut self, rest: &'r Rest<'a, 'r>, at: usize) -> Option<usize> {
        self.steps += 1;
        if self.steps > STEP_BUDGET {
            return None;
        }
        match rest {
            Rest::Done => Some(at),
            Rest::EndsAt(wanted) => {
                if at == *wanted {
                    return Some(at);
                }
                None
            }
            Rest::NoFurtherThan(limit) => {
                if at > *limit {
                    return None;
                }
                Some(at)
            }
            Rest::Seq { pieces, next } => self.sequence(pieces, at, next),
            Rest::Close { index, from, next } => {
                let saved = self.slots[*index];
                self.slots[*index] = Some((*from, at));
                match self.carry_on(next, at) {
                    Some(end) => Some(end),
                    None => {
                        self.slots[*index] = saved;
                        None
                    }
                }
            }
            Rest::Again {
                node,
                least,
                most,
                greed,
                clears,
                done,
                from,
                writes,
                next,
            } => {
                // A turn that read nothing and wrote no group ends the
                // repetition, which is what keeps `(a?)*` from turning
                // forever. One that wrote a group left the loop somewhere it
                // has not been, so it carries on.
                if at == *from && self.state_mark() == *writes {
                    return self.carry_on(next, at);
                }
                self.repeat(node, *least, *most, *greed, *clears, *done, at, next)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn repeat<'r>(
        &mut self,
        node: &'a Node,
        least: u32,
        most: Option<u32>,
        greed: Greed,
        clears: Option<usize>,
        done: u32,
        at: usize,
        rest: &'r Rest<'a, 'r>,
    ) -> Option<usize> {
        let room = most.is_none_or(|counted| done < counted);
        if done < least {
            let frame = Rest::Again {
                node,
                least,
                most,
                greed,
                clears,
                done: done + 1,
                from: at,
                writes: self.state_mark(),
                next: rest,
            };
            self.forget(clears);
            return self.run(node, at, &frame);
        }
        match greed {
            Greed::Greedy => {
                if room {
                    let saved = self.saved();
                    let frame = Rest::Again {
                        node,
                        least,
                        most,
                        greed,
                        clears,
                        done: done + 1,
                        from: at,
                        writes: self.state_mark(),
                        next: rest,
                    };
                    self.forget(clears);
                    if let Some(end) = self.run(node, at, &frame) {
                        return Some(end);
                    }
                    self.restore(saved);
                }
                self.carry_on(rest, at)
            }
            Greed::Lazy => {
                let saved = self.saved();
                if let Some(end) = self.carry_on(rest, at) {
                    return Some(end);
                }
                self.restore(saved);
                if room {
                    let frame = Rest::Again {
                        node,
                        least,
                        most,
                        greed,
                        clears,
                        done: done + 1,
                        from: at,
                        writes: self.state_mark(),
                        next: rest,
                    };
                    self.forget(clears);
                    return self.run(node, at, &frame);
                }
                None
            }
            Greed::Possessive => {
                let mut cursor = at;
                let mut counted = done;
                while most.is_none_or(|limit| counted < limit) {
                    let Some(end) = self.run(node, cursor, &Rest::Done) else {
                        break;
                    };
                    if end == cursor {
                        break;
                    }
                    cursor = end;
                    counted += 1;
                }
                if counted < least {
                    return None;
                }
                self.carry_on(rest, cursor)
            }
        }
    }

    fn look<'r>(
        &mut self,
        behind: bool,
        negated: bool,
        node: &'a Node,
        at: usize,
        rest: &'r Rest<'a, 'r>,
    ) -> Option<usize> {
        let saved = self.saved();
        let found = if behind {
            // A lookbehind reads text ending where it is written, so every
            // place it could start is tried, nearest first.
            (0..=at).rev().any(|from| {
                let taken = self.saved();
                if self.run(node, from, &Rest::EndsAt(at)).is_some() {
                    return true;
                }
                self.restore(taken);
                false
            })
        } else {
            self.run(node, at, &Rest::Done).is_some()
        };
        if found == negated {
            self.restore(saved);
            return None;
        }
        if negated {
            self.restore(saved.clone());
        }
        match self.carry_on(rest, at) {
            Some(end) => Some(end),
            None => {
                self.restore(saved);
                None
            }
        }
    }

    fn backreference<'r>(
        &mut self,
        target: &Target,
        folded: bool,
        at: usize,
        rest: &'r Rest<'a, 'r>,
    ) -> Option<usize> {
        // A name written on more than one group stands for whichever of them
        // matched, so each is tried in turn.
        for index in self.slot_indexes(target) {
            let Some((from, to)) = self.slots.get(index).copied().flatten() else {
                continue;
            };
            let width = to - from;
            if at + width > self.letters.len() {
                continue;
            }
            let same = (0..width).all(|step| {
                let wanted = self.letters[from + step];
                let held = self.letters[at + step];
                held == wanted || (folded && same_folded(held, wanted))
            });
            if same && let Some(end) = self.carry_on(rest, at + width) {
                return Some(end);
            }
        }
        None
    }

    /// `(?~x)` reads as far as it can without any of what it read holding a
    /// match for `x`.
    fn absent<'r>(&mut self, node: &'a Node, at: usize, rest: &'r Rest<'a, 'r>) -> Option<usize> {
        let mut end = self.letters.len();
        loop {
            if !self.holds_a_match(node, at, end) {
                let saved = self.saved();
                if let Some(done) = self.carry_on(rest, end) {
                    return Some(done);
                }
                self.restore(saved);
            }
            if end == at {
                return None;
            }
            end -= 1;
        }
    }

    /// Whether `node` matches anywhere inside the run from `from` to `to`.
    fn holds_a_match(&mut self, node: &'a Node, from: usize, to: usize) -> bool {
        for start in from..=to {
            let saved = self.saved();
            let found = self.run(node, start, &Rest::NoFurtherThan(to)).is_some();
            self.restore(saved);
            if found {
                return true;
            }
        }
        false
    }

    fn slot_index(&self, target: &Target) -> Option<usize> {
        self.slot_indexes(target).first().copied()
    }

    /// The group numbers a target names, the ones that matched first.
    fn slot_indexes(&self, target: &Target) -> Vec<usize> {
        match target {
            Target::Numbered(index) => vec![*index],
            Target::Relative(step) => {
                let counted = self.names.len() as isize - 1;
                let index = if *step < 0 { counted + 1 + step } else { *step };
                if index > 0 {
                    vec![index as usize]
                } else {
                    Vec::new()
                }
            }
            Target::Named(name) => {
                let mut found: Vec<usize> = self
                    .names
                    .iter()
                    .enumerate()
                    .filter(|(_, held)| held.as_deref() == Some(name.as_str()))
                    .map(|(index, _)| index)
                    .collect();
                found.sort_by_key(|index| self.slots[*index].is_none());
                found
            }
        }
    }
}

/// Whether two characters are the same once case is set aside.
fn same_folded(held: char, wanted: char) -> bool {
    held.to_lowercase().eq(wanted.to_lowercase())
}

/// The body of each numbered group, so a call can match it again.
fn collect_bodies<'a>(node: &'a Node, bodies: &mut Vec<Option<&'a Node>>) {
    match node {
        Node::Capture { index, node } => {
            if *index < bodies.len() {
                bodies[*index] = Some(node.as_ref());
            }
            collect_bodies(node, bodies);
        }
        Node::Concat(pieces) | Node::Alternate(pieces) => {
            for piece in pieces {
                collect_bodies(piece, bodies);
            }
        }
        Node::Repeat { node, .. }
        | Node::Group(node)
        | Node::Atomic(node)
        | Node::Look { node, .. }
        | Node::Absent(node) => collect_bodies(node, bodies),
        Node::Conditional {
            then_node,
            else_node,
            ..
        } => {
            collect_bodies(then_node, bodies);
            collect_bodies(else_node, bodies);
        }
        _ => {}
    }
}
