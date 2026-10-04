// Reading an encoding out of an argument, and deciding whether two
// encodings can be read together.

use super::*;

impl VirtualMachine {
    /// The encoding a setting was given. A name is looked up, and anything
    /// else already stands for an encoding.
    pub(crate) fn encoding_setting(
        &mut self,
        class_rc: &Rc<Class>,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // An Encoding stands for itself, and so does nil. Anything else is a
        // name, which an object of the program's own spells through `to_str`.
        let named = match value {
            Object::String(_) => value.clone(),
            Object::Nil | Object::Class(_) | Object::Module(_) => return Ok(value.clone()),
            other if self.responds_to(other, "to_str") => {
                Object::string(self.coerce_name_argument(other, position)?)
            }
            other => {
                let message = format!(
                    "no implicit conversion of {} into String",
                    self.builtins().class_of(other).name()
                );
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ));
            }
        };
        let found =
            self.call_class_methods(class_rc, "find", std::slice::from_ref(&named), position)?;
        Ok(found.unwrap_or(named))
    }
}

/// A value of the class named, for asking whether that class answers a
/// method natively.
pub(crate) fn sample_of_class(named: &str) -> Option<Object> {
    match named {
        "Integer" => Some(Object::Int(0)),
        "Float" => Some(Object::Float(0.0)),
        "String" => Some(Object::string("")),
        "Symbol" => Some(Object::symbol("held")),
        "Array" => Some(Object::array(Vec::new())),
        "Hash" => Some(Object::Dict(Rc::new(std::cell::RefCell::new(
            indexmap::IndexMap::new(),
        )))),
        "NilClass" => Some(Object::Nil),
        "TrueClass" => Some(Object::Bool(true)),
        "FalseClass" => Some(Object::Bool(false)),
        _ => None,
    }
}

impl VirtualMachine {
    /// The encoding `Encoding.compatible?` answers for two objects, following
    /// Ruby's negotiation: an encoding both can be read in, or None.
    pub(crate) fn compatible_encoding(
        &mut self,
        first: &Object,
        second: &Object,
    ) -> Option<String> {
        let first_encoding = self.encoding_name_of(first)?;
        let second_encoding = self.encoding_name_of(second)?;
        if first_encoding == second_encoding {
            return Some(first_encoding);
        }
        let first_text = as_encoded_text(first);
        let second_text = as_encoded_text(second);
        let first_is_string = matches!(first, Object::String(_));
        let second_is_string = matches!(second, Object::String(_));
        if let Some(text) = &second_text
            && second_is_string
            && text.as_str().is_empty()
        {
            return Some(first_encoding);
        }
        if first_is_string
            && second_is_string
            && first_text
                .as_ref()
                .is_some_and(|text| text.as_str().is_empty())
        {
            if encoding_reads_alongside_ascii(&first_encoding) && holds_only_ascii(second) {
                return Some(first_encoding);
            }
            return Some(second_encoding);
        }
        if !encoding_reads_alongside_ascii(&first_encoding)
            || !encoding_reads_alongside_ascii(&second_encoding)
        {
            return None;
        }
        // An object whose encoding follows what it holds, rather than a
        // string, settles the answer as soon as it is plain ASCII.
        if !second_is_string && second_encoding == "US-ASCII" {
            return Some(first_encoding);
        }
        if !first_is_string && first_encoding == "US-ASCII" {
            return Some(second_encoding);
        }
        let (left, right, left_encoding, right_encoding, right_is_string) = if first_is_string {
            (
                first,
                second,
                first_encoding,
                second_encoding,
                second_is_string,
            )
        } else {
            (second, first, second_encoding, first_encoding, false)
        };
        if !matches!(left, Object::String(_)) {
            return None;
        }
        let left_ascii = holds_only_ascii(left);
        if right_is_string {
            let right_ascii = holds_only_ascii(right);
            if left_ascii != right_ascii {
                if left_ascii {
                    return Some(right_encoding);
                }
                return Some(left_encoding);
            }
            if right_ascii {
                return Some(left_encoding);
            }
        }
        if left_ascii {
            return Some(right_encoding);
        }
        None
    }

    /// The encoding an object reports, or None for one that carries none.
    pub(crate) fn encoding_name_of(&mut self, value: &Object) -> Option<String> {
        match value {
            Object::String(text) => Some(text.encoding_name()),
            // A symbol named in ASCII is written in ASCII, whatever the
            // source naming it was written in.
            Object::Symbol(text) => Some(if text.as_str().is_ascii() {
                "US-ASCII".to_string()
            } else {
                text.encoding_name()
            }),
            Object::Regex(pattern, flags) => Some(self.pattern_encoding_name(pattern, flags)),
            Object::Class(class_rc)
                if class_rc
                    .superclass()
                    .is_some_and(|parent| parent.name() == "Encoding") =>
            {
                Some(class_rc.name().to_string())
            }
            _ => None,
        }
    }

    /// The encoding a pattern is read in: the modifier it was written with,
    /// else the encoding of the string it was built from, else the encoding a
    /// literal is read in.
    pub(crate) fn pattern_encoding_name(&self, pattern: &Rc<String>, flags: &str) -> String {
        if let Some((held, named)) = self
            .forced_pattern_encodings
            .get(&(Rc::as_ptr(pattern) as usize))
            && held
                .upgrade()
                .is_some_and(|alive| Rc::ptr_eq(&alive, pattern))
        {
            return named.clone();
        }
        // A pattern built to match in one encoding, or from text in an
        // encoding that spells ASCII another way, keeps the one it was built
        // in.
        let recorded = self.recorded_pattern_encoding(pattern);
        if let Some(named) = &recorded
            && (flags.contains('u') || !encoding_reads_alongside_ascii(named))
        {
            return named.clone();
        }
        if flags.contains('u') {
            return "UTF-8".to_string();
        }
        if flags.contains('s') {
            return "Windows-31J".to_string();
        }
        if flags.contains('e') {
            return "EUC-JP".to_string();
        }
        let beyond_ascii = pattern_reaches_beyond_ascii(pattern);
        if flags.contains('n') {
            return if beyond_ascii {
                "ASCII-8BIT"
            } else {
                "US-ASCII"
            }
            .to_string();
        }
        if !beyond_ascii {
            return "US-ASCII".to_string();
        }
        recorded.unwrap_or_else(|| "UTF-8".to_string())
    }
}

impl VirtualMachine {
    /// Record the encoding of the text a pattern was built from.
    pub(crate) fn record_pattern_encoding(&mut self, pattern: &Rc<String>, named: String) {
        self.pattern_encodings.insert(
            Rc::as_ptr(pattern) as usize,
            (Rc::downgrade(pattern), named),
        );
    }

    /// Make a pattern report `named` as its encoding, whatever it holds.
    pub(crate) fn force_pattern_encoding(&mut self, pattern: &Rc<String>, named: String) {
        self.forced_pattern_encodings.insert(
            Rc::as_ptr(pattern) as usize,
            (Rc::downgrade(pattern), named),
        );
    }

    /// Record a pattern as built by `Regexp.new` rather than written.
    pub(crate) fn record_built_pattern(&mut self, pattern: &Rc<String>) {
        self.built_patterns
            .insert(Rc::as_ptr(pattern) as usize, Rc::downgrade(pattern));
    }

    /// Whether a pattern was built by `Regexp.new`, if it is the pattern the
    /// entry was made for rather than one made later at the same address.
    pub(crate) fn pattern_was_built(&self, pattern: &Rc<String>) -> bool {
        self.built_patterns
            .get(&(Rc::as_ptr(pattern) as usize))
            .and_then(std::rc::Weak::upgrade)
            .is_some_and(|held| Rc::ptr_eq(&held, pattern))
    }

    /// The encoding recorded for a pattern, if it is the pattern the entry
    /// was made for rather than one built later at the same address.
    pub(crate) fn recorded_pattern_encoding(&self, pattern: &Rc<String>) -> Option<String> {
        let (held, named) = self
            .pattern_encodings
            .get(&(Rc::as_ptr(pattern) as usize))?;
        let alive = held.upgrade()?;
        Rc::ptr_eq(&alive, pattern).then(|| named.clone())
    }

    /// Whether a pattern matches in one encoding whatever the text it is
    /// matched against is tagged with. A pattern written with an encoding
    /// after it does, and so does one in any encoding but US-ASCII.
    pub(crate) fn pattern_fixes_encoding(&self, pattern: &Rc<String>, flags: &str) -> bool {
        let named = flags.contains('u') || flags.contains('e') || flags.contains('s');
        // A `\u` escape names a character outside ASCII whether or not the
        // pattern spells it out.
        let spelled = !pattern.is_ascii() || pattern.contains("\\u");
        named || spelled || self.pattern_encoding_name(pattern, flags) != "US-ASCII"
    }
}

/// The text behind a String or a Symbol, which is what an encoding
/// negotiation reads.
fn as_encoded_text(value: &Object) -> Option<Rc<crate::object::StringValue>> {
    match value {
        Object::String(text) | Object::Symbol(text) => Some(Rc::clone(text)),
        _ => None,
    }
}

/// Whether an encoding lays ASCII out one byte to a character, which is what
/// lets text in it be read alongside text in another such encoding.
pub(crate) fn encoding_reads_alongside_ascii(named: &str) -> bool {
    !super::encoding_settings::is_dummy_encoding(named)
        && !named.starts_with("UTF-16")
        && !named.starts_with("UTF-32")
}

/// Whether everything an object holds is plain ASCII, which is what makes it
/// readable in any encoding that lays ASCII out the same way.
fn holds_only_ascii(value: &Object) -> bool {
    let Some(text) = as_encoded_text(value) else {
        return false;
    };
    if !encoding_reads_alongside_ascii(&text.encoding_name()) {
        return false;
    }
    crate::vm::native_methods::string_methods::binary_bytes(&text)
        .iter()
        .all(|byte| byte.is_ascii())
}
