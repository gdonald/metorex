// Reading lines in, and opening a stream.

use super::*;

impl VirtualMachine {
    /// `chomp` and `chop` with no receiver rewrite `$_` in place, which
    /// is the line `-n` read. `chomp` takes the separator from `$/`.
    pub(crate) fn chomp_line(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        let line = match self.globals().get("_") {
            Some(Object::String(text)) => text.as_str().to_string(),
            _ => String::new(),
        };
        let separator = match arguments.first() {
            Some(Object::String(text)) => Some(text.as_str().to_string()),
            _ => match self.globals().get("/") {
                Some(Object::String(text)) => Some(text.as_str().to_string()),
                _ => None,
            },
        };
        let trimmed = if name == "chop" {
            let mut trimmed = line.clone();
            if trimmed.ends_with("\r\n") {
                trimmed.truncate(trimmed.len() - 2);
            } else {
                trimmed.pop();
            }
            trimmed
        } else {
            chomped(&line, separator.as_deref())
        };
        let result = Object::string(trimmed);
        self.globals_mut().set_variable("_", result.clone());
        Ok(result)
    }

    /// `open(path, mode = "r", perm = nil, **options)` opens a file.
    /// An argument answering `to_open` is asked to open itself, and
    /// whatever it answers is what `open` hands back.
    pub(crate) fn open_stream(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Any conversion below invokes a method, which would consume
        // the block, so it is taken first and put back at the end.
        let pending = self.pending_block.take();
        let (positional, _keywords) =
            crate::vm::native_methods::kernel_conversion::split_conversion_keywords(&arguments);
        let Some(target) = positional.first() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 3),
                0,
                position,
            ));
        };
        let target = &target.clone();
        if self.responds_to(target, "to_open") {
            let Some((class, method)) = self.lookup_method(target, "to_open") else {
                return Ok(Object::Nil);
            };
            let opened = self.invoke_method(
                class,
                method,
                target.clone(),
                arguments[1..].to_vec(),
                position,
            )?;
            if let Some(Object::Block(block)) = pending {
                return self.execute_block_callable(&block, vec![opened], position);
            }
            return Ok(opened);
        }
        // Only the file form is limited to path, mode, and
        // permissions; `to_open` takes whatever it is given.
        if positional.len() > 3 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 3),
                positional.len(),
                position,
            ));
        }
        let path = self.coerce_load_path(target, position)?;
        let mut open_arguments = vec![Object::string(path)];
        if let Some(mode) = positional.get(1)
            && !matches!(mode, Object::Nil)
        {
            open_arguments.push(mode.clone());
        }
        let Some(Object::Class(file_class)) = self.globals().get("File") else {
            return Err(MetorexError::runtime_error(
                "File is not defined",
                crate::vm::utils::position_to_location(position),
            ));
        };
        self.pending_block = pending;
        self.call_file_dir_methods(&file_class, "open", &open_arguments, position)
            .map(|opened| opened.unwrap_or(Object::Nil))
    }

    /// Kernel#readline — `gets` that refuses to answer nil: at end of
    /// input it raises EOFError.
    pub(crate) fn read_line_or_fail(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                format!("readline() expects 0 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        match self.read_raw_line_from_stdin(position)? {
            Some(line) => Ok(Object::string(line)),
            None => {
                let message = "end of file reached".to_string();
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("EOFError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
        }
    }

    /// Kernel#readlines — every remaining line, as an Array.
    pub(crate) fn read_all_lines(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                format!("readlines() expects 0 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let mut lines = Vec::new();
        while let Some(line) = self.read_raw_line_from_stdin(position)? {
            lines.push(Object::string(line));
        }
        Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            lines,
        ))))
    }

    /// `gets` is ARGF's, so a stand-in installed on ARGF answers here.
    pub(crate) fn read_line(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                format!("gets() expects 0 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let argf = self.globals().get("ARGF").unwrap_or(Object::Nil);
        if let Some((class, method)) = self.lookup_method(&argf, "gets")
            && !method.is_undefined
        {
            return self.invoke_method(class, method, argf, vec![], position);
        }
        self.read_line_from_stdin(position)
    }
}
