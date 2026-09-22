// What a name stands for, and how large it is.

use super::*;

impl VirtualMachine {
    /// What a name stands for, and how large it is.
    pub(crate) fn call_file_attribute_methods(
        &mut self,
        _class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
            _ => Ok(None),
        }
    }
}
