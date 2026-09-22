// `String#unpack`: reading the bytes back as the directives describe them.

use super::*;

impl VirtualMachine {
    /// `String#unpack` — read the bytes back as the directives describe them.
    pub(crate) fn string_unpack(
        &mut self,
        text: &str,
        format: &str,
        pointer: u64,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let bytes = string_to_bytes(text);
        let directives = parse_format(format, "unpack", position)?;
        let mut out: Vec<Object> = Vec::new();
        let mut at = 0usize;
        for directive in &directives {
            let remaining = bytes.len().saturating_sub(at);
            match directive.code {
                // An address rather than text. The string it names is the one
                // `pack` was given, which only a string `pack` built, or a
                // copy of one, carries an address into.
                'P' | 'p' => {
                    if remaining < POINTER_WIDTH {
                        out.push(Object::Nil);
                        at = bytes.len();
                        continue;
                    }
                    if pointer == 0 {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "no associated pointer",
                            position,
                        ));
                    }
                    let mut named = [0u8; POINTER_WIDTH];
                    named.copy_from_slice(&bytes[at..at + POINTER_WIDTH]);
                    at += POINTER_WIDTH;
                    let named = u64::from_ne_bytes(named);
                    let Some(held) = self.packed_pointers.get(&named).cloned() else {
                        out.push(Object::Nil);
                        continue;
                    };
                    let Object::String(held) = held else {
                        out.push(Object::Nil);
                        continue;
                    };
                    let wanted = match (directive.code, &directive.count) {
                        ('p', _) | (_, Count::Rest) => None,
                        (_, Count::Exactly(width)) => Some(*width),
                        (_, Count::One) => Some(1),
                    };
                    let source = string_to_bytes(&held.as_str());
                    let stops = source
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(source.len());
                    let ends = wanted.map_or(stops, |width| width.min(stops));
                    out.push(bytes_to_string(&source[..ends]));
                }
                // A run of bytes, read as text. 'a' keeps what is there, 'A'
                // drops the trailing spaces and nulls, and 'Z' stops at the
                // first null.
                'a' | 'A' | 'Z' => {
                    // `Z*` reads up to the first null and steps past it, so
                    // the next directive starts on what follows.
                    if directive.code == 'Z' && directive.count == Count::Rest {
                        let end = bytes[at..]
                            .iter()
                            .position(|byte| *byte == 0)
                            .map_or(bytes.len(), |offset| at + offset);
                        let held: String =
                            bytes[at..end].iter().map(|byte| *byte as char).collect();
                        at = (end + 1).min(bytes.len());
                        out.push(Object::binary_string(held));
                        continue;
                    }
                    let taken = match directive.count {
                        Count::Rest => remaining,
                        Count::One => remaining.min(1),
                        Count::Exactly(n) => remaining.min(n),
                    };
                    let slice = &bytes[at..at + taken];
                    at += taken;
                    let held: String = slice.iter().map(|byte| *byte as char).collect();
                    let settled = match directive.code {
                        'A' => held.trim_end_matches(['\0', ' ']).to_string(),
                        'Z' => match held.find('\0') {
                            Some(end) => held[..end].to_string(),
                            None => held,
                        },
                        _ => held,
                    };
                    out.push(Object::binary_string(settled));
                }
                // Bits, most or least significant first.
                'b' | 'B' => {
                    let wanted = match directive.count {
                        Count::Rest => remaining * 8,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let available = remaining * 8;
                    let taken = wanted.min(available);
                    let mut written = String::with_capacity(taken);
                    for step in 0..taken {
                        let byte = bytes[at + step / 8];
                        let bit = if directive.code == 'B' {
                            (byte >> (7 - step % 8)) & 1
                        } else {
                            (byte >> (step % 8)) & 1
                        };
                        written.push(if bit == 1 { '1' } else { '0' });
                    }
                    at += taken.div_ceil(8);
                    out.push(ascii_string(written));
                }
                // Nibbles, high or low half first.
                'h' | 'H' => {
                    let wanted = match directive.count {
                        Count::Rest => remaining * 2,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let available = remaining * 2;
                    let taken = wanted.min(available);
                    let mut written = String::with_capacity(taken);
                    for step in 0..taken {
                        let byte = bytes[at + step / 2];
                        let nibble = if directive.code == 'H' {
                            if step % 2 == 0 { byte >> 4 } else { byte & 0xf }
                        } else if step % 2 == 0 {
                            byte & 0xf
                        } else {
                            byte >> 4
                        };
                        written.push(char::from_digit(u32::from(nibble), 16).unwrap_or('0'));
                    }
                    at += taken.div_ceil(2);
                    out.push(ascii_string(written));
                }
                // A UTF-8 character read back as its code point.
                'U' => {
                    let wanted = match directive.count {
                        Count::Rest => usize::MAX,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    // Bytes that spell no character are refused rather than
                    // read as the one that stands in for a broken run.
                    let source = match std::str::from_utf8(&bytes[at..]) {
                        Ok(text) => text,
                        Err(problem) if problem.valid_up_to() > 0 => {
                            std::str::from_utf8(&bytes[at..at + problem.valid_up_to()])
                                .expect("the bytes read as text up to here")
                        }
                        Err(_) => {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "malformed UTF-8 character",
                                position,
                            ));
                        }
                    };
                    let mut read = 0;
                    for character in source.chars() {
                        if read >= wanted {
                            break;
                        }
                        out.push(Object::Int(character as i64));
                        at += character.len_utf8();
                        read += 1;
                    }
                    if read < wanted && wanted != usize::MAX && at < bytes.len() {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "malformed UTF-8 character",
                            position,
                        ));
                    }
                }
                // Skip forward, back, or to a fixed place.
                // `x` steps forward, and `x*` runs to the end.
                'x' => {
                    let step = match directive.count {
                        Count::Rest => remaining,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if at + step > bytes.len() {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "x outside of string",
                            position,
                        ));
                    }
                    at += step;
                }
                // `X` steps back, and `X*` steps back by however many bytes
                // are left, which there has to be room for.
                'X' => {
                    let step = match directive.count {
                        Count::Rest => remaining,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if step > at {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "X outside of string",
                            position,
                        ));
                    }
                    at -= step;
                }
                // `@` moves to a fixed place, and `@*` names no place, so it
                // leaves the index where it stands.
                '@' => {
                    if let Count::Exactly(place) = directive.count {
                        if place > bytes.len() {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "@ outside of string",
                                position,
                            ));
                        }
                        at = place;
                    } else if directive.count == Count::One {
                        at = 0;
                    }
                }
                // The integer directives, which differ only in width, order,
                // and whether they are signed.
                code if integer_width(code, directive.native).is_some() => {
                    let width =
                        integer_width(code, directive.native).expect("the guard read a width");
                    let wanted = match directive.count {
                        Count::Rest => remaining / width,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        if at + width > bytes.len() {
                            // Ruby fills the count a format asked for with
                            // nil once the bytes run out, and answers nothing
                            // at all for `*`.
                            if directive.count != Count::Rest {
                                out.push(Object::Nil);
                            }
                            at = bytes.len();
                            continue;
                        }
                        let value = read_integer(&bytes[at..at + width], code, directive.order);
                        at += width;
                        out.push(Object::integer(num_bigint::BigInt::from(value)));
                    }
                }
                // The floating point directives. 'e' and 'g' name the two
                // byte orders outright, and 'f'/'d' follow the platform's.
                'f' | 'F' | 'e' | 'd' | 'D' | 'E' | 'g' | 'G' => {
                    let width = if matches!(directive.code, 'f' | 'F' | 'e' | 'g') {
                        4
                    } else {
                        8
                    };
                    let big = matches!(directive.code, 'g' | 'G');
                    let wanted = match directive.count {
                        Count::Rest => remaining / width,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        if at + width > bytes.len() {
                            if directive.count != Count::Rest {
                                out.push(Object::Nil);
                            }
                            at = bytes.len();
                            continue;
                        }
                        let mut slice = bytes[at..at + width].to_vec();
                        if big {
                            slice.reverse();
                        }
                        at += width;
                        let value = if width == 4 {
                            f64::from(f32::from_le_bytes(
                                slice.try_into().expect("four bytes were taken"),
                            ))
                        } else {
                            f64::from_le_bytes(slice.try_into().expect("eight bytes were taken"))
                        };
                        out.push(Object::Float(value));
                    }
                }
                // A BER compressed integer: seven bits per byte, most
                // significant first, with the top bit set on every byte but
                // the last.
                'w' => {
                    let wanted = match directive.count {
                        Count::Rest => usize::MAX,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let mut read = 0;
                    while read < wanted && at < bytes.len() {
                        let mut value = num_bigint::BigInt::from(0);
                        while let Some(byte) = bytes.get(at) {
                            at += 1;
                            value = value * 128 + num_bigint::BigInt::from(byte & 0x7f);
                            if byte & 0x80 == 0 {
                                break;
                            }
                        }
                        out.push(Object::integer(value));
                        read += 1;
                    }
                }
                // Base64, in the strict form 'm0' asks for and the lenient
                // one every other count admits.
                'm' => {
                    let held: String = bytes[at..].iter().map(|byte| *byte as char).collect();
                    at = bytes.len();
                    // `m0` reads strict base64, where anything outside the
                    // alphabet is a fault rather than something to skip.
                    if directive.count == Count::Exactly(0) && !is_strict_base64(&held) {
                        let message = "invalid base64".to_string();
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("ArgumentError", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        });
                    }
                    out.push(bytes_to_string(&decode_base64(&held)));
                }
                // Quoted printable.
                'M' => {
                    let held: String = bytes[at..].iter().map(|byte| *byte as char).collect();
                    at = bytes.len();
                    out.push(bytes_to_string(&decode_quoted_printable(&held)));
                }
                // Uuencoding, as `uuencode` writes it.
                'u' => {
                    let held: String = bytes[at..].iter().map(|byte| *byte as char).collect();
                    at = bytes.len();
                    out.push(bytes_to_string(&decode_uu(&held)));
                }
                other => {
                    return Err(unknown_directive(other, format, "unpack", position));
                }
            }
        }
        Ok(out)
    }
}
