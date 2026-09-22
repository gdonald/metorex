// The order a Range method is looked for in. Each step answers `None` when
// the name is none of its own, and the next step is tried.

use super::*;

/// One step of the chain: every group of Range methods reads the same
/// arguments and answers the same way.
type RangeMethodStep = fn(
    &mut VirtualMachine,
    &Object,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

const STEPS: &[RangeMethodStep] = &[
    VirtualMachine::call_range_walking_method,
    VirtualMachine::call_range_mapping_method,
    VirtualMachine::call_range_bounds_method,
    VirtualMachine::call_range_describing_method,
];

impl VirtualMachine {
    /// The Range method `method_name` names, run against `receiver`, or
    /// `None` when the receiver is not a Range and no group answers.
    pub(crate) fn call_range_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if !matches!(receiver, Object::Range { .. }) {
            return Ok(None);
        }
        // A subclass instance stands wherever a range does, so an argument
        // that is one is read as the range behind it.
        let arguments: Vec<Object> = arguments
            .iter()
            .map(|held| crate::vm::native_methods::as_range(held).unwrap_or_else(|| held.clone()))
            .collect();
        let arguments = arguments.as_slice();
        for step in STEPS {
            if let Some(result) = step(self, receiver, method_name, arguments, position)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}
