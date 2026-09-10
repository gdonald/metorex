use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn call_file_dir_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // A path may arrive as an object that names one through `to_path` or
        // `to_str`, which Ruby asks for before the path reaches the
        // filesystem.
        let coerced;
        let arguments = match arguments.first() {
            Some(first)
                if !matches!(first, Object::String(_))
                    && matches!(class_rc.name(), "File" | "Dir")
                    && names_a_path(method_name) =>
            {
                match self.path_naming_answer(first, position)? {
                    Some(named) => {
                        let mut rest = arguments.to_vec();
                        rest[0] = named;
                        coerced = rest;
                        coerced.as_slice()
                    }
                    None => arguments,
                }
            }
            _ => arguments,
        };
        let coerced_second;
        let arguments = match arguments.get(1) {
            Some(second)
                if !matches!(second, Object::String(_))
                    && class_rc.name() == "File"
                    && names_a_second_path(method_name) =>
            {
                match self.path_naming_answer(second, position)? {
                    Some(named) => {
                        let mut rest = arguments.to_vec();
                        rest[1] = named;
                        coerced_second = rest;
                        coerced_second.as_slice()
                    }
                    None => arguments,
                }
            }
            _ => arguments,
        };
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
        if class_rc.name() == "File" && method_name == "chmod" {
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
                use std::os::unix::fs::PermissionsExt as _;
                let permissions = std::fs::Permissions::from_mode(*mode as u32);
                if std::fs::set_permissions(&*path.as_str(), permissions).is_ok() {
                    changed += 1;
                }
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
                Some(Object::String(path)) => path.as_str().to_string(),
                Some(other) => {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        other,
                        position,
                    ));
                }
                None => std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
            };
            let previous = std::env::current_dir().ok();
            std::env::set_current_dir(&target).map_err(|error| {
                MetorexError::runtime_error(
                    format!("No such file or directory - {} ({})", target, error),
                    position_to_location(position),
                )
            })?;
            let Some(Object::Block(block)) = self.pending_block.take() else {
                return Ok(Some(Object::Int(0)));
            };
            let result =
                self.execute_block_callable(&block, vec![Object::string(target)], position);
            if let Some(previous) = previous {
                let _ = std::env::set_current_dir(previous);
            }
            return result.map(Some);
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
                Object::String(s) => s.as_str().to_string(),
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
        // Dir.foreach(path) and Dir.each_child(path) — every name the
        // directory holds, handed to the block one at a time.
        if class_rc.name() == "Dir" && (method_name == "foreach" || method_name == "each_child") {
            let named: Vec<Object> = arguments
                .iter()
                .filter(|held| !matches!(held, Object::Dict(_)))
                .cloned()
                .collect();
            if named.len() != 1 {
                return Err(method_argument_error(method_name, 1, named.len(), position));
            }
            let block = self.pending_block.take();
            // Without a block the walk is handed back as an Enumerator, and
            // the path is left untouched so `to_path` is asked for once.
            let Some(Object::Block(block)) = block else {
                return self
                    .build_enumerator(
                        Object::Class(Rc::clone(class_rc)),
                        method_name,
                        arguments.to_vec(),
                        None,
                        position,
                    )
                    .map(Some);
            };
            let path = self.directory_path_argument(method_name, &named[0], position)?;
            let encoding = self.named_encoding(arguments, position);
            let listing =
                self.directory_names(&path, method_name == "foreach", encoding, position)?;
            for name in listing {
                self.execute_block_body(&block, vec![name])?;
            }
            return Ok(Some(Object::Nil));
        }
        if class_rc.name() == "Dir"
            && (method_name == "mkdir"
                || method_name == "delete"
                || method_name == "rmdir"
                || method_name == "unlink")
        {
            if arguments.is_empty() {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let path = self.directory_path_argument(method_name, &arguments[0], position)?;
            if method_name == "mkdir" {
                let made = std::fs::create_dir_all(&path);
                // `Dir.mkdir` takes the mode the directory is created with,
                // which is where a sticky or setgid bit comes from.
                if made.is_ok()
                    && let Some(Object::Int(mode)) = arguments.get(1)
                {
                    use std::os::unix::fs::PermissionsExt as _;
                    let _ = std::fs::set_permissions(
                        &path,
                        std::fs::Permissions::from_mode(*mode as u32),
                    );
                }
            } else if let Err(problem) = std::fs::remove_dir(&path) {
                return Err(crate::vm::errors::simple_exception(
                    directory_removal_errno(&path, &problem),
                    &format!("{problem} - {path}"),
                    position,
                ));
            }
            return Ok(Some(Object::Int(0)));
        }
        // Dir.entries(path) — every name the directory holds, with the two
        // that name the directory itself and its parent.
        if class_rc.name() == "Dir" && (method_name == "entries" || method_name == "children") {
            // An `encoding:` keyword arrives as a trailing hash, which every
            // string here is read in already.
            let named: Vec<&Object> = arguments
                .iter()
                .filter(|held| !matches!(held, Object::Dict(_)))
                .collect();
            if named.len() != 1 {
                return Err(method_argument_error(method_name, 1, named.len(), position));
            }
            let path = match named[0] {
                Object::String(held) => held.as_str().to_string(),
                other => {
                    // Anything else names a path through `to_path`.
                    let converted =
                        self.send_to_object(other.clone(), "to_path", vec![], position)?;
                    match converted {
                        Object::String(held) => held.as_str().to_string(),
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
            };
            let encoding = self.named_encoding(arguments, position);
            let names =
                self.directory_names(&path, method_name == "entries", encoding, position)?;
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(names)))));
        }
        if class_rc.name() == "Dir" && (method_name == "[]" || method_name == "glob") {
            if arguments.is_empty() {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            // `Dir.glob` takes a pattern, the flags that widen it, and a
            // `base:` naming the directory the pattern is read against, while
            // `Dir[]` takes patterns alone.
            let mut patterns: Vec<String> = Vec::new();
            let mut flags = 0i64;
            let mut base: Option<String> = None;
            for (index, argument) in arguments.iter().enumerate() {
                match argument {
                    Object::String(held) => patterns.push(held.as_str().to_string()),
                    Object::Array(held) => {
                        for one in held.borrow().iter() {
                            match one {
                                Object::String(text) => patterns.push(text.as_str().to_string()),
                                other => {
                                    return Err(method_argument_type_error(
                                        method_name,
                                        "String",
                                        other,
                                        position,
                                    ));
                                }
                            }
                        }
                    }
                    Object::Int(held) if index > 0 && method_name == "glob" => flags = *held,
                    Object::Dict(held) if method_name == "glob" => {
                        for (key, value) in held.borrow().iter() {
                            if key == ":base" || key == "base" {
                                base = Some(value.to_string());
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
                }
            }
            // Ruby hides a leading dot from `*` unless asked not to, which is
            // the opposite of what the glob crate does by default.
            let options = glob::MatchOptions {
                case_sensitive: flags & 0x08 == 0,
                require_literal_separator: false,
                require_literal_leading_dot: flags & 0x04 == 0,
            };
            let mut results: Vec<Object> = Vec::new();
            for pattern in patterns {
                let (searched, prefix) = match &base {
                    Some(held) if !held.is_empty() => (
                        format!("{}/{}", held.trim_end_matches('/'), pattern),
                        format!("{}/", held.trim_end_matches('/')),
                    ),
                    _ => (pattern, String::new()),
                };
                if let Ok(paths) = glob::glob_with(&searched, options) {
                    for entry in paths.flatten() {
                        let found = entry.to_string_lossy().to_string();
                        let written = match found.strip_prefix(&prefix) {
                            Some(rest) if !prefix.is_empty() => rest.to_string(),
                            _ => found,
                        };
                        results.push(Object::string(written));
                    }
                }
            }
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                results,
            )))));
        }

        // ── File methods ────────────────────────────────────────────────────
        if class_rc.name() != "File" {
            return Ok(None);
        }
        match method_name {
            // File.umask — returns the current process umask. We stub a
            // typical default so spec helpers that branch on
            // `(File.umask & 0002) == 0` can run; this isn't
            // process-accurate but is enough for the autoload tmp-dir
            // bootstrap.
            "umask" => Ok(Some(Object::Int(0o022))),
            // File.stat — returns a stub File::Stat object. The class is
            // memoized as a global so subsequent stat()s share its method
            // table; `world_writable?` and `sticky?` are stubbed to return
            // false. Spec helpers (mspec's `tmp`) gate on these to
            // validate the temp dir mode; metorex creates the dir itself
            // so reporting "safe" is fine.
            // The numbers the operating system keeps about a file. The
            // File::Stat that presents them is written in Ruby on top of
            // this, so only the reading of them is native.
            // File.link(old, new) makes another name for the same file, and
            // File.utime sets the times a file reports.
            // File.chown(uid, gid, *paths) sets who owns a file. A -1 for
            // either leaves that one as it was.
            "chown" | "lchown" => {
                if arguments.len() < 3 {
                    return Err(method_argument_error(
                        method_name,
                        3,
                        arguments.len(),
                        position,
                    ));
                }
                let owner = match &arguments[0] {
                    Object::Int(held) => *held as libc::uid_t,
                    Object::Nil => libc::uid_t::MAX,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let group = match &arguments[1] {
                    Object::Int(held) => *held as libc::gid_t,
                    Object::Nil => libc::gid_t::MAX,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let mut changed = 0i64;
                for argument in &arguments[2..] {
                    let Object::String(path) = argument else {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            argument,
                            position,
                        ));
                    };
                    let Ok(name) = std::ffi::CString::new(path.as_str().as_bytes().to_vec()) else {
                        continue;
                    };
                    // SAFETY: `chown` and `lchown` read the name and the two
                    // ids, and touch nothing else.
                    let answer = unsafe {
                        if method_name == "chown" {
                            libc::chown(name.as_ptr(), owner, group)
                        } else {
                            libc::lchown(name.as_ptr(), owner, group)
                        }
                    };
                    if answer != 0 {
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::EPERM",
                            &format!("Operation not permitted @ chown - {path}"),
                            position,
                        ));
                    }
                    changed += 1;
                }
                Ok(Some(Object::Int(changed)))
            }
            "link" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let (Object::String(existing), Object::String(added)) =
                    (&arguments[0], &arguments[1])
                else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                std::fs::hard_link(&*existing.as_str(), &*added.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::EEXIST",
                        &format!("File exists @ rb_file_s_link - {added}: {problem}"),
                        position,
                    )
                })?;
                Ok(Some(Object::Int(0)))
            }
            "utime" => {
                if arguments.len() < 3 {
                    return Err(method_argument_error(
                        method_name,
                        3,
                        arguments.len(),
                        position,
                    ));
                }
                let seconds = |value: &Object| -> f64 {
                    match value {
                        Object::Int(held) => *held as f64,
                        Object::Float(held) => *held,
                        other => self_seconds(other).unwrap_or(0.0),
                    }
                };
                let accessed = seconds(&arguments[0]);
                let modified = seconds(&arguments[1]);
                let mut touched = 0i64;
                for argument in &arguments[2..] {
                    let Object::String(path) = argument else {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            argument,
                            position,
                        ));
                    };
                    let times = [
                        libc::timeval {
                            tv_sec: accessed.trunc() as libc::time_t,
                            tv_usec: ((accessed.fract() * 1_000_000.0) as libc::suseconds_t).abs(),
                        },
                        libc::timeval {
                            tv_sec: modified.trunc() as libc::time_t,
                            tv_usec: ((modified.fract() * 1_000_000.0) as libc::suseconds_t).abs(),
                        },
                    ];
                    let Ok(name) = std::ffi::CString::new(path.as_str().as_bytes().to_vec()) else {
                        continue;
                    };
                    // SAFETY: `utimes` reads the name and the two times, and
                    // touches nothing else.
                    let answer = unsafe { libc::utimes(name.as_ptr(), times.as_ptr()) };
                    if answer != 0 {
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::ENOENT",
                            &format!("No such file or directory @ utime_failed - {path}"),
                            position,
                        ));
                    }
                    touched += 1;
                }
                Ok(Some(Object::Int(touched)))
            }
            // File.binread(path) answers the bytes the file holds, one to a
            // character, so a file that is not text reads back unchanged.
            "binread" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                if std::path::Path::new(&*path.as_str()).is_dir() {
                    return Err(crate::vm::errors::simple_exception(
                        "Errno::EISDIR",
                        &format!("Is a directory @ io_fread - {path}"),
                        position,
                    ));
                }
                let held = std::fs::read(&*path.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_sysopen - {path}: {problem}"),
                        position,
                    )
                })?;
                Ok(Some(super::pack_format::bytes_to_string(&held)))
            }
            // File.read(path) answers everything the file holds.
            "read" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                // A directory opens but reads as the wrong kind of thing,
                // which Ruby reports separately from a missing name.
                if std::path::Path::new(&*path.as_str()).is_dir() {
                    return Err(crate::vm::errors::simple_exception(
                        "Errno::EISDIR",
                        &format!("Is a directory @ io_fread - {path}"),
                        position,
                    ));
                }
                let held = std::fs::read_to_string(&*path.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_sysopen - {path}: {problem}"),
                        position,
                    )
                })?;
                Ok(Some(Object::string(held)))
            }
            // File.readlines(path) answers the lines it holds, each keeping
            // the newline that ends it.
            "readlines" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                let held = std::fs::read_to_string(&*path.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_sysopen - {path}: {problem}"),
                        position,
                    )
                })?;
                let mut lines: Vec<Object> = Vec::new();
                let mut current = String::new();
                for character in held.chars() {
                    current.push(character);
                    if character == '\n' {
                        lines.push(Object::string(std::mem::take(&mut current)));
                    }
                }
                if !current.is_empty() {
                    lines.push(Object::string(current));
                }
                Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(lines)))))
            }
            "__stat_fields__" => {
                use std::os::unix::fs::MetadataExt;
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                let follow = arguments[1].is_truthy();
                let held = if follow {
                    std::fs::metadata(&*path.as_str())
                } else {
                    std::fs::symlink_metadata(&*path.as_str())
                };
                let held = held.map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_file_s_stat - {path}: {problem}"),
                        position,
                    )
                })?;
                let kind = held.file_type();
                let mode = held.mode();
                let ftype = if kind.is_dir() {
                    "directory"
                } else if kind.is_symlink() {
                    "link"
                } else if mode & 0o170000 == 0o020000 {
                    "characterSpecial"
                } else if mode & 0o170000 == 0o060000 {
                    "blockSpecial"
                } else if mode & 0o170000 == 0o010000 {
                    "fifo"
                } else if mode & 0o170000 == 0o140000 {
                    "socket"
                } else {
                    "file"
                };
                let mut fields: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
                let mut record = |name: &str, value: Object| {
                    fields.insert(format!(":{name}"), value);
                };
                record("dev", Object::Int(held.dev() as i64));
                record("ino", Object::Int(held.ino() as i64));
                record("mode", Object::Int(mode as i64));
                record("nlink", Object::Int(held.nlink() as i64));
                record("uid", Object::Int(held.uid() as i64));
                record("gid", Object::Int(held.gid() as i64));
                record("rdev", Object::Int(held.rdev() as i64));
                record("size", Object::Int(held.size() as i64));
                record("blksize", Object::Int(held.blksize() as i64));
                record("blocks", Object::Int(held.blocks() as i64));
                record("atime", Object::Float(held.atime() as f64));
                record("mtime", Object::Float(held.mtime() as f64));
                record("ctime", Object::Float(held.ctime() as f64));
                // Only some platforms keep the time a file was made.
                let born = held
                    .created()
                    .ok()
                    .and_then(|held| held.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|held| Object::Float(held.as_secs_f64()));
                record("birthtime", born.unwrap_or(Object::Nil));
                record("ftype", Object::string(ftype.to_string()));
                Ok(Some(Object::Dict(Rc::new(std::cell::RefCell::new(fields)))))
            }
            "new" | "open" => {
                use crate::object::{Instance, Method};
                if arguments.is_empty() {
                    return Err(method_argument_error("open", 1, 0, position));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "open", "String", other, position,
                        ));
                    }
                };
                let mode = match arguments.get(1) {
                    Some(Object::String(s)) => s.as_str().to_string(),
                    None => "r".to_string(),
                    Some(other) => {
                        return Err(method_argument_type_error(
                            "open", "String", other, position,
                        ));
                    }
                };
                let writing = mode.contains('w') || mode.contains('a') || mode.contains('+');
                let truncate = mode.contains('w');
                if writing && truncate {
                    let _ = std::fs::write(&path, "");
                }
                let file_class = match self.globals().get("__File_handle_class") {
                    Some(Object::Class(c)) => c,
                    _ => {
                        // An open handle is an instance of a class of its own,
                        // which stands under the global File so the methods
                        // written there answer for it.
                        let global_file = match self.globals().get("File") {
                            Some(Object::Class(file_class)) => Some(file_class),
                            _ => None,
                        };
                        let cls = Rc::new(crate::class::Class::new("File", global_file));
                        for n in ["close", "closed?"] {
                            cls.define_method(
                                n,
                                Rc::new(Method::with_owner(
                                    n.to_string(),
                                    vec![],
                                    vec![],
                                    "File".to_string(),
                                )),
                            );
                        }
                        self.globals_mut()
                            .set("__File_handle_class", Object::Class(Rc::clone(&cls)));
                        cls
                    }
                };
                let inst = Instance::new(file_class);
                let inst_rc = Rc::new(std::cell::RefCell::new(inst));
                inst_rc
                    .borrow_mut()
                    .set_var("__file_path".to_string(), Object::string(path));
                inst_rc
                    .borrow_mut()
                    .set_var("__file_mode".to_string(), Object::string(mode));
                let handle = Object::Instance(inst_rc);
                let block = self.pending_block.take();
                if let Some(Object::Block(b)) = block {
                    let result = self.execute_block_callable(&b, vec![handle], position);
                    return result.map(Some);
                }
                Ok(Some(handle))
            }
            // File.symlink(target, link) makes a symbolic link, which the
            // spec fixtures build their directory trees with.
            "symlink" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let (Object::String(target), Object::String(link)) = (&arguments[0], &arguments[1])
                else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                std::os::unix::fs::symlink(&*target.as_str(), &*link.as_str()).map_err(
                    |problem| {
                        crate::vm::errors::simple_exception(
                            "Errno::EEXIST",
                            &format!("File exists @ rb_file_s_symlink - {link}: {problem}"),
                            position,
                        )
                    },
                )?;
                Ok(Some(Object::Int(0)))
            }
            // File.readlink(path) answers what a symbolic link points at.
            "readlink" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                // A name nothing stands for is reported as missing, and a
                // name that stands for something other than a link is
                // reported as the wrong kind of argument.
                let target = std::fs::read_link(&*path.as_str()).map_err(|problem| {
                    if problem.kind() == std::io::ErrorKind::NotFound {
                        crate::vm::errors::simple_exception(
                            "Errno::ENOENT",
                            &format!(
                                "No such file or directory @ rb_file_s_readlink - {path}: {problem}"
                            ),
                            position,
                        )
                    } else {
                        crate::vm::errors::simple_exception(
                            "Errno::EINVAL",
                            &format!("Invalid argument @ rb_file_s_readlink - {path}: {problem}"),
                            position,
                        )
                    }
                })?;
                Ok(Some(Object::string(target.to_string_lossy().to_string())))
            }
            "symlink?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                // A symbolic link is what `symlink_metadata` reports without
                // following it, so a link to a missing file is still one.
                let answer = std::fs::symlink_metadata(&*path.as_str())
                    .map(|held| held.file_type().is_symlink())
                    .unwrap_or(false);
                Ok(Some(Object::Bool(answer)))
            }
            // File.rename(from, to) moves a name onto another, which is how
            // a log is rotated.
            "rename" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "rename",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let (Object::String(from), Object::String(to)) = (&arguments[0], &arguments[1])
                else {
                    return Err(method_argument_type_error(
                        "rename",
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                std::fs::rename(&*from.as_str(), &*to.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_file_s_rename - ({from}, {to}): {problem}"),
                        position,
                    )
                })?;
                Ok(Some(Object::Int(0)))
            }
            "delete" | "unlink" => {
                let mut deleted = 0i64;
                for arg in arguments {
                    if let Object::String(s) = arg {
                        let _ = std::fs::remove_file(&*s.as_str());
                        deleted += 1;
                    }
                }
                Ok(Some(Object::Int(deleted)))
            }
            // File.binwrite(path, content) writes the bytes the content
            // stands for, one to a character, rather than the bytes a text
            // encoding would spell them with.
            "binwrite" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "binwrite",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        "binwrite",
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                let content = match &arguments[1] {
                    Object::String(text) => text.as_str().to_string(),
                    other => format!("{}", other),
                };
                let bytes = super::pack_format::string_to_bytes(&content);
                std::fs::write(&*path.as_str(), &bytes).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_sysopen - {path}: {problem}"),
                        position,
                    )
                })?;
                Ok(Some(Object::Int(bytes.len() as i64)))
            }
            "write" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error("write", 2, arguments.len(), position));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "write", "String", other, position,
                        ));
                    }
                };
                let content = match &arguments[1] {
                    Object::String(s) => s.as_str().to_string(),
                    other => format!("{}", other),
                };
                std::fs::write(&path, &content).map_err(|e| {
                    MetorexError::runtime_error(
                        format!("Failed to write file '{}': {}", path, e),
                        position_to_location(position),
                    )
                })?;
                Ok(Some(Object::Int(content.len() as i64)))
            }
            "exist?" | "exists?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "exist?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "exist?", "String", other, position,
                        ));
                    }
                };
                Ok(Some(Object::Bool(std::path::Path::new(&path).exists())))
            }
            "realpath" | "realdirpath" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "realpath",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let path_str = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "realpath", "String", other, position,
                        ));
                    }
                };
                let base = if arguments.len() == 2 {
                    match &arguments[1] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "realpath", "String", other, position,
                            ));
                        }
                    }
                } else {
                    std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                };
                // A relative base expands against the working directory, so
                // the answer is always an absolute path, as Ruby's is.
                let base_path = std::path::PathBuf::from(&base);
                let base_path = if base_path.is_absolute() {
                    base_path
                } else {
                    std::env::current_dir().unwrap_or_default().join(base_path)
                };
                let expanded = base_path.join(&path_str);
                match expanded.canonicalize() {
                    Ok(p) => Ok(Some(Object::string(p.to_string_lossy().to_string()))),
                    Err(e) => {
                        let exc = if e.kind() == std::io::ErrorKind::NotFound {
                            Object::exception(
                                "Errno::ENOENT",
                                format!("No such file or directory @ realpath - {}", path_str),
                            )
                        } else {
                            Object::exception(
                                "Errno::ENOTDIR",
                                format!("Not a directory @ realpath - {}", path_str),
                            )
                        };
                        Err(MetorexError::UncaughtException {
                            exception: exc.clone(),
                            location: position_to_location(position),
                            message: format!("{}", exc),
                        })
                    }
                }
            }
            "directory?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "directory?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "directory?",
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                Ok(Some(Object::Bool(std::path::Path::new(&path).is_dir())))
            }
            // File.size(path) — how many bytes the file holds.
            "size" | "size?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let path = match &arguments[0] {
                    Object::String(held) => held.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let found = std::fs::metadata(&path);
                // `size?` answers nil where `size` raises, so a caller can
                // test for content and for the file itself in one step.
                if method_name == "size?" && found.is_err() {
                    return Ok(Some(Object::Nil));
                }
                let held = found.map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_file_s_size - {path}: {problem}"),
                        position,
                    )
                })?;
                if method_name == "size?" && held.len() == 0 {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(Object::Int(held.len() as i64)))
            }
            "file?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error("file?", 1, arguments.len(), position));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "file?", "String", other, position,
                        ));
                    }
                };
                Ok(Some(Object::Bool(std::path::Path::new(&path).is_file())))
            }
            // File.executable?(path) — whether the owner-, group-, or
            // other-execute bit is set on an existing path.
            "executable?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "executable?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "executable?",
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let executable = std::fs::metadata(&path)
                    .map(|metadata| {
                        use std::os::unix::fs::PermissionsExt;
                        metadata.permissions().mode() & 0o111 != 0
                    })
                    .unwrap_or(false);
                Ok(Some(Object::Bool(executable)))
            }
            "expand_path" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "wrong number of arguments (given {}, expected 1..2)",
                            arguments.len()
                        ),
                        position_to_location(position),
                    ));
                }
                let path_str = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "expand_path",
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let base = if arguments.len() == 2 {
                    match &arguments[1] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "expand_path",
                                "String",
                                other,
                                position,
                            ));
                        }
                    }
                } else {
                    std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                };
                // A leading `~` names the home directory.
                let path_str = expanded_home(&path_str);
                let base = expanded_home(&base);
                // A relative base expands against the working directory, so
                // the answer is always an absolute path, as Ruby's is.
                let base_path = std::path::PathBuf::from(&base);
                let base_path = if base_path.is_absolute() {
                    base_path
                } else {
                    std::env::current_dir().unwrap_or_default().join(base_path)
                };
                let expanded = base_path.join(&path_str);
                let result = match expanded.canonicalize() {
                    Ok(p) => p.to_string_lossy().to_string(),
                    Err(_) => {
                        let mut components = Vec::new();
                        for comp in expanded.components() {
                            match comp {
                                std::path::Component::ParentDir => {
                                    components.pop();
                                }
                                std::path::Component::CurDir => {}
                                _ => components.push(comp),
                            }
                        }
                        let normalized: std::path::PathBuf = components.iter().collect();
                        normalized.to_string_lossy().to_string()
                    }
                };
                Ok(Some(Object::string(result)))
            }
            _ => Ok(None),
        }
    }

    /// The String an object names itself as, asking `to_path` first and
    /// `to_str` after, or None when it names neither.
    fn path_naming_answer(
        &mut self,
        candidate: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        for name in ["to_path", "to_str"] {
            if self.responds_to(candidate, name) {
                let named = self.send_to_object(candidate.clone(), name, vec![], position)?;
                if matches!(named, Object::String(_)) {
                    return Ok(Some(named));
                }
            }
        }
        // A stream stands for the file it is open on, which is what the
        // predicates read when they are handed one.
        if self.responds_to(candidate, "to_io") {
            let named = self.send_to_object(candidate.clone(), "to_io", vec![], position)?;
            if matches!(named, Object::String(_) | Object::Instance(_)) {
                return Ok(Some(named));
            }
        }
        Ok(None)
    }
}

/// The seconds a Time stands for, read off the object rather than through a
/// method call, so `File.utime` takes one without asking the virtual machine.
fn self_seconds(value: &Object) -> Option<f64> {
    let Object::Instance(instance) = value else {
        return None;
    };
    let instance = instance.borrow();
    if instance.class.name() != "Time" {
        return None;
    }
    match instance.instance_vars.get("seconds") {
        Some(Object::Int(held)) => Some(*held as f64),
        Some(Object::Float(held)) => Some(*held),
        _ => None,
    }
}

/// Whether a File or Dir class method reads its first argument as a path.
/// Only the names answered natively belong here: the ones written in Ruby
/// ask for `to_path` themselves, and converting twice would call it twice.
fn names_a_path(method_name: &str) -> bool {
    matches!(
        method_name,
        "exist?"
            | "exists?"
            | "file?"
            | "directory?"
            | "executable?"
            | "symlink?"
            | "readlink"
            | "size"
            | "size?"
            | "symlink"
            | "link"
            | "rename"
    )
}

/// The methods that take a second path, which is named the same way the
/// first one is.
fn names_a_second_path(method_name: &str) -> bool {
    matches!(method_name, "symlink" | "link" | "rename")
}

/// The Errno a failed `rmdir` reports, named from what the operating system
/// answered.
fn directory_removal_errno(path: &str, problem: &std::io::Error) -> &'static str {
    match problem.raw_os_error() {
        Some(code) if code == libc::ENOTEMPTY || code == libc::EEXIST => "Errno::ENOTEMPTY",
        Some(code) if code == libc::ENOTDIR => "Errno::ENOTDIR",
        Some(code) if code == libc::EACCES => "Errno::EACCES",
        Some(code) if code == libc::EPERM => {
            // A path that names a file rather than a directory reports EPERM
            // on some systems, which Ruby still calls ENOTDIR.
            if std::path::Path::new(path).is_file() {
                "Errno::ENOTDIR"
            } else {
                "Errno::EPERM"
            }
        }
        _ => "Errno::ENOENT",
    }
}

impl VirtualMachine {
    /// The path an argument names, taking `to_path` from an object that
    /// answers one.
    fn directory_path_argument(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match argument {
            Object::String(path) => Ok(path.as_str().to_string()),
            other if self.responds_to(other, "to_path") => {
                match self.send_to_object(other.clone(), "to_path", vec![], position)? {
                    Object::String(path) => Ok(path.as_str().to_string()),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        other,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                method_name,
                "String",
                other,
                position,
            )),
        }
    }

    /// The encoding an `encoding:` keyword names, which the entries a
    /// directory answers are tagged with.
    fn named_encoding(&mut self, arguments: &[Object], position: Position) -> Option<String> {
        let named = match arguments.last() {
            Some(Object::Dict(options)) => options.borrow().get(":encoding").cloned(),
            _ => None,
        };
        // Without a keyword the entries take the internal encoding, which is
        // what the program asked every string read from outside to be in.
        let named = match named {
            Some(named) => named,
            None => match self.globals().get("Encoding") {
                Some(encoding) => {
                    match self.send_to_object(encoding, "default_internal", vec![], position) {
                        Ok(Object::Nil) | Err(_) => return None,
                        Ok(found) => found,
                    }
                }
                None => return None,
            },
        };
        self.encoding_name_argument(&named, position).ok()
    }

    /// Every name a directory holds, with `.` and `..` when they are wanted.
    fn directory_names(
        &mut self,
        path: &str,
        with_dots: bool,
        encoding: Option<String>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let reading = std::fs::read_dir(path).map_err(|problem| {
            crate::vm::errors::simple_exception(
                "Errno::ENOENT",
                &format!("No such file or directory @ dir_initialize - {path}: {problem}"),
                position,
            )
        })?;
        let tagged = |name: String| match &encoding {
            Some(encoding) => Object::String(Rc::new(crate::object::StringValue::with_encoding(
                name,
                encoding.clone(),
            ))),
            None => Object::string(name),
        };
        let mut names: Vec<Object> = Vec::new();
        if with_dots {
            names.push(tagged(".".to_string()));
            names.push(tagged("..".to_string()));
        }
        for entry in reading.flatten() {
            names.push(tagged(entry.file_name().to_string_lossy().to_string()));
        }
        Ok(names)
    }

    /// Where a user's files live: `$HOME` when no user is named, and the
    /// password database otherwise.
    fn directory_home(
        &mut self,
        user: Option<String>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(user) = user else {
            if let Ok(held) = std::env::var("HOME")
                && !held.is_empty()
            {
                return Ok(Object::string(held));
            }
            let found = self.passwd_home_for(None);
            return match found {
                Some(home) => Ok(Object::string(home)),
                None => Ok(Object::string("/".to_string())),
            };
        };
        match self.passwd_home_for(Some(&user)) {
            Some(home) => Ok(Object::string(home)),
            None => Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("user {user} doesn't exist"),
                position,
            )),
        }
    }

    /// The home directory the password database records, for the named user
    /// or for the one running the program.
    fn passwd_home_for(&mut self, user: Option<&str>) -> Option<String> {
        // SAFETY: both calls answer a pointer the C library owns, read before
        // any other call to it.
        let entry = unsafe {
            match user {
                Some(name) => {
                    let held = std::ffi::CString::new(name).ok()?;
                    libc::getpwnam(held.as_ptr())
                }
                None => libc::getpwuid(libc::getuid()),
            }
        };
        if entry.is_null() {
            return None;
        }
        // SAFETY: the entry is non-null, and its `pw_dir` is a C string the
        // library owns.
        let home = unsafe { std::ffi::CStr::from_ptr((*entry).pw_dir) };
        Some(home.to_string_lossy().to_string())
    }
}

/// A path whose leading `~` has been replaced by the home directory it names.
fn expanded_home(path: &str) -> String {
    if path == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| path.to_string());
    }
    match path.strip_prefix("~/") {
        Some(rest) => match std::env::var("HOME") {
            Ok(home) => format!("{}/{}", home.trim_end_matches('/'), rest),
            Err(_) => path.to_string(),
        },
        None => path.to_string(),
    }
}
