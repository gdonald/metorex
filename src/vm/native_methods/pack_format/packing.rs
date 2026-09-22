// `Array#pack`: writing the items out as the directives describe them.

use super::*;

impl VirtualMachine {
    /// `Array#pack` — write the items out as the directives describe them.
    pub(crate) fn array_pack(
        &mut self,
        items: &[Object],
        format: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let directives = parse_format(format, "pack", position)?;
        // Nothing written is nothing but ASCII, which is what Ruby tags an
        // empty result as.
        if directives.is_empty() {
            return Ok(Object::String(std::rc::Rc::new(
                crate::object::StringValue::with_encoding(String::new(), "US-ASCII"),
            )));
        }
        let mut out: Vec<u8> = Vec::new();
        let mut taken = 0usize;
        let mut pointer = 0u64;
        for directive in &directives {
            let left = items.len().saturating_sub(taken);
            match directive.code {
                // A run of bytes written from one string. 'a' pads with
                // nulls, 'A' with spaces, and 'Z' adds a null of its own.
                'a' | 'A' | 'Z' => {
                    let held = match items.get(taken) {
                        Some(Object::String(text)) => text.as_str().to_string(),
                        // nil packs as nothing, which the padding then fills.
                        Some(Object::Nil) => String::new(),
                        Some(other) => self.coerce_pack_string(other, position)?,
                        None => return Err(too_few_items(position)),
                    };
                    taken += 1;
                    let source = string_to_bytes(&held);
                    let width = match directive.count {
                        Count::Rest => source.len() + usize::from(directive.code == 'Z'),
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let filler = if directive.code == 'A' { b' ' } else { 0 };
                    for step in 0..width {
                        out.push(source.get(step).copied().unwrap_or(filler));
                    }
                }
                // Bits and nibbles, written from a string of '0'/'1' or of
                // hexadecimal digits.
                'b' | 'B' | 'h' | 'H' => {
                    let held = match items.get(taken) {
                        Some(Object::String(text)) => text.as_str().to_string(),
                        Some(Object::Nil) => String::new(),
                        Some(other) => self.coerce_pack_string(other, position)?,
                        None => return Err(too_few_items(position)),
                    };
                    taken += 1;
                    // Ruby reads the bits and nibbles off the bytes of the
                    // string, so a character spelled in several bytes
                    // contributes one digit per byte.
                    let digits = string_to_bytes(&held);
                    let wanted = match directive.count {
                        Count::Rest => digits.len(),
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let bits_each = if matches!(directive.code, 'b' | 'B') {
                        1
                    } else {
                        4
                    };
                    let per_byte = 8 / bits_each;
                    let mut current = 0u8;
                    for step in 0..wanted {
                        let value = match digits.get(step) {
                            // A bit is the low bit of the character that
                            // names it, so '0' and '1' read as themselves and
                            // any other character contributes its own.
                            Some(digit) if bits_each == 1 => *digit & 1,
                            // A letter counts nine past its own low half, so
                            // 'a' through 'f' land on ten through fifteen and
                            // every other letter follows the same step. Any
                            // other character contributes its low half alone.
                            Some(digit) => {
                                let low = *digit & 0xf;
                                if digit.is_ascii_alphabetic() {
                                    (low + 9) & 0xf
                                } else {
                                    low
                                }
                            }
                            None => 0,
                        };
                        let slot = step % per_byte;
                        let shift = match directive.code {
                            'B' => 7 - slot,
                            'b' => slot,
                            'H' => 4 - slot * 4,
                            _ => slot * 4,
                        };
                        current |= value << shift;
                        if slot == per_byte - 1 {
                            out.push(current);
                            current = 0;
                        }
                    }
                    if wanted % per_byte != 0 {
                        out.push(current);
                    }
                }
                // A code point written as UTF-8.
                'U' => {
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let Some(item) = items.get(taken) else {
                            return Err(too_few_items(position));
                        };
                        let point = self.coerce_pack_integer(item, position)?;
                        taken += 1;
                        if !(0..=0xffff_ffff).contains(&point) {
                            return Err(crate::vm::errors::simple_exception(
                                "RangeError",
                                &format!("pack(U): value out of range: {}", point),
                                position,
                            ));
                        }
                        out.extend_from_slice(&utf8_style_bytes(point as u64));
                    }
                }
                // A BER compressed integer.
                'w' => {
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let Some(item) = items.get(taken) else {
                            return Err(too_few_items(position));
                        };
                        let mut value = self.coerce_pack_integer(item, position)?;
                        // A compressed integer has no sign to carry, so a
                        // negative one cannot be written at all.
                        if value < 0 {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "can't compress negative numbers",
                                position,
                            ));
                        }
                        taken += 1;
                        let mut written = vec![(value & 0x7f) as u8];
                        value >>= 7;
                        while value > 0 {
                            written.push(((value & 0x7f) as u8) | 0x80);
                            value >>= 7;
                        }
                        written.reverse();
                        out.extend_from_slice(&written);
                    }
                }
                // Padding and placement, which take no item.
                'x' => {
                    let step = match directive.count {
                        Count::Rest => 0,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    out.extend(std::iter::repeat_n(0u8, step));
                }
                'X' => {
                    let step = match directive.count {
                        Count::Rest => 0,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if step > out.len() {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "X outside of string",
                            position,
                        ));
                    }
                    out.truncate(out.len() - step);
                }
                '@' => {
                    let place = match directive.count {
                        Count::Rest => 0,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if place > out.len() {
                        out.resize(place, 0);
                    } else {
                        out.truncate(place);
                    }
                }
                // The floating point directives.
                'f' | 'F' | 'e' | 'd' | 'D' | 'E' | 'g' | 'G' => {
                    let width = if matches!(directive.code, 'f' | 'F' | 'e' | 'g') {
                        4
                    } else {
                        8
                    };
                    let big = matches!(directive.code, 'g' | 'G');
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let Some(item) = items.get(taken) else {
                            return Err(too_few_items(position));
                        };
                        let value = self.coerce_pack_float(item, position)?;
                        taken += 1;
                        let mut written = if width == 4 {
                            (value as f32).to_le_bytes().to_vec()
                        } else {
                            value.to_le_bytes().to_vec()
                        };
                        if big {
                            written.reverse();
                        }
                        out.extend_from_slice(&written);
                    }
                }
                // The integer directives.
                code if integer_width(code, directive.native).is_some() => {
                    let width =
                        integer_width(code, directive.native).expect("the guard read a width");
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let Some(item) = items.get(taken) else {
                            return Err(too_few_items(position));
                        };
                        let value = self.coerce_pack_integer(item, position)?;
                        taken += 1;
                        out.extend_from_slice(&write_integer(value, width, code, directive.order));
                    }
                }
                // Uuencoding, written in lines of the length the count names.
                'u' | 'm' | 'M' => {
                    let held = match items.get(taken) {
                        Some(Object::String(text)) => text.as_str().to_string(),
                        // Quoted-printable writes whatever an object says it
                        // is, rather than asking it for a string of its own.
                        Some(other) if directive.code == 'M' => {
                            match self.send_to_object(other.clone(), "to_s", vec![], position)? {
                                Object::String(text) => text.as_str().to_string(),
                                spelled => spelled.to_string(),
                            }
                        }
                        Some(other) => self.coerce_pack_string(other, position)?,
                        None => return Err(too_few_items(position)),
                    };
                    taken += 1;
                    let source = string_to_bytes(&held);
                    let written = match directive.code {
                        'u' => encode_uu(&source, uu_line_length(&directive.count)),
                        'M' => {
                            encode_quoted_printable(&source, quoted_line_length(&directive.count))
                        }
                        // `m0` asks for base64 with no line breaks in it.
                        _ => encode_base64(&source, base64_line_length(&directive.count)),
                    };
                    out.extend_from_slice(written.as_bytes());
                }
                // 'P' and 'p' write where a string stands rather than what it
                // holds. Metorex hands out a number of its own for each one
                // and keeps the string under it, so `unpack` reads it back.
                'P' | 'p' => {
                    let held = match items.get(taken) {
                        Some(Object::Nil) => None,
                        Some(other @ Object::String(_)) => Some(other.clone()),
                        Some(other) => {
                            Some(Object::string(self.coerce_pack_string(other, position)?))
                        }
                        None => return Err(too_few_items(position)),
                    };
                    taken += 1;
                    let named = match held {
                        None => 0,
                        Some(value) => {
                            let named = (self.packed_pointers.len() as u64 + 1) * 8;
                            self.packed_pointers.insert(named, value);
                            pointer = named;
                            named
                        }
                    };
                    out.extend_from_slice(&named.to_ne_bytes());
                }
                other => {
                    return Err(unknown_directive(other, format, "pack", position));
                }
            }
        }
        let named = pack_result_encoding(&directives);
        // Text written as code points reads back as the characters those
        // bytes spell, where the bytes spell any. Anything else stands for
        // the bytes themselves.
        if named == "UTF-8"
            && let Ok(text) = String::from_utf8(out.clone())
        {
            let made = crate::object::StringValue::with_encoding(text, named);
            made.set_pointer(pointer);
            return Ok(Object::String(std::rc::Rc::new(made)));
        }
        let made =
            crate::object::StringValue::with_encoding(bytes_to_string(&out).to_string(), named);
        made.mark_bytes();
        made.set_pointer(pointer);
        Ok(Object::String(std::rc::Rc::new(made)))
    }

    /// One item an integer directive was given, read as a number.
    pub(crate) fn coerce_pack_integer(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<i128, MetorexError> {
        match value {
            Object::Int(number) => return Ok(*number as i128),
            Object::Float(number) => return Ok(*number as i128),
            Object::BigInt(number) => {
                // Only the low bytes are written, so a number too wide for
                // the directive is taken modulo what fits.
                let wrapped: num_bigint::BigInt =
                    number.as_ref() & num_bigint::BigInt::from(u128::MAX);
                return Ok(wrapped.try_into().unwrap_or(0));
            }
            _ => {}
        }
        if self.responds_to(value, "to_int")
            && let Object::Int(number) =
                self.send_to_object(value.clone(), "to_int", vec![], position)?
        {
            return Ok(number as i128);
        }
        let message = format!(
            "no implicit conversion of {} into Integer",
            self.builtins().class_of(value).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    /// One item a float directive was given, read as a number.
    pub(crate) fn coerce_pack_float(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        match value {
            Object::Float(number) => return Ok(*number),
            Object::Int(number) => return Ok(*number as f64),
            _ => {}
        }
        // Any other number converts through `to_f`, which is how a Rational
        // or a Complex with no imaginary part reaches a double. Anything that
        // is not a number at all is refused.
        let class = self.builtins().class_of(value);
        if crate::vm::method_invocation::descends_from(&class, "Numeric")
            || self.responds_to(value, "to_f")
        {
            // A number that has no `to_f`, or one whose `to_f` answers
            // something else, is refused the same way a non-number is.
            if let Ok(Object::Float(number)) =
                self.send_to_object(value.clone(), "to_f", vec![], position)
            {
                return Ok(number);
            }
        }
        let message = format!("can't convert {} into Float", class.name());
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    /// One item a string directive was given, read as a String.
    pub(crate) fn coerce_pack_string(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if self.responds_to(value, "to_str")
            && let Object::String(text) =
                self.send_to_object(value.clone(), "to_str", vec![], position)?
        {
            return Ok(text.as_str().to_string());
        }
        let message = format!(
            "no implicit conversion of {} into String",
            self.builtins().class_of(value).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }
}
