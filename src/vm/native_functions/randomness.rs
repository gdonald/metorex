// The generator `rand` draws from, and the hooks a global carries.

use super::*;

impl VirtualMachine {
    /// Kernel#rand — a Float in [0, 1) with no argument, an Integer
    /// below the given bound, or a value drawn from a Range.
    pub(crate) fn random_draw(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() > 1 {
            return Err(MetorexError::runtime_error(
                format!("rand() expects 0-1 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let Some(limit) = arguments.first().cloned() else {
            return Ok(Object::Float(self.next_random_float()));
        };
        self.random_below(limit, position)
    }

    /// Kernel#trace_var — run a hook whenever the named global is
    /// assigned. The hook comes from a block, a Proc argument, or a
    /// String of code to evaluate.
    pub(crate) fn trace_global(
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
        let name = global_name_from(&arguments[0]);
        let hook = match arguments.get(1) {
            Some(hook) => hook.clone(),
            None => match self.pending_block.take() {
                Some(block) => block,
                None => {
                    let message = "tracing requires a block or a proc".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
            },
        };
        self.traced_globals.entry(name).or_default().push(hook);
        Ok(Object::Nil)
    }

    /// Kernel#untrace_var — drop the hooks on a global. With a second
    /// argument only that hook goes.
    pub(crate) fn untrace_global(
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
        let name = global_name_from(&arguments[0]);
        match arguments.get(1) {
            Some(hook) => {
                if let Some(hooks) = self.traced_globals.get_mut(&name) {
                    hooks.retain(|existing| !existing.equals(hook));
                }
            }
            None => {
                self.traced_globals.remove(&name);
            }
        }
        Ok(Object::Nil)
    }

    /// Kernel#srand — reseed the generator and answer the seed it had.
    /// With no argument it picks one, so successive calls differ.
    pub(crate) fn seed_generator(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() > 1 {
            return Err(MetorexError::runtime_error(
                format!("srand() expects 0-1 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let seed = match arguments.first() {
            None => Object::Int(crate::vm::core::seed_from_clock() as i64),
            Some(given) => self.coerce_to_seed(given, position)?,
        };
        let words = seed.as_big_integer().unwrap_or_default();
        let previous = std::mem::replace(&mut self.random_seed, seed);
        self.seed_random(&words);
        Ok(previous)
    }
}
