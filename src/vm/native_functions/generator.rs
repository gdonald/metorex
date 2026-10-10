// The Mersenne Twister `rand` draws from.

use super::*;

impl VirtualMachine {
    /// The Integer `srand` seeds with, kept at its full width so the next
    /// call answers the same number back. A Float truncates and any other
    /// object must answer `#to_int`, as Ruby requires.
    pub(crate) fn coerce_to_seed(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match given {
            Object::Int(_) | Object::BigInt(_) => Ok(given.clone()),
            Object::Float(seed) => Ok(Object::Int(*seed as i64)),
            other => {
                let Some((class, method)) = self.lookup_method(other, "to_int") else {
                    let message = format!(
                        "no implicit conversion of {} into Integer",
                        crate::vm::errors::conversion_subject(other)
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                let converted =
                    self.invoke_method(class, method, other.clone(), vec![], position)?;
                self.coerce_to_seed(&converted, position)
            }
        }
    }

    /// Run the hooks `trace_var` registered for `name`, with the value just
    /// assigned. A String hook is evaluated as code, the way Ruby's is.
    pub(crate) fn fire_global_trace(
        &mut self,
        name: &str,
        value: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Some(hooks) = self.traced_globals.get(name).cloned() else {
            return Ok(());
        };
        for hook in hooks {
            match hook {
                Object::String(code) => {
                    let tokens = crate::lexer::Lexer::new(&code.as_str()).tokenize();
                    let statements =
                        crate::parser::Parser::new(tokens)
                            .parse()
                            .map_err(|errors| {
                                MetorexError::runtime_error(
                                    format!(
                                        "trace_var: parse error: {}",
                                        errors
                                            .iter()
                                            .map(|error| error.to_string())
                                            .collect::<Vec<_>>()
                                            .join("; ")
                                    ),
                                    crate::vm::utils::position_to_location(position),
                                )
                            })?;
                    for statement in &statements {
                        self.execute_statement(statement)?;
                    }
                }
                callable => {
                    self.invoke_callable(callable, vec![value.clone()], position)?;
                }
            }
        }
        Ok(())
    }

    /// Seed the generator from a whole number, the way Ruby seeds one: a
    /// single word spreads out on its own, and a wider seed is mixed in word
    /// by word.
    pub(crate) fn seed_random(&mut self, seed: &num_bigint::BigInt) {
        let magnitude = if *seed < num_bigint::BigInt::from(0) {
            -seed.clone()
        } else {
            seed.clone()
        };
        let mut words: Vec<u32> = Vec::new();
        let mut held = magnitude;
        let step = num_bigint::BigInt::from(1u64 << 32);
        while held > num_bigint::BigInt::from(0) {
            let low = &held % &step;
            words.push(low.to_string().parse::<u64>().unwrap_or(0) as u32);
            held /= &step;
        }
        if words.is_empty() {
            words.push(0);
        }
        // A seed whose top word is one carries nothing that word does not
        // already say, which is the trim Ruby makes before mixing.
        if words.len() > 1 && words[words.len() - 1] == 1 {
            words.pop();
        }
        self.random_words = vec![0u32; MT_WORDS];
        self.random_at = MT_WORDS;
        if words.len() <= 1 {
            self.seed_random_word(words[0]);
            return;
        }
        self.seed_random_word(19650218);
        let mut at = 1usize;
        let mut from = 0usize;
        let mut count = MT_WORDS.max(words.len());
        while count > 0 {
            let previous = self.random_words[at - 1];
            self.random_words[at] = (self.random_words[at]
                ^ (previous ^ (previous >> 30)).wrapping_mul(1664525))
            .wrapping_add(words[from])
            .wrapping_add(from as u32);
            at += 1;
            from += 1;
            if at >= MT_WORDS {
                self.random_words[0] = self.random_words[MT_WORDS - 1];
                at = 1;
            }
            if from >= words.len() {
                from = 0;
            }
            count -= 1;
        }
        let mut count = MT_WORDS - 1;
        while count > 0 {
            let previous = self.random_words[at - 1];
            self.random_words[at] = (self.random_words[at]
                ^ (previous ^ (previous >> 30)).wrapping_mul(1566083941))
            .wrapping_sub(at as u32);
            at += 1;
            if at >= MT_WORDS {
                self.random_words[0] = self.random_words[MT_WORDS - 1];
                at = 1;
            }
            count -= 1;
        }
        self.random_words[0] = 0x8000_0000;
        self.random_at = MT_WORDS;
    }

    /// The spread a single-word seed makes across the whole state.
    fn seed_random_word(&mut self, seed: u32) {
        self.random_words = vec![0u32; MT_WORDS];
        self.random_words[0] = seed;
        for at in 1..MT_WORDS {
            let previous = self.random_words[at - 1];
            self.random_words[at] = (previous ^ (previous >> 30))
                .wrapping_mul(1812433253)
                .wrapping_add(at as u32);
        }
        self.random_at = MT_WORDS;
    }

    /// The next word the generator answers.
    fn next_random_word(&mut self) -> u32 {
        if self.random_words.len() != MT_WORDS {
            let seed = num_bigint::BigInt::from(crate::vm::core::seed_from_clock());
            self.seed_random(&seed);
        }
        if self.random_at >= MT_WORDS {
            for at in 0..MT_WORDS {
                let mixed = (self.random_words[at] & 0x8000_0000)
                    | (self.random_words[(at + 1) % MT_WORDS] & 0x7fff_ffff);
                let mut next = self.random_words[(at + MT_STEP) % MT_WORDS] ^ (mixed >> 1);
                if mixed & 1 == 1 {
                    next ^= 0x9908_b0df;
                }
                self.random_words[at] = next;
            }
            self.random_at = 0;
        }
        let mut held = self.random_words[self.random_at];
        self.random_at += 1;
        held ^= held >> 11;
        held ^= (held << 7) & 0x9d2c_5680;
        held ^= (held << 15) & 0xefc6_0000;
        held ^ (held >> 18)
    }

    /// Advance the generator and answer the next 64 bits.
    fn next_random_bits(&mut self) -> u64 {
        let high = self.next_random_word() as u64;
        let low = self.next_random_word() as u64;
        (high << 32) | low
    }

    /// The next draw as a Float in [0, 1), read from two words the way Ruby
    /// reads one.
    pub(crate) fn next_random_float(&mut self) -> f64 {
        let high = (self.next_random_word() >> 5) as f64;
        let low = (self.next_random_word() >> 6) as f64;
        (high * 67108864.0 + low) * (1.0 / 9007199254740992.0)
    }

    /// The next draw as an Integer in [0, bound) for a bound past the
    /// machine word. Words are drawn until the value is under the bound,
    /// which keeps every value in the range equally likely.
    pub(crate) fn next_random_big(&mut self, bound: &num_bigint::BigInt) -> num_bigint::BigInt {
        let width = bound.bits();
        let words = width.div_ceil(64) as usize;
        loop {
            let mut drawn = num_bigint::BigInt::from(0);
            for _ in 0..words {
                drawn = (drawn << 64) + num_bigint::BigInt::from(self.next_random_bits());
            }
            drawn >>= (words as u64 * 64) - width;
            if drawn < *bound {
                return drawn;
            }
        }
    }

    /// The next draw as an Integer in [0, bound). Ruby fills a mask wide
    /// enough for the bound a word at a time and draws again whenever the
    /// value lands past it, which is what keeps the sequence the same.
    pub(crate) fn next_random_int(&mut self, bound: i64) -> i64 {
        if bound <= 0 {
            return 0;
        }
        let top = bound as u64 - 1;
        if top == 0 {
            return 0;
        }
        let mut mask = 1u64;
        while mask < top {
            mask = (mask << 1) | 1;
        }
        loop {
            let mut value = 0u64;
            let mut landed = true;
            for place in (0..2).rev() {
                if (mask >> (place * 32)) & 0xffff_ffff == 0 {
                    continue;
                }
                value |= (self.next_random_word() as u64) << (place * 32);
                value &= mask;
                if top < value {
                    landed = false;
                    break;
                }
            }
            if landed {
                return value as i64;
            }
        }
    }

    /// `Kernel#rand(limit)` for every argument shape Ruby accepts.
    pub(crate) fn random_below(
        &mut self,
        limit: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match limit {
            // Ruby ignores the sign, and a bound of zero means "no bound",
            // which draws a Float instead.
            Object::Int(bound) => match bound.unsigned_abs() {
                0 => Ok(Object::Float(self.next_random_float())),
                magnitude => Ok(Object::Int(self.next_random_int(magnitude as i64))),
            },
            // A Float bound truncates. `rand(0.999)` truncates to zero, so it
            // draws a Float the way `rand(0)` does.
            Object::Float(bound) => match bound.abs().trunc() as i64 {
                0 => Ok(Object::Float(self.next_random_float())),
                magnitude => Ok(Object::Int(self.next_random_int(magnitude))),
            },
            // A bound past the machine word is drawn word by word, since
            // there is no single draw wide enough to cover it.
            Object::BigInt(ref bound) => {
                let magnitude = if **bound < num_bigint::BigInt::from(0) {
                    -(**bound).clone()
                } else {
                    (**bound).clone()
                };
                if magnitude == num_bigint::BigInt::from(0) {
                    return Ok(Object::Float(self.next_random_float()));
                }
                Ok(Object::integer(self.next_random_big(&magnitude)))
            }
            Object::Range {
                ref start,
                ref end,
                exclusive,
                ..
            } => self.random_in_range(start, end, exclusive, position),
            other => {
                // Anything else is asked for an Integer bound.
                let Some((class, method)) = self.lookup_method(&other, "to_int") else {
                    let message = format!(
                        "no implicit conversion of {} into Integer",
                        crate::vm::errors::conversion_subject(&other)
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                let converted = self.invoke_method(class, method, other, vec![], position)?;
                self.random_below(converted, position)
            }
        }
    }

    /// `Kernel#rand(range)`. An all-Integer range draws an Integer; a Float on
    /// either side draws a Float. A backwards range answers nil.
    fn random_in_range(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let (Object::Int(low), Object::Int(high)) = (start, end) {
            let span = if exclusive {
                high - low
            } else {
                high - low + 1
            };
            if span <= 0 {
                return Ok(Object::Nil);
            }
            return Ok(Object::Int(low + self.next_random_int(span)));
        }
        let (Some(low), Some(high)) = (numeric_value(start), numeric_value(end)) else {
            return self.random_across_width(start, end, exclusive, position);
        };
        if high < low || (exclusive && high == low) {
            return Ok(Object::Nil);
        }
        if high == low {
            return Ok(Object::Float(low));
        }
        Ok(Object::Float(low + self.next_random_float() * (high - low)))
    }

    /// A range whose ends are neither Integers nor Floats. Ruby measures the
    /// width between them with `-` and adds a number of that size back onto
    /// the start, so any type that subtracts and adds can bound a draw.
    fn random_across_width(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let width = self
            .apply_named_method(end, "-", vec![start.clone()], position)
            .map_err(|_| bad_range_value(position))?;
        let drawn = match &width {
            Object::Float(measured) => Object::Float(self.next_random_float() * measured),
            _ => {
                let counted = match &width {
                    Object::Int(counted) => *counted,
                    _ => match self.apply_named_method(&width, "to_int", vec![], position) {
                        Ok(Object::Int(counted)) => counted,
                        _ => return Err(bad_range_value(position)),
                    },
                };
                let counted = if exclusive { counted } else { counted + 1 };
                if counted <= 0 {
                    return Err(bad_range_value(position));
                }
                Object::Int(self.next_random_int(counted))
            }
        };
        self.apply_named_method(start, "+", vec![drawn], position)
    }
}

/// The global's name without its `$`, however it was named.
/// The low machine word of a seed, which is all the generator reads. A seed
/// wider than 64 bits still has to drive the same state word.
/// How many words the Mersenne Twister keeps, and how far a step reaches.
pub(crate) const MT_WORDS: usize = 624;
pub(crate) const MT_STEP: usize = 397;

#[allow(dead_code)]
pub(crate) fn seed_low_bits(seed: &Object) -> u64 {
    match seed {
        Object::BigInt(wide) => {
            let (sign, digits) = wide.to_u64_digits();
            let magnitude = digits.first().copied().unwrap_or(0);
            if matches!(sign, num_bigint::Sign::Minus) {
                (magnitude as i64).wrapping_neg() as u64
            } else {
                magnitude
            }
        }
        Object::Int(narrow) => *narrow as u64,
        _ => 0,
    }
}

pub(crate) fn global_name_from(named: &Object) -> String {
    let text = match named {
        Object::Symbol(name) | Object::String(name) => name.as_str().to_string(),
        other => other.to_string(),
    };
    text.strip_prefix('$').unwrap_or(&text).to_string()
}
