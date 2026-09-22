// Writing elements out as bytes.

use super::*;

impl VirtualMachine {
    /// Write a packed result into the String a `buffer:` keyword named. The
    /// writing starts at the end of what the buffer holds, or at the offset a
    /// leading `@` directive names, and the buffer is the answer.
    pub(crate) fn pack_into_buffer(
        &mut self,
        buffer: Object,
        format: &str,
        packed: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Object::String(held) = &buffer else {
            return Ok(buffer);
        };
        if held.is_frozen() {
            return Err(self.frozen_modification_error(&buffer, position));
        }
        let existing = crate::vm::native_methods::pack_format::string_to_bytes(&held.as_str());
        let written = match packed {
            Object::String(text) => {
                crate::vm::native_methods::pack_format::string_to_bytes(&text.as_str())
            }
            _ => Vec::new(),
        };
        let (start, tail) = match leading_pack_offset(format) {
            Some(offset) => (offset, written.get(offset..).unwrap_or(&[]).to_vec()),
            None => (existing.len(), written),
        };
        let mut made = existing;
        made.resize(start, 0);
        made.extend_from_slice(&tail);
        held.replace_text(made.iter().map(|byte| *byte as char).collect::<String>());
        held.mark_bytes();
        Ok(buffer)
    }
}

/// The absolute offset a format's leading `@` directive names, where it has
/// one. Any other format writes at the end of what the buffer holds.
pub(crate) fn leading_pack_offset(format: &str) -> Option<usize> {
    let digits = format.strip_prefix('@')?;
    let counted: String = digits.chars().take_while(char::is_ascii_digit).collect();
    counted.parse().ok()
}
