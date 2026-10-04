// Turning a value into the String or status a function needs.

use super::*;

impl VirtualMachine {
    /// Coerce `abort`'s argument to a String the way Ruby does: a String is
    /// taken as is, anything else must answer `to_str`, and a receiver without
    /// one raises TypeError.
    /// Coerce a `load` / `require` argument to a path: a String stands, and
    /// anything else goes through `to_path` and then `to_str`, each of which
    /// must answer a String.
    pub(crate) fn coerce_load_path(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(path) = argument {
            return Ok(path.as_str().to_string());
        }
        let refuse = |vm: &mut Self, value: &Object| {
            let message = format!(
                "no implicit conversion of {} into String",
                vm.conversion_name(value)
            );
            MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            }
        };
        let mut value = argument.clone();
        if self.responds_to(&value, "to_path") {
            value = self.invoke_named_conversion(&value, "to_path", position)?;
            if let Object::String(path) = &value {
                return Ok(path.as_str().to_string());
            }
        }
        if !self.responds_to(&value, "to_str") {
            return Err(refuse(self, &value));
        }
        let converted = self.invoke_named_conversion(&value, "to_str", position)?;
        match converted {
            Object::String(path) => Ok(path.as_str().to_string()),
            other => Err(refuse(self, &other)),
        }
    }

    /// Call a conversion method by name on a value.
    fn invoke_named_conversion(
        &mut self,
        value: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some((class, method)) = self.lookup_method(value, method_name) else {
            return Ok(Object::Nil);
        };
        self.invoke_method(class, method, value.clone(), Vec::new(), position)
    }

    /// The status `exit` was asked for: an Integer as it stands, true or
    /// false as 0 or 1, a Float truncated, and anything else through `to_int`.
    pub(crate) fn exit_status_argument(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let Some(argument) = argument else {
            return Ok(0);
        };
        match argument {
            Object::Int(status) => Ok(*status),
            Object::Bool(success) => Ok(if *success { 0 } else { 1 }),
            Object::Float(status) => Ok(status.trunc() as i64),
            other => {
                let source = self.builtins().class_of(other).name().to_string();
                let refuse = |message: String| MetorexError::UncaughtException {
                    exception: Object::exception("TypeError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                };
                let Some((class, method)) = self.lookup_method(other, "to_int") else {
                    return Err(refuse(if matches!(other, Object::Nil) {
                        "no implicit conversion from nil to integer".to_string()
                    } else {
                        format!("no implicit conversion of {} into Integer", source)
                    }));
                };
                let converted =
                    self.invoke_method(class, method, other.clone(), Vec::new(), position)?;
                match converted {
                    Object::Int(status) => Ok(status),
                    produced => Err(refuse(format!(
                        "can't convert {} to Integer ({}#to_int gives {})",
                        source,
                        source,
                        self.builtins().class_of(&produced).name()
                    ))),
                }
            }
        }
    }

    /// Coerce the argument to `` ` `` to a String: one is taken as is, and
    /// anything else has to answer `to_str`.
    pub(crate) fn coerce_command_argument(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = argument {
            return Ok(text.as_str().to_string());
        }
        let source = self.builtins().class_of(argument).name().to_string();
        let Some((class, method)) = self.lookup_method(argument, "to_str") else {
            let message = format!("no implicit conversion of {} into String", source);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let converted =
            self.invoke_method(class, method, argument.clone(), Vec::new(), position)?;
        match converted {
            Object::String(text) => Ok(text.as_str().to_string()),
            other => {
                let produced = self.builtins().class_of(&other).name().to_string();
                let message = format!(
                    "can't convert {} to String ({}#to_str gives {})",
                    source, source, produced
                );
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("TypeError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
        }
    }

    /// Get the string representation of an object by calling to_s or inspect if available.
    pub(crate) fn get_string_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        self.refuse_moved_conversion(obj, "to_s", position)?;
        // First try to_s, then inspect, then fall back to Display
        match obj {
            // `:name.to_s` is the bare name; only `inspect` keeps the colon.
            Object::Symbol(name) => Ok(name.as_str().to_string()),
            Object::Instance(_) => {
                // Try to_s first
                if let Some((class, method)) = self.lookup_method(obj, "to_s") {
                    let result =
                        self.invoke_method(class, method, obj.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.to_string());
                    }
                }
                // Try inspect as fallback
                if let Some((class, method)) = self.lookup_method(obj, "inspect") {
                    let result =
                        self.invoke_method(class, method, obj.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.to_string());
                    }
                }
                // Classes whose `to_s` is native, such as Rational, have no
                // entry in any method map for the lookups above to find.
                let class = self.builtins().class_of(obj);
                if let Some(Object::String(rendered)) =
                    self.call_native_method(&class, obj, "to_s", &[], position)?
                {
                    return Ok(rendered.to_string());
                }
                // Fall back to default Display
                Ok(format!("{}", obj))
            }
            _ => Ok(format!("{}", obj)),
        }
    }

    /// The string `inspect` produces for an object, tagged with the encoding
    /// it is written in. Text that encoding has no room for is escaped rather
    /// than refused.
    pub(crate) fn inspected_object(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let rendered = self.inspect_object(obj, position)?;
        let Object::String(text) = &rendered else {
            return Ok(rendered);
        };
        let writing = self.inspect_result_encoding();
        let held = text.encoding_name();
        // Text in an encoding that spells ASCII its own way is never read as
        // ASCII, however small the numbers its bytes happen to be.
        let reads_as_ascii =
            crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(&held)
                && text.as_str().is_ascii();
        if held == writing || reads_as_ascii {
            return Ok(rendered);
        }
        Ok(crate::vm::native_methods::string_methods::escaped_text(
            text,
        ))
    }

    /// The string `inspect` produces for an object, preferring a method the
    /// object defines over the native rendering.
    pub(crate) fn get_inspect_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        self.refuse_moved_conversion(obj, "inspect", position)?;
        let rendered = self.inspect_object(obj, position)?;
        Ok(match &rendered {
            Object::String(text) => text.to_string(),
            other => format!("{}", other),
        })
    }

    /// What `inspect` answered for an object, as the object it answered. A
    /// method the object defines is preferred over the native rendering.
    fn inspect_object(&mut self, obj: &Object, position: Position) -> Result<Object, MetorexError> {
        if let Some((class, method)) = self.lookup_method(obj, "inspect")
            && !method.is_undefined
        {
            let result = self.invoke_method(class, method, obj.clone(), vec![], position)?;
            if matches!(result, Object::String(_)) {
                return Ok(result);
            }
            // Ruby asks whatever `inspect` answered for its own `to_s`, and
            // shows that object's default form when `to_s` is no String
            // either. Neither one is asked for `to_str`.
            if let Some((class, method)) = self.lookup_method(&result, "to_s")
                && !method.is_undefined
            {
                let spelled =
                    self.invoke_method(class, method, result.clone(), vec![], position)?;
                if matches!(spelled, Object::String(_)) {
                    return Ok(spelled);
                }
                return Ok(Object::string(format!("{}", spelled)));
            }
            return Ok(Object::string(format!("{}", result)));
        }
        let class = self.builtins().class_of(obj);
        if let Some(rendered @ Object::String(_)) =
            self.call_native_method(&class, obj, "inspect", &[], position)?
        {
            return Ok(rendered);
        }
        // The per-class tables answer for the classes they know. Everything
        // else falls to Object's own, which is where an instance of a class
        // the program wrote is written out with the variables it holds.
        if let Some(rendered @ Object::String(_)) =
            self.call_object_method(obj, "inspect", &[], position)?
        {
            return Ok(rendered);
        }
        Ok(Object::string(format!("{}", obj)))
    }

    pub(crate) fn coerce_abort_message(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = argument {
            return Ok(text.as_str().to_string());
        }
        if let Some((class, method)) = self.lookup_method(argument, "to_str")
            && !method.is_undefined
        {
            let converted =
                self.invoke_method(class, method, argument.clone(), vec![], position)?;
            if let Object::String(text) = converted {
                return Ok(text.as_str().to_string());
            }
        }
        let source_class = self.builtins().class_of(argument).name().to_string();
        let message = format!("no implicit conversion of {} into String", source_class);
        Err(MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }
}
