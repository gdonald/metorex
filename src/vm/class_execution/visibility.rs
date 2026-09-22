// The visibility a freshly defined method is given.

use super::*;

/// Mark a freshly defined method with the visibility currently in force in
/// the class body, as set by a bare `private` or `protected`.
pub(crate) fn apply_current_visibility(class: &Rc<Class>, method_name: &str) {
    match class.current_visibility().as_str() {
        "private" => class.set_method_private(method_name.to_string()),
        "protected" => class.set_method_protected(method_name.to_string()),
        // Redefining a method under the default visibility makes it public
        // again, whatever it was before.
        _ => class.clear_method_visibility(method_name),
    }
}

/// Whether a bare `module_function` is in force in this body, so a method
/// defined here is also copied to the module object.
pub(crate) fn module_function_is_active(class: &Rc<Class>) -> bool {
    class.current_visibility() == MODULE_FUNCTION_VISIBILITY
}

/// Whether `needle` appears anywhere in `module_rc`'s ancestor chain
/// (mixins of mixins, plus superclass mixins). Used to detect a cyclic
/// `include`/`append_features` request before the chain is mutated.
pub(crate) fn module_includes(module_rc: &Rc<Class>, needle: &Rc<Class>) -> bool {
    let mut stack: Vec<Rc<Class>> = vec![Rc::clone(module_rc)];
    let mut seen: Vec<*const Class> = Vec::new();
    while let Some(current) = stack.pop() {
        let ptr = Rc::as_ptr(&current);
        if seen.contains(&ptr) {
            continue;
        }
        seen.push(ptr);
        if Rc::ptr_eq(&current, needle) {
            return true;
        }
        for mixin in current.mixin_chain() {
            stack.push(mixin);
        }
        if let Some(sc) = current.superclass() {
            stack.push(sc);
        }
    }
    false
}
