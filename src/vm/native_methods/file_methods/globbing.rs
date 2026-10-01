// Every path a glob pattern names.

use super::*;

impl VirtualMachine {
    /// `Dir.glob` and `Dir[]`: every path the patterns name, read against the
    /// directory `base:` names or the one the program stands in.
    pub(crate) fn glob_paths(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut patterns: Vec<String> = Vec::new();
        let mut flags = 0i64;
        let mut base: Option<String> = None;
        let mut sorted = true;
        // One String is a pattern of its own, where a list hands over each
        // pattern as a path, and the two refuse a NUL byte differently.
        let mut listed = false;
        for (index, argument) in arguments.iter().enumerate() {
            match argument {
                Object::Array(held) => {
                    listed = true;
                    for one in held.borrow().iter().cloned().collect::<Vec<Object>>() {
                        patterns.push(self.glob_pattern_text(&one, position)?);
                    }
                }
                Object::Int(held) if index > 0 => flags = *held,
                Object::Dict(held) if names_keywords(argument) || method_name == "glob" => {
                    let entries: Vec<(String, Object)> = held
                        .borrow()
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    for (key, value) in entries {
                        match key.as_str() {
                            ":base" => {
                                base = match &value {
                                    Object::Nil => None,
                                    Object::String(text) => Some(path_text(text)),
                                    other => Some(self.glob_pattern_text(other, position)?),
                                }
                            }
                            ":sort" => match value {
                                Object::Bool(held) => sorted = held,
                                _ => {
                                    return Err(crate::vm::errors::simple_exception(
                                        "ArgumentError",
                                        "expected true or false as sort:",
                                        position,
                                    ));
                                }
                            },
                            _ => {}
                        }
                    }
                }
                other => patterns.push(self.glob_pattern_text(other, position)?),
            }
        }
        let options = crate::vm::native_methods::glob::Options {
            dotmatch: flags & 0x04 != 0,
            noescape: flags & 0x01 != 0,
            casefold: flags & 0x08 != 0,
        };
        let root = match &base {
            None => std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            Some(held) if held.is_empty() => {
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            }
            Some(held) => {
                let named = std::path::PathBuf::from(held);
                if named.is_absolute() {
                    named
                } else {
                    std::env::current_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from("."))
                        .join(named)
                }
            }
        };
        let against_base = base.as_ref().is_some_and(|held| !held.is_empty());
        let mut found: Vec<Object> = Vec::new();
        let several = listed || patterns.len() > 1;
        for pattern in patterns {
            if pattern.contains('\0') {
                let message = if several {
                    "path name contains null byte"
                } else {
                    "nul-separated glob pattern is deprecated"
                };
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    message,
                    position,
                ));
            }
            for spread in crate::vm::native_methods::glob::brace_expansions(&pattern) {
                let mut held =
                    crate::vm::native_methods::glob::matching_paths(&root, &spread, &options);
                // Read against a directory named on its own, a pattern that
                // walks down through the subdirectories names that directory
                // itself as well.
                if against_base && spread.starts_with("**/") && spread.ends_with('/') {
                    held.insert(0, "/".to_string());
                }
                if sorted {
                    held.sort();
                }
                found.extend(held.into_iter().map(Object::string));
            }
        }
        // A block is handed each path and the call answers nothing.
        if let Some(Object::Block(block)) = self.pending_block.take() {
            for path in found {
                self.execute_block_callable(&block, vec![path], position)?;
            }
            return Ok(Object::Nil);
        }
        Ok(Object::Array(Rc::new(std::cell::RefCell::new(found))))
    }

    /// The text a glob pattern is written as. A pattern in an encoding that
    /// spells no ASCII cannot name a path.
    pub(crate) fn glob_pattern_text(
        &mut self,
        held: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = held {
            let named = text.encoding_name();
            if !crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(&named) {
                return Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &format!("ASCII incompatible encoding: {named}"),
                    position,
                ));
            }
            return Ok(path_text(text));
        }
        self.path_argument_text("glob", held, position)
    }
}

/// A path resolved one part at a time, the way Ruby resolves one: a link is
/// followed as it is reached, and a step up takes one part off what has been
/// resolved so far rather than being handed to the system as written.
pub(crate) fn walked_real_path(path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    use std::path::Component;
    let mut held = std::path::PathBuf::from("/");
    for part in path.components() {
        match part {
            Component::RootDir | Component::Prefix(_) => held = std::path::PathBuf::from("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                held.pop();
            }
            Component::Normal(name) => {
                held.push(name);
                if std::fs::symlink_metadata(&held)?.file_type().is_symlink() {
                    let target = std::fs::read_link(&held)?;
                    held = match target.is_absolute() {
                        true => target,
                        false => held.parent().unwrap_or(&held).join(&target),
                    };
                    held = held.canonicalize()?;
                }
            }
        }
    }
    Ok(held)
}
