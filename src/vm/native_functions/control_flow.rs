// Raising, catching, throwing, and looping.

use super::*;

impl VirtualMachine {
    /// `fail` is Ruby's other spelling of `raise`.
    /// `fail` is Ruby's other spelling of `raise`. Both are reachable
    /// as methods, so `send(:raise, ...)` and a singleton that makes
    /// `raise` public find them here.
    pub(crate) fn raise_exception(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A class, a message and a backtrace, with `cause:` allowed
        // alongside them.
        let counted = match arguments.last() {
            Some(Object::Dict(pairs)) if pairs.borrow().contains_key("__MX_KWARGS__") => {
                arguments.len() - 1
            }
            _ => arguments.len(),
        };
        if counted > 3 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(0, 3),
                counted,
                position,
            ));
        }
        let exception = self.build_raise_exception(&arguments, position)?;
        let message = match &exception {
            Object::Exception(_) => crate::vm::utils::format_exception(&exception),
            _ => String::new(),
        };
        Err(MetorexError::UncaughtException {
            exception,
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }

    /// Kernel#loop — run the block until `break` or StopIteration.
    /// `break value` is the loop's value; a StopIteration (or a
    /// subclass) ends the loop and yields the iterator's result. Every
    /// other exception propagates.
    pub(crate) fn run_loop(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                format!("loop() expects 0 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        // Without a block, `loop` answers an enumerator that yields
        // forever, which is what `loop.size` and `loop.each` read.
        let block = match self.pending_block.take() {
            Some(Object::Block(block)) => block,
            _ => {
                let Some(enumerator_class) = self.globals().get("Enumerator") else {
                    let message = "uninitialized constant Enumerator".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("NameError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                return self.send_to_object(enumerator_class, "endless", Vec::new(), position);
            }
        };
        loop {
            match block.call(self, Vec::new(), position) {
                Ok(_) => {}
                Err(MetorexError::BlockBreak { value, .. }) => return Ok(value),
                // A StopIteration ends the loop, which answers the
                // result the finished iterator carried.
                Err(MetorexError::UncaughtException { exception, .. })
                    if self.exception_matches(&exception, &["StopIteration".to_string()])? =>
                {
                    if let Object::Exception(details) = &exception {
                        let result = details.borrow().instance_vars.get("result").cloned();
                        if let Some(result) = result {
                            return Ok(result);
                        }
                    }
                    return Ok(Object::Nil);
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(crate) fn catch_tag(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() > 1 {
            return Err(MetorexError::runtime_error(
                format!("catch() expects 0-1 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let block = match self.pending_block.take() {
            Some(Object::Block(b)) => b,
            _ => {
                let message = "no block given (yield)".to_string();
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("LocalJumpError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                });
            }
        };
        let tag = match arguments.first() {
            Some(tag) => tag.clone(),
            None => match self.globals().get("Object") {
                Some(Object::Class(object_class)) => Object::instance(object_class),
                _ => Object::Nil,
            },
        };
        self.catch_tags.push(tag.clone());
        let block_arguments = if block.binding_parameters().is_empty() {
            Vec::new()
        } else {
            vec![tag.clone()]
        };
        let result = block.call(self, block_arguments, position);
        self.catch_tags.pop();
        match result {
            Err(MetorexError::Throw {
                tag: thrown, value, ..
            }) if throw_tags_match(&tag, &thrown) => Ok(value),
            other => other,
        }
    }

    /// `throw tag, value` unwinds to the matching `catch`. Ruby raises
    /// UncaughtThrowError when no live catch holds the tag, rather than
    /// unwinding out of the program.
    pub(crate) fn throw_tag(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() || arguments.len() > 2 {
            let message = format!(
                "wrong number of arguments (given {}, expected 1..2)",
                arguments.len()
            );
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        let tag = arguments[0].clone();
        if !self
            .catch_tags
            .iter()
            .any(|live| throw_tags_match(live, &tag))
        {
            let message = format!(
                "uncaught throw {}",
                crate::vm::native_methods::array_methods::inspect_element(&tag)
            );
            let exception = Object::exception("UncaughtThrowError", message.clone());
            if let Object::Exception(details) = &exception {
                let mut details = details.borrow_mut();
                details
                    .instance_vars
                    .insert(crate::vm::THROW_TAG_KEY.to_string(), tag);
                details.instance_vars.insert(
                    crate::vm::THROW_VALUE_KEY.to_string(),
                    arguments.get(1).cloned().unwrap_or(Object::Nil),
                );
            }
            return Err(MetorexError::UncaughtException {
                exception,
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        Err(MetorexError::Throw {
            tag,
            value: arguments.get(1).cloned().unwrap_or(Object::Nil),
            location: crate::vm::utils::position_to_location(position),
        })
    }
}
