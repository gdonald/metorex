// Writing one line to the system log. The C library takes the message, and
// with LOG_PERROR asked for it also reaches the error stream, which is where
// a program watching its own output sees what it logged.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

/// The option bit asking for the message to reach the error stream too.
const LOG_PERROR: i64 = 0x20;

impl VirtualMachine {
    /// `Syslog.__write__(ident, options, priority, message)`.
    pub(crate) fn syslog_write(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (
            Some(Object::String(ident)),
            Some(Object::Int(options)),
            Some(Object::Int(priority)),
            Some(Object::String(message)),
        ) = (
            arguments.first(),
            arguments.get(1),
            arguments.get(2),
            arguments.get(3),
        )
        else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Exact(4),
                arguments.len(),
                position,
            ));
        };
        let named = std::ffi::CString::new(ident.as_str().as_bytes().to_vec()).unwrap_or_default();
        let written =
            std::ffi::CString::new(message.as_str().as_bytes().to_vec()).unwrap_or_default();
        let carried = std::ffi::CString::new("%s").expect("a literal with no zero byte");
        // SAFETY: each pointer names a CString that outlives the call, and
        // the format string takes exactly the one argument given after it.
        unsafe {
            libc::openlog(named.as_ptr(), *options as i32, 0);
            libc::syslog(*priority as i32, carried.as_ptr(), written.as_ptr());
            libc::closelog();
        }
        // The C library writes to stderr itself under LOG_PERROR on some
        // systems and not others, so the line is written here instead and
        // that bit is kept from it.
        if options & LOG_PERROR != 0 {
            let line = format!("{}: {}", ident.as_str(), message.as_str());
            self.emit_warning_to_stderr(&line, position);
        }
        Ok(Object::Nil)
    }
}
