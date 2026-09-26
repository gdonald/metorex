use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Instance, Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

/// Instance variable on a Process::Status: the exit code, or nil when signaled.
const STATUS_EXITSTATUS: &str = "__exitstatus";
/// Instance variable on a Process::Status: the signal number, or nil.
const STATUS_TERMSIG: &str = "__termsig";
/// Instance variable on a Process::Status: the child's process id.
const STATUS_PID: &str = "__pid";
/// Global holding the status of the most recently waited-for child.
const LAST_STATUS_GLOBAL: &str = "__process_last_status";

impl VirtualMachine {
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
            // A child that a signal ended did not succeed or fail, so there
            // is nothing to report either way.
            "success?" => Ok(Some(match exitstatus {
                Object::Int(code) => Object::Bool(code == 0),
                _ => Object::Nil,
            })),
            "==" => {
                let Some(other) = _arguments.first() else {
                    return Ok(Some(Object::Bool(false)));
                };
                let held = match &exitstatus {
                    Object::Int(code) => code << 8,
                    _ => match &termsig {
                        Object::Int(signal) => *signal,
                        _ => 0,
                    },
                };
                Ok(Some(Object::Bool(match other {
                    Object::Int(number) => *number == held,
                    Object::Instance(_) => {
                        matches!(
                            self.call_process_status_method(other, "to_i", &[], _position)?,
                            Some(Object::Int(number)) if number == held
                        )
                    }
                    _ => false,
                })))
            }
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

    /// Take the last status aside, leaving none behind. A thread body starts
    /// with no child of its own, which is what Ruby reports inside one.
    pub(crate) fn take_last_status(&mut self) -> Option<Object> {
        let held = self.globals().get(LAST_STATUS_GLOBAL);
        self.globals_mut().set(LAST_STATUS_GLOBAL, Object::Nil);
        held
    }

    /// Put back what `take_last_status` set aside.
    pub(crate) fn restore_last_status(&mut self, held: Option<Object>) {
        self.globals_mut()
            .set(LAST_STATUS_GLOBAL, held.unwrap_or(Object::Nil));
    }

    /// Record a finished child's status so `Process.last_status` reads it back.
    pub(crate) fn record_last_status(
        &mut self,
        status: &std::process::ExitStatus,
        pid: Option<i64>,
    ) {
        let (exitstatus, termsig) = match status.code() {
            Some(code) => (Object::Int(code as i64), Object::Nil),
            None => (Object::Nil, Object::Int(terminating_signal(status))),
        };
        let status_class = self.memoized_class(
            "__Process_Status_class",
            "Process::Status",
            &["exited?", "=="],
        );
        let instance = Rc::new(RefCell::new(Instance::new(status_class)));
        instance
            .borrow_mut()
            .set_var(STATUS_EXITSTATUS.to_string(), exitstatus);
        instance
            .borrow_mut()
            .set_var(STATUS_TERMSIG.to_string(), termsig);
        if let Some(pid) = pid {
            instance
                .borrow_mut()
                .set_var(STATUS_PID.to_string(), Object::Int(pid));
        }
        self.globals_mut()
            .set(LAST_STATUS_GLOBAL, Object::Instance(instance));
    }
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
        self.wait_for_child_with(requested, 0, position)
    }

    /// Wait for a child, with the flags Ruby's `waitpid` takes. `WNOHANG`
    /// answers a pid of zero rather than waiting for one still running.
    pub(crate) fn wait_for_child_with(
        &mut self,
        requested: i32,
        flags: libc::c_int,
        position: Position,
    ) -> Result<(i32, Object), MetorexError> {
        let mut raw_status: libc::c_int = 0;
        // SAFETY: `waitpid` only writes through the status pointer given.
        let pid = unsafe { libc::waitpid(requested, &mut raw_status, flags) };
        // A child that has not finished answers a pid of zero, which the
        // caller reports as nil.
        if pid == 0 && flags & libc::WNOHANG != 0 {
            return Ok((0, Object::Nil));
        }
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

    /// A class built once and kept in globals, so every instance of it shares
    /// one method table and compares equal by class.
    fn memoized_class(&mut self, global: &str, name: &str, methods: &[&str]) -> Rc<Class> {
        if let Some(Object::Class(existing)) = self.globals().get(global) {
            return existing;
        }
        // The class descends from the one it stands for, so the methods
        // written for that class in the core library answer for it as well.
        let parent = match self.globals().get(name) {
            Some(Object::Class(found)) => Some(found),
            _ => None,
        };
        let class = Rc::new(Class::new(name, parent));
        for method in methods {
            class.define_method(
                *method,
                Rc::new(Method::with_owner(
                    method.to_string(),
                    Vec::new(),
                    Vec::new(),
                    name.to_string(),
                )),
            );
        }
        self.globals_mut()
            .set(global, Object::Class(Rc::clone(&class)));
        class
    }

    /// A Process::Status carrying the parts a wait reported.
    fn build_process_status(&mut self, exitstatus: Object, termsig: Object, pid: i64) -> Object {
        let status_class = self.memoized_class(
            "__Process_Status_class",
            "Process::Status",
            &["exited?", "=="],
        );
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
