// Reading and checking the arguments a class method was given.

use super::*;

impl VirtualMachine {
    /// The elements `Array.new` builds from its arguments and block, which is
    /// also what a subclass of Array starts out holding.
    pub(crate) fn build_array_elements(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut elements = Vec::new();
        self.collect_array_elements(arguments, position, &mut elements)?;
        Ok(elements)
    }

    /// The same, filling `elements` as it goes so a `break` out of the block
    /// leaves behind what it had already built.
    pub(crate) fn collect_array_elements(
        &mut self,
        arguments: &[Object],
        position: Position,
        elements: &mut Vec<Object>,
    ) -> Result<(), MetorexError> {
        if arguments.len() > 2 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(0, 2),
                arguments.len(),
                position,
            ));
        }
        // `Array.new { ... }` names no size, so the block has nothing to
        // fill and Ruby says it went unused.
        if arguments.is_empty() && matches!(self.pending_block, Some(Object::Block(_))) {
            self.pending_block = None;
            let file = self
                .current_source_file
                .clone()
                .unwrap_or_else(|| "-".to_string());
            let message = format!(
                "{}:{}: warning: given block not used\n",
                file, position.line
            );
            self.warn_through_warning_module(message, position)?;
        }
        // A lone array names the elements rather than a size, and anything
        // that reads as one through `to_ary` names them the same way. A
        // default alongside it has nothing to fill.
        let read_as_array = match arguments.first() {
            Some(held @ Object::Array(_)) => Some(held.clone()),
            Some(held) if crate::vm::native_methods::array_subclass_value(held).is_some() => {
                crate::vm::native_methods::array_subclass_value(held)
            }
            Some(held)
                if !matches!(held, Object::Int(_) | Object::Nil)
                    && self.responds_to(held, "to_ary") =>
            {
                Some(self.send_to_object(held.clone(), "to_ary", vec![], position)?)
            }
            _ => None,
        };
        if let Some(Object::Array(held)) = read_as_array {
            if arguments.len() > 1 {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "no implicit conversion of Array into Integer",
                    position,
                ));
            }
            self.pending_block.take();
            elements.extend(held.borrow().iter().cloned());
            return Ok(());
        }
        let size = match arguments.first() {
            None => 0_i64,
            Some(Object::Int(n)) => *n,
            // A count past the widest array Ruby builds is refused rather
            // than attempted.
            Some(Object::BigInt(_)) => return Err(array_size_too_big(position)),
            Some(Object::Nil) => {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "no implicit conversion from nil to integer",
                    position,
                ));
            }
            Some(other) if self.responds_to(other, "to_int") => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(held) => held,
                    _ => {
                        return Err(method_argument_type_error(
                            "Array.new",
                            "Integer",
                            other,
                            position,
                        ));
                    }
                }
            }
            Some(other) => {
                return Err(method_argument_type_error(
                    "Array.new",
                    "Integer",
                    other,
                    position,
                ));
            }
        };
        if size < 0 {
            let message = "negative array size".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        if size > WIDEST_ARRAY {
            return Err(array_size_too_big(position));
        }
        let block = self.pending_block.take();
        // A block fills every place, so a default named alongside it has
        // nothing left to fill and Ruby says so.
        if matches!(block, Some(Object::Block(_))) && arguments.len() > 1 {
            let file = self
                .current_source_file
                .clone()
                .unwrap_or_else(|| "-".to_string());
            let message = format!(
                "{}:{}: warning: block supersedes default value argument\n",
                file, position.line
            );
            self.warn_through_warning_module(message, position)?;
        }
        if let Some(Object::Block(body)) = block {
            // The block may take 0 args (`Array.new(10) { rand }`) or 1
            // (`Array.new(10) { |i| ... }`). Match metorex's strict arity
            // check by only passing the index when the block declares a
            // positional parameter for it.
            let pass_index = body
                .parameters
                .iter()
                .any(|name| !name.starts_with('&') && !name.starts_with('*'));
            for index in 0..size {
                let block_arguments = if pass_index {
                    vec![Object::Int(index)]
                } else {
                    vec![]
                };
                let value = self.execute_block_callable(&body, block_arguments, position)?;
                elements.push(value);
            }
        } else {
            let default = arguments.get(1).cloned().unwrap_or(Object::Nil);
            for _ in 0..size {
                elements.push(default.clone());
            }
        }
        Ok(())
    }

    /// Coerce a name argument to a String: Strings and Symbols are used
    /// directly, anything else goes through `to_str`. Raises TypeError when
    /// that conversion is missing or returns a non-String.
    pub(crate) fn coerce_name_argument(
        &mut self,
        arg: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match arg {
            Object::Symbol(s) => Ok(s.as_str().to_string()),
            Object::String(s) => Ok(crate::vm::native_methods::name_text(s)),
            other => {
                let other_obj = other.clone();
                let source_class = self.builtins().class_of(other).name().to_string();
                let converted =
                    if let Some((cls, method)) = self.lookup_method(&other_obj, "to_str") {
                        self.invoke_method(cls, method, other_obj, Vec::new(), position)?
                    } else {
                        let msg = format!("no implicit conversion of {} into String", source_class);
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    };
                match converted {
                    Object::String(s) => Ok(s.as_str().to_string()),
                    other => {
                        let converted_class = self.builtins().class_of(&other).name().to_string();
                        let msg = format!("can't convert {} to String", converted_class);
                        let exc = Object::exception("TypeError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                }
            }
        }
    }

    /// Coerce a class-variable name argument to its storage key (the name with
    /// the leading `@@` removed). Strings and Symbols are used directly; any
    /// other object is converted via `to_str`. Raises TypeError when that
    /// conversion is missing or returns a non-String, and NameError when the
    /// resulting name is not a valid class variable name.
    pub(crate) fn coerce_class_variable_name(
        &mut self,
        arg: &Object,
        receiver: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let name = self.coerce_name_argument(arg, position)?;
        if let Some(rest) = name.strip_prefix("@@")
            && is_valid_class_variable_ident(rest)
        {
            return Ok(rest.to_string());
        }
        let msg = format!("`{}' is not allowed as a class variable name", name);
        Err(crate::vm::errors::invalid_name_error(
            msg, arg, receiver, position,
        ))
    }

    /// Coerce an instance-variable name argument to its storage key (the name
    /// with the leading `@` removed). Strings and Symbols are used directly;
    /// any other object is converted via `to_str`. Raises TypeError when that
    /// conversion is missing or returns a non-String, and NameError when the
    /// resulting name is not a valid instance variable name.
    pub(crate) fn coerce_instance_variable_name(
        &mut self,
        arg: &Object,
        receiver: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let name = self.coerce_name_argument(arg, position)?;
        if let Some(rest) = name.strip_prefix('@')
            && is_valid_class_variable_ident(rest)
        {
            return Ok(rest.to_string());
        }
        let msg = format!("`{}' is not allowed as an instance variable name", name);
        Err(crate::vm::errors::invalid_name_error(
            msg, arg, receiver, position,
        ))
    }

    /// What a queue starts off holding: the elements an enumerable answers
    /// for `to_a`, in the order it answers them.
    pub(crate) fn queue_seed_argument(
        &mut self,
        held: &Object,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let Object::Array(elements) = held {
            return Ok(elements.borrow().clone());
        }
        let named = self.builtins().class_of(held).ruby_name().to_string();
        if !self.responds_to(held, "to_a") {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("can't convert {named} into Array"),
                position,
            ));
        }
        let answered = self.send_to_object(held.clone(), "to_a", vec![], position)?;
        match answered {
            Object::Array(elements) => Ok(elements.borrow().clone()),
            other => {
                let gives = self.builtins().class_of(&other).ruby_name().to_string();
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("can't convert {named} into Array ({named}#to_a gives {gives})"),
                    position,
                ))
            }
        }
    }

    /// How many a SizedQueue holds, which is a whole number above zero.
    pub(crate) fn queue_capacity_argument(
        &mut self,
        held: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let counted: i64 = match held {
            Object::Int(counted) => *counted,
            other => {
                let named = self.coerce_integer_argument(other, position)?;
                named.try_into().unwrap_or(i64::MAX)
            }
        };
        if counted <= 0 {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "queue size must be positive",
                position,
            ));
        }
        Ok(counted)
    }
}

/// The most elements an array Ruby builds may hold, which is what it refuses
/// a larger count against.
const WIDEST_ARRAY: i64 = 1152921504606846975;

/// Ruby's ArgumentError for an array asked to hold more than it can.
fn array_size_too_big(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", "array size too big", position)
}

/// Validate the identifier portion of a class variable name (the part after
/// `@@`): it must start with a letter or underscore and contain only
/// alphanumerics and underscores.
pub(crate) fn is_valid_class_variable_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || !c.is_ascii() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
}

impl VirtualMachine {
    /// The path an argument names, taking `to_path` from an object that
    /// answers one and refusing anything else.
    pub(crate) fn path_name_argument(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match argument {
            Object::String(path) => Ok(path.as_str().to_string()),
            // A path may be named by an object that answers `to_path`, and
            // by one that answers `to_str` the way any String argument may.
            other if self.responds_to(other, "to_path") || self.responds_to(other, "to_str") => {
                let asked = if self.responds_to(other, "to_path") {
                    "to_path"
                } else {
                    "to_str"
                };
                match self.send_to_object(other.clone(), asked, vec![], position)? {
                    Object::String(path) => Ok(path.as_str().to_string()),
                    converted => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &converted,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                method_name,
                "String",
                other,
                position,
            )),
        }
    }
}
