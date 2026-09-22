// The values a format string reads, by name, by number, or in order.

use super::*;

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
    pub(crate) fn format_argument_list(
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
        let mut answer =
            crate::vm::native_methods::string_methods::AnswerText::new(&format_encoding);
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
    pub(crate) fn read_directive(
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
    pub(crate) fn starred_number(
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
    pub(crate) fn named_value(
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
    pub(crate) fn directive_value(
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
}

/// The flags, width and precision written between a `%` and the letter that
/// says what to write.
#[derive(Default)]
pub(crate) struct Directive {
    pub(crate) left_align: bool,
    pub(crate) plus: bool,
    pub(crate) space: bool,
    pub(crate) zero: bool,
    pub(crate) alternate: bool,
    pub(crate) width: Option<i64>,
    pub(crate) precision: Option<i64>,
    /// The `<name>` a directive was written with, which takes its value from
    /// the Hash rather than from the next argument.
    pub(crate) name: Option<String>,
    /// The `N$` a directive was written with, counting from one.
    pub(crate) numbered: Option<usize>,
}

/// Where the values a format string writes come from, and which ways of
/// naming them have been used so far. Ruby refuses a format that mixes them.
pub(crate) struct Arguments {
    pub(crate) values: Vec<Object>,
    pub(crate) keywords: Option<Object>,
    pub(crate) next: usize,
    pub(crate) took_in_order: bool,
    pub(crate) took_by_number: bool,
    pub(crate) took_by_name: bool,
}

/// Ruby's ArgumentError for a format string that asks for something it was
/// not given.
pub(crate) fn argument_error(message: &str, position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", message, position)
}

/// Ruby's TypeError for a value a directive cannot read.
pub(crate) fn type_error(message: String, position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("TypeError", &message, position)
}

impl Arguments {
    /// The next value in order, which a format may not ask for once it has
    /// asked for one by number.
    pub(crate) fn in_order(&mut self, position: Position) -> Result<Object, MetorexError> {
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
    pub(crate) fn by_number(
        &mut self,
        at: usize,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
