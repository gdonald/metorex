//! Signal names and numbers, plus the handler table `Process.kill` consults.

use crate::object::Object;

/// The signals `Signal.list` reports, in the order Ruby lists them. Numbers
/// come from libc so each platform reports its own values.
pub(crate) fn signal_table() -> Vec<(&'static str, i32)> {
    vec![
        ("EXIT", 0),
        ("HUP", libc::SIGHUP),
        ("INT", libc::SIGINT),
        ("QUIT", libc::SIGQUIT),
        ("ILL", libc::SIGILL),
        ("TRAP", libc::SIGTRAP),
        ("ABRT", libc::SIGABRT),
        ("FPE", libc::SIGFPE),
        ("KILL", libc::SIGKILL),
        ("BUS", libc::SIGBUS),
        ("SEGV", libc::SIGSEGV),
        ("SYS", libc::SIGSYS),
        ("PIPE", libc::SIGPIPE),
        ("ALRM", libc::SIGALRM),
        ("TERM", libc::SIGTERM),
        ("URG", libc::SIGURG),
        ("STOP", libc::SIGSTOP),
        ("TSTP", libc::SIGTSTP),
        ("CONT", libc::SIGCONT),
        ("CHLD", libc::SIGCHLD),
        ("TTIN", libc::SIGTTIN),
        ("TTOU", libc::SIGTTOU),
        ("IO", libc::SIGIO),
        ("XCPU", libc::SIGXCPU),
        ("XFSZ", libc::SIGXFSZ),
        ("VTALRM", libc::SIGVTALRM),
        ("PROF", libc::SIGPROF),
        ("WINCH", libc::SIGWINCH),
        ("USR1", libc::SIGUSR1),
        ("USR2", libc::SIGUSR2),
        // Ruby lists the older spelling of SIGCHLD alongside it. It comes
        // last so a lookup by number answers the name the signal goes by.
        ("CLD", libc::SIGCHLD),
    ]
}

/// The number a signal name stands for. Accepts the bare name and the `SIG`
/// prefixed spelling, either of which Ruby takes.
pub(crate) fn number_for_name(name: &str) -> Option<i32> {
    let bare = name.strip_prefix("SIG").unwrap_or(name);
    signal_table()
        .into_iter()
        .find(|(candidate, _)| *candidate == bare)
        .map(|(_, number)| number)
}

/// The name a signal number stands for, without its `SIG` prefix.
pub(crate) fn name_for_number(number: i32) -> Option<&'static str> {
    signal_table()
        .into_iter()
        .find(|(_, candidate)| *candidate == number)
        .map(|(name, _)| name)
}

/// The signal an argument to `Signal.trap` or `Process.kill` names, as a
/// `(name, number)` pair. A negative number asks for the process group, which
/// names the same signal.
pub(crate) fn signal_from_object(value: &Object) -> Option<(String, i32)> {
    match value {
        Object::Symbol(name) | Object::String(name) => {
            number_for_name(&name.as_str()).map(|number| {
                (
                    name.as_str()
                        .strip_prefix("SIG")
                        .unwrap_or(&*name.as_str())
                        .to_string(),
                    number,
                )
            })
        }
        Object::Int(number) => {
            let number = number.unsigned_abs() as i32;
            name_for_number(number).map(|name| (name.to_string(), number))
        }
        _ => None,
    }
}

/// The handler name a `Signal.trap` command argument stands for, or `None`
/// when the argument is a callable to run instead.
pub(crate) fn handler_name(command: &Object) -> Option<String> {
    let text = match command {
        Object::Symbol(name) | Object::String(name) => name.as_str().to_string(),
        Object::Nil => "IGNORE".to_string(),
        _ => return None,
    };
    Some(match text.as_str() {
        "SIG_IGN" | "IGNORE" => "IGNORE".to_string(),
        "SIG_DFL" | "DEFAULT" => "DEFAULT".to_string(),
        "SYSTEM_DEFAULT" => "SYSTEM_DEFAULT".to_string(),
        _ => text,
    })
}

impl crate::vm::VirtualMachine {
    /// `Signal.list` — every known signal name mapped to its number.
    pub(crate) fn signal_list(&self) -> Object {
        let mut entries = indexmap::IndexMap::new();
        for (name, number) in signal_table() {
            entries.insert(name.to_string(), Object::Int(number as i64));
        }
        Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(entries)))
    }

    /// `Signal.signame(number)` — the name that number goes by, and nil for
    /// a number no signal uses. Anything but an Integer is asked for `to_int`.
    pub(crate) fn signal_name(
        &mut self,
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, crate::error::MetorexError> {
        if arguments.len() != 1 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Exact(1),
                arguments.len(),
                position,
            ));
        }
        let number = self.coerce_signal_number(&arguments[0], position)?;
        let Ok(number) = i32::try_from(number) else {
            return Ok(Object::Nil);
        };
        Ok(match name_for_number(number) {
            Some(name) => Object::string(name),
            None => Object::Nil,
        })
    }

    /// A signal number argument: an Integer as it stands, and anything else
    /// through `to_int`.
    fn coerce_signal_number(
        &mut self,
        argument: &Object,
        position: crate::lexer::Position,
    ) -> Result<i64, crate::error::MetorexError> {
        if let Object::Int(number) = argument {
            return Ok(*number);
        }
        let source = self.builtins().class_of(argument).name().to_string();
        let refuse = |message: String| crate::error::MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        };
        let Some((class, method)) = self.lookup_method(argument, "to_int") else {
            return Err(refuse(format!(
                "no implicit conversion of {} into Integer",
                source
            )));
        };
        match self.invoke_method(class, method, argument.clone(), Vec::new(), position)? {
            Object::Int(number) => Ok(number),
            produced => Err(refuse(format!(
                "can't convert {} to Integer ({}#to_int gives {})",
                source,
                source,
                self.builtins().class_of(&produced).name()
            ))),
        }
    }

    /// `Signal.trap(signal, command)` — install a handler and answer the one
    /// it replaced. A block stands in for the command argument.
    pub(crate) fn install_signal_trap(
        &mut self,
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, crate::error::MetorexError> {
        let block = self.pending_block.take();
        let Some((name, _)) = arguments.first().and_then(signal_from_object) else {
            let given = arguments.first().cloned().unwrap_or(Object::Nil);
            return Err(self.signal_name_error(&given, position));
        };
        let command = match (block, arguments.get(1)) {
            (Some(block), _) => block,
            (None, Some(command)) => match handler_name(command) {
                Some(name) => Object::string(name),
                None => command.clone(),
            },
            (None, None) => Object::string("DEFAULT"),
        };
        let previous = self
            .signal_handlers
            .insert(name, command)
            .unwrap_or_else(|| Object::string("DEFAULT"));
        Ok(previous)
    }

    /// `Process.kill(signal, *pids)` — answers how many processes were
    /// signalled. A signal aimed at this process runs its handler right here,
    /// which for the default disposition means raising.
    pub(crate) fn send_signal(
        &mut self,
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, crate::error::MetorexError> {
        if arguments.is_empty() {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                0,
                position,
            ));
        }
        let Some((name, number)) = arguments.first().and_then(signal_from_object) else {
            let given = arguments.first().cloned().unwrap_or(Object::Nil);
            return Err(self.signal_name_error(&given, position));
        };
        let own_pid = std::process::id() as i64;
        let mut delivered = 0;
        for target in &arguments[1..] {
            let Object::Int(pid) = target else {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!(
                        "no implicit conversion of {} into Integer",
                        self.builtins().class_of(target).name()
                    ),
                    position,
                ));
            };
            delivered += 1;
            // A signal the program can answer for itself runs whatever
            // `Signal.trap` left in force. Everything else, and every signal
            // sent to another process, goes to the operating system.
            if *pid == own_pid && !matches!(name.as_str(), "KILL" | "STOP") {
                self.run_signal_handler(&name, number, position)?;
                continue;
            }
            // A signal this process sends to itself is delivered before the
            // call returns, so the program never runs on past it.
            if *pid == own_pid {
                // SAFETY: `raise` delivers the signal to this process, which
                // an uncatchable one ends before returning.
                unsafe { libc::raise(number) };
                loop {
                    // SAFETY: `pause` waits for a signal and nothing else.
                    unsafe { libc::pause() };
                }
            }
            // SAFETY: `kill` sends one signal to one process and touches
            // nothing else.
            if unsafe { libc::kill(*pid as libc::pid_t, number) } < 0 {
                let code = std::io::Error::last_os_error();
                let named = match code.raw_os_error() {
                    Some(number) if number == libc::ESRCH => "Errno::ESRCH",
                    Some(number) if number == libc::EPERM => "Errno::EPERM",
                    Some(number) if number == libc::EINVAL => "Errno::EINVAL",
                    _ => "SystemCallError",
                };
                return Err(crate::vm::errors::simple_exception(
                    named,
                    &format!("{code} - kill(2)"),
                    position,
                ));
            }
        }
        Ok(Object::Int(delivered))
    }

    /// Run whatever `Signal.trap` left in force for `name`. The default
    /// disposition raises, which is how a Ruby program sees a signal at all.
    fn run_signal_handler(
        &mut self,
        name: &str,
        number: i32,
        position: crate::lexer::Position,
    ) -> Result<(), crate::error::MetorexError> {
        let handler = self.signal_handlers.get(name).cloned();
        match handler {
            Some(Object::String(disposition)) => match &*disposition.as_str() {
                "IGNORE" => Ok(()),
                _ => Err(self.signal_exception(name, number, position)),
            },
            Some(callable) => {
                self.invoke_callable(callable, vec![Object::Int(number as i64)], position)?;
                Ok(())
            }
            None => Err(self.signal_exception(name, number, position)),
        }
    }

    /// The exception a signal raises: `Interrupt` for SIGINT, and
    /// `SignalException` carrying the signal's name for everything else.
    fn signal_exception(
        &mut self,
        name: &str,
        number: i32,
        position: crate::lexer::Position,
    ) -> crate::error::MetorexError {
        let (class_name, message) = if name == "INT" {
            ("Interrupt", String::new())
        } else {
            ("SignalException", format!("SIG{}", name))
        };
        let exception = Object::exception(class_name, message.clone());
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.message_given = !message.is_empty();
            details
                .instance_vars
                .insert(SIGNO_KEY.to_string(), Object::Int(number as i64));
        }
        crate::error::MetorexError::UncaughtException {
            exception,
            location: crate::vm::utils::position_to_location(position),
            message,
        }
    }

    /// The ArgumentError Ruby raises for a signal it cannot name.
    fn signal_name_error(
        &mut self,
        given: &Object,
        position: crate::lexer::Position,
    ) -> crate::error::MetorexError {
        // Ruby names the signal the way it spells one, so a bare name is
        // reported with its `SIG` prefix.
        let named = match given {
            Object::Symbol(name) | Object::String(name) => {
                format!(
                    "SIG{}",
                    name.as_str().strip_prefix("SIG").unwrap_or(&*name.as_str())
                )
            }
            other => other.to_string(),
        };
        let message = format!("unsupported signal `{}'", named);
        crate::error::MetorexError::UncaughtException {
            exception: Object::exception("ArgumentError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        }
    }
}

/// Where a signal exception keeps its number. Not an `@` name, so a program's
/// own instance variables cannot collide with it.
pub(crate) const SIGNO_KEY: &str = "__signo__";

/// The signal number a SignalException carries, and None for an exception
/// that names no signal.
pub fn signal_number_of(details: &crate::object::Exception) -> Option<libc::c_int> {
    if !matches!(
        details.exception_type.as_str(),
        "SignalException" | "Interrupt"
    ) {
        return None;
    }
    if let Some(crate::object::Object::Int(number)) = details.instance_vars.get(SIGNO_KEY) {
        return Some(*number as libc::c_int);
    }
    // An exception raised by the program names its signal in the message,
    // with or without the `SIG` in front.
    let named = details.message.trim();
    let named = named.strip_prefix("SIG").unwrap_or(named);
    number_for_name(named)
}
