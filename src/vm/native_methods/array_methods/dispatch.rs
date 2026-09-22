// The order an Array method is looked for in. Each step answers `None` when
// the name is none of its own, and the next step is tried.

use super::*;

/// One step of the chain: every group of Array methods reads the same
/// arguments and answers the same way.
type ArrayMethodStep = fn(
    &mut VirtualMachine,
    &Object,
    &Rc<RefCell<Vec<Object>>>,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

const STEPS: &[ArrayMethodStep] = &[
    VirtualMachine::call_array_basic_method,
    VirtualMachine::call_array_access_method,
    VirtualMachine::call_array_iteration_method,
    VirtualMachine::call_array_folding_method,
    VirtualMachine::call_array_ordering_method,
    VirtualMachine::call_array_joining_method,
    VirtualMachine::call_array_edge_method,
    VirtualMachine::call_array_filtering_method,
    VirtualMachine::call_array_searching_method,
    VirtualMachine::call_array_aggregate_method,
    VirtualMachine::call_array_writing_method,
    VirtualMachine::call_array_predicate_method,
    VirtualMachine::call_array_in_place_method,
];

impl VirtualMachine {
    /// The Array method `method_name` names, run against `receiver`, or
    /// `None` when the receiver is not an Array and no group answers.
    pub(crate) fn call_array_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Array(array_rc) = receiver else {
            return Ok(None);
        };
        // Every method that changes the array in place refuses a frozen one.
        const MUTATORS: &[&str] = &[
            "initialize",
            "<<",
            "append",
            "push",
            "pop",
            "shift",
            "unshift",
            "prepend",
            "insert",
            "delete_at",
            "clear",
            "concat",
            "replace",
            "fill",
            "compact!",
            "flatten!",
            "map!",
            "collect!",
            "reject!",
            "select!",
            "filter!",
            "reverse!",
            "rotate!",
            "shuffle!",
            "slice!",
            "sort!",
            "sort_by!",
            "uniq!",
            "[]=",
        ];
        // A method that would modify the array refuses on a frozen one. The
        // block-taking ones answer an Enumerator first when no block is given,
        // which Ruby allows even on a frozen array.
        let answers_enumerator = matches!(
            method_name,
            "map!"
                | "collect!"
                | "reject!"
                | "select!"
                | "filter!"
                | "keep_if"
                | "delete_if"
                | "sort_by!"
        ) && !matches!(self.pending_block, Some(Object::Block(_)));
        if MUTATORS.contains(&method_name) && !answers_enumerator && self.object_is_frozen(receiver)
        {
            return Err(self.frozen_modification_error(receiver, position));
        }
        for step in STEPS {
            if let Some(result) = step(self, receiver, array_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}
