// Every object the running program can still reach.

use super::*;

/// The names every object answers for itself, which a module keeping no
/// account of its own names still answers truthfully.
pub(crate) const ANSWERED_BY_EVERY_OBJECT: &[&str] = &[
    "==",
    "!=",
    "===",
    "class",
    "clone",
    "display",
    "dup",
    "enum_for",
    "eql?",
    "equal?",
    "freeze",
    "frozen?",
    "hash",
    "inspect",
    "instance_of?",
    "instance_variable_defined?",
    "instance_variable_get",
    "instance_variable_set",
    "instance_variables",
    "is_a?",
    "itself",
    "kind_of?",
    "method",
    "methods",
    "nil?",
    "object_id",
    "public_send",
    "respond_to?",
    "send",
    "singleton_class",
    "tap",
    "then",
    "to_enum",
    "to_s",
    "yield_self",
    "__id__",
    "__send__",
];

/// Collect an object and everything it leads to, passing over one already
/// collected so a structure that holds itself is walked once.
pub(crate) fn gather_reachable(
    object: Object,
    seen: &mut std::collections::HashSet<usize>,
    found: &mut Vec<Object>,
) {
    let identity = match &object {
        Object::String(held) => Rc::as_ptr(held) as usize,
        Object::Array(held) => Rc::as_ptr(held) as usize,
        Object::Dict(held) => Rc::as_ptr(held) as usize,
        Object::Instance(held) => Rc::as_ptr(held) as usize,
        Object::Class(held) | Object::Module(held) => Rc::as_ptr(held) as usize,
        _ => 0,
    };
    if identity == 0 || !seen.insert(identity) {
        return;
    }
    found.push(object.clone());
    match &object {
        Object::Array(held) => {
            for element in held.borrow().iter() {
                gather_reachable(element.clone(), seen, found);
            }
        }
        Object::Dict(held) => {
            for (_, value) in held.borrow().iter() {
                gather_reachable(value.clone(), seen, found);
            }
        }
        Object::Instance(held) => {
            let names: Vec<Object> = held.borrow().instance_vars.values().cloned().collect();
            for value in names {
                gather_reachable(value, seen, found);
            }
        }
        _ => {}
    }
}
