// Links between names, and moving a name onto another.

use super::*;

impl VirtualMachine {
    /// Links between names, and moving a name onto another.
    pub(crate) fn call_file_link_methods(
        &mut self,
        _class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
                        crate::vm::init::two_path_error(
                            &problem,
                            "rb_file_s_symlink",
                            &target.as_str(),
                            &link.as_str(),
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
            _ => Ok(None),
        }
    }
}
