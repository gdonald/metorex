use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Instance, Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

/// Instance variable holding everything an `IO.popen` handle has left to read.
const POPEN_OUTPUT: &str = "__popen_output";
/// Instance variable holding the id of the child behind the handle.
const POPEN_HANDLE: &str = "__popen_handle";
/// Instance variable holding what has been written to the child's input.
const POPEN_INPUT: &str = "__popen_input";
/// Instance variable holding the child's process id.
const POPEN_PID: &str = "__popen_pid";
/// Instance variable recording whether the handle has been closed.
const POPEN_CLOSED: &str = "__popen_closed";
/// Instance variable on a Process::Status: the exit code, or nil when signaled.
const STATUS_EXITSTATUS: &str = "__exitstatus";
/// Instance variable on a Process::Status: the signal number, or nil.
const STATUS_TERMSIG: &str = "__termsig";
/// Instance variable on a Process::Status: the child's process id.
const STATUS_PID: &str = "__pid";
/// Global holding the status of the most recently waited-for child.
const LAST_STATUS_GLOBAL: &str = "__process_last_status";

impl VirtualMachine {
    /// `IO.popen(command)` and `IO.popen(command, options)`. The command runs
    /// through `/bin/sh` and is waited for before the handle is handed out, so
    /// `#read` answers everything the child wrote. `err: [:child, :out]`
    /// merges the child's stderr into that output.
    pub(crate) fn call_io_class_method(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if class_rc.name() != "IO" || method_name != "popen" {
            return Ok(None);
        }
        if arguments.is_empty() {
            return Err(method_argument_error("popen", 1, 0, position));
        }
        // The command is either a String the shell reads or an argv Array,
        // which names the program and its arguments outright.
        let mut child = match &arguments[0] {
            Object::String(text) => {
                let merge_stderr = arguments.iter().skip(1).any(child_out_redirect);
                // Merging through the shell keeps the child's two streams
                // interleaved as they were written, which capturing them
                // separately would lose. The whole command is grouped so the
                // redirect applies to it as a unit.
                let shell_command = if merge_stderr {
                    format!("{{ {} ; }} 2>&1", text)
                } else {
                    (**text).clone()
                };
                let mut child = std::process::Command::new("/bin/sh");
                child.arg("-c").arg(&shell_command);
                child
            }
            Object::Array(parts) => {
                let parts = parts.borrow();
                let mut words = Vec::with_capacity(parts.len());
                for part in parts.iter() {
                    match part {
                        Object::String(word) => words.push((**word).clone()),
                        other => {
                            return Err(method_argument_type_error(
                                "popen", "String", other, position,
                            ));
                        }
                    }
                }
                let Some((program, rest)) = words.split_first() else {
                    return Err(method_argument_error("popen", 1, 0, position));
                };
                let mut child = std::process::Command::new(program);
                child.args(rest);
                child
            }
            other => {
                return Err(method_argument_type_error(
                    "popen", "String", other, position,
                ));
            }
        };
        child.stdin(std::process::Stdio::piped());
        child.stdout(std::process::Stdio::piped());
        child.stderr(std::process::Stdio::inherit());
        let spawned = child.spawn().map_err(|error| {
            MetorexError::runtime_error(
                format!("Failed to run {}: {}", arguments[0], error),
                position_to_location(position),
            )
        })?;
        let pid = spawned.id() as i64;

        // The child stays alive behind the handle so the block can write to
        // its input before reading what it wrote back.
        let handle_id = self.next_popen_id;
        self.next_popen_id += 1;
        self.popen_children.insert(handle_id, spawned);

        let handle_class = self.memoized_class("__IO_popen_class", "IO", &["close", "closed?"]);
        let instance = Rc::new(RefCell::new(Instance::new(handle_class)));
        {
            let mut borrowed = instance.borrow_mut();
            borrowed.set_var(POPEN_HANDLE.to_string(), Object::Int(handle_id as i64));
            borrowed.set_var(POPEN_PID.to_string(), Object::Int(pid));
            borrowed.set_var(POPEN_INPUT.to_string(), Object::string(String::new()));
            borrowed.set_var(POPEN_CLOSED.to_string(), Object::Bool(false));
        }
        let handle = Object::Instance(Rc::clone(&instance));

        let Some(Object::Block(block)) = self.pending_block.take() else {
            return Ok(Some(handle));
        };
        let result = self.execute_block_callable(&block, vec![handle], position);
        // Whatever the block did, the child is waited for once it returns.
        self.finish_popen(&instance)?;
        result.map(Some)
    }

    /// Send whatever was written to a popen handle, wait for the child, and
    /// keep what it wrote. Answers the output, which a later `read` hands out.
    fn finish_popen(&mut self, instance: &Rc<RefCell<Instance>>) -> Result<String, MetorexError> {
        if let Some(Object::String(output)) = instance.borrow().get_var(POPEN_OUTPUT) {
            return Ok((**output).clone());
        }
        let handle_id = match instance.borrow().get_var(POPEN_HANDLE) {
            Some(Object::Int(id)) => *id as u64,
            _ => return Ok(String::new()),
        };
        let input = match instance.borrow().get_var(POPEN_INPUT) {
            Some(Object::String(text)) => (**text).clone(),
            _ => String::new(),
        };
        let Some(mut child) = self.popen_children.remove(&handle_id) else {
            return Ok(String::new());
        };
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write as _;
            let _ = stdin.write_all(input.as_bytes());
        }
        let finished = child.wait_with_output().map_err(|error| {
            MetorexError::runtime_error(
                format!("Failed to wait for the command: {}", error),
                crate::error::SourceLocation::new(0, 0, 0),
            )
        })?;
        let output = String::from_utf8_lossy(&finished.stdout).to_string();
        self.record_last_status(&finished.status);
        instance
            .borrow_mut()
            .set_var(POPEN_OUTPUT.to_string(), Object::string(output.clone()));
        Ok(output)
    }

    /// The reading and writing halves of an `IO.popen` handle.
    pub(crate) fn call_io_handle_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(instance) = receiver else {
            return Ok(None);
        };
        if instance.borrow().get_var(POPEN_HANDLE).is_none() {
            return Ok(None);
        }
        match method_name {
            // Everything the child wrote that has not been read yet, which is
            // an empty string rather than nil once the handle is drained.
            "read" => {
                let remaining = self.finish_popen(instance)?;
                instance
                    .borrow_mut()
                    .set_var(POPEN_OUTPUT.to_string(), Object::string(String::new()));
                Ok(Some(Object::string(remaining)))
            }
            // Writing buffers until the input is closed, which is when the
            // child is handed everything at once.
            "puts" | "print" | "write" | "<<" => {
                let mut written = String::new();
                for argument in arguments {
                    let text = self.get_string_representation(argument, position)?;
                    written.push_str(&text);
                    if method_name == "puts" && !text.ends_with('\n') {
                        written.push('\n');
                    }
                }
                if method_name == "puts" && arguments.is_empty() {
                    written.push('\n');
                }
                let existing = match instance.borrow().get_var(POPEN_INPUT) {
                    Some(Object::String(text)) => (**text).clone(),
                    _ => String::new(),
                };
                instance.borrow_mut().set_var(
                    POPEN_INPUT.to_string(),
                    Object::string(format!("{}{}", existing, written)),
                );
                Ok(Some(if method_name == "<<" {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "close_write" => {
                self.finish_popen(instance)?;
                Ok(Some(Object::Nil))
            }
            "pid" => Ok(Some(
                instance
                    .borrow()
                    .get_var(POPEN_PID)
                    .cloned()
                    .unwrap_or(Object::Nil),
            )),
            "close" => {
                self.finish_popen(instance)?;
                instance
                    .borrow_mut()
                    .set_var(POPEN_CLOSED.to_string(), Object::Bool(true));
                Ok(Some(Object::Nil))
            }
            "closed?" => Ok(Some(
                instance
                    .borrow()
                    .get_var(POPEN_CLOSED)
                    .cloned()
                    .unwrap_or(Object::Bool(false)),
            )),
            _ => Ok(None),
        }
    }

    /// `Process::Status` readers. A status carries either an exit code or the
    /// signal that ended the child, never both.
    pub(crate) fn call_process_status_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        _arguments: &[Object],
        _position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(instance) = receiver else {
            return Ok(None);
        };
        let exitstatus = instance
            .borrow()
            .get_var(STATUS_EXITSTATUS)
            .cloned()
            .unwrap_or(Object::Nil);
        let termsig = instance
            .borrow()
            .get_var(STATUS_TERMSIG)
            .cloned()
            .unwrap_or(Object::Nil);
        match method_name {
            "exited?" => Ok(Some(Object::Bool(!matches!(exitstatus, Object::Nil)))),
            "exitstatus" => Ok(Some(exitstatus)),
            "signaled?" => Ok(Some(Object::Bool(!matches!(termsig, Object::Nil)))),
            // Nothing metorex waits for is left stopped, so a status never
            // reports one.
            "stopped?" => Ok(Some(Object::Bool(false))),
            "stopsig" => Ok(Some(Object::Nil)),
            "pid" => Ok(Some(
                instance
                    .borrow()
                    .get_var(STATUS_PID)
                    .cloned()
                    .unwrap_or(Object::Nil),
            )),
            "termsig" => Ok(Some(termsig)),
            "success?" => Ok(Some(Object::Bool(matches!(exitstatus, Object::Int(0))))),
            "to_i" => Ok(Some(match exitstatus {
                Object::Int(code) => Object::Int(code << 8),
                _ => match termsig {
                    Object::Int(signal) => Object::Int(signal),
                    _ => Object::Int(0),
                },
            })),
            _ => Ok(None),
        }
    }

    /// `Process.last_status` — the status of the last child this process
    /// waited for, and nil before there has been one.
    pub(crate) fn process_last_status(&self) -> Object {
        self.globals()
            .get(LAST_STATUS_GLOBAL)
            .unwrap_or(Object::Nil)
    }

    /// Record a finished child's status so `Process.last_status` reads it back.
    pub(crate) fn record_last_status(&mut self, status: &std::process::ExitStatus) {
        let (exitstatus, termsig) = match status.code() {
            Some(code) => (Object::Int(code as i64), Object::Nil),
            None => (Object::Nil, Object::Int(terminating_signal(status))),
        };
        let status_class =
            self.memoized_class("__Process_Status_class", "Process::Status", &["exited?"]);
        let instance = Rc::new(RefCell::new(Instance::new(status_class)));
        instance
            .borrow_mut()
            .set_var(STATUS_EXITSTATUS.to_string(), exitstatus);
        instance
            .borrow_mut()
            .set_var(STATUS_TERMSIG.to_string(), termsig);
        self.globals_mut()
            .set(LAST_STATUS_GLOBAL, Object::Instance(instance));
    }

    /// A class built once and kept in globals, so every instance of it shares
    /// one method table and compares equal by class.
    fn memoized_class(&mut self, global: &str, name: &str, methods: &[&str]) -> Rc<Class> {
        if let Some(Object::Class(existing)) = self.globals().get(global) {
            return existing;
        }
        let class = Rc::new(Class::new(name, None));
        for method_name in methods {
            class.define_method(
                *method_name,
                Rc::new(Method::with_owner(
                    (*method_name).to_string(),
                    vec![],
                    vec![],
                    name.to_string(),
                )),
            );
        }
        self.globals_mut()
            .set(global, Object::Class(Rc::clone(&class)));
        class
    }
}

/// True when an `IO.popen` options hash asks for the child's stderr to join
/// its stdout, which mspec writes as `err: [:child, :out]`.
fn child_out_redirect(options: &Object) -> bool {
    let Object::Dict(entries) = options else {
        return false;
    };
    let Some(target) = entries.borrow().get(":err").cloned() else {
        return false;
    };
    let Object::Array(items) = target else {
        return false;
    };
    let items = items.borrow();
    items.len() == 2
        && matches!(&items[0], Object::Symbol(name) if name.as_str() == "child")
        && matches!(&items[1], Object::Symbol(name) if name.as_str() == "out")
}

/// The signal that ended a child, on platforms that report one.
#[cfg(unix)]
fn terminating_signal(status: &std::process::ExitStatus) -> i64 {
    use std::os::unix::process::ExitStatusExt as _;
    status.signal().unwrap_or(0) as i64
}

#[cfg(not(unix))]
fn terminating_signal(_status: &std::process::ExitStatus) -> i64 {
    0
}

impl VirtualMachine {
    /// Wait for a child process, answering its id and the Process::Status it
    /// ended with. A requested id of -1 waits for any child.
    pub(crate) fn wait_for_child(
        &mut self,
        requested: i32,
        position: Position,
    ) -> Result<(i32, Object), MetorexError> {
        let mut raw_status: libc::c_int = 0;
        // SAFETY: `waitpid` only writes through the status pointer given.
        let pid = unsafe { libc::waitpid(requested, &mut raw_status, 0) };
        if pid <= 0 {
            let message = "No child processes".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("Errno::ECHILD", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        let exited = libc::WIFEXITED(raw_status);
        let (exitstatus, termsig) = if exited {
            (
                Object::Int(libc::WEXITSTATUS(raw_status) as i64),
                Object::Nil,
            )
        } else if libc::WIFSIGNALED(raw_status) {
            (Object::Nil, Object::Int(libc::WTERMSIG(raw_status) as i64))
        } else {
            (Object::Int(0), Object::Nil)
        };
        let status = self.build_process_status(exitstatus, termsig, pid as i64);
        self.globals_mut().set(LAST_STATUS_GLOBAL, status.clone());
        Ok((pid, status))
    }

    /// A Process::Status carrying the parts a wait reported.
    fn build_process_status(&mut self, exitstatus: Object, termsig: Object, pid: i64) -> Object {
        let status_class =
            self.memoized_class("__Process_Status_class", "Process::Status", &["exited?"]);
        let instance = Rc::new(RefCell::new(Instance::new(status_class)));
        {
            let mut borrowed = instance.borrow_mut();
            borrowed.set_var(STATUS_EXITSTATUS.to_string(), exitstatus);
            borrowed.set_var(STATUS_TERMSIG.to_string(), termsig);
            borrowed.set_var(STATUS_PID.to_string(), Object::Int(pid));
        }
        Object::Instance(instance)
    }
}
