//! `SystemCallError.new` and the `Errno::EXXX` classes it hands back.

use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::native_methods::rational_methods::complex_parts;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

/// Where a SystemCallError keeps the number it was built with, which `#errno`
/// answers. Not an `@` name, so a program's own instance variables cannot
/// collide with it.
pub(crate) const ERRNO_VALUE_KEY: &str = "__errno_value__";

impl VirtualMachine {
    /// `SystemCallError.new(message = nil, errno = nil, location = nil)`, and
    /// `Errno::EXXX.new(message = nil, location = nil)` where the class names
    /// the number itself. The number decides which class comes back and what
    /// the message reads, with a custom message and location folded in.
    pub(crate) fn build_system_call_error(
        &mut self,
        class: &Rc<Class>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // An Errno class already names its number, so its arguments are the
        // custom message and the location.
        if let Some(number) = Self::declared_errno(class) {
            let custom = self.optional_message(arguments.first(), position)?;
            let location = self.optional_location(arguments.get(1), position)?;
            let default = Self::errno_message(class, number);
            return Ok(self.errno_exception(Rc::clone(class), number, default, custom, location));
        }

        // SystemCallError itself takes the number as an argument, in the
        // second position unless it is the only argument.
        if arguments.is_empty() {
            let message = "wrong number of arguments (given 0, expected 1..3)".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        // A lone number is the errno rather than the message.
        let numeric_only_argument = arguments.len() == 1
            && (matches!(arguments[0], Object::Int(_) | Object::Float(_))
                || complex_parts(&arguments[0]).is_some());
        let errno_argument = match numeric_only_argument {
            true => Some(&arguments[0]),
            false => arguments.get(1),
        };
        let custom = match numeric_only_argument {
            true => None,
            false => self.optional_message(arguments.first(), position)?,
        };
        let location = self.optional_location(arguments.get(2), position)?;
        let Some(errno_argument) = errno_argument else {
            // No number at all: the exception carries the message it was
            // given and nothing else.
            let message = custom.unwrap_or_else(|| "unknown error".to_string());
            let exception = Object::exception(class.name(), message);
            if let Object::Exception(details) = &exception {
                details.borrow_mut().class = Some(Rc::clone(class));
            }
            return Ok(exception);
        };
        let number = self.coerce_errno(errno_argument, position)?;
        let target = self.errno_class_for_number(number).unwrap_or_else(|| {
            match self.globals().get("SystemCallError") {
                Some(Object::Class(system_call_error)) => system_call_error,
                _ => Rc::clone(class),
            }
        });
        let default = Self::errno_message(&target, number);
        Ok(self.errno_exception(target, number, default, custom, location))
    }

    /// Build the exception, recording the number `#errno` answers.
    fn errno_exception(
        &self,
        class: Rc<Class>,
        number: i64,
        default: String,
        custom: Option<String>,
        location: Option<String>,
    ) -> Object {
        let message = match (custom, location) {
            (None, _) => default,
            (Some(custom), None) => format!("{} - {}", default, custom),
            (Some(custom), Some(location)) => format!("{} @ {} - {}", default, location, custom),
        };
        let exception = Object::exception(class.name(), message);
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.class = Some(class);
            details
                .instance_vars
                .insert(ERRNO_VALUE_KEY.to_string(), Object::Int(number));
        }
        exception
    }

    /// The number an Errno class stands for, inherited by a subclass of one.
    /// None for SystemCallError itself and for a subclass naming no number.
    pub(crate) fn declared_errno(class: &Rc<Class>) -> Option<i64> {
        let mut cursor = Some(Rc::clone(class));
        while let Some(current) = cursor {
            if let Some(Object::Int(number)) = current.get_class_var("Errno") {
                return Some(number);
            }
            cursor = current.superclass();
        }
        None
    }

    /// What the number reads as: the text its Errno class stands for, or an
    /// unknown-error line for a number no class names.
    fn errno_message(class: &Rc<Class>, number: i64) -> String {
        let mut cursor = Some(Rc::clone(class));
        while let Some(current) = cursor {
            if let Some(Object::String(message)) =
                current.get_class_var(crate::vm::init::ERRNO_MESSAGE_KEY)
            {
                return message.as_str().to_string();
            }
            cursor = current.superclass();
        }
        match i32::try_from(number) {
            Ok(number) => crate::vm::init::errno_description(number),
            Err(_) => format!("Unknown error: {}", number),
        }
    }

    /// The `Errno::EXXX` class carrying `number`, when one does.
    fn errno_class_for_number(&self, number: i64) -> Option<Rc<Class>> {
        let Some(Object::Module(errno) | Object::Class(errno)) = self.globals().get("Errno") else {
            return None;
        };
        errno
            .class_var_names()
            .into_iter()
            .find_map(|name| match errno.get_class_var(&name) {
                Some(Object::Class(class))
                    if class.get_class_var("Errno") == Some(Object::Int(number)) =>
                {
                    Some(class)
                }
                _ => None,
            })
    }

    /// A custom message argument: a String as it stands, nil as if it were
    /// not passed, and anything else through `to_str`.
    fn optional_message(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<Option<String>, MetorexError> {
        match argument {
            None | Some(Object::Nil) => Ok(None),
            Some(Object::String(text)) => Ok(Some(text.as_str().to_string())),
            Some(other) => {
                let source = self.builtins().class_of(other).name().to_string();
                let Some((class, method)) = self.lookup_method(other, "to_str") else {
                    let message = format!("no implicit conversion of {} into String", source);
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                };
                match self.invoke_method(class, method, other.clone(), Vec::new(), position)? {
                    Object::String(text) => Ok(Some(text.as_str().to_string())),
                    produced => {
                        let message = format!(
                            "can't convert {} to String ({}#to_str gives {})",
                            source,
                            source,
                            self.builtins().class_of(&produced).name()
                        );
                        Err(MetorexError::UncaughtException {
                            exception: Object::exception("TypeError", message.clone()),
                            location: position_to_location(position),
                            message,
                        })
                    }
                }
            }
        }
    }

    /// A location argument, which Ruby renders with `to_s`.
    fn optional_location(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<Option<String>, MetorexError> {
        match argument {
            None | Some(Object::Nil) => Ok(None),
            Some(Object::String(text)) => Ok(Some(text.as_str().to_string())),
            Some(other) => Ok(Some(self.coerce_name_argument(other, position)?)),
        }
    }

    /// The errno argument as a number: an Integer as it stands, a Float or a
    /// real Complex truncated, and anything else through `to_int`. A Complex
    /// with an imaginary part cannot become one at all.
    fn coerce_errno(&mut self, argument: &Object, position: Position) -> Result<i64, MetorexError> {
        let refuse = |class_name: &str, message: String| MetorexError::UncaughtException {
            exception: Object::exception(class_name, message.clone()),
            location: position_to_location(position),
            message,
        };
        match argument {
            Object::Int(number) => Ok(*number),
            Object::Float(number) => Ok(number.trunc() as i64),
            _ => {
                if let Some((real, imaginary)) = complex_parts(argument) {
                    let rendered = self.render_object(argument, position)?;
                    let real = match (&real, &imaginary) {
                        (Object::Int(real), Object::Int(0)) => Some(*real),
                        (Object::Float(real), Object::Int(0)) => Some(real.trunc() as i64),
                        _ => None,
                    };
                    return real.ok_or_else(|| {
                        refuse(
                            "RangeError",
                            format!("can't convert {} into Integer", rendered),
                        )
                    });
                }
                let source = self.builtins().class_of(argument).name().to_string();
                let Some((class, method)) = self.lookup_method(argument, "to_int") else {
                    return Err(refuse(
                        "TypeError",
                        format!("no implicit conversion of {} into Integer", source),
                    ));
                };
                match self.invoke_method(class, method, argument.clone(), Vec::new(), position)? {
                    Object::Int(number) => Ok(number),
                    produced => Err(refuse(
                        "TypeError",
                        format!(
                            "can't convert {} to Integer ({}#to_int gives {})",
                            source,
                            source,
                            self.builtins().class_of(&produced).name()
                        ),
                    )),
                }
            }
        }
    }

    /// How the object reads in an error message, which is its `to_s`.
    fn render_object(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match self.send_to_object(value.clone(), "to_s", vec![], position)? {
            Object::String(text) => Ok(text.as_str().to_string()),
            rendered => Ok(rendered.to_string()),
        }
    }
}
