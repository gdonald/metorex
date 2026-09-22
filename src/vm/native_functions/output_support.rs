// Writing one value out, and the streams a write goes to.

use super::*;

impl VirtualMachine {
    /// What a command wrote, as a String in the encoding this process reads
    /// and writes. Bytes that spell no text stand for themselves.
    pub(crate) fn command_output(&mut self, written: &[u8]) -> Object {
        let held = self.globals().get("__Encoding_default_external");
        let named = match held {
            Some(setting) => {
                match self.send_to_object(setting, "name", Vec::new(), Position::default()) {
                    Ok(Object::String(text)) => text.as_str().to_string(),
                    _ => "UTF-8".to_string(),
                }
            }
            None => "UTF-8".to_string(),
        };
        let made = match std::str::from_utf8(written) {
            Ok(text) => crate::object::StringValue::new(text),
            Err(_) => crate::object::StringValue::from_bytes(
                written.iter().map(|byte| *byte as char).collect::<String>(),
            ),
        };
        made.set_encoding(named);
        Object::String(std::rc::Rc::new(made))
    }

    /// Write one `puts` argument. An Array is written a line per element,
    /// however deeply nested, and an empty one writes a line of its own. A
    /// string that already ends in a newline is not given a second.
    pub(crate) fn puts_object(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        if let Object::Array(elements) = value {
            let elements = elements.borrow().clone();
            // An empty Array still writes a line, as `puts []` does.
            if elements.is_empty() {
                self.write_to_stdout("\n", position)?;
                return Ok(());
            }
            for element in &elements {
                self.puts_object(element, position)?;
            }
            return Ok(());
        }
        let output = self.get_string_representation(value, position)?;
        if output.ends_with('\n') {
            self.write_to_stdout(&output, position)?;
        } else {
            self.write_to_stdout(&format!("{}\n", output), position)?;
        }
        Ok(())
    }

    /// Write `text` where `$stdout` points. The default is the process's own
    /// stdout; when a program (or a spec harness) assigns an object with its
    /// own `write`, the text goes there instead.
    pub(crate) fn write_to_stdout(
        &mut self,
        text: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.write_to_stream("stdout", text, position)
    }

    /// The `$stderr` counterpart of `write_to_stdout`.
    /// The `Thread::Backtrace::Location` class the prelude defines. Its
    /// instances carry a `path`, `lineno`, `label`, and `absolute_path`,
    /// which is what both `caller_locations` and
    /// `Exception#backtrace_locations` hand out.
    pub(crate) fn backtrace_location_class(&mut self) -> std::rc::Rc<crate::class::Class> {
        use std::rc::Rc;
        if let Some(Object::Class(thread)) = self.globals().get("Thread")
            && let Some(Object::Class(backtrace)) = thread.get_class_var("Backtrace")
            && let Some(Object::Class(location)) = backtrace.get_class_var("Location")
        {
            return location;
        }
        Rc::new(crate::class::Class::new(
            "Thread::Backtrace::Location",
            None,
        ))
    }

    pub(crate) fn write_to_stderr(
        &mut self,
        text: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.write_to_stream("stderr", text, position)
    }

    fn write_to_stream(
        &mut self,
        stream: &str,
        text: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let target = self.globals().get(stream).unwrap_or(Object::Nil);
        if let Some((class, method)) = self.lookup_method(&target, "write")
            && !method.is_undefined
        {
            let argument = Object::string(text.to_string());
            self.invoke_method(class, method, target, vec![argument], position)?;
            return Ok(());
        }
        // A stream reassigned to a File handle answers `write` natively, with
        // no entry in a method table to find.
        if let Object::Instance(_) = &target {
            let argument = Object::string(text.to_string());
            let class = self.builtins().class_of(&target);
            if self
                .call_native_method(&class, &target, "write", &[argument], position)?
                .is_some()
            {
                return Ok(());
            }
        }
        write_to_standard_stream(stream, text);
        Ok(())
    }
}

/// Write to one of the standard streams. A stream whose other end has gone
/// ends the program the way the signal would.
pub(crate) fn write_to_standard_stream(stream: &str, text: &str) {
    use std::io::Write as _;
    let sent = if stream == "stderr" {
        let mut held = std::io::stderr();
        held.write_all(text.as_bytes()).and_then(|()| held.flush())
    } else {
        let mut held = std::io::stdout();
        held.write_all(text.as_bytes()).and_then(|()| held.flush())
    };
    if let Err(trouble) = sent
        && trouble.kind() == std::io::ErrorKind::BrokenPipe
    {
        die_of_a_broken_pipe();
    }
}

/// End the program the way a signal would when the other end of the standard
/// stream has gone. Ruby lets SIGPIPE through for the standard streams, and a
/// program in a pipeline is told to stop that way.
pub(crate) fn die_of_a_broken_pipe() -> ! {
    // SAFETY: both calls name a signal this process may send itself, and the
    // disposition restored is the one every program starts with.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        libc::raise(libc::SIGPIPE);
    }
    std::process::exit(141)
}
