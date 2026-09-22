// Opening a file by name.

use super::*;

impl VirtualMachine {
    /// Opening a file by name.
    pub(crate) fn call_file_opening_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "new" | "open" | "__open_path__" => {
                use crate::object::Instance;
                if arguments.is_empty() {
                    return Err(method_argument_error("open", 1, 0, position));
                }
                // A name is a String or anything that spells itself as one,
                // which `to_path` and `to_str` are for.
                let held = arguments[0].clone();
                // A count of arguments past the name, the mode, and the
                // permissions is one too many, options aside.
                let counted = arguments
                    .iter()
                    .filter(|given| !names_keywords(given))
                    .count();
                if counted > 3 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("wrong number of arguments (given {counted}, expected 1..3)"),
                        position,
                    ));
                }
                // A number in the name's place is a descriptor the program
                // already holds rather than a path.
                if let Object::Int(number) = &held {
                    if *number < 0 {
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::EBADF",
                            "Bad file descriptor",
                            position,
                        ));
                    }
                    let rest: Vec<Object> = arguments[1..].to_vec();
                    let mut passed = vec![held.clone()];
                    passed.extend(rest);
                    return self
                        .send_to_object(
                            Object::Class(Rc::clone(class_rc)),
                            "for_fd",
                            passed,
                            position,
                        )
                        .map(Some);
                }
                let path = self.directory_path_argument("open", &held, position)?;
                // The options may name the mode instead of a second argument
                // naming it, which is what `mode:` is for.
                let named_mode = match arguments.last() {
                    Some(Object::Dict(options)) => match options.borrow().get(":mode") {
                        Some(Object::String(spelled)) => Some(spelled.as_str().to_string()),
                        _ => None,
                    },
                    _ => None,
                };
                // The options may also name the flags a numeric mode would.
                let named_flags = match arguments.last() {
                    Some(Object::Dict(options)) => match options.borrow().get(":flags") {
                        Some(Object::Int(bits)) => Some(*bits),
                        _ => None,
                    },
                    _ => None,
                };
                let (mode, numeric) = match arguments.get(1) {
                    Some(Object::String(s)) => (s.as_str().to_string(), None),
                    // A number names the flags the file is opened with, which
                    // say the same as a mode written out.
                    Some(Object::Int(bits)) => (mode_of_flags(*bits), Some(*bits)),
                    Some(Object::Dict(_)) | None => {
                        (named_mode.unwrap_or_else(|| "r".to_string()), None)
                    }
                    Some(other) => {
                        return Err(method_argument_type_error(
                            "open", "String", other, position,
                        ));
                    }
                };
                let flags = (numeric.unwrap_or(0) | named_flags.unwrap_or(0)) as libc::c_int;
                let writing = mode.contains('w') || mode.contains('a') || mode.contains('+');
                let truncate = mode.contains('w');
                let there = std::path::Path::new(&path).exists();
                // Asked for a file of its own, a name that already stands for
                // one is refused.
                // A mode that names writing from the start brings the file
                // into being, where one that only adds to what is there does
                // not.
                let creates =
                    mode.starts_with('w') || mode.starts_with('a') || flags & libc::O_CREAT != 0;
                if flags & libc::O_EXCL != 0 && creates && there {
                    return Err(crate::vm::errors::simple_exception(
                        "Errno::EEXIST",
                        &format!("File exists @ rb_sysopen - {path}"),
                        position,
                    ));
                }
                // A permission argument settles what a file brought into being
                // may be read and written by, less what the umask takes off.
                // It is applied once the stream is open, so a file made
                // read-only is still written through the stream that made it.
                let allowed = match arguments.get(2) {
                    Some(Object::Int(wanted)) if !there && creates => {
                        // SAFETY: reading the mask means setting it and
                        // putting it straight back.
                        let masked = unsafe {
                            let held = libc::umask(0);
                            libc::umask(held);
                            held
                        };
                        Some((*wanted as u32) & !(masked as u32))
                    }
                    _ => None,
                };
                if creates && !there {
                    let _ = std::fs::write(&path, "");
                }
                if writing && truncate && there {
                    let _ = std::fs::write(&path, "");
                }
                // `File.new` hands nothing to a block, and says so.
                if method_name == "new" && matches!(self.pending_block, Some(Object::Block(_))) {
                    self.pending_block = None;
                    let file = self
                        .current_source_file
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let message = format!(
                        "{}:{}: warning: File::new() does not take block; use File::open() instead\n",
                        file, position.line
                    );
                    self.warn_through_warning_module(message, position)?;
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
                        self.globals_mut()
                            .set("__File_handle_class", Object::Class(Rc::clone(&cls)));
                        cls
                    }
                };
                let inst = Instance::new(file_class);
                let inst_rc = Rc::new(std::cell::RefCell::new(inst));
                // The name is kept as the program spelled it, tag and all, so
                // `path` hands back a String in the same encoding.
                let named = match &held {
                    Object::String(spelled) => Object::String(Rc::clone(spelled)),
                    _ => Object::string(path.clone()),
                };
                inst_rc
                    .borrow_mut()
                    .set_var("__file_path".to_string(), named);
                inst_rc
                    .borrow_mut()
                    .set_var("__file_mode".to_string(), Object::string(mode));
                // `encoding: "UTF-8:ISO-8859-1"` names the encodings a file
                // is read and written in, the way a mode string's own suffix
                // does.
                if let Some(held @ Object::Dict(options)) = arguments.last() {
                    // The rest of what the options say is read on the Ruby
                    // side, where the encodings they name are objects.
                    inst_rc
                        .borrow_mut()
                        .set_var("__file_options".to_string(), held.clone());
                    if let Some(named) = options.borrow().get(":encoding") {
                        inst_rc
                            .borrow_mut()
                            .set_var("__file_encoding".to_string(), named.clone());
                    }
                    // `newline: :crlf` names what ends a line the stream
                    // writes, which is not the newline everywhere.
                    if let Some(named) = options.borrow().get(":newline") {
                        inst_rc
                            .borrow_mut()
                            .set_var("__file_newline".to_string(), named.clone());
                    }
                }
                let handle = Object::Instance(inst_rc);
                // Opening happens now rather than at the first read, so a
                // name that stands for nothing is refused here, and a stream
                // opened for writing brings the file into being.
                let block = self.pending_block.take();
                self.send_to_object(handle.clone(), "__stream_handle__", Vec::new(), position)?;
                if let Some(allowed) = allowed {
                    use std::os::unix::fs::PermissionsExt as _;
                    let _ =
                        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(allowed));
                }
                if let Some(Object::Block(b)) = block {
                    let result = self.execute_block_callable(&b, vec![handle], position);
                    return result.map(Some);
                }
                Ok(Some(handle))
            }
            _ => Ok(None),
        }
    }
}
