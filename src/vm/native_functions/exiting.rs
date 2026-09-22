// Ending the program.

use super::*;

impl VirtualMachine {
    pub(crate) fn exit_program(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let code = self.exit_status_argument(arguments.first(), position)?;
        if name == "exit!" {
            std::process::exit(code as i32);
        }
        // `exit` raises SystemExit so `ensure` blocks and a rescue of
        // SystemExit still see it. An uncaught one ends the program
        // with this status.
        let exception = Object::Exception(std::rc::Rc::new(std::cell::RefCell::new(
            crate::object::Exception {
                exception_type: "SystemExit".to_string(),
                message: "exit".to_string(),
                backtrace: None,
                location: None,
                cause: None,
                status: Some(code),
                name: None,
                receiver: None,
                backtrace_array: None,
                backtrace_sites: None,
                backtrace_locations_array: None,
                class: None,
                instance_vars: indexmap::IndexMap::new(),
                message_given: true,
                cause_settled: false,
            },
        )));
        Err(MetorexError::UncaughtException {
            exception,
            location: crate::vm::utils::position_to_location(position),
            message: "exit".to_string(),
        })
    }

    pub(crate) fn abort_program(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let message = match arguments.first() {
            Some(argument) => {
                let text = self.coerce_abort_message(argument, position)?;
                self.emit_warning_to_stderr(&text, position);
                text
            }
            None => "SystemExit".to_string(),
        };
        let exception = Object::Exception(std::rc::Rc::new(std::cell::RefCell::new(
            crate::object::Exception {
                exception_type: "SystemExit".to_string(),
                message: message.clone(),
                backtrace: None,
                location: None,
                cause: None,
                status: Some(1),
                name: None,
                receiver: None,
                backtrace_array: None,
                backtrace_sites: None,
                backtrace_locations_array: None,
                class: None,
                instance_vars: indexmap::IndexMap::new(),
                message_given: true,
                cause_settled: false,
            },
        )));
        Err(MetorexError::UncaughtException {
            exception,
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }
}
