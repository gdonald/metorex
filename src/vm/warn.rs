// Kernel#warn.
//
// The message is assembled here, then handed to `Warning.warn`, which the
// prelude defines in Ruby so a program can replace it. Whether the category
// travels along as a keyword depends on the arity of the `Warning.warn` in
// force, matching MRI.

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

/// The keyword names `warn` accepts. A trailing Hash counts as keywords only
/// when every key is one of these, so `warn({a: 1})` still prints the Hash.
const WARN_KEYWORDS: &[&str] = &["uplevel", "category"];

impl VirtualMachine {
    /// `Kernel#warn(*messages, uplevel: nil, category: nil)`.
    pub(crate) fn kernel_warn(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (messages, keywords) = split_warn_keywords(arguments);

        let category = match keyword_value(&keywords, "category") {
            Some(Object::Nil) | None => Object::Nil,
            Some(value) => self.coerce_warn_category(&value, position)?,
        };
        let prefix = match keyword_value(&keywords, "uplevel") {
            Some(Object::Nil) | None => String::new(),
            Some(value) => {
                let level = self.coerce_uplevel(&value, position)?;
                self.warning_prefix(level, position)
            }
        };

        // `$VERBOSE` set to nil silences the warning before anything is built.
        if messages.is_empty() || matches!(self.globals().get("VERBOSE"), None | Some(Object::Nil))
        {
            return Ok(Object::Nil);
        }

        let mut text = String::new();
        for message in &messages {
            for line in flatten_warn_message(message) {
                let rendered = self.string_for_warning(&line, position)?;
                text.push_str(&rendered);
                if !rendered.ends_with('\n') {
                    text.push('\n');
                }
            }
        }
        let text = format!("{}{}", prefix, text);

        self.dispatch_to_warning_module(text, category, position)?;
        Ok(Object::Nil)
    }

    /// `category:` is either nil or something that answers `to_sym`.
    fn coerce_warn_category(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Object::Symbol(name) = value {
            return Ok(Object::Symbol(Rc::clone(name)));
        }
        if let Object::String(text) = value {
            return Ok(Object::Symbol(Rc::new((**text).clone())));
        }
        if self.responds_to(value, "to_sym") {
            let converted = self.send_to_object(value.clone(), "to_sym", vec![], position)?;
            if let Object::Symbol(_) = converted {
                return Ok(converted);
            }
        }
        Err(type_error(
            format!(
                "no implicit conversion of {} into Symbol",
                value.type_name()
            ),
            position,
        ))
    }

    /// `uplevel:` counts stack frames, so it has to be a non-negative Integer
    /// or something that converts to one.
    fn coerce_uplevel(&mut self, value: &Object, position: Position) -> Result<i64, MetorexError> {
        let level = match value {
            Object::Int(number) => *number,
            Object::Float(number) => *number as i64,
            // These answer `to_i` but not `to_int`, so Ruby refuses them.
            Object::String(_) | Object::Array(_) | Object::Dict(_) | Object::Nil => {
                return Err(type_error(
                    format!(
                        "no implicit conversion of {} into Integer",
                        value.type_name()
                    ),
                    position,
                ));
            }
            other => match self.integer_conversion_of(other, position)? {
                Some(number) => number,
                None => {
                    return Err(type_error(
                        format!(
                            "no implicit conversion of {} into Integer",
                            other.type_name()
                        ),
                        position,
                    ));
                }
            },
        };
        if level < 0 {
            let message = format!("negative level ({})", level);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        Ok(level)
    }

    /// `to_int`, then `to_i`, answering None when the object has neither.
    /// Both are tried by sending rather than by asking `respond_to?`, which
    /// does not report the conversions a class implements natively.
    fn integer_conversion_of(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<i64>, MetorexError> {
        for name in ["to_int", "to_i"] {
            match self.send_to_object(value.clone(), name, vec![], position) {
                Ok(Object::Int(number)) => return Ok(Some(number)),
                Ok(Object::Float(number)) => return Ok(Some(number as i64)),
                Ok(_) => continue,
                Err(MetorexError::UncaughtException { exception, .. })
                    if is_no_method_error(&exception) =>
                {
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    /// The `path:line: warning: ` a message with `uplevel:` carries. Level 0
    /// names the line the `warn` call itself sits on, level 1 the line that
    /// called that method, and so on outward. A level past the top of the
    /// stack leaves only the `warning: ` part.
    fn warning_prefix(&self, level: i64, position: Position) -> String {
        let stack = self.call_stack();
        let (line, path) = if level == 0 {
            (Some(position.line), self.current_source_file.clone())
        } else {
            match stack
                .len()
                .checked_sub(level as usize)
                .map(|index| &stack[index])
            {
                Some(frame) => {
                    let line = frame.location().and_then(|location| {
                        location
                            .rsplit(':')
                            .nth(1)
                            .and_then(|line| line.parse::<usize>().ok())
                    });
                    (line, frame.source_file().map(|file| file.to_string()))
                }
                None => (None, None),
            }
        };
        let path = path.or_else(|| {
            self.current_file
                .as_ref()
                .map(|file| file.display().to_string())
        });
        match (line, path) {
            (Some(line), Some(path)) => format!("{}:{}: warning: ", path, line),
            (Some(line), None) => format!("{}: warning: ", line),
            (None, _) => "warning: ".to_string(),
        }
    }

    /// Hand the assembled text to `Warning.warn`. MRI passes `category:` only
    /// when the method in force takes more than the one message argument, so
    /// a replacement written as `def warn(message)` still works.
    fn dispatch_to_warning_module(
        &mut self,
        text: String,
        category: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(warning) = self.globals().get("Warning") else {
            self.write_to_stderr(&text, position)?;
            return Ok(Object::Nil);
        };
        let mut arguments = vec![Object::string(text.clone())];
        if warning_warn_takes_keywords(&warning) {
            let mut keywords = indexmap::IndexMap::new();
            keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
            keywords.insert(":category".to_string(), category);
            arguments.push(Object::Dict(Rc::new(RefCell::new(keywords))));
        }
        self.send_to_object(warning, "warn", arguments, position)
    }

    /// `to_s` for one warning line, honoring a user-defined `to_s`.
    fn string_for_warning(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match value {
            Object::Nil => Ok(String::new()),
            other => {
                let rendered = self.send_to_object(other.clone(), "to_s", vec![], position)?;
                match rendered {
                    Object::String(text) => Ok((*text).clone()),
                    other => Ok(other.to_string()),
                }
            }
        }
    }
}

/// Whether the `Warning.warn` in force accepts anything past the message. A
/// method taking exactly one required positional argument does not.
fn warning_warn_takes_keywords(warning: &Object) -> bool {
    let (Object::Module(class) | Object::Class(class)) = warning else {
        return false;
    };
    let Some(method) = class
        .singleton_class_slot()
        .as_ref()
        .and_then(|singleton| singleton.find_method("warn"))
        .or_else(|| class.find_method("__class__warn"))
    else {
        return true;
    };
    let only_the_message = method.parameters.len() == 1
        && method.variadic_param.is_none()
        && method.keyword_parameters.is_empty()
        && method.keyword_rest_parameter.is_none();
    !only_the_message
}

/// Peel a trailing keyword Hash off the argument list.
fn split_warn_keywords(mut arguments: Vec<Object>) -> (Vec<Object>, Option<Object>) {
    let is_keywords = matches!(arguments.last(), Some(Object::Dict(entries))
    if !entries.borrow().is_empty()
        && entries
            .borrow()
            .keys()
            .filter(|key| key.as_str() != "__MX_KWARGS__")
            .all(|key| match key.strip_prefix(':') {
                Some(name) => WARN_KEYWORDS.contains(&name),
                None => false,
            }));
    if is_keywords {
        let keywords = arguments.pop();
        (arguments, keywords)
    } else {
        (arguments, None)
    }
}

/// One keyword's value, when the call supplied it.
fn keyword_value(keywords: &Option<Object>, name: &str) -> Option<Object> {
    let Some(Object::Dict(entries)) = keywords else {
        return None;
    };
    entries.borrow().get(&format!(":{}", name)).cloned()
}

/// An Array argument warns with each element on its own line.
fn flatten_warn_message(message: &Object) -> Vec<Object> {
    match message {
        Object::Array(elements) => elements.borrow().clone(),
        other => vec![other.clone()],
    }
}

/// A TypeError carrying `message`.
fn type_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("TypeError", message.clone()),
        location: crate::vm::utils::position_to_location(position),
        message,
    }
}

/// Whether an exception object is a NoMethodError or NameError.
fn is_no_method_error(exception: &Object) -> bool {
    match exception {
        Object::Exception(details) => {
            matches!(
                details.borrow().exception_type.as_str(),
                "NoMethodError" | "NameError"
            )
        }
        _ => false,
    }
}
