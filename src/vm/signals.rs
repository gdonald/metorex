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
        _ => return None,
    };
    Some(match text.as_str() {
        "SIG_IGN" | "IGNORE" => "IGNORE".to_string(),
        "SIG_DFL" | "DEFAULT" => "DEFAULT".to_string(),
        "SYSTEM_DEFAULT" => "SYSTEM_DEFAULT".to_string(),
        _ => text,
    })
}

/// One past the highest signal number any platform metorex runs on uses.
const SIGNAL_SLOTS: usize = 65;

/// Which signals have arrived and not been handled yet, set by the operating
/// system's handler and cleared as the interpreter runs the Ruby one.
static PENDING: [std::sync::atomic::AtomicBool; SIGNAL_SLOTS] =
    [const { std::sync::atomic::AtomicBool::new(false) }; SIGNAL_SLOTS];

/// Whether any signal is pending, so the check made before every statement
/// reads one flag.
static ANY_PENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Which signals a Ruby handler has asked to catch.
static CAUGHT: [std::sync::atomic::AtomicBool; SIGNAL_SLOTS] =
    [const { std::sync::atomic::AtomicBool::new(false) }; SIGNAL_SLOTS];

/// Which interpreter's thread caught each signal. Only that thread runs the
/// handler, since the handler is that interpreter's own and a process may
/// run several interpreters on threads of their own.
static CAUGHT_BY: [std::sync::atomic::AtomicUsize; SIGNAL_SLOTS] =
    [const { std::sync::atomic::AtomicUsize::new(0) }; SIGNAL_SLOTS];

/// The number the next thread to catch a signal is known by. Zero stands for
/// no thread at all.
static NEXT_CATCHER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

thread_local! {
    static CATCHER: usize = NEXT_CATCHER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// Whether any signal has arrived that no thread has taken yet.
fn any_signal_waiting() -> bool {
    PENDING
        .iter()
        .any(|slot| slot.load(std::sync::atomic::Ordering::SeqCst))
}

/// What the operating system runs when a caught signal arrives. It only notes
/// the signal, since nothing else is safe to do inside a signal handler.
extern "C" fn note_signal(number: libc::c_int) {
    if let Some(slot) = PENDING.get(number as usize) {
        slot.store(true, std::sync::atomic::Ordering::SeqCst);
        ANY_PENDING.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Hand a signal to the operating system's handler for it: noting it for the
/// interpreter when a Ruby handler is written for it, and otherwise ignoring
/// it or leaving it to the system as the handler says. A signal no Ruby
/// handler ever caught keeps whatever the process already does with it.
fn set_disposition(number: i32, command: &Object) {
    let Some(caught) = CAUGHT.get(number as usize) else {
        return;
    };
    let catching = !matches!(command, Object::String(_) | Object::Nil);
    if !catching && !caught.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let handler = if catching {
        note_signal as extern "C" fn(libc::c_int) as libc::sighandler_t
    } else if matches!(command, Object::Nil)
        || matches!(command, Object::String(held) if *held.as_str() == *"IGNORE")
    {
        libc::SIG_IGN
    } else {
        libc::SIG_DFL
    };
    caught.store(catching, std::sync::atomic::Ordering::SeqCst);
    // A signal that arrived while it was caught has no handler to run once
    // it no longer is.
    if !catching {
        PENDING[number as usize].store(false, std::sync::atomic::Ordering::SeqCst);
    }
    CAUGHT_BY[number as usize].store(
        if catching {
            CATCHER.with(|own| *own)
        } else {
            0
        },
        std::sync::atomic::Ordering::SeqCst,
    );
    // SAFETY: the action is zeroed and then given a handler that only stores
    // to atomics, which is safe to run whenever the signal arrives.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = handler;
        action.sa_flags = libc::SA_RESTART;
        libc::sigemptyset(&mut action.sa_mask);
        libc::sigaction(number, &action, std::ptr::null_mut());
    }
}

impl crate::vm::VirtualMachine {
    /// Run the Ruby handler of every signal that has arrived since the last
    /// time this was asked, lowest number first.
    pub(crate) fn deliver_pending_signals(
        &mut self,
        position: crate::lexer::Position,
    ) -> Result<(), crate::error::MetorexError> {
        if !ANY_PENDING.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        let own = CATCHER.with(|own| *own);
        let mut mine = Vec::new();
        for (number, slot) in PENDING.iter().enumerate() {
            if CAUGHT_BY[number].load(std::sync::atomic::Ordering::SeqCst) == own
                && slot.swap(false, std::sync::atomic::Ordering::SeqCst)
            {
                mine.push(number as i32);
            }
        }
        // Only a thread that took a signal clears the flag, and only once no
        // slot is left. A thread clearing it while a signal waits for its own
        // catcher would leave that catcher reading nothing pending. A slot
        // set while the flag is being cleared is caught by reading the slots
        // once more.
        if !mine.is_empty() && !any_signal_waiting() {
            ANY_PENDING.store(false, std::sync::atomic::Ordering::SeqCst);
            if any_signal_waiting() {
                ANY_PENDING.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        for number in mine {
            if let Some(name) = name_for_number(number) {
                self.run_signal_handler(name, number, position)?;
            }
        }
        Ok(())
    }

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
        let given = arguments.first().cloned().unwrap_or(Object::Nil);
        // `EXIT` is not a signal the operating system sends: it names what to
        // run as the program ends.
        let exiting = matches!(&given, Object::Symbol(held) | Object::String(held) if *held.as_str() == *"EXIT");
        let (name, number) = if exiting {
            ("EXIT".to_string(), 0)
        } else {
            let (name, number) = self.signal_named_by(&given, position)?;
            if let Some(refused) = self.refuse_to_trap(&name, position) {
                return Err(refused);
            }
            (name, number)
        };
        let command = match (block, arguments.get(1)) {
            (Some(block), _) => block,
            (None, Some(command)) => match handler_name(command) {
                Some(name) => Object::string(name),
                None => command.clone(),
            },
            (None, None) => Object::string("DEFAULT"),
        };
        if !exiting {
            set_disposition(number, &command);
        }
        // A signal nothing was ever written for is answered for by the
        // operating system, which is what Ruby says of it.
        let previous = self
            .signal_handlers
            .insert(name, command)
            .unwrap_or_else(|| Object::string("SYSTEM_DEFAULT"));
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
        let given = arguments.first().cloned().unwrap_or(Object::Nil);
        // A signal named with a leading minus, or a negative number, goes to
        // the process group each pid leads rather than to the process.
        let (given, to_the_group) = match &given {
            Object::Int(number) => (given.clone(), *number < 0),
            Object::Symbol(text) | Object::String(text) if text.as_str().starts_with('-') => {
                (Object::string(&text.as_str()[1..]), true)
            }
            _ => (given.clone(), false),
        };
        let (name, number) = self.signal_named_by(&given, position)?;
        let own_pid = std::process::id() as i64;
        let mut delivered = 0;
        for target in &arguments[1..] {
            let pid = self.process_id_argument(Some(target), position)? as i64;
            delivered += 1;
            // A signal the program can answer for itself runs whatever
            // `Signal.trap` left in force. Everything else, and every signal
            // sent to another process, goes to the operating system. Signal 0
            // only asks whether the process could be signaled.
            if pid == own_pid
                && !to_the_group
                && number != 0
                && !matches!(name.as_str(), "KILL" | "STOP")
            {
                // A signal a program sends itself arrives inside the call
                // that sent it, which is what a report of it names.
                let frame = crate::vm::CallFrame::method(
                    "Process.kill".to_string(),
                    Some(format!("{}:{}", position.line, position.column)),
                    "Process.kill".to_string(),
                    "Process.kill".to_string(),
                )
                .with_source_file(self.current_source_file.clone());
                self.with_call_frame(frame, |vm| vm.run_signal_handler(&name, number, position))?;
                continue;
            }
            // A signal this process sends to itself is delivered before the
            // call returns, so the program never runs on past it.
            if pid == own_pid && !to_the_group && number != 0 {
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
            let aimed_at = if to_the_group { -pid } else { pid };
            if unsafe { libc::kill(aimed_at as libc::pid_t, number) } < 0 {
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
    pub(crate) fn run_signal_handler(
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
            // Nothing at all is written for a signal set aside with nil,
            // which is the same as ignoring it.
            Some(Object::Nil) => Ok(()),
            Some(held) => {
                // Anything that answers to `call` when the signal arrives
                // runs, whether it could when it was written down or not.
                self.send_to_object(held, "call", vec![Object::Int(number as i64)], position)?;
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
        // The signal was sent from somewhere, and that is where a report says
        // the program was when it arrived.
        let exception = self.add_stack_trace_to_exception(exception, position);
        crate::error::MetorexError::UncaughtException {
            exception,
            location: crate::vm::utils::position_to_location(position),
            message,
        }
    }

    /// The ArgumentError Ruby raises for a signal it cannot name.
    /// The signal an argument names, taken the way Ruby takes it: a Symbol,
    /// a String, or an Integer, and anything else only through `to_str`.
    /// A number with no signal behind it, a name nothing answers to, and a
    /// value of any other kind are each refused in their own words.
    pub(crate) fn signal_named_by(
        &mut self,
        given: &Object,
        position: crate::lexer::Position,
    ) -> Result<(String, i32), crate::error::MetorexError> {
        let refuse = |message: String| crate::error::MetorexError::UncaughtException {
            exception: Object::exception("ArgumentError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        };
        let named = match given {
            Object::Symbol(_) | Object::String(_) | Object::Int(_) => given.clone(),
            other if self.responds_to(other, "to_str") => {
                self.send_to_object(other.clone(), "to_str", Vec::new(), position)?
            }
            other => {
                // Ruby names nil, true and false outright rather than by the
                // class they belong to.
                return Err(refuse(format!(
                    "bad signal type {}",
                    match other {
                        Object::Nil => "NilClass".to_string(),
                        Object::Bool(true) => "TrueClass".to_string(),
                        Object::Bool(false) => "FalseClass".to_string(),
                        held => self.builtins().class_of(held).name().to_string(),
                    }
                )));
            }
        };
        if let Object::Int(number) = &named
            && crate::vm::signals::name_for_number(number.unsigned_abs() as i32).is_none()
        {
            return Err(refuse(format!("invalid signal number ({})", number)));
        }
        match signal_from_object(&named) {
            Some(held) => Ok(held),
            None => Err(self.signal_name_error(&named, position)),
        }
    }

    /// Whether a signal is one the interpreter keeps for itself, or one the
    /// operating system does not let a program answer for.
    fn refuse_to_trap(
        &mut self,
        name: &str,
        position: crate::lexer::Position,
    ) -> Option<crate::error::MetorexError> {
        let message = match name {
            "KILL" | "STOP" => format!("Signal already used by VM or OS: SIG{}", name),
            "SEGV" | "BUS" | "ILL" | "FPE" | "VTALRM" => {
                format!("can't trap reserved signal: SIG{}", name)
            }
            _ => return None,
        };
        Some(crate::error::MetorexError::UncaughtException {
            exception: Object::exception("ArgumentError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }

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
