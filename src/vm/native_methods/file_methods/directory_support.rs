// Reading a directory, and where a home directory sits.

use super::*;

impl VirtualMachine {
    /// The path an argument names, taking `to_path` from an object that
    /// answers one.
    pub(crate) fn directory_path_argument(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let path = self.path_argument_text(method_name, argument, position)?;
        // NUL ends a name the operating system reads, so one holding it names
        // something other than what was written.
        if path.contains('\0') {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "path name contains null byte",
                position,
            ));
        }
        Ok(path)
    }

    /// The text a path argument spells, NUL bytes and all, for a caller that
    /// reads it as a pattern rather than as a name.
    pub(crate) fn path_argument_text(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match argument {
            Object::String(path) => Ok(path_text(path)),
            // A name is spelled by `to_path` where an object has one, and by
            // `to_str` otherwise. Only an object that answers neither is the
            // wrong kind of argument.
            other => {
                for named in ["to_path", "to_str"] {
                    // A name written for the object itself is what counts
                    // here: a Kernel method of the same name would answer for
                    // everything and spell nothing.
                    if self
                        .lookup_method(other, named)
                        .is_none_or(|(_, found)| found.is_undefined)
                    {
                        continue;
                    }
                    if let Object::String(path) =
                        self.send_to_object(other.clone(), named, vec![], position)?
                    {
                        return Ok(path_text(&path));
                    }
                }
                Err(method_argument_type_error(
                    method_name,
                    "String",
                    other,
                    position,
                ))
            }
        }
    }

    /// The encoding an `encoding:` keyword names, which the entries a
    /// directory answers are tagged with.
    pub(crate) fn named_encoding(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Option<String> {
        let named = match arguments.last() {
            Some(Object::Dict(options)) => options.borrow().get(":encoding").cloned(),
            _ => None,
        };
        // Without a keyword the caller works out the encoding from what the
        // program asked its surroundings to be read in.
        let named = named?;
        self.encoding_name_argument(&named, position).ok()
    }

    /// Every name a directory holds, with `.` and `..` when they are wanted.
    pub(crate) fn directory_names(
        &mut self,
        path: &str,
        with_dots: bool,
        encoding: Option<String>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if path.contains('\0') {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "path name contains null byte",
                position,
            ));
        }
        let reading = std::fs::read_dir(path).map_err(|problem| {
            crate::vm::errors::simple_exception(
                "Errno::ENOENT",
                &format!("No such file or directory @ dir_initialize - {path}: {problem}"),
                position,
            )
        })?;
        // A name comes off the file system in the encoding the program reads
        // its surroundings in, unless it was told to read the directory in
        // one of its own.
        let named_encoding = |name: &str| match self.globals().get(name) {
            Some(Object::Class(held)) => Some(held.name().to_string()),
            _ => None,
        };
        let external = named_encoding("__Encoding_default_external")
            .unwrap_or_else(|| crate::object::string_value::DEFAULT_ENCODING.to_string());
        let internal = named_encoding("__Encoding_default_internal");
        let tagged = |name: String| {
            let held = match &encoding {
                Some(encoding) => encoding.clone(),
                // A name carries over into the encoding a program asked its
                // text to be read in, as long as its bytes spell something in
                // the encoding the file system names it in. One whose bytes
                // do not keeps that encoding, since there is no conversion to
                // make.
                None => match &internal {
                    Some(internal) if reads_as(&name, &external) => internal.clone(),
                    _ => external.clone(),
                },
            };
            Object::String(Rc::new(crate::object::StringValue::with_encoding(
                name, held,
            )))
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
    pub(crate) fn directory_home(
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

/// The Errno a failed `rmdir` reports, named from what the operating system
/// answered.
pub(crate) fn directory_removal_errno(path: &str, problem: &std::io::Error) -> &'static str {
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

/// The path `realdirpath` answers for a name whose last part stands for
/// nothing: the directories are resolved, a link in the last place is
/// followed once, and the name itself is kept.
pub(crate) fn directory_real_path(
    expanded: &std::path::Path,
) -> std::io::Result<std::path::PathBuf> {
    let followed = match std::fs::symlink_metadata(expanded) {
        Ok(held) if held.file_type().is_symlink() => {
            let target = std::fs::read_link(expanded)?;
            if target.is_absolute() {
                target
            } else {
                expanded
                    .parent()
                    .unwrap_or(std::path::Path::new("/"))
                    .join(target)
            }
        }
        _ => expanded.to_path_buf(),
    };
    match (followed.parent(), followed.file_name()) {
        (Some(holding), Some(named)) => Ok(holding.canonicalize()?.join(named)),
        _ => followed.canonicalize(),
    }
}

/// Whether a name's bytes spell characters in the encoding the file system
/// names it in, which is what decides whether it carries over into the
/// encoding a program asked its text to be read in.
fn reads_as(name: &str, encoding: &str) -> bool {
    crate::vm::native_methods::string_methods::encoding_reads_bytes(name.as_bytes(), encoding)
}

/// The Errno a directory operation failed with, named the way Ruby names it.
pub(crate) fn directory_error(
    error: &std::io::Error,
    operation: &str,
    path: &str,
    position: Position,
) -> MetorexError {
    let named = match error.raw_os_error() {
        Some(code) if code == libc::EACCES => "Errno::EACCES",
        Some(code) if code == libc::ENOTDIR => "Errno::ENOTDIR",
        Some(code) if code == libc::ELOOP => "Errno::ELOOP",
        Some(code) if code == libc::ENAMETOOLONG => "Errno::ENAMETOOLONG",
        _ => "Errno::ENOENT",
    };
    let message = format!("No such file or directory @ {} - {}", operation, path);
    crate::vm::errors::simple_exception(named, &message, position)
}
