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
        // ── Dir methods ─────────────────────────────────────────────────────
        if class_rc.name() == "Dir" && (method_name == "pwd" || method_name == "getwd") {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            return Ok(Some(Object::string(cwd)));
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
        if class_rc.name() == "Dir"
            && (method_name == "mkdir" || method_name == "delete" || method_name == "rmdir")
        {
            if arguments.is_empty() {
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
            let _ = if method_name == "mkdir" {
                std::fs::create_dir_all(&path)
            } else {
                std::fs::remove_dir(&path)
            };
            return Ok(Some(Object::Int(0)));
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
            let mut results: Vec<Object> = Vec::new();
            for arg in arguments {
                let pattern = match arg {
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
                if let Ok(paths) = glob::glob(&pattern) {
                    for entry in paths.flatten() {
                        results.push(Object::string(entry.to_string_lossy().to_string()));
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
            "read" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error("read", 1, arguments.len(), position));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "read", "String", other, position,
                        ));
                    }
                };
                let contents = std::fs::read_to_string(&path).map_err(|e| {
                    MetorexError::runtime_error(
                        format!("Failed to read file '{}': {}", path, e),
                        position_to_location(position),
                    )
                })?;
                Ok(Some(Object::string(contents)))
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
                let base_path = std::path::PathBuf::from(&base);
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
                let base_path = std::path::PathBuf::from(&base);
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
}
