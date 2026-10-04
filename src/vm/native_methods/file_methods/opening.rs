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
                    let block = self.pending_block.take();
                    let opened = self.send_to_object(
                        Object::Class(Rc::clone(class_rc)),
                        "for_fd",
                        passed,
                        position,
                    )?;
                    return match block {
                        Some(Object::Block(block)) => {
                            self.run_and_close(&block, opened, position).map(Some)
                        }
                        _ => Ok(Some(opened)),
                    };
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
                    Some(Object::Nil) => (named_mode.unwrap_or_else(|| "r".to_string()), None),
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
                if numeric.is_none() && !valid_access_mode(&mode) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("invalid access mode {mode}"),
                        position,
                    ));
                }
                // An encoding named in the mode may not be named again as an
                // option.
                if mode.contains(':')
                    && let Some(Object::Dict(options)) = arguments.last()
                    && [":encoding", ":external_encoding", ":internal_encoding"]
                        .iter()
                        .any(|key| options.borrow().contains_key(*key))
                {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "encoding specified twice",
                        position,
                    ));
                }
                let binary_mode = mode.split(':').next().unwrap_or("").contains('b');
                // A stream read in an encoding that spells ASCII another way
                // reads bytes it cannot split into lines unless it reads
                // them as bytes.
                let options = match arguments.last() {
                    Some(Object::Dict(options)) => Some(options.borrow().clone()),
                    _ => None,
                };
                let option_named = |key: &str| {
                    options
                        .as_ref()
                        .and_then(|held| held.get(key).cloned())
                        .filter(|held| !matches!(held, Object::Nil))
                };
                let mut named_encodings = mode.split(':').skip(1).map(str::to_string);
                let mut external = named_encodings.next();
                let mut internal = named_encodings.next();
                for key in [":encoding", ":external_encoding"] {
                    if let Some(held) = option_named(key)
                        && let Ok(name) = self.encoding_name_argument(&held, position)
                    {
                        let mut parts = name.split(':').map(str::to_string);
                        external = parts.next();
                        internal = internal.or(parts.next());
                    }
                }
                if let Some(held) = option_named(":internal_encoding")
                    && let Ok(name) = self.encoding_name_argument(&held, position)
                {
                    internal = Some(name);
                }
                let readable = mode.starts_with('r') || mode.contains('+');
                let binmode_option = option_named(":binmode").is_some_and(|held| held.is_truthy());
                let wide = |name: &str| {
                    let name = name.to_ascii_uppercase();
                    name.starts_with("UTF-16") || name.starts_with("UTF-32")
                };
                if numeric.is_none()
                    && readable
                    && !binary_mode
                    && !binmode_option
                    && internal.is_none()
                    && external.as_deref().is_some_and(wide)
                {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "ASCII incompatible encoding needs binmode",
                        position,
                    ));
                }
                if binary_mode
                    && let Some(Object::Dict(options)) = arguments.last()
                    && options.borrow().contains_key(":newline")
                {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "newline decorator with binary mode",
                        position,
                    ));
                }
                let mut flags = (numeric.unwrap_or(0) | named_flags.unwrap_or(0)) as libc::c_int;
                // `x` asks for a file of its own, which is what O_EXCL says.
                if mode.split(':').next().unwrap_or("").contains('x') {
                    flags |= libc::O_EXCL | libc::O_CREAT;
                }
                // Numeric flags say for themselves whether the file is written
                // and cut down, where a mode string says it by its letters.
                let (writing, truncate) = match numeric {
                    Some(bits) => {
                        let bits = bits as libc::c_int;
                        (
                            bits & libc::O_ACCMODE != libc::O_RDONLY,
                            bits & libc::O_TRUNC != 0,
                        )
                    }
                    None => (
                        mode.contains('w') || mode.contains('a') || mode.contains('+'),
                        mode.contains('w'),
                    ),
                };
                let there = std::path::Path::new(&path).exists();
                // Asked for a file of its own, a name that already stands for
                // one is refused.
                // A mode that names writing from the start brings the file
                // into being, where one that only adds to what is there does
                // not.
                let creates = match numeric {
                    Some(_) => flags & libc::O_CREAT != 0,
                    None => {
                        mode.starts_with('w') || mode.starts_with('a') || flags & libc::O_CREAT != 0
                    }
                };
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
                // Only a regular file is cut down here. Opening anything else
                // for writing, such as a FIFO, waits for its other end.
                if writing && truncate && std::path::Path::new(&path).is_file() {
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
                let inst_rc = Instance::new(Rc::clone(class_rc));
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
                // A file opened by numeric flags is opened with those flags.
                if let Some(bits) = numeric {
                    inst_rc
                        .borrow_mut()
                        .set_var("__file_flags".to_string(), Object::Int(bits));
                }
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
                    return self.run_and_close(&b, handle, position).map(Some);
                }
                Ok(Some(handle))
            }
            _ => Ok(None),
        }
    }
}

impl VirtualMachine {
    /// Hand an open file to a block, then close it through `close` whatever
    /// the block did. A close that finds it closed already says nothing.
    fn run_and_close(
        &mut self,
        block: &crate::object::BlockStatement,
        handle: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let result = self.execute_block_callable(block, vec![handle.clone()], position);
        let closed = self.send_to_object(handle, "close", vec![], position);
        if let Err(problem) = closed
            && !reports_a_closed_stream(&problem)
        {
            return Err(problem);
        }
        result
    }
}

/// Whether an error is the IOError a stream raises for being closed already.
fn reports_a_closed_stream(problem: &MetorexError) -> bool {
    match problem {
        MetorexError::UncaughtException {
            exception: Object::Exception(details),
            ..
        } => {
            let details = details.borrow();
            details.exception_type == "IOError" && details.message == "closed stream"
        }
        _ => false,
    }
}
