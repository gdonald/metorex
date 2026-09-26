// The names a directory holds, walked or gathered or matched.

use super::*;

impl VirtualMachine {
    /// The names a directory holds, walked or gathered or matched.
    pub(crate) fn call_dir_entry_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
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
                // `Dir.mkdir` takes the mode the directory is created with,
                // which is where a sticky or setgid bit comes from. Anything
                // but an Integer has to answer `to_int` for it.
                let mode = match arguments.get(1) {
                    None | Some(Object::Nil) => None,
                    Some(Object::Int(mode)) => Some(*mode),
                    Some(other) => {
                        let held = self.coerce_integer_argument(other, position)?;
                        Some(i64::try_from(held).unwrap_or(0))
                    }
                };
                // Only the last name in the path is made, so a missing
                // directory above it is reported rather than filled in.
                std::fs::create_dir(&path).map_err(|problem| {
                    let named = match problem.raw_os_error() {
                        Some(code) if code == libc::EEXIST => "Errno::EEXIST",
                        Some(code) if code == libc::EACCES => "Errno::EACCES",
                        Some(code) if code == libc::ENOTDIR => "Errno::ENOTDIR",
                        _ => "Errno::ENOENT",
                    };
                    crate::vm::errors::simple_exception(
                        named,
                        &format!("{problem} - {path}"),
                        position,
                    )
                })?;
                if let Some(mode) = mode {
                    use std::os::unix::fs::PermissionsExt as _;
                    let _ = std::fs::set_permissions(
                        &path,
                        std::fs::Permissions::from_mode(mode as u32),
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
                Object::String(held) => path_text(held),
                other => {
                    // Anything else names a path through `to_path`.
                    let converted =
                        self.send_to_object(other.clone(), "to_path", vec![], position)?;
                    match converted {
                        Object::String(held) => path_text(&held),
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
            return self.glob_paths(method_name, arguments, position).map(Some);
        }
        Ok(None)
    }
}
