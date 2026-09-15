//! `Kernel#sprintf`, `Kernel#format` and `String#%`: the directives a format
//! string carries, the values they take, and the text each one writes.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use num_bigint::BigInt;
use num_bigint::Sign;
use std::rc::Rc;

/// The flags, width and precision written between a `%` and the letter that
/// says what to write.
#[derive(Default)]
struct Directive {
    left_align: bool,
    plus: bool,
    space: bool,
    zero: bool,
    alternate: bool,
    width: Option<i64>,
    precision: Option<i64>,
    /// The `<name>` a directive was written with, which takes its value from
    /// the Hash rather than from the next argument.
    name: Option<String>,
    /// The `N$` a directive was written with, counting from one.
    numbered: Option<usize>,
}

/// Where the values a format string writes come from, and which ways of
/// naming them have been used so far. Ruby refuses a format that mixes them.
struct Arguments {
    values: Vec<Object>,
    keywords: Option<Object>,
    next: usize,
    took_in_order: bool,
    took_by_number: bool,
    took_by_name: bool,
}

/// Ruby's ArgumentError for a format string that asks for something it was
/// not given.
fn argument_error(message: &str, position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", message, position)
}

/// Ruby's TypeError for a value a directive cannot read.
fn type_error(message: String, position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("TypeError", &message, position)
}

impl Arguments {
    /// The next value in order, which a format may not ask for once it has
    /// asked for one by number.
    fn in_order(&mut self, position: Position) -> Result<Object, MetorexError> {
        if self.took_by_number {
            return Err(argument_error(
                "unnumbered(1) mixed with numbered",
                position,
            ));
        }
        self.took_in_order = true;
        let Some(value) = self.values.get(self.next) else {
            return Err(argument_error("too few arguments", position));
        };
        self.next += 1;
        Ok(value.clone())
    }

    /// The value at a written position, counting from one.
    fn by_number(&mut self, at: usize, position: Position) -> Result<Object, MetorexError> {
        if self.took_in_order {
            return Err(argument_error("numbered(2) after unnumbered(1)", position));
        }
        self.took_by_number = true;
        if at == 0 || at > self.values.len() {
            return Err(argument_error("too few arguments", position));
        }
        Ok(self.values[at - 1].clone())
    }
}

impl VirtualMachine {
    /// The text a format string writes, with each directive filled in from
    /// the values beside it.
    pub(crate) fn evaluate_string_format(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !matches!(&left, Object::String(_)) {
            return Err(type_error(
                format!("no implicit conversion of {} into String", left.type_name()),
                position,
            ));
        }
        let values = self.format_argument_list(right, position)?;
        self.format_with_values(&left, values, position)
    }

    /// The values a `String#%` reads, which is the Array its right operand
    /// stands for, or the operand itself where it stands for none.
    fn format_argument_list(
        &mut self,
        right: Object,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let Object::Array(held) = &right {
            return Ok(held.borrow().clone());
        }
        if let Some(Object::Array(held)) = crate::vm::native_methods::array_subclass_value(&right) {
            return Ok(held.borrow().clone());
        }
        if !self.responds_to(&right, "to_ary") {
            return Ok(vec![right]);
        }
        match self.send_to_object(right.clone(), "to_ary", vec![], position)? {
            Object::Array(held) => Ok(held.borrow().clone()),
            Object::Nil => Ok(vec![right]),
            other => Err(type_error(
                format!(
                    "can't convert {} to Array ({}#to_ary gives {})",
                    self.class_name_of(&right),
                    self.class_name_of(&right),
                    self.class_name_of(&other)
                ),
                position,
            )),
        }
    }

    /// The same, told outright which values the directives read, so a lone
    /// Array argument is not mistaken for a list of them.
    pub(crate) fn format_with_values(
        &mut self,
        left: &Object,
        values: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Object::String(format) = left else {
            return Err(type_error(
                format!("no implicit conversion of {} into String", left.type_name()),
                position,
            ));
        };
        let written = format.as_str().to_string();
        let format_encoding = format.encoding_name();
        // A lone Hash names its values as well as standing in a position, and
        // which of the two it is settles at the directive that reads it.
        let keywords = match values.as_slice() {
            [Object::Dict(_)] => Some(values[0].clone()),
            _ => None,
        };
        let mut arguments = Arguments {
            values,
            keywords,
            next: 0,
            took_in_order: false,
            took_by_number: false,
            took_by_name: false,
        };
        let mut answer = super::native_methods::string_methods::AnswerText::new(&format_encoding);
        // A character named by its code in an encoding that is not Unicode is
        // written as the bytes the number holds, so the answer carries bytes.
        let mut wrote_bytes = false;
        let letters: Vec<char> = written.chars().collect();
        let mut at = 0usize;
        while at < letters.len() {
            if letters[at] != '%' {
                let mut plain = String::new();
                while at < letters.len() && letters[at] != '%' {
                    plain.push(letters[at]);
                    at += 1;
                }
                answer.add(&plain, &format_encoding, position)?;
                continue;
            }
            at += 1;
            if at >= letters.len() {
                return Err(argument_error("incomplete format specifier", position));
            }
            if letters[at] == '%' {
                answer.add("%", &format_encoding, position)?;
                at += 1;
                continue;
            }
            let mut directive = Directive::default();
            let Some(spec) =
                self.read_directive(&letters, &mut at, &mut directive, &mut arguments, position)?
            else {
                // `%{name}` writes the value it names and carries no letter
                // of its own.
                let value = self.named_value(&mut arguments, &directive, position)?;
                let (text, encoding) =
                    self.text_through(value, "to_s", &mut wrote_bytes, position)?;
                let text = cut_to_precision(&text, directive.precision);
                answer.add(&pad(&text, &directive, ' '), &encoding, position)?;
                continue;
            };
            let (text, encoding) = self.written_directive(
                spec,
                &directive,
                &mut arguments,
                &format_encoding,
                &mut wrote_bytes,
                position,
            )?;
            answer.add(&text, &encoding, position)?;
        }
        // With `$VERBOSE` on Ruby points out values the format never reached.
        // A Hash naming its values is not counted, since a format reads only
        // the names it carries.
        // A Hash beside the format names its values, so leaving one of them
        // unread is not a value the format missed.
        let unused = !arguments.took_by_name
            && !arguments.took_by_number
            && arguments.keywords.is_none()
            && arguments.next < arguments.values.len();
        if unused && matches!(self.globals().get("DEBUG"), Some(Object::Bool(true))) {
            return Err(argument_error(
                "too many arguments for format string",
                position,
            ));
        }
        if unused && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
            eprintln!("warning: too many arguments for format string");
        }
        let encoding = answer.encoding();
        let made = crate::object::StringValue::with_encoding(answer.text(), encoding);
        if format.holds_bytes() || wrote_bytes {
            made.mark_bytes();
        }
        Ok(Object::String(Rc::new(made)))
    }

    /// The letter a directive ends with, reading the flags, the width and the
    /// precision that come before it. None for `%{name}`, which writes the
    /// value it names and ends there.
    fn read_directive(
        &mut self,
        letters: &[char],
        at: &mut usize,
        directive: &mut Directive,
        arguments: &mut Arguments,
        position: Position,
    ) -> Result<Option<char>, MetorexError> {
        let mut width_written = false;
        let mut precision_written = false;
        while *at < letters.len() {
            match letters[*at] {
                '-' => {
                    directive.left_align = true;
                    *at += 1;
                }
                '+' => {
                    directive.plus = true;
                    *at += 1;
                }
                ' ' => {
                    directive.space = true;
                    *at += 1;
                }
                '0' if !width_written => {
                    directive.zero = true;
                    *at += 1;
                }
                '#' => {
                    directive.alternate = true;
                    *at += 1;
                }
                '{' => {
                    directive.name = Some(read_name(letters, at, '}'));
                    return Ok(None);
                }
                '<' => {
                    directive.name = Some(read_name(letters, at, '>'));
                }
                '*' => {
                    if width_written {
                        return Err(argument_error("width given twice", position));
                    }
                    *at += 1;
                    let taken = self.starred_number(letters, at, arguments, position)?;
                    if taken < 0 {
                        directive.left_align = true;
                        directive.width = Some(-taken);
                    } else {
                        directive.width = Some(taken);
                    }
                    width_written = true;
                }
                '0'..='9' => {
                    let start = *at;
                    let counted = read_number(letters, at, position)?;
                    if letters.get(*at) == Some(&'$') {
                        *at += 1;
                        if directive.numbered.is_some() {
                            return Err(argument_error("value given twice", position));
                        }
                        directive.numbered = Some(counted as usize);
                    } else if letters[start] == '0' && !width_written {
                        // A leading zero is the flag that pads with zeros, and
                        // the digits after it are the width.
                        directive.zero = true;
                        *at = start + 1;
                    } else {
                        if width_written {
                            return Err(argument_error("width given twice", position));
                        }
                        directive.width = Some(counted);
                        width_written = true;
                    }
                }
                '.' => {
                    if precision_written {
                        return Err(argument_error("precision given twice", position));
                    }
                    precision_written = true;
                    *at += 1;
                    if letters.get(*at) == Some(&'*') {
                        *at += 1;
                        directive.precision =
                            Some(self.starred_number(letters, at, arguments, position)?);
                    } else {
                        directive.precision = Some(read_number(letters, at, position)?);
                    }
                }
                other => {
                    *at += 1;
                    return Ok(Some(other));
                }
            }
        }
        Err(argument_error("incomplete format specifier", position))
    }

    /// A width or a precision written as `*`, taken from a value beside the
    /// format rather than from the format itself.
    fn starred_number(
        &mut self,
        letters: &[char],
        at: &mut usize,
        arguments: &mut Arguments,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let taken = if letters.get(*at).is_some_and(|held| held.is_ascii_digit()) {
            let counted = read_number(letters, at, position)?;
            if letters.get(*at) == Some(&'$') {
                *at += 1;
            }
            arguments.by_number(counted as usize, position)?
        } else {
            arguments.in_order(position)?
        };
        let counted = self.whole_number(&taken, position)?;
        counted
            .to_string()
            .parse::<i64>()
            .map_err(|_| argument_error("width too big", position))
    }

    /// The value a `%<name>` or `%{name}` directive names, read from the Hash
    /// beside the format.
    fn named_value(
        &mut self,
        arguments: &mut Arguments,
        directive: &Directive,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.took_in_order || arguments.took_by_number {
            return Err(argument_error("named after unnumbered(1)", position));
        }
        arguments.took_by_name = true;
        let name = directive.name.clone().unwrap_or_default();
        let Some(held) = arguments.keywords.clone() else {
            return Err(argument_error("one hash required", position));
        };
        let key = Object::symbol(name.clone());
        let known = self
            .send_to_object(held.clone(), "key?", vec![key.clone()], position)?
            .is_truthy();
        if known {
            return self.send_to_object(held, "[]", vec![key], position);
        }
        // A Hash with a default answers for a key it does not hold, and only
        // a default of nil leaves the name unanswered.
        let found = self.send_to_object(held.clone(), "[]", vec![key.clone()], position)?;
        if !matches!(found, Object::Nil) {
            return Ok(found);
        }
        let message = format!("key{{{}}} not found", name);
        let trouble = Object::exception("KeyError", message.clone());
        if let Object::Exception(cell) = &trouble {
            let mut held_cell = cell.borrow_mut();
            held_cell.receiver = Some(Box::new(held));
            held_cell
                .instance_vars
                .insert(crate::vm::KEY_ERROR_KEY.to_string(), Object::symbol(name));
        }
        Err(MetorexError::UncaughtException {
            exception: trouble,
            location: position_to_location(position),
            message,
        })
    }

    /// The value a directive writes, taken by name, by number, or in order.
    fn directive_value(
        &mut self,
        directive: &Directive,
        arguments: &mut Arguments,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if directive.name.is_some() {
            return self.named_value(arguments, directive, position);
        }
        match directive.numbered {
            Some(at) => arguments.by_number(at, position),
            None => arguments.in_order(position),
        }
    }

    /// The text one directive writes, already padded to its width.
    fn written_directive(
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
    fn written_character(
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
    fn text_through(
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
    fn whole_number(&mut self, value: &Object, position: Position) -> Result<BigInt, MetorexError> {
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
    fn fractional_number(
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
    fn class_name_of(&self, value: &Object) -> String {
        self.builtins().class_of(value).name().to_string()
    }
}

/// The name written between `<` and `>` or between `{` and `}`.
fn read_name(letters: &[char], at: &mut usize, closing: char) -> String {
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
fn read_number(letters: &[char], at: &mut usize, position: Position) -> Result<i64, MetorexError> {
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
const WIDEST: i64 = 1 << 30;

/// The first `precision` characters of a text, which is as much of it as a
/// string directive writes.
fn cut_to_precision(text: &str, precision: Option<i64>) -> String {
    let Some(precision) = precision else {
        return text.to_string();
    };
    text.chars().take(precision.max(0) as usize).collect()
}

/// A written value padded out to the width its directive names.
fn pad(text: &str, directive: &Directive, filler: char) -> String {
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
fn base_of(spec: char) -> u32 {
    match spec {
        'b' | 'B' => 2,
        'o' => 8,
        'x' | 'X' => 16,
        _ => 10,
    }
}

/// The `0b`, `0x` and the rest a `#` flag writes before the digits.
fn alternate_prefix(spec: char) -> &'static str {
    match spec {
        'b' => "0b",
        'B' => "0B",
        'x' => "0x",
        'X' => "0X",
        _ => "",
    }
}

/// The text a whole-number directive writes.
fn written_whole(number: &BigInt, spec: char, directive: &Directive) -> String {
    let base = base_of(spec);
    let negative = matches!(number.sign(), Sign::Minus);
    // A negative number is written as a run of the base's largest digit
    // reaching back forever, unless a sign was asked for.
    let complement = negative && base != 10 && !directive.plus && !directive.space;
    let sign = if negative && !complement {
        "-"
    } else if directive.plus {
        "+"
    } else if directive.space {
        " "
    } else {
        ""
    };
    let mut digits = if complement {
        complement_digits(number, base, spec == 'X')
    } else {
        let size = if negative {
            -number.clone()
        } else {
            number.clone()
        };
        let written = size.to_str_radix(base);
        if spec == 'X' {
            written.to_uppercase()
        } else {
            written
        }
    };
    // A precision of zero writes nothing at all for a value of zero.
    if directive.precision == Some(0) && matches!(number.sign(), Sign::NoSign) {
        digits = String::new();
    }
    let mut prefix = String::new();
    if directive.alternate && !matches!(number.sign(), Sign::NoSign) {
        prefix.push_str(alternate_prefix(spec));
    }
    if let Some(precision) = directive.precision.filter(|held| *held > 0) {
        let filler = if complement {
            largest_digit(base, spec == 'X')
        } else {
            '0'
        };
        let counted = digits.chars().count() as i64;
        if counted < precision {
            let room: String =
                std::iter::repeat_n(filler, (precision - counted) as usize).collect();
            digits = insert_after_marker(&digits, &room, complement);
        }
    } else if directive.alternate && spec == 'o' && !complement && !digits.starts_with('0') {
        digits.insert(0, '0');
    }
    let body = format!("{}{}{}", sign, prefix, digits);
    // Zeros pad from the inside, after any sign and any prefix, and a run
    // reaching back forever pads with the base's largest digit instead.
    if directive.zero
        && !directive.left_align
        && directive.precision.is_none()
        && let Some(width) = directive.width
    {
        {
            let counted = body.chars().count() as i64;
            if counted < width {
                let filler = if complement {
                    largest_digit(base, spec == 'X')
                } else {
                    '0'
                };
                let room: String =
                    std::iter::repeat_n(filler, (width - counted) as usize).collect();
                let padded = if complement {
                    insert_after_marker(&digits, &room, true)
                } else {
                    format!("{}{}", room, digits)
                };
                return format!("{}{}{}", sign, prefix, padded);
            }
        }
    }
    pad(&body, directive, ' ')
}

/// The largest digit a base writes, which is what a run reaching back forever
/// is made of.
fn largest_digit(base: u32, upper: bool) -> char {
    let digit = std::char::from_digit(base - 1, base).expect("a digit of this base");
    if upper {
        digit.to_ascii_uppercase()
    } else {
        digit
    }
}

/// Put text straight after the `..d` that opens a run reaching back forever,
/// or at the front when there is none.
fn insert_after_marker(digits: &str, room: &str, complement: bool) -> String {
    if !complement {
        return format!("{}{}", room, digits);
    }
    let mut letters = digits.chars();
    let marker: String = letters.by_ref().take(3).collect();
    format!("{}{}{}", marker, room, letters.collect::<String>())
}

/// A negative number written the way Ruby writes one in a base of its own:
/// `..` and the base's largest digit stand for the run of them reaching back
/// forever, and the digits after it are what the run leaves.
fn complement_digits(number: &BigInt, base: u32, upper: bool) -> String {
    let largest = largest_digit(base, upper);
    // `~n` is the number whose digits, each taken from the largest, spell the
    // run's tail.
    let flipped = -(number + 1u32);
    let written = flipped.to_str_radix(base);
    let mut digits = String::new();
    for letter in written.chars() {
        let held = letter.to_digit(base).unwrap_or(0);
        let mut turned =
            std::char::from_digit(base - 1 - held, base).expect("a digit of this base");
        if upper {
            turned = turned.to_ascii_uppercase();
        }
        digits.push(turned);
    }
    let tail = digits.trim_start_matches(largest).to_string();
    format!("..{}{}", largest, tail)
}

/// The text a fractional directive writes.
fn written_fraction(number: f64, spec: char, directive: &Directive) -> String {
    if number.is_nan() || number.is_infinite() {
        let body = if number.is_nan() {
            "NaN".to_string()
        } else if number.is_sign_negative() {
            "-Inf".to_string()
        } else if directive.plus {
            "+Inf".to_string()
        } else if directive.space {
            " Inf".to_string()
        } else {
            "Inf".to_string()
        };
        return pad(&body, directive, ' ');
    }
    let negative = number.is_sign_negative();
    let sign = if negative {
        "-"
    } else if directive.plus {
        "+"
    } else if directive.space {
        " "
    } else {
        ""
    };
    let size = number.abs();
    let digits = match spec {
        'f' => decimal_notation(
            size,
            directive.precision.unwrap_or(6).max(0) as usize,
            directive.alternate,
        ),
        'e' | 'E' => {
            let written = exponent_form(
                size,
                directive.precision.unwrap_or(6).max(0) as usize,
                directive.alternate,
            );
            if spec == 'E' {
                written.to_uppercase()
            } else {
                written
            }
        }
        'g' | 'G' => {
            let written = shortest_form(size, directive.precision, directive.alternate);
            if spec == 'G' {
                written.to_uppercase()
            } else {
                written
            }
        }
        _ => {
            let written = hex_form(size, directive.precision, directive.alternate);
            if spec == 'A' {
                written.to_uppercase()
            } else {
                written
            }
        }
    };
    let body = format!("{}{}", sign, digits);
    if directive.zero
        && !directive.left_align
        && let Some(width) = directive.width
    {
        let counted = body.chars().count() as i64;
        if counted < width {
            let room: String = std::iter::repeat_n('0', (width - counted) as usize).collect();
            // A hexadecimal fraction pads after the `0x` that opens it.
            let opens = if digits.len() > 2 && digits[..2].eq_ignore_ascii_case("0x") {
                2
            } else {
                0
            };
            return format!("{}{}{}{}", sign, &digits[..opens], room, &digits[opens..]);
        }
    }
    pad(&body, directive, ' ')
}

/// A number written out in full, with as many places after the point as asked
/// for. A `#` flag keeps the point even where no places follow.
fn decimal_notation(size: f64, places: usize, alternate: bool) -> String {
    let written = format!("{:.*}", places, size);
    if places == 0 && alternate {
        return format!("{}.", written);
    }
    written
}

/// A number written as one digit, a point, and a power of ten.
fn exponent_form(size: f64, places: usize, alternate: bool) -> String {
    let mut power = if size == 0.0 {
        0i32
    } else {
        size.abs().log10().floor() as i32
    };
    let mut lead = if size == 0.0 {
        0.0
    } else {
        size / 10f64.powi(power)
    };
    // Rounding the lead can carry it up to ten, which belongs to the next
    // power instead.
    let rounded = format!("{:.*}", places, lead);
    if rounded.starts_with("10") {
        power += 1;
        lead = size / 10f64.powi(power);
    }
    let mut written = format!("{:.*}", places, lead);
    if places == 0 && alternate {
        written.push('.');
    }
    format!(
        "{}e{}{:02}",
        written,
        if power < 0 { '-' } else { '+' },
        power.abs()
    )
}

/// A number written in whichever of the two forms is shorter, which is what
/// `%g` asks for. Trailing zeros go unless a `#` flag keeps them.
fn shortest_form(size: f64, precision: Option<i64>, alternate: bool) -> String {
    let places = precision.unwrap_or(6).max(1) as usize;
    let power = if size == 0.0 {
        0i32
    } else {
        // The power the number would be written with, after rounding it to
        // the digits asked for.
        let first = size.abs().log10().floor() as i32;
        let rounded = format!("{:.*e}", places - 1, size);
        rounded
            .rsplit('e')
            .next()
            .and_then(|held| held.parse::<i32>().ok())
            .unwrap_or(first)
    };
    if power < -4 || power >= places as i32 {
        let written = exponent_form(size, places - 1, alternate);
        if alternate {
            return written;
        }
        let (digits, power) = written.split_once('e').unwrap_or((written.as_str(), ""));
        return format!("{}e{}", trimmed_zeros(digits), power);
    }
    let room = (places as i32 - 1 - power).max(0) as usize;
    let written = format!("{:.*}", room, size);
    if alternate {
        if room == 0 {
            return format!("{}.", written);
        }
        return written;
    }
    trimmed_zeros(&written)
}

/// A written number with the zeros trailing its fraction taken off, and the
/// point with them where nothing is left after it.
fn trimmed_zeros(written: &str) -> String {
    if !written.contains('.') {
        return written.to_string();
    }
    let trimmed = written.trim_end_matches('0');
    trimmed.trim_end_matches('.').to_string()
}

/// A number written as a hexadecimal fraction and a power of two, which is
/// what `%a` asks for.
fn hex_form(size: f64, precision: Option<i64>, alternate: bool) -> String {
    if size == 0.0 {
        return match precision {
            Some(places) if places > 0 => {
                format!("0x0.{}p+0", "0".repeat(places as usize))
            }
            _ if alternate => "0x0.p+0".to_string(),
            _ => "0x0p+0".to_string(),
        };
    }
    let bits = size.to_bits();
    let raw_power = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = bits & 0x000f_ffff_ffff_ffff;
    let (lead, mantissa, power) = if raw_power == 0 {
        (0u64, mantissa, -1022)
    } else {
        (1u64, mantissa, raw_power - 1023)
    };
    // The mantissa holds thirteen hexadecimal digits.
    let mut digits: String = (0..13)
        .map(|place| {
            let shift = 48 - place * 4;
            let held = ((mantissa >> shift) & 0xf) as u32;
            std::char::from_digit(held, 16).expect("a hexadecimal digit")
        })
        .collect();
    if let Some(places) = precision {
        digits = rounded_hex(&digits, places.max(0) as usize);
    } else {
        digits = digits.trim_end_matches('0').to_string();
    }
    let mut written = format!("0x{}", lead);
    if !digits.is_empty() {
        written.push('.');
        written.push_str(&digits);
    } else if alternate {
        written.push('.');
    }
    format!(
        "{}p{}{}",
        written,
        if power < 0 { '-' } else { '+' },
        power.abs()
    )
}

/// Hexadecimal digits cut to the count asked for, with the last one rounded
/// to the nearest.
fn rounded_hex(digits: &str, places: usize) -> String {
    if places >= digits.len() {
        let mut written = digits.to_string();
        while written.len() < places {
            written.push('0');
        }
        return written;
    }
    let mut kept: Vec<u32> = digits[..places]
        .chars()
        .map(|held| held.to_digit(16).unwrap_or(0))
        .collect();
    let next = digits[places..]
        .chars()
        .next()
        .and_then(|held| held.to_digit(16))
        .unwrap_or(0);
    if next >= 8 {
        let mut at = kept.len();
        while at > 0 {
            at -= 1;
            if kept[at] == 15 {
                kept[at] = 0;
            } else {
                kept[at] += 1;
                break;
            }
        }
    }
    kept.into_iter()
        .map(|held| std::char::from_digit(held, 16).expect("a hexadecimal digit"))
        .collect()
}
