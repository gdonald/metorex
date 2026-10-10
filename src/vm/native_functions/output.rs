// Writing values out.

use super::*;

impl VirtualMachine {
    pub(crate) fn put_lines(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Ruby's `puts` hands its arguments to `$stdout.puts`, which
        // is what a program replacing that stream relies on.
        let target = self.globals().get("stdout").unwrap_or(Object::Nil);
        if !matches!(target, Object::Nil) && self.responds_to(&target, "puts") {
            self.send_to_object(target, "puts", arguments, position)?;
            return Ok(Object::Nil);
        }
        if arguments.is_empty() {
            self.write_to_stdout("\n", position)?;
        }
        for arg in &arguments {
            self.puts_object(arg, position)?;
        }
        Ok(Object::Nil)
    }

    /// Kernel#print writes its arguments with no separator. With no
    /// arguments it writes `$_`, the last line `gets` read.
    pub(crate) fn print_values(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() {
            let last_line = self.globals().get("_").unwrap_or(Object::Nil);
            if !matches!(last_line, Object::Nil) {
                let output = self.get_string_representation(&last_line, position)?;
                self.write_to_stdout(&output, position)?;
            }
            return Ok(Object::Nil);
        }
        for arg in &arguments {
            let output = self.get_string_representation(arg, position)?;
            self.write_to_stdout(&output, position)?;
        }
        Ok(Object::Nil)
    }

    /// Kernel#p writes each argument's `inspect` on its own line and
    /// answers with the argument, the argument list, or nil for none.
    /// `printf(io, format, *args)` writes to that io, and
    /// `printf(format, *args)` writes to `$stdout`.
    pub(crate) fn print_formatted(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() {
            return Ok(Object::Nil);
        }
        let first_is_format =
            matches!(&arguments[0], Object::String(_)) || !self.responds_to(&arguments[0], "write");
        let (target, rest) = if first_is_format {
            (None, arguments.as_slice())
        } else {
            (Some(arguments[0].clone()), &arguments[1..])
        };
        let Some(format) = rest.first() else {
            return Ok(Object::Nil);
        };
        let format = match format {
            Object::String(text) => Object::String(std::rc::Rc::clone(text)),
            other => Object::string(self.coerce_load_path(other, position)?),
        };
        let values = Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            rest[1..].to_vec(),
        )));
        let rendered = self.evaluate_string_format(format, values, position)?;
        let text = match &rendered {
            Object::String(text) => text.as_str().to_string(),
            other => other.to_string(),
        };
        match target {
            Some(target) => {
                self.send_to_object(target, "write", vec![Object::string(text)], position)?;
            }
            None => self.write_to_stdout(&text, position)?,
        }
        Ok(Object::Nil)
    }

    /// `pp` loads the pp library the first time it is called and prints
    /// each value through `PP.pp`, which breaks a wide structure across lines.
    pub(crate) fn pretty_print_values(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.require_feature(vec![Object::string("pp")], position)?;
        let printer = self.globals().get("PP").unwrap_or(Object::Nil);
        for argument in &arguments {
            self.send_to_object(printer.clone(), "pp", vec![argument.clone()], position)?;
        }
        match arguments.len() {
            0 => Ok(Object::Nil),
            1 => Ok(arguments.into_iter().next().unwrap_or(Object::Nil)),
            _ => Ok(Object::array(arguments)),
        }
    }

    /// `p` writes each value's `inspect` on a line of its own.
    pub(crate) fn inspect_values(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        for argument in &arguments {
            let rendered = self.get_inspect_representation(argument, position)?;
            self.write_to_stdout(&format!("{}\n", rendered), position)?;
        }
        match arguments.len() {
            0 => Ok(Object::Nil),
            1 => Ok(arguments.into_iter().next().unwrap()),
            _ => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                arguments,
            )))),
        }
    }
}
