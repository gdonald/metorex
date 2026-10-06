// Putting elements in order, with the block when one is given.

use super::*;

impl VirtualMachine {
    /// The elements in order, each pair ordered with the block when one is
    /// given and with `<=>` otherwise. Elements that compare equal keep the
    /// order they came in.
    pub(crate) fn sort_elements(
        &mut self,
        elements: Vec<Object>,
        block: Option<Rc<crate::object::BlockStatement>>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        self.merge_sort(elements, |machine, left, right| {
            machine.compare_elements(left, right, &block, position)
        })
    }

    /// The elements of `(key, element)` pairs ordered by their keys with
    /// `<=>`. Pairs whose keys compare equal keep the order they came in.
    pub(crate) fn sort_by_keys(
        &mut self,
        keyed: Vec<(Object, Object)>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let sorted = self.merge_sort(keyed, |machine, (left, _), (right, _)| {
            machine.compare_elements(left, right, &None, position)
        })?;
        Ok(sorted.into_iter().map(|(_, element)| element).collect())
    }

    /// A merge sort whose comparison can raise, which is any comparison that
    /// runs Ruby code. Runs of one, then two, then four are merged, taking
    /// the left run's item while it is not greater than the right's.
    fn merge_sort<T: Clone>(
        &mut self,
        items: Vec<T>,
        mut compare: impl FnMut(&mut Self, &T, &T) -> Result<i64, MetorexError>,
    ) -> Result<Vec<T>, MetorexError> {
        let length = items.len();
        let mut current = items;
        let mut merged = Vec::with_capacity(length);
        let mut width = 1;
        while width < length {
            merged.clear();
            let mut start = 0;
            while start < length {
                let middle = (start + width).min(length);
                let end = (start + 2 * width).min(length);
                let (mut left, mut right) = (start, middle);
                while left < middle && right < end {
                    if compare(self, &current[right], &current[left])? < 0 {
                        merged.push(current[right].clone());
                        right += 1;
                    } else {
                        merged.push(current[left].clone());
                        left += 1;
                    }
                }
                merged.extend_from_slice(&current[left..middle]);
                merged.extend_from_slice(&current[right..end]);
                start = end;
            }
            std::mem::swap(&mut current, &mut merged);
            width *= 2;
        }
        Ok(current)
    }

    /// Order two elements with `<=>`, or with the block when one is given.
    /// A comparison that answers nil is an ArgumentError, which is what Ruby
    /// raises when the two cannot be ordered.
    pub(crate) fn compare_elements(
        &mut self,
        left: &Object,
        right: &Object,
        block: &Option<Rc<crate::object::BlockStatement>>,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let answer = match block {
            Some(block) => {
                self.execute_block_callable(block, vec![left.clone(), right.clone()], position)?
            }
            None => {
                // A class with no `<=>` at all cannot be ordered, and Ruby
                // reports the missing method rather than a failed comparison.
                if matches!(left, Object::Instance(_)) && !self.responds_to(left, "<=>") {
                    let wording = self.receiver_wording_for(left, position);
                    return Err(crate::vm::errors::undefined_method_error_worded(
                        "<=>",
                        left,
                        std::slice::from_ref(right),
                        wording,
                        position,
                    ));
                }
                self.send_to_object(left.clone(), "<=>", vec![right.clone()], position)?
            }
        };
        match answer {
            Object::Int(order) => Ok(order),
            Object::Float(order) => Ok(order as i64),
            // A bignum answers only by its sign, and an object of the
            // program's own is asked how it compares to zero, which is what
            // Ruby reduces a block result to.
            Object::BigInt(ref value) => {
                Ok(num_bigint::BigInt::from(0).cmp(value).reverse() as i64)
            }
            Object::Instance(_) => {
                // Ruby asks a value that is not already a number which side
                // of zero it falls on, reading `>` first and then `<`.
                for (named, order) in [(">", 1), ("<", -1)] {
                    if self
                        .send_to_object(answer.clone(), named, vec![Object::Int(0)], position)?
                        .is_truthy()
                    {
                        return Ok(order);
                    }
                }
                Ok(0)
            }
            _ => Err(comparison_failed(left, right, position)),
        }
    }

    /// The elements of `elements` with later duplicates dropped. A block
    /// decides what counts as a duplicate by naming a key for each element.
    /// The unique elements of an array as it stands, read one index at a
    /// time so an element appended while the walk runs is visited too.
    pub(crate) fn unique_live_elements(
        &mut self,
        array_rc: &Rc<std::cell::RefCell<Vec<Object>>>,
        block: Option<Rc<crate::object::BlockStatement>>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut seen: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
        let mut unique = Vec::new();
        let mut index = 0usize;
        loop {
            let Some(element) = array_rc.borrow().get(index).cloned() else {
                break;
            };
            let key = match &block {
                None => element.clone(),
                Some(block) => {
                    self.execute_block_callable(block, vec![element.clone()], position)?
                }
            };
            let slot = self.dict_slot_in(&seen, &key, false, position)?;
            if !seen.contains_key(&slot) {
                crate::vm::native_methods::remember_key_object(&mut seen, &slot, &key);
                seen.insert(slot, Object::Nil);
                unique.push(element);
            }
            index += 1;
        }
        Ok(unique)
    }
}

/// The ArgumentError Ruby raises when two values cannot be ordered.
pub(crate) fn comparison_failed(left: &Object, right: &Object, position: Position) -> MetorexError {
    // An instance reports its own class; every other kind reports the class
    // Ruby names it with, which `class_of` does not know for true and nil.
    let name_of = |value: &Object| match value {
        Object::Instance(instance) => instance.borrow().class.ruby_name(),
        other => crate::vm::native_methods::define_method::ruby_class_name(other).to_string(),
    };
    let message = format!(
        "comparison of {} with {} failed",
        name_of(left),
        name_of(right)
    );
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
}
