// How an array reads back as source.

use super::*;

/// Render array elements the way `Array#inspect` does, recursing so a nested
/// array's own strings and symbols keep their quoting.
pub(crate) fn inspect_elements(elements: &[Object]) -> String {
    let parts: Vec<String> = elements.iter().map(inspect_element).collect();
    format!("[{}]", parts.join(", "))
}

/// `inspect` for a nested array, which prints `[...]` when the array reaches
/// itself rather than recursing forever.
pub(crate) fn inspect_nested(nested: &Rc<RefCell<Vec<Object>>>) -> String {
    let elements = nested.borrow().clone();
    crate::object::render_guarded(Rc::as_ptr(nested) as usize, || inspect_elements(&elements))
        .unwrap_or_else(|| "[...]".to_string())
}

pub(crate) fn inspect_element(element: &Object) -> String {
    match element {
        Object::String(s) => format!("{:?}", s.as_str()),
        Object::Symbol(s) => crate::object::inspect_symbol(&s.as_str()),
        Object::Nil => "nil".to_string(),
        Object::Array(nested) => inspect_nested(nested),
        other => other.to_string(),
    }
}
