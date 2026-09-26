// The working directory, the scratch directory, and where a user's
// files live.

use super::*;

impl VirtualMachine {
    /// The working directory, the scratch directory, and where a user's
    /// files live.
    pub(crate) fn call_dir_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // ── Dir methods ─────────────────────────────────────────────────────
        if class_rc.name() == "Dir" && (method_name == "pwd" || method_name == "getwd") {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            return Ok(Some(Object::string(cwd)));
        }
        // `Dir.chdir(path)` changes the working directory. The block form
        // restores the previous one afterwards and answers the block's value.
        // `File.chmod(mode, *paths)` answers how many it changed.
        // `fnmatch` asks whether a name matches a glob pattern, without
        // looking at the file system at all.
        if class_rc.name() == "File" && matches!(method_name, "fnmatch" | "fnmatch?") {
            if arguments.len() < 2 || arguments.len() > 3 {
                return Err(crate::vm::errors::argument_count_error(
                    crate::vm::errors::Arity::Range(2, 3),
                    arguments.len(),
                    position,
                ));
            }
            let pattern = self.directory_path_argument(method_name, &arguments[0], position)?;
            let path = self.directory_path_argument(method_name, &arguments[1], position)?;
            let flags = match arguments.get(2) {
                None => 0,
                Some(held) => {
                    let asked = self.coerce_integer_argument(held, position)?;
                    i64::try_from(&asked).unwrap_or(0)
                }
            };
            return Ok(Some(Object::Bool(path_matches_pattern(
                &pattern, &path, flags,
            ))));
        }
        if class_rc.name() == "File" && method_name == "chmod" {
            if arguments.is_empty() {
                return Err(method_argument_error(
                    method_name,
                    2,
                    arguments.len(),
                    position,
                ));
            }
            let asked = self.coerce_integer_argument(&arguments[0], position)?;
            let Ok(mode) = i64::try_from(&asked) else {
                return Err(crate::vm::errors::simple_exception(
                    "RangeError",
                    "bignum too big to convert into `long'",
                    position,
                ));
            };
            let mut changed = 0;
            for named in arguments.iter().skip(1) {
                let path = self.directory_path_argument(method_name, named, position)?;
                use std::os::unix::fs::PermissionsExt as _;
                let permissions = std::fs::Permissions::from_mode(mode as u32);
                if let Err(error) = std::fs::set_permissions(&path, permissions) {
                    return Err(directory_error(&error, "chmod", &path, position));
                }
                changed += 1;
            }
            return Ok(Some(Object::Int(changed)));
        }
        // `mkfifo` makes a named pipe, which is a file every reader and
        // writer opens by name rather than inheriting across a fork.
        if class_rc.name() == "File" && method_name == "mkfifo" {
            if arguments.is_empty() || arguments.len() > 2 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            // A name may arrive as a String or as something that answers
            // `to_path`, and anything else is the wrong kind of argument.
            let path = match &arguments[0] {
                Object::String(path) => Rc::clone(path),
                other if self.responds_to(other, "to_path") => {
                    match self.send_to_object(other.clone(), "to_path", vec![], position)? {
                        Object::String(path) => path,
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                other,
                                position,
                            ));
                        }
                    }
                }
                other => {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        other,
                        position,
                    ));
                }
            };
            let mode = match arguments.get(1) {
                Some(Object::Int(mode)) => *mode as libc::mode_t,
                _ => 0o666,
            };
            let named =
                std::ffi::CString::new(path.as_str().as_bytes().to_vec()).map_err(|_| {
                    MetorexError::runtime_error(
                        format!("Invalid path - {}", path.as_str()),
                        position_to_location(position),
                    )
                })?;
            // SAFETY: `mkfifo` reads the name and the mode and touches
            // nothing else.
            let made = unsafe { libc::mkfifo(named.as_ptr(), mode) };
            if made != 0 {
                let problem = std::io::Error::last_os_error();
                let (class, wording) = match problem.raw_os_error() {
                    Some(libc::ENOENT) => ("Errno::ENOENT", "No such file or directory"),
                    Some(libc::EACCES) => ("Errno::EACCES", "Permission denied"),
                    Some(libc::EROFS) => ("Errno::EROFS", "Read-only file system"),
                    _ => ("Errno::EEXIST", "File exists"),
                };
                return Err(crate::vm::errors::simple_exception(
                    class,
                    &format!("{wording} @ file_mkfifo - {}: {problem}", path.as_str()),
                    position,
                ));
            }
            return Ok(Some(Object::Int(0)));
        }
        // `lchmod` changes the mode of a symlink itself rather than of what it
        // points at, which only some systems allow at all.
        if class_rc.name() == "File" && method_name == "lchmod" {
            let Some(Object::Int(mode)) = arguments.first() else {
                return Err(method_argument_error(
                    method_name,
                    2,
                    arguments.len(),
                    position,
                ));
            };
            let mut changed = 0;
            for path in arguments.iter().skip(1) {
                let Object::String(path) = path else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        path,
                        position,
                    ));
                };
                let named =
                    std::ffi::CString::new(path.as_str().as_bytes().to_vec()).map_err(|_| {
                        MetorexError::runtime_error(
                            format!("Invalid path - {}", path.as_str()),
                            position_to_location(position),
                        )
                    })?;
                // SAFETY: `fchmodat` reads the name and the mode and touches
                // nothing else. AT_SYMLINK_NOFOLLOW is what makes it change
                // the link rather than what the link points at.
                if unsafe {
                    libc::fchmodat(
                        libc::AT_FDCWD,
                        named.as_ptr(),
                        *mode as libc::mode_t,
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } == 0
                {
                    changed += 1;
                }
            }
            return Ok(Some(Object::Int(changed)));
        }
        if class_rc.name() == "Dir" && method_name == "chdir" {
            let target = match arguments.first() {
                Some(named) => self.directory_path_argument(method_name, named, position)?,
                None => std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
            };
            let previous = std::env::current_dir().ok();
            if let Err(error) = std::env::set_current_dir(&target) {
                return Err(directory_error(&error, "chdir", &target, position));
            }
            let Some(Object::Block(block)) = self.pending_block.take() else {
                return Ok(Some(Object::Int(0)));
            };
            let result =
                self.execute_block_callable(&block, vec![Object::string(target)], position);
            if let Some(previous) = previous {
                // The directory the program came from may be gone by now,
                // which is what the program hears about rather than silently
                // being left somewhere else.
                if let Err(error) = std::env::set_current_dir(&previous) {
                    let named = previous.display().to_string();
                    return Err(directory_error(&error, "chdir", &named, position));
                }
            }
            return result.map(Some);
        }
        // `Dir.chroot` names the new root, which only a superuser may set.
        // A regular user gets EPERM, and a name that is not there gets the
        // Errno the system answered.
        if class_rc.name() == "Dir" && method_name == "chroot" {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let path = self.directory_path_argument(method_name, &arguments[0], position)?;
            let spelled = std::ffi::CString::new(path.clone()).map_err(|_| {
                crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "path name contains null byte",
                    position,
                )
            })?;
            let outcome = unsafe { libc::chroot(spelled.as_ptr()) };
            if outcome == 0 {
                return Ok(Some(Object::Int(0)));
            }
            let problem = std::io::Error::last_os_error();
            let named = match problem.raw_os_error() {
                Some(code) if code == libc::EPERM => "Errno::EPERM",
                Some(code) if code == libc::EACCES => "Errno::EACCES",
                Some(code) if code == libc::ENOTDIR => "Errno::ENOTDIR",
                _ => "Errno::ENOENT",
            };
            let message = format!("{} @ chroot - {}", problem, path);
            return Err(crate::vm::errors::simple_exception(
                named, &message, position,
            ));
        }
        if class_rc.name() == "Dir" && (method_name == "exist?" || method_name == "exists?") {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let path = match &arguments[0] {
                Object::String(s) => path_text(s),
                other => {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        other,
                        position,
                    ));
                }
            };
            return Ok(Some(Object::Bool(std::path::Path::new(&path).is_dir())));
        }
        // `Dir.tmpdir` names the directory the system sets aside for scratch
        // files, which is what the tmpdir library answers.
        if class_rc.name() == "Dir" && method_name == "tmpdir" {
            let named = std::env::var("TMPDIR")
                .ok()
                .filter(|held| !held.is_empty())
                .unwrap_or_else(|| "/tmp".to_string());
            let trimmed = named.strip_suffix('/').unwrap_or(&named);
            return Ok(Some(Object::string(trimmed.to_string())));
        }
        // Dir.empty?(path) — whether the directory holds no names of its own.
        if class_rc.name() == "Dir" && method_name == "empty?" {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let path = self.directory_path_argument(method_name, &arguments[0], position)?;
            let Ok(details) = std::fs::metadata(&path) else {
                return Err(crate::vm::errors::simple_exception(
                    "Errno::ENOENT",
                    &format!("No such file or directory - {path}"),
                    position,
                ));
            };
            // A path that names something other than a directory is not empty,
            // whatever it holds.
            if !details.is_dir() {
                return Ok(Some(Object::Bool(false)));
            }
            let empty = std::fs::read_dir(&path)
                .map(|mut reading| reading.next().is_none())
                .unwrap_or(false);
            return Ok(Some(Object::Bool(empty)));
        }
        // Dir.home(user = nil) — where the named user's files live, and where
        // the current user's do when no name is given.
        if class_rc.name() == "Dir" && method_name == "home" {
            if arguments.len() > 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let named = match arguments.first() {
                None | Some(Object::Nil) => None,
                Some(Object::String(name)) => Some(name.as_str().to_string()),
                Some(other) => {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        other,
                        position,
                    ));
                }
            };
            return self.directory_home(named, position).map(Some);
        }
        Ok(None)
    }
}
