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
        let output = match spawned {
            Ok(child) => {
                let child_pid = child.id() as i64;
                child.wait_with_output().map(|output| (output, child_pid))
            }
            Err(error) => Err(error),
        };
        let (output, child_pid) = match output {
            Ok(pair) => pair,
            Err(error) => {
                let message = format!("No such file or directory - {} ({})", command, error);
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("Errno::ENOENT", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                });
            }
        };
        self.record_last_status(&output.status, Some(child_pid));
        // A command the shell could not find ends with status 127,
        // which Ruby reports as Errno::ENOENT from the spawn itself.
        if output.status.code() == Some(127) {
            let message = format!("No such file or directory - {}", command);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("Errno::ENOENT", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        Ok(self.command_output(&output.stdout))
    }

    /// `exec` replaces this process with the command, so nothing after
    /// it runs. A command the shell cannot find raises Errno::ENOENT.
    pub(crate) fn exec_program(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(first) = arguments.first() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                0,
                position,
            ));
        };
        let program = self.coerce_command_argument(first, position)?;
        let mut rest = Vec::new();
        for argument in arguments.iter().skip(1) {
            rest.push(self.coerce_command_argument(argument, position)?);
        }
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        // A command with nothing for the shell to do is run directly,
        // so a missing program is reported as ENOENT rather than
        // becoming the shell's own "command not found" exit.
        let needs_shell = rest.is_empty()
            && program.contains(|c: char| " \t\n|&;<>()$`\\\"'*?[]#~=%".contains(c));
        // A program that cannot be found is refused before anything
        // is started. Leaving it to the spawn reports differently
        // from one platform to the next: some hand back the error,
        // and some start a child that exits 127.
        if !needs_shell && findable_program(&program).is_none() {
            let message = format!("No such file or directory - {}", program);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("Errno::ENOENT", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        let status = if !rest.is_empty() {
            std::process::Command::new(&program).args(&rest).status()
        } else if needs_shell {
            std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&program)
                .status()
        } else {
            std::process::Command::new(&program).status()
        };
        match status {
            Ok(status) => std::process::exit(status.code().unwrap_or(0)),
            Err(_) => {
                let message = format!("No such file or directory - {}", program);
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("Errno::ENOENT", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
        }
    }

    /// `spawn` starts a command and answers its process id without
    /// waiting for it, which is what `Process.wait` is then given.
    pub(crate) fn spawn_program(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A Hash in front names the environment the child runs with,
        // and one at the back names how it is run rather than what it
        // is run with.
        let mut given: &[Object] = &arguments;
        let mut environment = None;
        if let Some(Object::Dict(entries)) = given.first() {
            environment = Some(entries.borrow().clone());
            given = &given[1..];
        }
        let mut settings = None;
        if given.len() > 1
            && let Some(Object::Dict(entries)) = given.last()
        {
            settings = Some(entries.borrow().clone());
            given = &given[..given.len() - 1];
        }
        let Some(command) = given.first() else {
            return Err(MetorexError::runtime_error(
                "spawn requires at least 1 argument".to_string(),
                crate::vm::utils::position_to_location(position),
            ));
        };
        // `[program, argv0]` names the program and what it is told it is
        // called.
        let (command, called) = match command {
            Object::Array(pair) if pair.borrow().len() == 2 => {
                let pair = pair.borrow();
                (
                    pair[0].clone(),
                    Some(self.get_string_representation(&pair[1], position)?),
                )
            }
            other => (other.clone(), None),
        };
        let program = self.get_string_representation(&command, position)?;
        let mut rest = Vec::new();
        for argument in &given[1..] {
            rest.push(self.get_string_representation(argument, position)?);
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
        if let Some(environment) = environment {
            for (name, value) in environment.iter() {
                let name = name.trim_start_matches(':');
                match value {
                    Object::Nil => {
                        running.env_remove(name);
                    }
                    held => {
                        running.env(name, self.get_string_representation(held, position)?);
                    }
                }
            }
        }
        if let Some(settings) = &settings
            && let Some(directory) = settings.get(":chdir")
        {
            let file = self.globals().get("File").unwrap_or(Object::Nil);
            let named = self.send_to_object(file, "path", vec![directory.clone()], position)?;
            running.current_dir(self.get_string_representation(&named, position)?);
        }
        // The files a redirection opens stay open until the child has been
        // started, since the child takes its copies of them then.
        let mut held_open: Vec<std::fs::File> = Vec::new();
        if let Some(settings) = &settings {
            let moves = self.spawn_redirections(settings, &mut held_open, position)?;
            if !moves.is_empty() {
                use std::os::unix::process::CommandExt;
                // SAFETY: the child only calls `dup2` between the fork and
                // the exec, which is what redirecting its streams takes.
                unsafe {
                    running.pre_exec(move || {
                        for (target, source) in &moves {
                            let from = match source {
                                RedirectSource::Descriptor(number) => *number,
                                RedirectSource::ChildStream(number) => *number,
                            };
                            if libc::dup2(from, *target) < 0 {
                                return Err(std::io::Error::last_os_error());
                            }
                            // A child's own streams wait for room or for
                            // input, whatever the end it was handed did.
                            let flags = libc::fcntl(*target, libc::F_GETFL);
                            libc::fcntl(*target, libc::F_SETFL, flags & !libc::O_NONBLOCK);
                        }
                        Ok(())
                    });
                }
            }
        }
        // `pgroup: true` starts the child in a process group of its
        // own, which is what keeps a signal to this group from
        // reaching it.
        let own_group = matches!(
            settings.as_ref().and_then(|held| held.get(":pgroup")),
            Some(Object::Bool(true)) | Some(Object::Int(0))
        );
        if own_group {
            use std::os::unix::process::CommandExt;
            // SAFETY: the child calls `setpgid` on itself between the
            // fork and the exec, which is what it is for.
            unsafe {
                running.pre_exec(|| {
                    libc::setpgid(0, 0);
                    Ok(())
                });
            }
        }
        let started = running.spawn();
        drop(held_open);
        match started {
            Ok(child) => Ok(Object::Int(i64::from(child.id()))),
            Err(problem) => {
                let message = format!("No such file or directory - {program} ({problem})");
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("Errno::ENOENT", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
        }
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
        let (status, pid) = run_to_completion(&reached, &words, &redirects);
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

/// Where a redirected stream of a spawned child comes from: a descriptor this
/// process holds, or another of the child's own streams.
#[derive(Clone, Copy)]
enum RedirectSource {
    Descriptor(libc::c_int),
    ChildStream(libc::c_int),
}

/// The stream a redirection names, as `:in`, `:out`, `:err` or a number.
fn redirected_stream(named: &str) -> Option<libc::c_int> {
    match named.trim_start_matches(':') {
        "in" => Some(0),
        "out" => Some(1),
        "err" => Some(2),
        number => number.parse().ok(),
    }
}

impl VirtualMachine {
    /// The moves a spawned child makes to its streams before it runs, in the
    /// order they are made: a file, a stream of this process, or a descriptor
    /// by number first, and another of the child's own streams last, since
    /// that one is read once the others are in place.
    fn spawn_redirections(
        &mut self,
        settings: &indexmap::IndexMap<String, Object>,
        held_open: &mut Vec<std::fs::File>,
        position: Position,
    ) -> Result<Vec<(libc::c_int, RedirectSource)>, MetorexError> {
        use std::os::fd::AsRawFd as _;
        let mut moves = Vec::new();
        let mut last = Vec::new();
        for (key, value) in settings.iter() {
            let Some(target) = redirected_stream(key) else {
                continue;
            };
            let source = match value {
                Object::Int(number) => RedirectSource::Descriptor(*number as libc::c_int),
                Object::String(path) => {
                    let opened = open_redirect_file(&path.as_str(), target, None)
                        .map_err(|problem| redirect_error(&path.as_str(), &problem, position))?;
                    let number = opened.as_raw_fd();
                    held_open.push(opened);
                    RedirectSource::Descriptor(number)
                }
                Object::Array(parts) => {
                    let parts = parts.borrow().clone();
                    match parts.first() {
                        Some(Object::Symbol(child)) if &*child.as_str() == "child" => {
                            let Some(Object::Symbol(other)) = parts.get(1) else {
                                continue;
                            };
                            let Some(number) = redirected_stream(&other.as_str()) else {
                                continue;
                            };
                            last.push((target, RedirectSource::ChildStream(number)));
                            continue;
                        }
                        Some(Object::String(path)) => {
                            let mode = match parts.get(1) {
                                Some(Object::String(mode)) => Some(mode.as_str().to_string()),
                                _ => None,
                            };
                            let opened =
                                open_redirect_file(&path.as_str(), target, mode.as_deref())
                                    .map_err(|problem| {
                                        redirect_error(&path.as_str(), &problem, position)
                                    })?;
                            let number = opened.as_raw_fd();
                            held_open.push(opened);
                            RedirectSource::Descriptor(number)
                        }
                        _ => continue,
                    }
                }
                // A stream of this process lends the child its descriptor.
                other if self.responds_to(other, "fileno") => {
                    match self.send_to_object(other.clone(), "fileno", vec![], position)? {
                        Object::Int(number) => RedirectSource::Descriptor(number as libc::c_int),
                        _ => continue,
                    }
                }
                _ => continue,
            };
            moves.push((target, source));
        }
        moves.extend(last);
        Ok(moves)
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
