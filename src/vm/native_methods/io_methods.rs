use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Instance, Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

/// Instance variable on a Process::Status: the status `waitpid` reported,
/// which every reader decodes.
const STATUS_RAW: &str = "__raw_status";
/// Instance variable on a Process::Status: the child's process id.
const STATUS_PID: &str = "__pid";
/// Global holding the status of the most recently waited-for child.
const LAST_STATUS_GLOBAL: &str = "__process_last_status";

/// How a signal reads in a status's description: `SIGKILL (signal 9)`, or
/// `signal 70` for a number no signal goes by.
fn described_signal(number: i32) -> String {
    match crate::vm::signals::name_for_number(number) {
        Some(name) => format!("SIG{name} (signal {number})"),
        None => format!("signal {number}"),
    }
}

/// What `Process::Status#to_s` answers for `raw`, after the pid.
fn described_status(raw: libc::c_int) -> String {
    if libc::WIFSTOPPED(raw) {
        return format!("stopped {}", described_signal(libc::WSTOPSIG(raw)));
    }
    if libc::WIFSIGNALED(raw) {
        let mut described = described_signal(libc::WTERMSIG(raw));
        if libc::WCOREDUMP(raw) {
            described.push_str(" (core dumped)");
        }
        return described;
    }
    format!("exit {}", libc::WEXITSTATUS(raw))
}

impl VirtualMachine {
    /// `Process::Status` readers, each decoding the status `waitpid`
    /// reported the way MRI's does.
    pub(crate) fn call_process_status_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        _position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(instance) = receiver else {
            return Ok(None);
        };
        let raw = match instance.borrow().get_var(STATUS_RAW) {
            Some(Object::Int(raw)) => *raw as libc::c_int,
            _ => 0,
        };
        let pid = instance
            .borrow()
            .get_var(STATUS_PID)
            .cloned()
            .unwrap_or(Object::Nil);
        let when = |held: bool, value: i64| {
            if held {
                Object::Int(value)
            } else {
                Object::Nil
            }
        };
        Ok(Some(match method_name {
            "exited?" => Object::Bool(libc::WIFEXITED(raw)),
            "exitstatus" => when(libc::WIFEXITED(raw), libc::WEXITSTATUS(raw) as i64),
            "signaled?" => Object::Bool(libc::WIFSIGNALED(raw)),
            "termsig" => when(libc::WIFSIGNALED(raw), libc::WTERMSIG(raw) as i64),
            "stopped?" => Object::Bool(libc::WIFSTOPPED(raw)),
            "stopsig" => when(libc::WIFSTOPPED(raw), libc::WSTOPSIG(raw) as i64),
            "coredump?" => Object::Bool(libc::WIFSIGNALED(raw) && libc::WCOREDUMP(raw)),
            "pid" => pid,
            // A child that a signal ended did not succeed or fail, so there
            // is nothing to report either way.
            "success?" => {
                if libc::WIFEXITED(raw) {
                    Object::Bool(libc::WEXITSTATUS(raw) == 0)
                } else {
                    Object::Nil
                }
            }
            "to_i" => Object::Int(raw as i64),
            "to_s" => Object::string(format!("pid {} {}", pid, described_status(raw))),
            "inspect" => Object::string(format!(
                "#<Process::Status: pid {} {}>",
                pid,
                described_status(raw)
            )),
            "==" => {
                let Some(other) = arguments.first() else {
                    return Ok(Some(Object::Bool(false)));
                };
                Object::Bool(match other {
                    Object::Int(number) => *number == raw as i64,
                    Object::Instance(held) => matches!(
                        held.borrow().get_var(STATUS_RAW),
                        Some(Object::Int(number)) if *number == raw as i64
                    ),
                    _ => false,
                })
            }
            _ => return Ok(None),
        }))
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
        let status = self.build_process_status(raw_wait_status(status), pid.unwrap_or(-1));
        self.globals_mut().set(LAST_STATUS_GLOBAL, status);
    }
}

/// The status `waitpid` reported for a child the standard library waited
/// for.
fn raw_wait_status(status: &std::process::ExitStatus) -> libc::c_int {
    use std::os::unix::process::ExitStatusExt as _;
    status.into_raw()
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
        let Some((pid, status)) = self.reap_child(requested, flags, position)? else {
            let message = "No child processes".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("Errno::ECHILD", message.clone()),
                location: position_to_location(position),
                message,
            });
        };
        if pid != 0 {
            self.globals_mut().set(LAST_STATUS_GLOBAL, status.clone());
        }
        Ok((pid, status))
    }

    /// Wait for a child, handing the wait to the scheduler of a fiber that is
    /// not blocking when the scheduler answers `process_wait`.
    pub(crate) fn wait_for_child_through_scheduler(
        &mut self,
        requested: i32,
        flags: libc::c_int,
        position: Position,
    ) -> Result<(i32, Object), MetorexError> {
        let scheduler = match self.current_scheduler() {
            Some(held) if flags & libc::WNOHANG == 0 && self.responds_to(&held, "process_wait") => {
                held
            }
            _ => return self.wait_for_child_with(requested, flags, position),
        };
        let status = self.send_to_object(
            scheduler,
            "process_wait",
            vec![
                Object::Int(i64::from(requested)),
                Object::Int(i64::from(flags)),
            ],
            position,
        )?;
        let pid = match self.send_to_object(status.clone(), "pid", Vec::new(), position)? {
            Object::Int(pid) => pid as i32,
            _ => 0,
        };
        self.globals_mut().set(LAST_STATUS_GLOBAL, status.clone());
        Ok((pid, status))
    }

    /// Wait for a child without recording `$?`, answering None when there is
    /// no child to wait for. A child still running under `WNOHANG` answers a
    /// pid of zero and a nil status.
    pub(crate) fn reap_child(
        &mut self,
        requested: i32,
        flags: libc::c_int,
        position: Position,
    ) -> Result<Option<(i32, Object)>, MetorexError> {
        let mut raw_status: libc::c_int = 0;
        let pid = self.waitpid_handing_turns(requested, &mut raw_status, flags, position)?;
        if pid == 0 && flags & libc::WNOHANG != 0 {
            return Ok(Some((0, Object::Nil)));
        }
        if pid <= 0 {
            return Ok(None);
        }
        let status = self.build_process_status(raw_status, pid as i64);
        Ok(Some((pid, status)))
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
        let class = Class::new(name, parent);
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

    /// A Process::Status for the status `waitpid` reported for `pid`.
    pub(crate) fn build_process_status(&mut self, raw: libc::c_int, pid: i64) -> Object {
        let status_class = self.memoized_class(
            "__Process_Status_class",
            "Process::Status",
            &["exited?", "=="],
        );
        let instance = Instance::new(status_class);
        {
            let mut borrowed = instance.borrow_mut();
            borrowed.set_var(STATUS_RAW.to_string(), Object::Int(raw as i64));
            borrowed.set_var(STATUS_PID.to_string(), Object::Int(pid));
        }
        Object::Instance(instance)
    }
}
