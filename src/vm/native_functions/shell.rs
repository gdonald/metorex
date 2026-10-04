// Running another program.

use super::*;

impl VirtualMachine {
    /// Kernel#` — run the command through the shell, answering what it
    /// wrote to stdout. Its stderr passes through to ours, and `$?`
    /// reports how it ended.
    pub(crate) fn shell_command(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(argument) = arguments.first() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Exact(1),
                0,
                position,
            ));
        };
        let command = self.coerce_command_argument(argument, position)?;
        // The shell reads bytes, so a command whose characters stand
        // for bytes is handed those rather than their text form.
        let written: std::ffi::OsString = match argument {
            Object::String(text) if text.holds_bytes() => {
                use std::os::unix::ffi::OsStringExt;
                std::ffi::OsString::from_vec(
                    crate::vm::native_methods::string_methods::binary_bytes(text),
                )
            }
            _ => std::ffi::OsString::from(command.clone()),
        };
        // Spawn rather than run to completion in one step, so the
        // child's process id is read before it is waited for and
        // `$?.pid` can report it.
        let spawned = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&written)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => {
                let message = format!("No such file or directory - {} ({})", command, error);
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("Errno::ENOENT", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                });
            }
        };
        let child_pid = libc::pid_t::try_from(child.id()).unwrap_or(-1);
        let written = match child.stdout.take() {
            Some(stdout) => self.read_handing_turns(stdout, position)?,
            None => Vec::new(),
        };
        let mut held: libc::c_int = 0;
        self.waitpid_handing_turns(child_pid, &mut held, 0, position)?;
        use std::os::unix::process::ExitStatusExt as _;
        let status = std::process::ExitStatus::from_raw(held);
        self.record_last_status(&status, Some(i64::from(child_pid)));
        // A command the shell could not find ends with status 127,
        // which Ruby reports as Errno::ENOENT from the spawn itself.
        if status.code() == Some(127) {
            let message = format!("No such file or directory - {}", command);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("Errno::ENOENT", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        Ok(self.command_output(&written))
    }

    /// `exec` replaces this process with the command, so nothing after it
    /// runs. What stops the command from starting is raised here, and this
    /// process carries on.
    pub(crate) fn exec_program(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut prepared = self.prepare_command(arguments, position)?;
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        use std::os::unix::process::CommandExt as _;
        // `exec` puts SIGPIPE back to its default and clears the signal mask
        // in this process before it replaces it, so a command that never
        // starts leaves both to be put back.
        let held_signals = SignalSettings::current();
        let problem = prepared.command.exec();
        held_signals.restore();
        drop(prepared.held_open);
        Err(start_error(&prepared.program, &problem, position))
    }

    /// `spawn` starts a command and answers its process id without
    /// waiting for it, which is what `Process.wait` is then given.
    pub(crate) fn spawn_program(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut prepared = self.prepare_command(arguments, position)?;
        let started = prepared.command.spawn();
        drop(prepared.held_open);
        match started {
            Ok(child) => Ok(Object::Int(i64::from(child.id()))),
            Err(problem) => Err(start_error(&prepared.program, &problem, position)),
        }
    }

    /// The command `exec` and `spawn` start, built from the arguments they
    /// share: an environment Hash in front, the command itself, the words it
    /// is given, and a Hash at the back saying how it is run.
    fn prepare_command(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<PreparedCommand, MetorexError> {
        let mut given: &[Object] = &arguments;
        let mut environment = None;
        if let Some(first) = given.first()
            && (matches!(first, Object::Dict(_)) || self.responds_to(first, "to_hash"))
        {
            let entries = match first {
                Object::Dict(entries) => entries.borrow().clone(),
                other => match self.send_to_object(other.clone(), "to_hash", vec![], position)? {
                    Object::Dict(entries) => entries.borrow().clone(),
                    _ => indexmap::IndexMap::new(),
                },
            };
            environment = Some(entries);
            given = &given[1..];
        }
        let mut settings = None;
        if let Some(Object::Dict(entries)) = given.last() {
            settings = Some(entries.borrow().clone());
            given = &given[..given.len() - 1];
        }
        let Some(command) = given.first() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                0,
                position,
            ));
        };
        // `[program, argv0]` names the program and what it is told it is
        // called, and an object answering `to_ary` stands for one.
        let pair = match command {
            Object::Array(pair) => Some(pair.borrow().clone()),
            Object::String(_) => None,
            other if self.responds_to(other, "to_ary") => {
                match self.send_to_object(other.clone(), "to_ary", vec![], position)? {
                    Object::Array(pair) => Some(pair.borrow().clone()),
                    _ => None,
                }
            }
            _ => None,
        };
        let (command, called) = match pair {
            Some(pair) if pair.len() == 2 => (
                pair[0].clone(),
                Some(self.command_word(&pair[1], position)?),
            ),
            Some(_) => {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "wrong first argument",
                    position,
                ));
            }
            None => (command.clone(), None),
        };
        let program = self.command_word(&command, position)?;
        let mut rest = Vec::new();
        for argument in &given[1..] {
            rest.push(self.command_argument(argument, position)?);
        }
        // An empty command names no program to start. One of only spaces is
        // left for the shell, which refuses it itself.
        if program.is_empty() && rest.is_empty() && called.is_none() {
            return Err(start_error(
                &program,
                &std::io::Error::from_raw_os_error(libc::ENOENT),
                position,
            ));
        }
        // A single command that the shell has nothing to read in runs as the
        // program it names, the way Ruby runs it.
        let plain = if rest.is_empty() && called.is_none() {
            shell_free_words(&program)
        } else {
            None
        };
        let mut running = match plain {
            Some(words) => {
                let mut named = std::process::Command::new(&words[0]);
                named.args(&words[1..]);
                named
            }
            None if rest.is_empty() && called.is_none() => {
                let mut shell = std::process::Command::new("/bin/sh");
                shell.arg("-c").arg(&program);
                shell
            }
            None => {
                let mut named = std::process::Command::new(&program);
                named.args(&rest);
                named
            }
        };
        if let Some(called) = called {
            use std::os::unix::process::CommandExt as _;
            running.arg0(called);
        }
        let settings = match &settings {
            Some(entries) => self.spawn_settings(entries, position)?,
            None => SpawnSettings::default(),
        };
        // Every other variable is dropped before the ones named are set.
        if settings.unsetenv_others {
            running.env_clear();
        }
        if let Some(environment) = environment {
            for (slot, value) in environment.iter() {
                if crate::vm::native_methods::hash_methods::is_internal_key(slot) {
                    continue;
                }
                let key =
                    crate::vm::native_methods::hash_methods::reconstruct_key(&environment, slot);
                let name = self.environment_word(&key, position)?;
                if name.contains('=') {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("environment name contains a equal : {name}"),
                        position,
                    ));
                }
                match value {
                    Object::Nil => {
                        running.env_remove(&name);
                    }
                    held => {
                        let value = self.environment_word(held, position)?;
                        running.env(&name, value);
                    }
                }
            }
        }
        if let Some(directory) = &settings.chdir {
            running.current_dir(directory);
        }
        let SpawnSettings {
            mut moves,
            held_open,
            umask,
            pgroup,
            close_others,
            limits,
            user,
            group,
            ..
        } = settings;
        use std::os::unix::process::CommandExt as _;
        // SAFETY: only system calls that are safe between a fork and an exec
        // run here: moving, closing and flagging descriptors, and setting the
        // mask, group, limits and ids the child starts with.
        unsafe {
            running.pre_exec(move || {
                // A source is read as this process holds it, so one that
                // another move replaces is copied out of the way first, to a
                // number above every target that closes at the exec.
                let above_targets = moves.iter().map(|(target, _)| *target).max().unwrap_or(2) + 1;
                for index in 0..moves.len() {
                    if let RedirectSource::Descriptor(from) = moves[index].1
                        && from != moves[index].0
                        && moves.iter().any(|(target, _)| *target == from)
                    {
                        let copy = libc::fcntl(from, libc::F_DUPFD_CLOEXEC, above_targets);
                        if copy < 0 {
                            return Err(std::io::Error::last_os_error());
                        }
                        moves[index].1 = RedirectSource::Descriptor(copy);
                    }
                }
                for (target, source) in &moves {
                    match source {
                        RedirectSource::Close => {
                            libc::close(*target);
                            continue;
                        }
                        RedirectSource::Descriptor(from) | RedirectSource::ChildStream(from)
                            if from != target && libc::dup2(*from, *target) < 0 =>
                        {
                            return Err(std::io::Error::last_os_error());
                        }
                        _ => {}
                    }
                    // A descriptor handed on under its own number is one
                    // `dup2` leaves as it was, so it is told to stay open
                    // across the exec.
                    let descriptor_flags = libc::fcntl(*target, libc::F_GETFD);
                    libc::fcntl(*target, libc::F_SETFD, descriptor_flags & !libc::FD_CLOEXEC);
                    // A child's own streams wait for room or for input,
                    // whatever the end it was handed did.
                    let flags = libc::fcntl(*target, libc::F_GETFL);
                    libc::fcntl(*target, libc::F_SETFL, flags & !libc::O_NONBLOCK);
                }
                // A descriptor left open across the exec is closed unless a
                // redirection named it. One marked to close on exec closes
                // then anyway, which is what keeps the pipe the exec reports
                // its failure through open until it has.
                if close_others {
                    for descriptor in 3..highest_descriptor() {
                        if moves.iter().any(|(target, _)| *target == descriptor) {
                            continue;
                        }
                        let descriptor_flags = libc::fcntl(descriptor, libc::F_GETFD);
                        if descriptor_flags >= 0 && descriptor_flags & libc::FD_CLOEXEC == 0 {
                            libc::close(descriptor);
                        }
                    }
                }
                if let Some(mask) = umask {
                    libc::umask(mask as libc::mode_t);
                }
                if let Some(leader) = pgroup
                    && libc::setpgid(0, leader) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                for (resource, current, maximum) in &limits {
                    let limit = libc::rlimit {
                        rlim_cur: *current as libc::rlim_t,
                        rlim_max: *maximum as libc::rlim_t,
                    };
                    if libc::setrlimit(*resource as _, &limit) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                if let Some(group) = group
                    && libc::setgid(group) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                if let Some(user) = user
                    && libc::setuid(user) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        Ok(PreparedCommand {
            command: running,
            held_open,
            program,
        })
    }

    /// One word of a command, which the operating system reads as a C
    /// string, so it may not hold a null byte.
    /// An argument handed to a program, as the bytes its String holds, so
    /// text in an encoding of its own reaches the program unchanged.
    fn command_argument(
        &mut self,
        word: &Object,
        position: Position,
    ) -> Result<std::ffi::OsString, MetorexError> {
        let text = self.command_word(word, position)?;
        if let Object::String(held) = word {
            use std::os::unix::ffi::OsStringExt;
            return Ok(std::ffi::OsString::from_vec(
                crate::vm::native_methods::string_methods::binary_bytes(held),
            ));
        }
        Ok(std::ffi::OsString::from(text))
    }

    fn command_word(&mut self, word: &Object, position: Position) -> Result<String, MetorexError> {
        let text = self.implicit_string(word, position)?;
        if text.contains('\0') {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "string contains null byte",
                position,
            ));
        }
        Ok(text)
    }

    /// A String, or what an object answering `to_str` hands back for one.
    fn implicit_string(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match self.implicit_conversion(given, "to_str", position)? {
            Some(Object::String(text)) => Ok(text.as_str().to_string()),
            _ => Err(self.conversion_error(given, "String", position)),
        }
    }

    /// An Integer, or what an object answering `to_int` hands back for one.
    fn implicit_integer(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        match self.implicit_conversion(given, "to_int", position)? {
            Some(Object::Int(number)) => Ok(number),
            _ => Err(self.conversion_error(given, "Integer", position)),
        }
    }

    /// The value itself when it is already of the kind wanted, and otherwise
    /// what the conversion method answers, if the value has one.
    fn implicit_conversion(
        &mut self,
        given: &Object,
        conversion: &str,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let already = match conversion {
            "to_str" => matches!(given, Object::String(_)),
            _ => matches!(given, Object::Int(_)),
        };
        if already {
            return Ok(Some(given.clone()));
        }
        if !self.responds_to(given, conversion) {
            return Ok(None);
        }
        self.send_to_object(given.clone(), conversion, vec![], position)
            .map(Some)
    }

    /// The TypeError a value that is not of the kind wanted raises.
    fn conversion_error(&mut self, given: &Object, kind: &str, position: Position) -> MetorexError {
        let class = self.builtins().class_of(given).name().to_string();
        crate::vm::errors::simple_exception(
            "TypeError",
            &format!("no implicit conversion of {class} into {kind}"),
            position,
        )
    }

    /// A name or a value in the environment a command is given, which is a
    /// C string the same way a word of the command is.
    fn environment_word(
        &mut self,
        word: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        self.command_word(word, position)
    }

    pub(crate) fn system_command(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(command) = arguments.first() else {
            return Err(MetorexError::runtime_error(
                "system requires at least 1 argument".to_string(),
                crate::vm::utils::position_to_location(position),
            ));
        };
        let program = self.get_string_representation(command, position)?;
        // A trailing Hash names where the child's streams go and
        // whether a failure is raised rather than reported.
        let mut given = &arguments[1..];
        let mut options = None;
        if let Some(Object::Dict(entries)) = given.last() {
            options = Some(entries.borrow().clone());
            given = &given[..given.len() - 1];
        }
        let mut rest = Vec::new();
        for arg in given {
            rest.push(self.get_string_representation(arg, position)?);
        }
        let mut raises = false;
        let mut redirects: Vec<(i32, String)> = Vec::new();
        if let Some(options) = &options {
            for (name, target) in options.iter() {
                match name.trim_start_matches(':') {
                    "exception" => raises = target.is_truthy(),
                    "out" => {
                        if let Object::String(path) = target {
                            redirects.push((1, path.as_str().to_string()));
                        }
                    }
                    "err" => {
                        if let Object::String(path) = target {
                            redirects.push((2, path.as_str().to_string()));
                        }
                    }
                    _ => {}
                }
            }
        }
        // One string holding a character the shell reads runs through
        // the shell, and anything else runs as the program it names.
        let mut reached = None;
        let words: Vec<String> = if !rest.is_empty() {
            let mut held = vec![program.clone()];
            held.extend(rest);
            held
        } else if needs_a_shell(&program) {
            // The shell is reached by its path and told its name is
            // `sh`, which is the name `$0` answers inside it.
            reached = Some("/bin/sh".to_string());
            vec!["sh".to_string(), "-c".to_string(), program.clone()]
        } else {
            program
                .split_whitespace()
                .map(|held| held.to_string())
                .collect()
        };
        if words.is_empty() {
            return Ok(Object::Nil);
        }
        let reached = reached.unwrap_or_else(|| words[0].clone());
        let (status, pid) = self.run_to_completion(&reached, &words, &redirects, position)?;
        self.record_last_status(&status, Some(pid));
        let code = status.code().unwrap_or(-1);
        if raises && code != 0 {
            // A child that never reached the program reports it the
            // way the operating system does.
            if code == 127 {
                let message = format!("No such file or directory - {program}");
                return Err(crate::vm::errors::simple_exception(
                    "Errno::ENOENT",
                    &message,
                    position,
                ));
            }
            let message = format!("Command failed with exit {code}: {}", words.join(" "));
            return Err(crate::vm::errors::simple_exception(
                "RuntimeError",
                &message,
                position,
            ));
        }
        // A command that never ran at all is reported as nothing
        // rather than as a failure.
        if code == 127 {
            return Ok(Object::Nil);
        }
        Ok(Object::Bool(status.success()))
    }

    /// Run a program to completion, answering how it ended and the process
    /// id it ran under.
    fn run_to_completion(
        &mut self,
        reached: &str,
        words: &[String],
        redirects: &[(i32, String)],
        position: Position,
    ) -> Result<(std::process::ExitStatus, i64), MetorexError> {
        use std::os::unix::process::ExitStatusExt as _;
        let child = start_program(reached, words, redirects);
        if child < 0 {
            return Ok((std::process::ExitStatus::from_raw(127 << 8), 0));
        }
        let mut held: libc::c_int = 0;
        self.waitpid_handing_turns(child, &mut held, 0, position)?;
        Ok((std::process::ExitStatus::from_raw(held), i64::from(child)))
    }

    /// `fork` splits the process through `Process._fork`. The child answers
    /// nil, or runs the block and exits with its status; the parent answers
    /// what `_fork` answered, which is the child's process id.
    pub(crate) fn fork_process(&mut self, position: Position) -> Result<Object, MetorexError> {
        use std::io::Write as _;
        let block = self.pending_block.take();
        let process = self.globals().get("Process").unwrap_or(Object::Nil);
        let answered = self.send_to_object(process, "_fork", vec![], position)?;
        if !matches!(answered, Object::Int(0)) {
            return Ok(answered);
        }
        // In the child. Without a block, `fork` answers nil and the
        // caller carries on as the child.
        let Some(Object::Block(block)) = block else {
            return Ok(Object::Nil);
        };
        let outcome = self.execute_block_callable(&block, Vec::new(), position);
        let _ = std::io::stdout().flush();
        let status = match outcome {
            Ok(_) => 0,
            Err(MetorexError::UncaughtException {
                exception: Object::Exception(details),
                ..
            }) if details.borrow().is_system_exit() => details.borrow().status.unwrap_or(0) as i32,
            Err(error) => {
                eprintln!("{}", error);
                1
            }
        };
        std::process::exit(status);
    }
}

impl VirtualMachine {
    /// `Process._fork` splits the process, answering the child's process id
    /// in the parent and 0 in the child.
    pub(crate) fn split_process(&mut self, position: Position) -> Result<Object, MetorexError> {
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        // SAFETY: the interpreter is the only thread that runs Ruby code, and
        // the child carries on with nothing but that thread.
        let child = unsafe { libc::fork() };
        if child < 0 {
            let message = "fork failed".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("Errno::EAGAIN", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        if child > 0 {
            return Ok(Object::Int(child as i64));
        }
        // Only the thread that called `fork` survives into the child, so
        // every other one is marked finished there.
        for thread in std::mem::take(&mut self.pending_threads) {
            if let Object::Instance(instance) = thread {
                instance
                    .borrow_mut()
                    .set_var("__thread_value".to_string(), Object::Nil);
            }
        }
        Ok(Object::Int(0))
    }
}

impl VirtualMachine {
    /// `Process.daemon` forks, and the parent leaves at once without running
    /// what `at_exit` registered. The child starts a session of its own, moves
    /// to `/` unless told to stay put, and reads and writes `/dev/null` in
    /// place of its standard streams unless told to keep them.
    pub(crate) fn detach_process(
        &mut self,
        stays_put: bool,
        keeps_streams: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !matches!(self.split_process(position)?, Object::Int(0)) {
            // SAFETY: `_exit` ends the parent without running its handlers,
            // which belong to the child now.
            unsafe { libc::_exit(0) };
        }
        // SAFETY: each call acts on this process alone: starting a session,
        // changing its directory, and moving its standard descriptors.
        unsafe {
            if libc::setsid() < 0 {
                return Err(crate::vm::errors::simple_exception(
                    "Errno::EPERM",
                    "Operation not permitted - setsid",
                    position,
                ));
            }
            if !stays_put {
                libc::chdir(c"/".as_ptr());
            }
            if !keeps_streams {
                let null = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
                if null >= 0 {
                    for descriptor in 0..3 {
                        libc::dup2(null, descriptor);
                    }
                    if null > 2 {
                        libc::close(null);
                    }
                }
            }
        }
        Ok(Object::Int(0))
    }
}

/// A command ready to start, and the files its redirections opened, which
/// stay open until it has.
struct PreparedCommand {
    command: std::process::Command,
    held_open: Vec<std::fs::File>,
    program: String,
}

/// The error a command that could not be started raises, named the way the
/// operating system named what stopped it.
fn start_error(program: &str, problem: &std::io::Error, position: Position) -> MetorexError {
    let (named, reason) = match problem.raw_os_error() {
        Some(code) if code == libc::EACCES => ("Errno::EACCES", "Permission denied"),
        Some(code) if code == libc::ENOEXEC => ("Errno::ENOEXEC", "Exec format error"),
        _ => ("Errno::ENOENT", "No such file or directory"),
    };
    crate::vm::errors::simple_exception(named, &format!("{reason} - {program}"), position)
}

/// Where a redirected stream of a spawned child comes from: a descriptor this
/// process holds, another of the child's own streams, or nowhere, which
/// closes it.
#[derive(Clone, Copy)]
enum RedirectSource {
    Descriptor(libc::c_int),
    ChildStream(libc::c_int),
    Close,
}

/// How a command is to be run, read from the Hash of options given to
/// `spawn` or `exec`.
#[derive(Default)]
struct SpawnSettings {
    chdir: Option<String>,
    unsetenv_others: bool,
    umask: Option<u32>,
    pgroup: Option<libc::pid_t>,
    close_others: bool,
    limits: Vec<(i32, u64, u64)>,
    user: Option<libc::uid_t>,
    group: Option<libc::gid_t>,
    /// The moves the child makes to its streams, in the order it makes them:
    /// files, descriptors and closes first, and another of its own streams
    /// last, since that one is read once the others are in place.
    moves: Vec<(libc::c_int, RedirectSource)>,
    /// The files a redirection opened, kept open until the child has its
    /// copies of them.
    held_open: Vec<std::fs::File>,
}

/// The limit a `:rlimit_` option names.
fn resource_named(name: &str) -> Option<i32> {
    Some(match name {
        "as" => libc::RLIMIT_AS as _,
        "core" => libc::RLIMIT_CORE as _,
        "cpu" => libc::RLIMIT_CPU as _,
        "data" => libc::RLIMIT_DATA as _,
        "fsize" => libc::RLIMIT_FSIZE as _,
        "memlock" => libc::RLIMIT_MEMLOCK as _,
        "nofile" => libc::RLIMIT_NOFILE as _,
        "nproc" => libc::RLIMIT_NPROC as _,
        "rss" => libc::RLIMIT_RSS as _,
        "stack" => libc::RLIMIT_STACK as _,
        _ => return None,
    })
}

/// One past the highest descriptor the child may hold open, which bounds the
/// descriptors `close_others` looks at.
fn highest_descriptor() -> libc::c_int {
    const MOST_LOOKED_AT: libc::c_int = 65536;
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `getrlimit` only writes the one struct given.
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) } < 0 {
        return MOST_LOOKED_AT;
    }
    limit.rlim_cur.min(MOST_LOOKED_AT as libc::rlim_t) as libc::c_int
}

impl VirtualMachine {
    /// Read the options Hash `spawn` and `exec` take. A Symbol names an
    /// option, and anything else names streams of the child to redirect.
    fn spawn_settings(
        &mut self,
        entries: &indexmap::IndexMap<String, Object>,
        position: Position,
    ) -> Result<SpawnSettings, MetorexError> {
        let mut settings = SpawnSettings::default();
        let mut last = Vec::new();
        let refuse = |message: String| {
            crate::vm::errors::simple_exception("ArgumentError", &message, position)
        };
        for (slot, value) in entries.iter() {
            if crate::vm::native_methods::hash_methods::is_internal_key(slot) {
                continue;
            }
            let key = crate::vm::native_methods::hash_methods::reconstruct_key(entries, slot);
            let targets: Vec<libc::c_int> = match &key {
                Object::Symbol(name) => match &*name.as_str() {
                    "in" => vec![0],
                    "out" => vec![1],
                    "err" => vec![2],
                    "chdir" => {
                        let file = self.globals().get("File").unwrap_or(Object::Nil);
                        let named =
                            self.send_to_object(file, "path", vec![value.clone()], position)?;
                        settings.chdir = Some(self.get_string_representation(&named, position)?);
                        continue;
                    }
                    "unsetenv_others" => {
                        settings.unsetenv_others = value.is_truthy();
                        continue;
                    }
                    "close_others" => {
                        settings.close_others = value.is_truthy();
                        continue;
                    }
                    "umask" => {
                        settings.umask = Some(self.implicit_integer(value, position)? as u32);
                        continue;
                    }
                    "pgroup" => {
                        settings.pgroup = match value {
                            Object::Nil | Object::Bool(false) => None,
                            Object::Bool(true) => Some(0),
                            Object::Int(number) if *number < 0 => {
                                return Err(refuse(format!(
                                    "negative process group ID : {number}"
                                )));
                            }
                            other => Some(self.implicit_integer(other, position)? as libc::pid_t),
                        };
                        continue;
                    }
                    "uid" => {
                        settings.user =
                            Some(self.implicit_integer(value, position)? as libc::uid_t);
                        continue;
                    }
                    "gid" => {
                        settings.group =
                            Some(self.implicit_integer(value, position)? as libc::gid_t);
                        continue;
                    }
                    named => {
                        let Some(resource) = named.strip_prefix("rlimit_").and_then(resource_named)
                        else {
                            return Err(refuse(format!("wrong exec option symbol: {named}")));
                        };
                        let (current, maximum) = match value {
                            Object::Array(pair) => {
                                let pair = pair.borrow().clone();
                                let current = self.implicit_integer(
                                    pair.first().unwrap_or(&Object::Nil),
                                    position,
                                )?;
                                let maximum = match pair.get(1) {
                                    Some(held) => self.implicit_integer(held, position)?,
                                    None => current,
                                };
                                (current as u64, maximum as u64)
                            }
                            other => {
                                let both = self.implicit_integer(other, position)? as u64;
                                (both, both)
                            }
                        };
                        settings.limits.push((resource, current, maximum));
                        continue;
                    }
                },
                Object::Int(number) => vec![*number as libc::c_int],
                Object::Array(named) => {
                    let named = named.borrow().clone();
                    let mut targets = Vec::new();
                    for one in &named {
                        targets.push(self.redirect_target(one, position)?);
                    }
                    targets
                }
                Object::String(_) => return Err(refuse("wrong exec option".to_string())),
                other => vec![self.redirect_target(other, position)?],
            };
            let Some(first) = targets.first().copied() else {
                continue;
            };
            let source = self.redirect_source(value, first, &mut settings.held_open, position)?;
            for target in targets {
                match source {
                    RedirectSource::ChildStream(_) => last.push((target, source)),
                    _ => settings.moves.push((target, source)),
                }
            }
        }
        settings.moves.extend(last);
        Ok(settings)
    }

    /// The stream of the child a redirection key names: `:in`, `:out` or
    /// `:err`, a descriptor number, or a stream of this process.
    fn redirect_target(
        &mut self,
        named: &Object,
        position: Position,
    ) -> Result<libc::c_int, MetorexError> {
        match named {
            Object::Symbol(name) => match &*name.as_str() {
                "in" => Ok(0),
                "out" => Ok(1),
                "err" => Ok(2),
                _ => Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    &format!("wrong exec redirect symbol: {}", name.as_str()),
                    position,
                )),
            },
            Object::Int(number) => Ok(*number as libc::c_int),
            other => self.stream_descriptor(other, position)?.ok_or_else(|| {
                crate::vm::errors::simple_exception("ArgumentError", "wrong exec option", position)
            }),
        }
    }

    /// Where a redirected stream comes from: a descriptor or a stream of this
    /// process, a file named, another of the child's own streams, or `:close`.
    fn redirect_source(
        &mut self,
        value: &Object,
        target: libc::c_int,
        held_open: &mut Vec<std::fs::File>,
        position: Position,
    ) -> Result<RedirectSource, MetorexError> {
        use std::os::fd::AsRawFd as _;
        let refuse = || {
            crate::vm::errors::simple_exception(
                "ArgumentError",
                "wrong exec redirect action",
                position,
            )
        };
        match value {
            Object::Int(number) => Ok(RedirectSource::Descriptor(*number as libc::c_int)),
            Object::Symbol(name) => match &*name.as_str() {
                "close" => Ok(RedirectSource::Close),
                "in" => Ok(RedirectSource::Descriptor(0)),
                "out" => Ok(RedirectSource::Descriptor(1)),
                "err" => Ok(RedirectSource::Descriptor(2)),
                _ => Err(refuse()),
            },
            Object::String(path) => {
                let opened = open_redirect_file(&path.as_str(), target, None)
                    .map_err(|problem| redirect_error(&path.as_str(), &problem, position))?;
                let number = opened.as_raw_fd();
                held_open.push(opened);
                Ok(RedirectSource::Descriptor(number))
            }
            Object::Array(parts) => {
                let parts = parts.borrow().clone();
                match parts.first() {
                    Some(Object::Symbol(child)) if &*child.as_str() == "child" => {
                        let Some(other) = parts.get(1) else {
                            return Err(refuse());
                        };
                        Ok(RedirectSource::ChildStream(
                            self.redirect_target(other, position)?,
                        ))
                    }
                    Some(Object::String(path)) => {
                        let mode = match parts.get(1) {
                            Some(Object::String(mode)) => Some(mode.as_str().to_string()),
                            _ => None,
                        };
                        let opened = open_redirect_file(&path.as_str(), target, mode.as_deref())
                            .map_err(|problem| {
                                redirect_error(&path.as_str(), &problem, position)
                            })?;
                        let number = opened.as_raw_fd();
                        held_open.push(opened);
                        Ok(RedirectSource::Descriptor(number))
                    }
                    _ => Err(refuse()),
                }
            }
            other => match self.stream_descriptor(other, position)? {
                Some(number) => Ok(RedirectSource::Descriptor(number)),
                None => Err(refuse()),
            },
        }
    }

    /// The descriptor behind a stream of this process, read through `fileno`
    /// or through the IO an object answering `to_io` stands for.
    fn stream_descriptor(
        &mut self,
        stream: &Object,
        position: Position,
    ) -> Result<Option<libc::c_int>, MetorexError> {
        let stream = if !self.responds_to(stream, "fileno") && self.responds_to(stream, "to_io") {
            self.send_to_object(stream.clone(), "to_io", vec![], position)?
        } else {
            stream.clone()
        };
        if !self.responds_to(&stream, "fileno") {
            return Ok(None);
        }
        match self.send_to_object(stream, "fileno", vec![], position)? {
            Object::Int(number) => Ok(Some(number as libc::c_int)),
            _ => Ok(None),
        }
    }
}

/// A file a redirection names, opened for reading when it stands for the
/// child's input and for writing otherwise, unless a mode says how.
fn open_redirect_file(
    path: &str,
    target: libc::c_int,
    mode: Option<&str>,
) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    match mode.unwrap_or(if target == 0 { "r" } else { "w" }) {
        "r" => options.read(true),
        "a" => options.append(true).create(true),
        "r+" => options.read(true).write(true),
        "w+" => options.read(true).write(true).create(true).truncate(true),
        _ => options.write(true).create(true).truncate(true),
    };
    options.open(path)
}

/// The error a redirection to a file that cannot be opened raises.
fn redirect_error(path: &str, problem: &std::io::Error, position: Position) -> MetorexError {
    let named = match problem.raw_os_error() {
        Some(code) if code == libc::EACCES => "Errno::EACCES",
        Some(code) if code == libc::EISDIR => "Errno::EISDIR",
        _ => "Errno::ENOENT",
    };
    crate::vm::errors::simple_exception(named, &format!("{problem} - {path}"), position)
}

/// The commands the shell answers itself, which have no program to run and so
/// have to go through `/bin/sh` however plainly they are written.
const SHELL_BUILTINS: &[&str] = &[
    "!", ".", ":", "break", "case", "continue", "do", "done", "elif", "else", "esac", "eval",
    "exec", "exit", "export", "fi", "for", "if", "in", "readonly", "return", "set", "shift",
    "then", "times", "trap", "unset", "until", "while",
];

/// The words of a command that needs no shell to run, or None when the shell
/// has to read it. Anything the shell would treat as more than a plain word
/// sends the command back through `/bin/sh`.
pub(crate) fn shell_free_words(command: &str) -> Option<Vec<String>> {
    const SHELL_CHARACTERS: &str = "*?{}[]<>()~&|\\$;'\"`\n#=";
    if command.contains(|character| SHELL_CHARACTERS.contains(character)) {
        return None;
    }
    let words: Vec<String> = command.split_whitespace().map(str::to_string).collect();
    let first = words.first()?;
    if SHELL_BUILTINS.contains(&first.as_str()) {
        return None;
    }
    Some(words)
}

/// The SIGPIPE disposition and the signal mask a process has.
struct SignalSettings {
    broken_pipe: libc::sighandler_t,
    mask: libc::sigset_t,
}

impl SignalSettings {
    fn current() -> Self {
        // SAFETY: the disposition is read by setting one and putting the old
        // one straight back, and the mask is read without being changed.
        unsafe {
            let broken_pipe = libc::signal(libc::SIGPIPE, libc::SIG_IGN);
            libc::signal(libc::SIGPIPE, broken_pipe);
            let mut mask: libc::sigset_t = std::mem::zeroed();
            libc::pthread_sigmask(libc::SIG_SETMASK, std::ptr::null(), &mut mask);
            SignalSettings { broken_pipe, mask }
        }
    }

    fn restore(&self) {
        // SAFETY: both were read from this process by `current`.
        unsafe {
            libc::signal(libc::SIGPIPE, self.broken_pipe);
            libc::pthread_sigmask(libc::SIG_SETMASK, &self.mask, std::ptr::null_mut());
        }
    }
}
