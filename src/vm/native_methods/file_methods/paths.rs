// Removing a name, and the path one resolves to.

use super::*;

impl VirtualMachine {
    /// Removing a name, and the path one resolves to.
    pub(crate) fn call_file_path_methods(
        &mut self,
        _class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "delete" | "unlink" => {
                let mut deleted = 0i64;
                for argument in &arguments.to_vec() {
                    // A name is a String or anything answering `to_path` or
                    // `to_str`, and anything else is refused.
                    let path = self.directory_path_argument(method_name, argument, position)?;
                    std::fs::remove_file(&path).map_err(|problem| {
                        crate::vm::init::system_call_error(&problem, "apply2files", &path, position)
                    })?;
                    deleted += 1;
                }
                Ok(Some(Object::Int(deleted)))
            }
            // File.binwrite(path, content) writes the bytes the content
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
                    Object::String(s) => path_text(s),
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
                let path_str = self.path_text("realpath", &arguments[0], position)?;
                let base = if arguments.len() == 2 {
                    match &arguments[1] {
                        Object::String(s) => path_text(s),
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
                let resolved = match expanded.canonicalize() {
                    // `realdirpath` asks only that the directories exist, so
                    // a last part that stands for nothing is kept as written.
                    Err(problem)
                        if method_name == "realdirpath"
                            && problem.kind() == std::io::ErrorKind::NotFound =>
                    {
                        directory_real_path(&expanded)
                    }
                    // A path that goes up out of a file, as `file/..` does,
                    // is refused by the system call on some platforms. Ruby
                    // walks the path itself and takes one step back off what
                    // it has resolved, which is what happens here.
                    Err(problem) if problem.raw_os_error() == Some(libc::ENOTDIR) => {
                        walked_real_path(&expanded)
                    }
                    other => other,
                };
                match resolved {
                    Ok(p) => Ok(Some(Object::string(p.to_string_lossy().to_string()))),
                    Err(e) => {
                        let exc = match e.raw_os_error() {
                            Some(libc::ELOOP) => Object::exception(
                                "Errno::ELOOP",
                                format!(
                                    "Too many levels of symbolic links @ realpath - {}",
                                    path_str
                                ),
                            ),
                            _ if e.kind() == std::io::ErrorKind::NotFound => Object::exception(
                                "Errno::ENOENT",
                                format!("No such file or directory @ realpath - {}", path_str),
                            ),
                            _ => Object::exception(
                                "Errno::ENOTDIR",
                                format!("Not a directory @ realpath - {}", path_str),
                            ),
                        };
                        Err(MetorexError::UncaughtException {
                            exception: exc.clone(),
                            location: position_to_location(position),
                            message: format!("{}", exc),
                        })
                    }
                }
            }
            _ => Ok(None),
        }
    }
}
