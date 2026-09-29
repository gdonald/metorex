// The Process module, and what the operating system reports about
// this process.

use super::*;

/// The Process methods the interpreter answers itself rather than through a
/// method written in Ruby, which `respond_to?` reports alongside those.
pub(crate) const PROCESS_NATIVE_METHODS: &[&str] = &[
    "_fork",
    "abort",
    "argv0",
    "clock_getres",
    "clock_gettime",
    "daemon",
    "egid",
    "egid=",
    "euid",
    "euid=",
    "exit",
    "exit!",
    "fork",
    "getpgid",
    "getpgrp",
    "getpriority",
    "getrlimit",
    "getsid",
    "gid",
    "gid=",
    "groups",
    "groups=",
    "initgroups",
    "kill",
    "last_status",
    "maxgroups",
    "maxgroups=",
    "pid",
    "ppid",
    "setpgid",
    "setpgrp",
    "setpriority",
    "setproctitle",
    "setrlimit",
    "setsid",
    "spawn",
    "times",
    "uid",
    "uid=",
    "wait",
    "wait2",
    "waitall",
    "waitpid",
    "waitpid2",
    "warmup",
];

impl VirtualMachine {
    /// The Process module method `method_name` names, or `None` when the
    /// name is none of its own.
    pub(crate) fn call_process_module_methods(
        &mut self,
        module_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "pid" => return Ok(Some(Object::Int(std::process::id() as i64))),
            // The name the main script was run under, which is what
            // `__FILE__` reports there too. The same frozen string
            // answers every time.
            "argv0" => {
                if let Some(held) = self.globals().get("__Process_argv0") {
                    return Ok(Some(held));
                }
                let named = self.script_name().unwrap_or_else(|| "-e".to_string());
                let made = Object::string(named);
                if let Object::String(text) = &made {
                    text.freeze();
                }
                self.globals_mut().set("__Process_argv0", made.clone());
                return Ok(Some(made));
            }
            // SAFETY: `getppid` reads the parent's process id and
            // touches nothing else.
            "ppid" => return Ok(Some(Object::Int(unsafe { libc::getppid() } as i64))),
            "kill" => return self.send_signal(arguments, position).map(Some),
            // SAFETY: `geteuid` and `getuid` read process ids and touch
            // nothing else.
            "euid" => return Ok(Some(Object::Int(unsafe { libc::geteuid() } as i64))),
            "uid" => return Ok(Some(Object::Int(unsafe { libc::getuid() } as i64))),
            // SAFETY: `getegid` and `getgid` read process ids and touch
            // nothing else.
            "egid" => return Ok(Some(Object::Int(unsafe { libc::getegid() } as i64))),
            "gid" => return Ok(Some(Object::Int(unsafe { libc::getgid() } as i64))),
            // The supplementary groups this process belongs to, which is
            // what decides whether a file it does not own is still one of
            // its group's.
            "groups" => return Ok(Some(current_groups())),
            // The process group and session a process belongs to, which
            // decide which processes a signal reaches together.
            // SAFETY: each of these reads or sets one process id and
            // touches nothing else.
            "getpgrp" => return Ok(Some(Object::Int(unsafe { libc::getpgrp() } as i64))),
            "setpgrp" => {
                let answered = unsafe { libc::setpgid(0, 0) };
                return self.process_result(answered, "setpgrp", position).map(Some);
            }
            "getpgid" => {
                let pid = self.process_id_argument(arguments.first(), position)?;
                let answered = unsafe { libc::getpgid(pid) };
                return self.process_result(answered, "getpgid", position).map(Some);
            }
            "setpgid" => {
                let pid = self.process_id_argument(arguments.first(), position)?;
                let group = self.process_id_argument(arguments.get(1), position)?;
                let answered = unsafe { libc::setpgid(pid, group) };
                return self.process_result(answered, "setpgid", position).map(Some);
            }
            "getsid" => {
                let pid = self.process_id_argument(arguments.first(), position)?;
                let answered = unsafe { libc::getsid(pid) };
                return self.process_result(answered, "getsid", position).map(Some);
            }
            // The soft and hard bounds the operating system keeps on
            // one kind of resource.
            "getrlimit" => {
                let resource = self.rlimit_resource(arguments.first(), position)?;
                let mut limits = libc::rlimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                };
                // SAFETY: `getrlimit` fills the struct handed to it and
                // touches nothing else.
                let answered = unsafe { libc::getrlimit(resource, &mut limits) };
                if answered != 0 {
                    return Err(self.errno_error("getrlimit", position));
                }
                return Ok(Some(Object::array(vec![
                    Object::Int(limits.rlim_cur as i64),
                    Object::Int(limits.rlim_max as i64),
                ])));
            }
            "setrlimit" => {
                let resource = self.rlimit_resource(arguments.first(), position)?;
                let soft = self.rlimit_value(arguments.get(1), position)?;
                let hard = match arguments.get(2) {
                    None => soft,
                    held => self.rlimit_value(held, position)?,
                };
                let limits = libc::rlimit {
                    rlim_cur: soft,
                    rlim_max: hard,
                };
                // SAFETY: `setrlimit` reads the struct handed to it and
                // touches nothing else.
                let answered = unsafe { libc::setrlimit(resource, &limits) };
                if answered != 0 {
                    return Err(self.errno_error("setrlimit", position));
                }
                return Ok(Some(Object::Nil));
            }
            "setsid" => {
                let answered = unsafe { libc::setsid() };
                return self.process_result(answered, "setsid", position).map(Some);
            }
            // How much of the processor a process is given before the
            // ones around it, where a lower number means more.
            "getpriority" => {
                let which = self.process_id_argument(arguments.first(), position)?;
                let who = self.process_id_argument(arguments.get(1), position)?;
                // SAFETY: `getpriority` reads a setting and touches
                // nothing else. A priority of -1 is a real answer, so
                // errno is cleared first to tell it from a failure.
                let answered = unsafe {
                    *errno_location() = 0;
                    libc::getpriority(which as _, who as _)
                };
                if answered == -1 && unsafe { *errno_location() } != 0 {
                    return Err(self.errno_error("getpriority", position));
                }
                return Ok(Some(Object::Int(answered as i64)));
            }
            "setpriority" => {
                let which = self.process_id_argument(arguments.first(), position)?;
                let who = self.process_id_argument(arguments.get(1), position)?;
                let level = self.process_id_argument(arguments.get(2), position)?;
                let answered = unsafe { libc::setpriority(which as _, who as _, level) };
                return self
                    .process_result(answered, "setpriority", position)
                    .map(Some);
            }
            // The supplementary groups a named user belongs to, which
            // only a process running as root may take on.
            "initgroups" => {
                let Some(Object::String(user)) = arguments.first() else {
                    return Err(method_argument_type_error(
                        "initgroups",
                        "String",
                        arguments.first().unwrap_or(&Object::Nil),
                        position,
                    ));
                };
                let group = self.process_id_argument(arguments.get(1), position)?;
                let Ok(named) = std::ffi::CString::new(user.as_str().as_bytes().to_vec()) else {
                    return Err(method_argument_type_error(
                        "initgroups",
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                // SAFETY: `initgroups` reads the name across the call and
                // sets this process's group list.
                let answered = unsafe { libc::initgroups(named.as_ptr(), group as _) };
                if answered < 0 {
                    return Err(self.errno_error("initgroups", position));
                }
                return Ok(Some(current_groups()));
            }
            "groups=" => {
                let Some(Object::Array(wanted)) = arguments.first() else {
                    return Err(method_argument_type_error(
                        "groups=",
                        "Array",
                        arguments.first().unwrap_or(&Object::Nil),
                        position,
                    ));
                };
                let held: Vec<libc::gid_t> = wanted
                    .borrow()
                    .iter()
                    .filter_map(|group| match group {
                        Object::Int(number) => Some(*number as libc::gid_t),
                        _ => None,
                    })
                    .collect();
                // SAFETY: `setgroups` reads the list across the call.
                let answered = unsafe { libc::setgroups(held.len() as _, held.as_ptr()) };
                if answered < 0 {
                    return Err(self.errno_error("setgroups", position));
                }
                return Ok(Some(arguments[0].clone()));
            }
            // The name this process shows under in a process listing,
            // which Ruby keeps apart from `$0`.
            "setproctitle" => {
                let title = match arguments.first() {
                    Some(Object::String(text)) => text.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "setproctitle",
                            "String",
                            other.unwrap_or(&Object::Nil),
                            position,
                        ));
                    }
                };
                self.set_process_title(&title);
                return Ok(Some(Object::string(title)));
            }
            // `_fork` is the hook `fork` runs through.
            "_fork" => return self.split_process(position).map(Some),
            // Carry on as a process of its own, detached from the terminal
            // and from the one that started it, which leaves at once.
            "daemon" => {
                let stays_put = arguments.first().is_some_and(Object::is_truthy);
                let keeps_streams = arguments.get(1).is_some_and(Object::is_truthy);
                return self
                    .detach_process(stays_put, keeps_streams, position)
                    .map(Some);
            }
            // A user or a group may be named rather than numbered, and
            // setting either needs the right to do so.
            "uid=" | "gid=" | "euid=" | "egid=" => {
                let wanted = self.account_id_argument(
                    method_name,
                    arguments.first(),
                    method_name.starts_with('u') || method_name == "euid=",
                    position,
                )?;
                // SAFETY: each of these sets one id and touches nothing
                // else.
                let answered = unsafe {
                    match method_name {
                        "uid=" => libc::setuid(wanted as libc::uid_t),
                        "euid=" => libc::seteuid(wanted as libc::uid_t),
                        "gid=" => libc::setgid(wanted as libc::gid_t),
                        _ => libc::setegid(wanted as libc::gid_t),
                    }
                };
                if answered < 0 {
                    return Err(self.errno_error(method_name, position));
                }
                return Ok(Some(Object::Int(wanted as i64)));
            }
            // How much processor time this program and the children it
            // waited for have used, in seconds.
            "times" => {
                let mut held: libc::tms = unsafe { std::mem::zeroed() };
                // SAFETY: `times` writes through the struct pointer given
                // and touches nothing else.
                unsafe { libc::times(&mut held) };
                // SAFETY: `sysconf` reads one setting.
                let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
                let ticks = if ticks > 0.0 { ticks } else { 100.0 };
                let seconds = |count: libc::clock_t| Object::Float(count as f64 / ticks);
                let tms = module_rc.get_class_var("Tms").unwrap_or(Object::Nil);
                return self
                    .send_to_object(
                        tms,
                        "new",
                        vec![
                            seconds(held.tms_utime),
                            seconds(held.tms_stime),
                            seconds(held.tms_cutime),
                            seconds(held.tms_cstime),
                        ],
                        position,
                    )
                    .map(Some);
            }
            // `last_status` takes nothing at all, and answers the status
            // of the last child this process waited for.
            "last_status" => {
                if !arguments.is_empty() {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Exact(0),
                        arguments.len(),
                        position,
                    ));
                }
                return Ok(Some(self.process_last_status()));
            }
            // Ruby documents `warmup` as a hint the implementation is
            // free to ignore, and answers true for having taken it.
            "warmup" => return Ok(Some(Object::Bool(true))),
            // The ceiling on the supplementary group list. Ruby keeps it
            // as a settable value rather than reading it back from the
            // operating system, so metorex holds the last one written.
            "maxgroups" => {
                return Ok(Some(
                    self.globals()
                        .get("__process_maxgroups")
                        .unwrap_or(Object::Int(DEFAULT_MAXGROUPS)),
                ));
            }
            "maxgroups=" => {
                let written = arguments.first().cloned().unwrap_or(Object::Nil);
                self.globals_mut()
                    .set("__process_maxgroups", written.clone());
                return Ok(Some(written));
            }
            // `Process.exit`, `.exit!`, and `.abort` end this process the
            // way the bare forms do.
            "exit" | "exit!" | "abort" | "spawn" => {
                return self
                    .call_native_function(method_name, arguments.to_vec(), position)
                    .map(Some);
            }

            // `Process::Status.wait` answers the status itself and leaves
            // `$?` alone. With no child to wait for, the status names a pid
            // of -1 rather than raising.
            "__wait_status__" => {
                let requested = match arguments.first() {
                    None | Some(Object::Nil) => -1,
                    Some(held) => self.process_id_argument(Some(held), position)?,
                };
                let flags = match arguments.get(1) {
                    Some(Object::Int(held)) => *held as libc::c_int,
                    _ => 0,
                };
                return Ok(Some(match self.reap_child(requested, flags) {
                    Some((0, _)) => Object::Nil,
                    Some((_, status)) => status,
                    None => self.build_process_status(Object::Nil, Object::Nil, -1),
                }));
            }
            // `wait` and `waitpid` answer the child's process id, and
            // `wait2` pairs it with the status. All three record `$?`.
            "wait" | "waitpid" | "wait2" | "waitpid2" => {
                // A process id may be spelled by an object that answers
                // `to_int`, which is what Ruby reads it through.
                let requested = match arguments.first() {
                    None | Some(Object::Nil) => -1,
                    Some(held) => self.process_id_argument(Some(held), position)?,
                };
                let flags = match arguments.get(1) {
                    Some(Object::Int(held)) => *held as libc::c_int,
                    _ => 0,
                };
                let (pid, status) = self.wait_for_child_with(requested, flags, position)?;
                // A child still running answers nothing at all.
                if pid == 0 {
                    return Ok(Some(Object::Nil));
                }
                if matches!(method_name, "wait2" | "waitpid2") {
                    let pair = vec![Object::Int(pid as i64), status];
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(pair)))));
                }
                return Ok(Some(Object::Int(pid as i64)));
            }
            // The reading a clock gives, in the unit named or in float
            // seconds when none is.
            "clock_gettime" | "clock_getres" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 2),
                        arguments.len(),
                        position,
                    ));
                }
                let nanoseconds = if method_name == "clock_gettime" {
                    self.clock_reading(&arguments[0], position)?
                } else {
                    self.clock_resolution(&arguments[0], position)?
                };
                let unit = match arguments.get(1) {
                    None | Some(Object::Nil) => "float_second".to_string(),
                    Some(Object::Symbol(named)) => named.as_str().to_string(),
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Symbol",
                            other,
                            position,
                        ));
                    }
                };
                return Ok(Some(clock_in_unit(nanoseconds, &unit, position)?));
            }
            "waitall" => {
                // Ruby waits for every child there is, so there is
                // nothing to name and nothing to pass.
                if !arguments.is_empty() {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Exact(0),
                        arguments.len(),
                        position,
                    ));
                }
                let mut results = Vec::new();
                while let Ok((pid, status)) = self.wait_for_child(-1, position) {
                    if pid <= 0 {
                        break;
                    }
                    results.push(Object::Array(Rc::new(std::cell::RefCell::new(vec![
                        Object::Int(pid as i64),
                        status,
                    ]))));
                }
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    results,
                )))));
            }
            _ => {}
        }
        Ok(None)
    }

    /// A process, group, or priority number an argument names, taking
    /// `to_int` from an object that answers one. A missing argument means
    /// this process.
    pub(crate) fn process_id_argument(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<i32, MetorexError> {
        match argument {
            None | Some(Object::Nil) => Ok(0),
            Some(Object::Int(number)) => Ok(*number as i32),
            Some(other) if self.answers_to(other, "to_int", position)? => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(number) => Ok(number as i32),
                    converted => Err(method_argument_type_error(
                        "Process", "Integer", &converted, position,
                    )),
                }
            }
            Some(other) => Err(method_argument_type_error(
                "Process", "Integer", other, position,
            )),
        }
    }

    /// What a process call answered: the number itself, or the errno the
    /// operating system left behind when it failed.
    fn process_result(
        &mut self,
        answered: i32,
        called: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if answered < 0 {
            return Err(self.errno_error(called, position));
        }
        Ok(Object::Int(answered as i64))
    }

    /// The exception the last failed system call names.
    fn errno_error(&mut self, called: &str, position: Position) -> MetorexError {
        let code = std::io::Error::last_os_error();
        let named = errno_constant(code.raw_os_error().unwrap_or(0));
        crate::vm::errors::simple_exception(named, &format!("{code} - {called}(2)"), position)
    }

    /// Rename this process so a listing shows the new name.
    fn set_process_title(&mut self, title: &str) {
        if let Some((start, length)) = original_argument_area() {
            // SAFETY: the area is the one the kernel copied the program's
            // arguments into, which belongs to this process for as long as it
            // runs and is written here in place, within its length.
            unsafe { write_process_title(start, length, title.as_bytes()) };
        }
    }

    /// The user or group an argument names: a number outright, a name looked
    /// up in the account database, or an object with `to_int`.
    fn account_id_argument(
        &mut self,
        method_name: &str,
        argument: Option<&Object>,
        is_user: bool,
        position: Position,
    ) -> Result<i32, MetorexError> {
        if let Some(Object::String(name)) = argument {
            let Ok(held) = std::ffi::CString::new(name.as_str().as_bytes().to_vec()) else {
                return Err(method_argument_type_error(
                    method_name,
                    "String",
                    &argument.cloned().unwrap_or(Object::Nil),
                    position,
                ));
            };
            // SAFETY: both calls answer a pointer the C library owns, read
            // before any other call to it.
            let found = unsafe {
                if is_user {
                    let entry = libc::getpwnam(held.as_ptr());
                    if entry.is_null() {
                        None
                    } else {
                        Some((*entry).pw_uid as i32)
                    }
                } else {
                    let entry = libc::getgrnam(held.as_ptr());
                    if entry.is_null() {
                        None
                    } else {
                        Some((*entry).gr_gid as i32)
                    }
                }
            };
            return match found {
                Some(id) => Ok(id),
                None => Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    &format!(
                        "can't find {} for {}",
                        if is_user { "user" } else { "group" },
                        name.as_str()
                    ),
                    position,
                )),
            };
        }
        self.process_id_argument(argument, position)
    }
}

/// The Errno class a number stands for, named the way Ruby names it.
fn errno_constant(code: i32) -> &'static str {
    match code {
        libc::EPERM => "Errno::EPERM",
        libc::ESRCH => "Errno::ESRCH",
        libc::EACCES => "Errno::EACCES",
        libc::EINVAL => "Errno::EINVAL",
        libc::ENOENT => "Errno::ENOENT",
        libc::ECHILD => "Errno::ECHILD",
        _ => "SystemCallError",
    }
}

/// The supplementary groups this process belongs to, which is what decides
/// whether a file it does not own is still one of its group's.
fn current_groups() -> Object {
    // The list the account belongs to, which is what `id -G` reports. The
    // one the kernel caches for the process is cut off at sixteen entries on
    // some systems, so the account database answers first.
    if let Some(named) = account_groups() {
        return Object::array(named.iter().map(|group| Object::Int(*group)).collect());
    }
    let mut held = [0 as libc::gid_t; 64];
    // SAFETY: `getgroups` fills at most the count it is given and reports how
    // many it wrote.
    let written = unsafe { libc::getgroups(held.len() as i32, held.as_mut_ptr()) };
    let counted = if written < 0 { 0 } else { written as usize };
    Object::array(
        held[..counted]
            .iter()
            .map(|group| Object::Int(*group as i64))
            .collect(),
    )
}

/// Every group the account this process runs as belongs to, read from the
/// account database rather than from the process's own cached list.
pub(crate) fn account_groups() -> Option<Vec<i64>> {
    // SAFETY: `getpwuid` answers a pointer into a static the library owns,
    // and `getgrouplist` fills the array it is handed up to the count it is
    // told, reporting how many the account has.
    unsafe {
        let account = libc::getpwuid(libc::getuid());
        if account.is_null() {
            return None;
        }
        let mut held = vec![0 as libc::gid_t; 256];
        let mut counted = held.len() as libc::c_int;
        let answered = libc::getgrouplist(
            (*account).pw_name,
            (*account).pw_gid as _,
            held.as_mut_ptr() as _,
            &mut counted,
        );
        if answered < 0 || counted <= 0 {
            return None;
        }
        held.truncate(counted as usize);
        Some(held.into_iter().map(|group| group as i64).collect())
    }
}

/// Where the C library keeps the number of the last failure. Each system
/// names the function that answers it differently.
#[cfg(not(target_os = "linux"))]
unsafe fn errno_location() -> *mut libc::c_int {
    unsafe { libc::__error() }
}

#[cfg(target_os = "linux")]
unsafe fn errno_location() -> *mut libc::c_int {
    unsafe { libc::__errno_location() }
}

/// Write a title over the arguments a process was started with, which is
/// where a process listing reads its command from. What the title does not
/// fill is cleared, and one longer than the area is cut to fit.
///
/// # Safety
/// `start` must point at `length` writable bytes.
unsafe fn write_process_title(start: *mut u8, length: usize, title: &[u8]) {
    if length == 0 {
        return;
    }
    let kept = title.len().min(length - 1);
    // SAFETY: the caller vouches for `length` bytes from `start`.
    unsafe {
        std::ptr::copy_nonoverlapping(title.as_ptr(), start, kept);
        std::ptr::write_bytes(start.add(kept), 0, length - kept);
    }
}

/// Where the arguments this process was started with sit, and how many bytes
/// they span: the first argument through the end of the last one that
/// follows on from it.
#[cfg(any(target_os = "macos", all(target_os = "linux", target_env = "gnu")))]
fn original_argument_area() -> Option<(*mut u8, usize)> {
    let (count, arguments) = original_arguments()?;
    // SAFETY: the array holds `count` pointers to the strings the process was
    // started with, which live as long as the process does.
    unsafe {
        if count == 0 || arguments.is_null() || (*arguments).is_null() {
            return None;
        }
        let start = *arguments as *mut u8;
        let mut end = start.add(libc::strlen(*arguments) + 1);
        for index in 1..count {
            let next = *arguments.add(index) as *mut u8;
            if next != end {
                break;
            }
            end = next.add(libc::strlen(next as *const libc::c_char) + 1);
        }
        Some((start, end.offset_from(start) as usize))
    }
}

#[cfg(not(any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))))]
fn original_argument_area() -> Option<(*mut u8, usize)> {
    None
}

/// How many arguments the process was started with, and the array of them.
#[cfg(target_os = "macos")]
fn original_arguments() -> Option<(usize, *const *const libc::c_char)> {
    // SAFETY: both calls answer pointers into the process's own start-up
    // state, which lives as long as the process does.
    unsafe {
        Some((
            *libc::_NSGetArgc() as usize,
            *libc::_NSGetArgv() as *const *const libc::c_char,
        ))
    }
}

/// How many arguments the process was started with, and the array of them,
/// as glibc handed them to the functions it runs before `main`.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn original_arguments() -> Option<(usize, *const *const libc::c_char)> {
    let count = STARTED_WITH_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    let arguments = STARTED_WITH.load(std::sync::atomic::Ordering::Relaxed);
    Some((count, arguments as *const *const libc::c_char))
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
static STARTED_WITH_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(all(target_os = "linux", target_env = "gnu"))]
static STARTED_WITH: std::sync::atomic::AtomicPtr<*const libc::c_char> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// glibc calls each function in `.init_array` with the arguments the process
/// was started with, which is the only place a Linux process is handed the
/// array itself rather than a copy.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
#[used]
#[unsafe(link_section = ".init_array")]
static KEEP_STARTED_WITH: extern "C" fn(
    libc::c_int,
    *const *const libc::c_char,
    *const *const libc::c_char,
) = keep_started_with;

#[cfg(all(target_os = "linux", target_env = "gnu"))]
extern "C" fn keep_started_with(
    count: libc::c_int,
    arguments: *const *const libc::c_char,
    _environment: *const *const libc::c_char,
) {
    STARTED_WITH_COUNT.store(count as usize, std::sync::atomic::Ordering::Relaxed);
    STARTED_WITH.store(
        arguments as *mut *const libc::c_char,
        std::sync::atomic::Ordering::Relaxed,
    );
}
