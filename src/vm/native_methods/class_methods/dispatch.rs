// The order a class method is looked for in. Each step answers `None` when
// the name is none of its own, and the next step is tried.

use super::*;

/// One step of the chain: every group of class methods reads the same
/// arguments and answers the same way.
type ClassMethodStep = fn(
    &mut VirtualMachine,
    &Rc<Class>,
    &str,
    &[Object],
    Position,
) -> Result<ClassMethodAnswer, MetorexError>;

const STEPS: &[ClassMethodStep] = &[
    VirtualMachine::call_primitive_class_methods,
    VirtualMachine::call_allocation_class_methods,
    VirtualMachine::call_reflection_class_methods,
    VirtualMachine::call_kernel_and_new_class_methods,
    VirtualMachine::call_autoload_class_methods,
    VirtualMachine::call_encoding_class_methods,
    VirtualMachine::call_autoload_query_class_methods,
    VirtualMachine::call_mixin_class_methods,
    VirtualMachine::call_regexp_class_methods,
    VirtualMachine::call_file_class_methods,
    VirtualMachine::call_thread_class_methods,
    VirtualMachine::call_collection_class_methods,
    VirtualMachine::call_module_naming_class_methods,
    VirtualMachine::call_instance_method_class_methods,
    VirtualMachine::call_attribute_class_methods,
    VirtualMachine::call_module_query_class_methods,
    VirtualMachine::call_constant_class_methods,
    VirtualMachine::call_method_definition_class_methods,
    VirtualMachine::call_class_variable_class_methods,
];

impl VirtualMachine {
    /// The class method `method_name` names, run against `class_rc`, or
    /// `None` when no group of class methods answers that name.
    pub(crate) fn call_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        for step in STEPS {
            match step(self, class_rc, method_name, arguments, position)? {
                Answered(result) => return Ok(Some(result)),
                Deferred => return Ok(None),
                Unclaimed => {}
            }
        }
        Ok(None)
    }
}
