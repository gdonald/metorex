// The order a String method is looked for in. Each step answers `None` when
// the name is none of its own, and the next step is tried.

use super::*;

/// One step of the chain: every group of String methods reads the same
/// arguments and answers the same way.
type StringMethodStep = fn(
    &mut VirtualMachine,
    &Object,
    &Rc<crate::object::StringValue>,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

const STEPS: &[StringMethodStep] = &[
    VirtualMachine::call_string_inspection_method,
    VirtualMachine::call_string_match_method,
    VirtualMachine::call_string_encode_method,
    VirtualMachine::call_string_case_method,
    VirtualMachine::call_string_trimming_method,
    VirtualMachine::call_string_byte_method,
    VirtualMachine::call_string_character_method,
    VirtualMachine::call_string_encoding_method,
    VirtualMachine::call_string_padding_method,
    VirtualMachine::call_string_slicing_method,
    VirtualMachine::call_string_searching_method,
    VirtualMachine::call_string_conversion_method,
    VirtualMachine::call_string_substitution_method,
];

impl VirtualMachine {
    /// The String method `method_name` names, run against `receiver`, or
    /// `None` when the receiver is not a String and no group answers.
    pub(crate) fn call_string_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(string_value) = receiver else {
            return Ok(None);
        };
        for step in STEPS {
            if let Some(result) = step(
                self,
                receiver,
                string_value,
                method_name,
                arguments,
                position,
            )? {
                return Ok(Some(result));
            }
        }
        self.call_string_set_method(receiver, method_name, arguments, position)
    }
}
