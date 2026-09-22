// Descending into nested arrays.

use super::*;

/// Copy `elements` into `flat`, descending into nested arrays until `depth`
/// levels have been unwrapped. A negative depth descends all the way.
/// `in_flight` holds the arrays being walked, so an array that contains
/// itself raises rather than recursing forever.
impl VirtualMachine {
    pub(crate) fn flatten_into(
        &mut self,
        elements: &[Object],
        depth: i64,
        in_flight: &mut Vec<usize>,
        flat: &mut Vec<Object>,
        position: Position,
    ) -> Result<(), MetorexError> {
        for element in elements {
            match element {
                Object::Array(nested) if depth != 0 => {
                    let address = Rc::as_ptr(nested) as usize;
                    if in_flight.contains(&address) {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "tries to flatten self",
                            position,
                        ));
                    }
                    let inner = nested.borrow().clone();
                    in_flight.push(address);
                    self.flatten_into(&inner, depth - 1, in_flight, flat, position)?;
                    in_flight.pop();
                }
                other if depth != 0 => match self.array_for_flatten(other, position)? {
                    Some(inner) => {
                        self.flatten_into(&inner, depth - 1, in_flight, flat, position)?
                    }
                    None => flat.push(other.clone()),
                },
                other => flat.push(other.clone()),
            }
        }
        Ok(())
    }

    /// The array an element spells, where it spells one. `flatten` asks every
    /// element that is not an Array, the way Ruby asks: an object saying it
    /// answers `to_ary`, or one with a `method_missing` to catch the call.
    fn array_for_flatten(
        &mut self,
        element: &Object,
        position: Position,
    ) -> Result<Option<Vec<Object>>, MetorexError> {
        if !matches!(element, Object::Instance(_)) {
            return Ok(None);
        }
        if let Some(held) = crate::vm::native_methods::array_subclass_value(element) {
            if let Object::Array(items) = held {
                return Ok(Some(items.borrow().clone()));
            }
            return Ok(None);
        }
        let named = Object::symbol("to_ary".to_string());
        // A redefined `respond_to?` has the last word, the way Ruby asks it
        // before reaching for a conversion at all.
        if self.lookup_method(element, "respond_to?").is_some()
            && !self
                .send_to_object(
                    element.clone(),
                    "respond_to?",
                    vec![named.clone(), Object::Bool(true)],
                    position,
                )?
                .is_truthy()
        {
            return Ok(None);
        }
        let answered = if self.responds_to(element, "to_ary") {
            self.send_to_object(element.clone(), "to_ary", vec![], position)?
        } else {
            // With no `to_ary` of its own, the object is asked only where it
            // has a `method_missing` to catch the call, and a refusal from
            // that stands for having no conversion at all.
            let mut says_it_does = false;
            if self.lookup_method(element, "respond_to_missing?").is_some() {
                says_it_does = self
                    .send_to_object(
                        element.clone(),
                        "respond_to_missing?",
                        vec![named, Object::Bool(true)],
                        position,
                    )?
                    .is_truthy();
                if !says_it_does {
                    return Ok(None);
                }
            }
            if self.lookup_method(element, "method_missing").is_none() {
                return Ok(None);
            }
            match self.send_to_object(element.clone(), "to_ary", vec![], position) {
                Ok(held) => held,
                Err(problem) => {
                    // An object that said it answers the call keeps the
                    // failure, and one that never claimed to spells no array.
                    if refused_the_call(&problem) && !says_it_does {
                        return Ok(None);
                    }
                    return Err(problem);
                }
            }
        };
        if let Object::Array(items) = &answered {
            return Ok(Some(items.borrow().clone()));
        }
        if let Some(Object::Array(items)) =
            crate::vm::native_methods::array_subclass_value(&answered)
        {
            return Ok(Some(items.borrow().clone()));
        }
        if matches!(answered, Object::Nil) {
            return Ok(None);
        }
        let message = format!(
            "can't convert {} to Array ({}#to_ary gives {})",
            self.builtins().class_of(element).name(),
            self.builtins().class_of(element).name(),
            self.builtins().class_of(&answered).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }
}

/// `rotate` moves the first `by` elements to the end, counting from the end
/// when negative. An empty array rotates to itself.
pub(crate) fn rotate_elements(elements: &[Object], by: i64) -> Vec<Object> {
    if elements.is_empty() {
        return Vec::new();
    }
    let length = elements.len() as i64;
    let offset = by.rem_euclid(length) as usize;
    let mut rotated = elements[offset..].to_vec();
    rotated.extend_from_slice(&elements[..offset]);
    rotated
}
