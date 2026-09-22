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
        let program = self.get_string_representation(command, position)?;
        let mut rest = Vec::new();
        for argument in &given[1..] {
            rest.push(self.get_string_representation(argument, position)?);
        }
        let mut running = if rest.is_empty() {
            let mut shell = std::process::Command::new("/bin/sh");
            shell.arg("-c").arg(&program);
            shell
        } else {
            let mut named = std::process::Command::new(&program);
            named.args(&rest);
            named
        };
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
            && let Some(Object::String(directory)) = settings.get(":chdir")
        {
            running.current_dir(directory.as_str().to_string());
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
        match running.spawn() {
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

    /// `fork` splits the process. The child answers nil, or runs the
    /// block and exits with its status; the parent answers the child's
    /// process id either way.
    pub(crate) fn fork_process(&mut self, position: Position) -> Result<Object, MetorexError> {
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        let block = self.pending_block.take();
        // SAFETY: `fork` is called with no other threads running, and
        // the child does nothing but run the block and exit.
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
        // Only the thread that called `fork` survives into the child,
        // so every other one is marked finished there.
        for thread in std::mem::take(&mut self.pending_threads) {
            if let Object::Instance(instance) = thread {
                instance
                    .borrow_mut()
                    .set_var("__thread_value".to_string(), Object::Nil);
            }
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
