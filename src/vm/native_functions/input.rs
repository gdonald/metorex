// Reading a line of standard input.

use super::*;

impl VirtualMachine {
    /// Read one line from stdin, without its line ending.
    pub(crate) fn read_line_from_stdin(
        &mut self,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match self.read_raw_line_from_stdin(position)? {
            Some(line) => Ok(Object::string(line)),
            None => Ok(Object::Nil),
        }
    }

    /// One line of stdin with its terminator trimmed, or `None` at end of
    /// input. `gets` answers nil there and `readline` raises EOFError.
    pub(crate) fn read_raw_line_from_stdin(
        &mut self,
        position: Position,
    ) -> Result<Option<String>, MetorexError> {
        let mut input = String::new();
        let read = std::io::stdin().read_line(&mut input).map_err(|error| {
            MetorexError::runtime_error(
                format!("Failed to read from stdin: {}", error),
                crate::vm::utils::position_to_location(position),
            )
        })?;
        if read == 0 {
            return Ok(None);
        }
        if input.ends_with('\n') {
            input.pop();
            if input.ends_with('\r') {
                input.pop();
            }
        }
        Ok(Some(input))
    }
}
