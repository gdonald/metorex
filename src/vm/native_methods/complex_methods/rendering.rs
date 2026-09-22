// What a Complex reads back as.

use super::*;

impl VirtualMachine {
    /// One part of a Complex as text, through the method the caller names.
    pub(crate) fn render_part(
        &mut self,
        value: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        match self.send_to_object(value.clone(), method_name, vec![], position)? {
            Object::String(text) => Ok(text.as_str().to_string()),
            other => Ok(other.to_string()),
        }
    }

    /// Whether a part sits below zero, which decides the sign between the two
    /// halves. A negative zero counts, since Ruby shows `1-0.0i`.
    pub(crate) fn part_is_negative(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if let Object::Float(number) = value {
            // NaN carries a sign bit that varies by platform, and Ruby prints
            // it with a `+` either way, so only a real negative counts. A
            // negative zero keeps its sign, which `-0.0 < 0` would miss.
            return Ok(!number.is_nan() && number.is_sign_negative());
        }
        let answer = self.send_to_object(value.clone(), "<", vec![Object::Int(0)], position)?;
        Ok(answer.is_truthy())
    }
}

/// Join the two rendered halves the way Ruby does: a sign between them, and a
/// `*` before the `i` when the imaginary half does not end in a digit.
pub(crate) fn format_complex_parts(real: &str, imaginary: &str, negative: bool) -> String {
    let magnitude = imaginary.strip_prefix('-').unwrap_or(imaginary);
    let separator = if negative { "-" } else { "+" };
    let suffix = match magnitude.chars().last() {
        Some(last) if last.is_ascii_digit() => "i",
        _ => "*i",
    };
    format!("{}{}{}{}", real, separator, magnitude, suffix)
}
