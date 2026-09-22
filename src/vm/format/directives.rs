// The text one directive writes, padded to its width.

use super::*;

impl VirtualMachine {
    /// The text one directive writes, already padded to its width.
    pub(crate) fn written_directive(
        &mut self,
        spec: char,
        directive: &Directive,
        arguments: &mut Arguments,
        format_encoding: &str,
        wrote_bytes: &mut bool,
        position: Position,
    ) -> Result<(String, String), MetorexError> {
        match spec {
            'b' | 'B' | 'd' | 'i' | 'o' | 'u' | 'x' | 'X' => {
                let value = self.directive_value(directive, arguments, position)?;
                let number = self.whole_number(&value, position)?;
                Ok((
                    written_whole(&number, spec, directive),
                    format_encoding.to_string(),
                ))
            }
            'e' | 'E' | 'f' | 'g' | 'G' | 'a' | 'A' => {
                let value = self.directive_value(directive, arguments, position)?;
                let number = self.fractional_number(&value, position)?;
                Ok((
                    written_fraction(number, spec, directive),
                    format_encoding.to_string(),
                ))
            }
            'c' => {
                let value = self.directive_value(directive, arguments, position)?;
                let text = self.written_character(value, format_encoding, wrote_bytes, position)?;
                Ok((pad(&text, directive, ' '), format_encoding.to_string()))
            }
            'p' | 's' => {
                let value = self.directive_value(directive, arguments, position)?;
                let asked = if spec == 'p' { "inspect" } else { "to_s" };
                let (text, encoding) = match (&value, spec) {
                    (Object::Nil, 's') => (String::new(), format_encoding.to_string()),
                    (Object::Nil, _) => ("nil".to_string(), format_encoding.to_string()),
                    _ => self.text_through(value, asked, wrote_bytes, position)?,
                };
                let text = cut_to_precision(&text, directive.precision);
                Ok((pad(&text, directive, ' '), encoding))
            }
            other => Err(argument_error(
                &format!("malformed format string - %{}", other),
                position,
            )),
        }
    }

    /// The character a `%c` directive writes. A String gives its first
    /// character and an Integer names one by its code.
    pub(crate) fn written_character(
        &mut self,
        value: Object,
        format_encoding: &str,
        wrote_bytes: &mut bool,
        position: Position,
    ) -> Result<String, MetorexError> {
        let held = match &value {
            Object::String(text) => Some(text.as_str().to_string()),
            _ if self.responds_to(&value, "to_str") => {
                match self.send_to_object(value.clone(), "to_str", vec![], position)? {
                    Object::String(text) => Some(text.as_str().to_string()),
                    _ => {
                        return Err(type_error(
                            format!("can't convert {} into String", self.class_name_of(&value)),
                            position,
                        ));
                    }
                }
            }
            _ => None,
        };
        if let Some(text) = held {
            return Ok(text.chars().next().map(String::from).unwrap_or_default());
        }
        let code = self.whole_number(&value, position)?;
        let Ok(code) = code.to_string().parse::<u32>() else {
            return Err(crate::vm::errors::simple_exception(
                "RangeError",
                &format!("{} out of char range", code),
                position,
            ));
        };
        // Only Unicode names a character by its code point. Every other
        // encoding spells the character with the bytes the number holds.
        match format_encoding {
            "UTF-8" => char::from_u32(code).map(String::from).ok_or_else(|| {
                crate::vm::errors::simple_exception(
                    "RangeError",
                    &format!("{} out of char range", code),
                    position,
                )
            }),
            "US-ASCII" if code < 0x80 => Ok(char::from(code as u8).to_string()),
            "US-ASCII" => Err(crate::vm::errors::simple_exception(
                "RangeError",
                &format!("{} out of char range", code),
                position,
            )),
            _ => {
                let mut bytes: Vec<u8> = code.to_be_bytes().to_vec();
                while bytes.len() > 1 && bytes[0] == 0 {
                    bytes.remove(0);
                }
                *wrote_bytes = true;
                Ok(crate::vm::native_methods::string_methods::bytes_as_text(
                    &bytes,
                ))
            }
        }
    }

    /// The text a value stands for, taken through `to_s` or `inspect`, along
    /// with the encoding it is written in.
    pub(crate) fn text_through(
        &mut self,
        value: Object,
        asked: &str,
        wrote_bytes: &mut bool,
        position: Position,
    ) -> Result<(String, String), MetorexError> {
        match self.send_to_object(value, asked, vec![], position)? {
            Object::String(text) => {
                if text.holds_bytes() {
                    *wrote_bytes = true;
                }
                Ok((text.as_str().to_string(), text.encoding_name()))
            }
            other => Ok((
                other.to_string(),
                crate::object::string_value::DEFAULT_ENCODING.to_string(),
            )),
        }
    }

    /// The whole number a directive writes, read the way Ruby reads one: a
    /// String through `Kernel#Integer`, and anything else through `to_int`
    /// and then `to_i`.
    pub(crate) fn whole_number(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<BigInt, MetorexError> {
        match value {
            Object::Int(held) => return Ok(BigInt::from(*held)),
            Object::BigInt(held) => return Ok((**held).clone()),
            Object::Float(held) => {
                return Ok(BigInt::from(held.trunc() as i64));
            }
            Object::String(text) => {
                let written = Object::string(text.as_str().to_string());
                return match self.kernel_integer(&[written], position)? {
                    Object::Int(held) => Ok(BigInt::from(held)),
                    Object::BigInt(held) => Ok((*held).clone()),
                    other => Err(type_error(
                        format!("can't convert {} into Integer", other.type_name()),
                        position,
                    )),
                };
            }
            Object::Nil => {
                return Err(type_error(
                    "no implicit conversion of nil into Integer".to_string(),
                    position,
                ));
            }
            _ => {}
        }
        for asked in ["to_int", "to_i"] {
            if !self.responds_to(value, asked) {
                continue;
            }
            return match self.send_to_object(value.clone(), asked, vec![], position)? {
                Object::Int(held) => Ok(BigInt::from(held)),
                Object::BigInt(held) => Ok((*held).clone()),
                _ => Err(type_error(
                    format!("can't convert {} into Integer", self.class_name_of(value)),
                    position,
                )),
            };
        }
        Err(type_error(
            format!(
                "no implicit conversion of {} into Integer",
                self.class_name_of(value)
            ),
            position,
        ))
    }

    /// The fractional number a directive writes, read through `to_f` the way
    /// `Kernel#Float` reads one.
    pub(crate) fn fractional_number(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        match value {
            Object::Float(held) => return Ok(*held),
            Object::Int(held) => return Ok(*held as f64),
            Object::BigInt(held) => {
                return Ok(held.to_string().parse::<f64>().unwrap_or(f64::INFINITY));
            }
            Object::String(text) => {
                let written = Object::string(text.as_str().to_string());
                return match self.kernel_float(&[written], position)? {
                    Object::Float(held) => Ok(held),
                    Object::Int(held) => Ok(held as f64),
                    other => Err(type_error(
                        format!("can't convert {} into Float", other.type_name()),
                        position,
                    )),
                };
            }
            Object::Nil => {
                return Err(type_error(
                    "can't convert nil into Float".to_string(),
                    position,
                ));
            }
            _ => {}
        }
        if self.responds_to(value, "to_f") {
            return match self.send_to_object(value.clone(), "to_f", vec![], position)? {
                Object::Float(held) => Ok(held),
                Object::Int(held) => Ok(held as f64),
                _ => Err(type_error(
                    format!("can't convert {} into Float", self.class_name_of(value)),
                    position,
                )),
            };
        }
        Err(type_error(
            format!("can't convert {} into Float", self.class_name_of(value)),
            position,
        ))
    }

    /// The name of the class a value is an instance of, which an error about
    /// a conversion has to report.
    pub(crate) fn class_name_of(&self, value: &Object) -> String {
        self.builtins().class_of(value).name().to_string()
    }
}

/// The name written between `<` and `>` or between `{` and `}`.
pub(crate) fn read_name(letters: &[char], at: &mut usize, closing: char) -> String {
    let mut name = String::new();
    *at += 1;
    while *at < letters.len() && letters[*at] != closing {
        name.push(letters[*at]);
        *at += 1;
    }
    if *at < letters.len() {
        *at += 1;
    }
    name
}

/// The number the digits at `at` spell, leaving `at` on the character after
/// them.
pub(crate) fn read_number(
    letters: &[char],
    at: &mut usize,
    position: Position,
) -> Result<i64, MetorexError> {
    let mut counted: i64 = 0;
    let mut over = false;
    while letters.get(*at).is_some_and(|held| held.is_ascii_digit()) {
        counted = counted
            .saturating_mul(10)
            .saturating_add(letters[*at] as i64 - '0' as i64);
        if counted > WIDEST {
            over = true;
        }
        *at += 1;
    }
    if over {
        return Err(argument_error("width too big", position));
    }
    Ok(counted)
}

/// The widest a field may be written, past which Ruby refuses the format.
pub(crate) const WIDEST: i64 = 1 << 30;

/// The first `precision` characters of a text, which is as much of it as a
/// string directive writes.
pub(crate) fn cut_to_precision(text: &str, precision: Option<i64>) -> String {
    let Some(precision) = precision else {
        return text.to_string();
    };
    text.chars().take(precision.max(0) as usize).collect()
}

/// A written value padded out to the width its directive names.
pub(crate) fn pad(text: &str, directive: &Directive, filler: char) -> String {
    let Some(width) = directive.width else {
        return text.to_string();
    };
    let counted = text.chars().count() as i64;
    if counted >= width {
        return text.to_string();
    }
    let room: String = std::iter::repeat_n(filler, (width - counted) as usize).collect();
    if directive.left_align {
        format!("{}{}", text, room)
    } else {
        format!("{}{}", room, text)
    }
}

/// The base a directive counts in.
pub(crate) fn base_of(spec: char) -> u32 {
    match spec {
        'b' | 'B' => 2,
        'o' => 8,
        'x' | 'X' => 16,
        _ => 10,
    }
}

/// The `0b`, `0x` and the rest a `#` flag writes before the digits.
pub(crate) fn alternate_prefix(spec: char) -> &'static str {
    match spec {
        'b' => "0b",
        'B' => "0B",
        'x' => "0x",
        'X' => "0X",
        _ => "",
    }
}
