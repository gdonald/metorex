// The order an Object method is looked for in. Each step answers `None` when
// the name is none of its own, and the next step is tried.

use super::*;

/// One step of the chain: every group of Object methods reads the same
/// arguments and answers the same way.
type ObjectMethodStep = fn(
    &mut VirtualMachine,
    &Object,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

const STEPS: &[ObjectMethodStep] = &[
    VirtualMachine::call_object_sending_method,
    VirtualMachine::call_object_state_method,
    VirtualMachine::call_object_initializing_method,
    VirtualMachine::call_object_clamping_method,
    VirtualMachine::call_object_singleton_method,
    VirtualMachine::call_object_describing_method,
    VirtualMachine::call_object_method_lookup_method,
    VirtualMachine::call_object_type_check_method,
    VirtualMachine::call_object_instance_variable_method,
    VirtualMachine::call_object_method_list_method,
    VirtualMachine::call_object_equality_method,
    VirtualMachine::call_object_evaluating_method,
];

impl VirtualMachine {
    /// The Object method `method_name` names, run against `receiver`, or
    /// `None` when no group answers that name.
    pub(crate) fn call_object_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let entered = self.enter_native_frame(Some(receiver), method_name, position);
        let answered = self.call_object_method_body(receiver, method_name, arguments, position);
        self.leave_native_call(entered, position, &answered);
        answered
    }

    fn call_object_method_body(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // An instance of BasicObject, or of a class rooted there rather than
        // at Object, answers only the handful of methods BasicObject defines.
        // Everything Kernel and Object add arrives through Object, which such
        // a class never inherits from.
        if !basic_object_answers(method_name)
            && self.bound_stub_depth == 0
            && self.rooted_at_basic_object(receiver)
            && !class_defines(receiver, method_name)
        {
            return Ok(None);
        }

        // Kernel#autoload / #autoload? — a top-level (or any non-module)
        // receiver registers the autoload on Object, Ruby's home for
        // top-level constants.
        if matches!(method_name, "autoload" | "autoload?")
            && let Some(Object::Class(object_class)) = self.globals().get("Object")
        {
            return self.call_class_methods(&object_class, method_name, arguments, position);
        }

        // Operators reached by name rather than by syntax — `1.send(:+, 2)`,
        // or a method body built from `:+.to_proc`. Route them back through
        // the binary-operator evaluator. A pair it has no rule for is left to
        // the receiver's own methods, which answer or refuse it as Ruby does.
        if arguments.len() == 1
            && let Some(op) = binary_op_for_method_name(method_name)
        {
            return match self.evaluate_binary_operation(
                &op,
                receiver.clone(),
                arguments[0].clone(),
                position,
            ) {
                Err(MetorexError::TypeError { .. }) => Ok(None),
                answered => answered.map(Some),
            };
        }

        // A Proc is already one, so `to_proc` answers the same object.
        if method_name == "to_proc" && matches!(receiver, Object::Block(_)) {
            return Ok(Some(receiver.clone()));
        }

        // Symbol#to_proc — `:foo.to_proc` is a two-parameter callable that
        // sends `foo` to its first argument.
        if method_name == "to_proc"
            && let Object::Symbol(name) = receiver
        {
            return Ok(Some(Object::Block(std::rc::Rc::new(symbol_to_proc_block(
                &name.as_str(),
                position,
            )))));
        }

        // Nil-specific conversions: in Ruby `nil.to_i == 0`, `nil.to_s == ""`,
        // `nil.to_a == []`, `nil.to_f == 0.0`. The dispatch above checks the
        // class of the receiver first, so we have to intercept here for Nil
        // before falling through to the generic Object methods.
        if matches!(receiver, Object::Nil) {
            match method_name {
                "to_i" => return Ok(Some(Object::Int(0))),
                "to_f" => return Ok(Some(Object::Float(0.0))),
                "to_a" => {
                    return Ok(Some(Object::Array(std::rc::Rc::new(
                        std::cell::RefCell::new(Vec::new()),
                    ))));
                }
                "to_h" => {
                    return Ok(Some(Object::Dict(std::rc::Rc::new(
                        std::cell::RefCell::new(indexmap::IndexMap::new()),
                    ))));
                }
                // One string that never changes, so asking twice answers the
                // same object.
                "to_s" => return Ok(Some(self.memoized_text("__nil_to_s", ""))),
                "inspect" => return Ok(Some(Object::string("nil"))),
                "to_r" | "rationalize" => {
                    if method_name == "rationalize" && arguments.len() > 1 {
                        let exc = Object::exception(
                            "ArgumentError",
                            format!(
                                "wrong number of arguments (given {}, expected 0..1)",
                                arguments.len()
                            ),
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: format!(
                                "wrong number of arguments (given {}, expected 0..1)",
                                arguments.len()
                            ),
                        });
                    }
                    // Return Rational(0, 1) — create an instance via global function
                    if let Some(Object::Class(rational_class)) = self.globals().get("Rational") {
                        let inst = crate::object::Instance::new(rational_class);
                        inst.borrow_mut()
                            .set_var("numerator".to_string(), Object::Int(0));
                        inst.borrow_mut()
                            .set_var("denominator".to_string(), Object::Int(1));
                        return Ok(Some(Object::Instance(inst)));
                    }
                    return Ok(Some(Object::Int(0)));
                }
                "to_c" => {
                    if let Some(Object::Class(complex_class)) = self.globals().get("Complex") {
                        let inst = crate::object::Instance::new(complex_class);
                        inst.borrow_mut()
                            .set_var("real".to_string(), Object::Int(0));
                        inst.borrow_mut()
                            .set_var("imaginary".to_string(), Object::Int(0));
                        return Ok(Some(Object::Instance(inst)));
                    }
                    return Ok(Some(Object::Int(0)));
                }
                _ => {}
            }
        }
        for step in STEPS {
            if let Some(result) = step(self, receiver, method_name, arguments, position)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}
