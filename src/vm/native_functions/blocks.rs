// Blocks held as objects, the hooks a program registers, and the
// refinements it brings into scope.

use super::*;

impl VirtualMachine {
    /// A receiverless `freeze` freezes `self` — inside a class or
    /// module body that is the class object itself.
    pub(crate) fn freeze_value(&mut self) -> Result<Object, MetorexError> {
        self.pending_block.take();
        let current_self = self.environment().get("self").unwrap_or(Object::Nil);
        match &current_self {
            Object::Class(class) | Object::Module(class) => class.freeze(),
            Object::Instance(instance) => instance.borrow_mut().frozen = true,
            _ => {}
        }
        Ok(current_self)
    }

    /// Kernel#lambda — the block becomes a lambda-style Proc. A proc
    /// handed over as `&expr` is not a literal block, so Ruby rejects
    /// it unless it is already a lambda.
    pub(crate) fn make_lambda(&mut self, position: Position) -> Result<Object, MetorexError> {
        let from_ampersand = self.pending_block_from_ampersand;
        match self.pending_block.take() {
            Some(Object::Block(block)) => {
                if block.is_lambda {
                    return Ok(Object::Block(block));
                }
                if from_ampersand {
                    let msg = "the lambda method requires a literal block";
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", msg.to_string()),
                        location: crate::vm::utils::position_to_location(position),
                        message: msg.to_string(),
                    });
                }
                let mut as_lambda = (*block).clone();
                as_lambda.is_lambda = true;
                Ok(Object::Block(std::rc::Rc::new(as_lambda)))
            }
            Some(other) => Ok(other),
            None => {
                let msg = "tried to create Proc object without a block";
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", msg.to_string()),
                    location: crate::vm::utils::position_to_location(position),
                    message: msg.to_string(),
                })
            }
        }
    }

    /// Kernel#proc — the block itself is the Proc.
    pub(crate) fn make_proc(&mut self, position: Position) -> Result<Object, MetorexError> {
        if let Some(block) = self.pending_block.take() {
            return Ok(block);
        }
        let msg = "tried to create Proc object without a block";
        Err(MetorexError::UncaughtException {
            exception: Object::exception("ArgumentError", msg.to_string()),
            location: crate::vm::utils::position_to_location(position),
            message: msg.to_string(),
        })
    }

    /// `END { ... }` registers its body once, however many times the
    /// line holding it is read.
    pub(crate) fn end_once(&mut self, position: Position) -> Result<Object, MetorexError> {
        let Some(block) = self.pending_block.take() else {
            let message = "called without a block".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let site = (
            self.current_source_file.clone().unwrap_or_default(),
            position.line,
            position.column,
        );
        if self.opened_blocks.insert(site) {
            self.at_exit_handlers.push(block.clone());
        }
        Ok(block)
    }

    pub(crate) fn register_at_exit(&mut self, position: Position) -> Result<Object, MetorexError> {
        let Some(block) = self.pending_block.take() else {
            let message = "called without a block".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        self.at_exit_handlers.push(block.clone());
        Ok(block)
    }

    pub(crate) fn use_refinement(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() != 1 {
            let exc = Object::exception(
                "ArgumentError",
                format!(
                    "wrong number of arguments (given {}, expected 1)",
                    arguments.len()
                ),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: crate::vm::utils::position_to_location(position),
                message: "wrong number of arguments for using".to_string(),
            });
        }
        let module = match &arguments[0] {
            Object::Module(m) => std::rc::Rc::clone(m),
            other => {
                let exc = Object::exception(
                    "TypeError",
                    format!(
                        "wrong argument type {} (expected Module)",
                        other.type_name()
                    ),
                );
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: crate::vm::utils::position_to_location(position),
                    message: "wrong argument type for using".to_string(),
                });
            }
        };
        // `using` is forbidden inside a method body.
        if self.inside_user_method() {
            let exc = Object::exception(
                "RuntimeError",
                "Module#using is not permitted in methods".to_string(),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: crate::vm::utils::position_to_location(position),
                message: "using in method".to_string(),
            });
        }
        self.activate_refinement(module);
        // Ruby answers with the enclosing class or module, or `main`
        // at the top level.
        Ok(match self.current_definee() {
            Some(definee) if definee.is_module() => Object::Module(definee),
            Some(definee) => Object::Class(definee),
            None => self.environment().get("self").unwrap_or(Object::Nil),
        })
    }
}
