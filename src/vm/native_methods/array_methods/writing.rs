// Writing elements into positions the array already has or grows to.

use super::*;

impl VirtualMachine {
    /// Writing elements into positions the array already has or grows to.
    pub(crate) fn call_array_writing_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `insert(index, *objects)` puts the objects before the element at
            // a non-negative index and after it for a negative one, padding
            // with nil when the index is past the end.
            "insert" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                if arguments.len() == 1 {
                    return Ok(Some(receiver.clone()));
                }
                let index: i64 = self
                    .coerce_integer_argument(&arguments[0], position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                let mut array = array_rc.borrow_mut();
                let length = array.len() as i64;
                let at = if index < 0 { index + length + 1 } else { index };
                if at < 0 {
                    let message = format!(
                        "index {} too small for array; minimum: {}",
                        index,
                        -length - 1
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }
                while (array.len() as i64) < at {
                    array.push(Object::Nil);
                }
                for (offset, value) in arguments[1..].iter().enumerate() {
                    array.insert(at as usize + offset, value.clone());
                }
                drop(array);
                Ok(Some(receiver.clone()))
            }
            // `[]=` in its three forms: one index, a start and a length, and
            // a Range. The span forms splice, and a value that is an Array
            // contributes its elements.
            "[]=" => {
                if arguments.len() < 2 || arguments.len() > 3 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let value = arguments[arguments.len() - 1].clone();
                let length = array_rc.borrow().len() as i64;
                // An instance of a Range subclass names a span the same way a
                // plain Range does.
                let first = crate::vm::native_methods::as_range(&arguments[0])
                    .unwrap_or_else(|| arguments[0].clone());
                // A Range start before the front is a RangeError, while the
                // other forms report an IndexError.
                let out_of_range_error =
                    if arguments.len() == 2 && matches!(&first, Object::Range { .. }) {
                        "RangeError"
                    } else {
                        "IndexError"
                    };
                let (start, span) = if arguments.len() == 3 {
                    let start = self.index_from(&arguments[0], length, position)?;
                    let span: i64 = self
                        .coerce_integer_argument(&arguments[1], position)?
                        .try_into()
                        .unwrap_or(i64::MAX);
                    if span < 0 {
                        let message = format!("negative length ({})", span);
                        return Err(crate::vm::errors::simple_exception(
                            "IndexError",
                            &message,
                            position,
                        ));
                    }
                    (start, Some(span))
                } else if let Object::Range { .. } = &first {
                    let (start, span) = self.range_bounds(&first, length, position)?;
                    (start, Some(span))
                } else {
                    (self.index_from(&arguments[0], length, position)?, None)
                };
                if start < 0 {
                    let message = format!(
                        "index {} too small for array; minimum: {}",
                        start - length,
                        -length
                    );
                    return Err(crate::vm::errors::simple_exception(
                        out_of_range_error,
                        &message,
                        position,
                    ));
                }
                // The replacement is read before the array is borrowed for
                // writing, since `a[0, 2] = a` names the same array on both
                // sides.
                // A span is filled with the elements of an Array, or with the
                // one value anything else stands for. An object that answers
                // `to_ary` contributes its elements too, while an instance of
                // an Array subclass is taken as the array it already is.
                let replacement = match &span {
                    None => Vec::new(),
                    Some(_) => match &value {
                        Object::Array(elements) => elements.borrow().clone(),
                        other => match crate::vm::native_methods::array_subclass_value(other) {
                            Some(Object::Array(elements)) => elements.borrow().clone(),
                            _ if self.responds_to(other, "to_ary") => {
                                match self.send_to_object(
                                    other.clone(),
                                    "to_ary",
                                    vec![],
                                    position,
                                )? {
                                    Object::Array(elements) => elements.borrow().clone(),
                                    converted => vec![converted],
                                }
                            }
                            _ => vec![other.clone()],
                        },
                    },
                };
                let mut array = array_rc.borrow_mut();
                while (array.len() as i64) < start {
                    array.push(Object::Nil);
                }
                match span {
                    None => {
                        let at = start as usize;
                        if at < array.len() {
                            array[at] = value.clone();
                        } else {
                            array.push(value.clone());
                        }
                    }
                    Some(span) => {
                        let at = start as usize;
                        let taken = (span as usize).min(array.len().saturating_sub(at));
                        array.splice(at..at + taken, replacement);
                    }
                }
                drop(array);
                Ok(Some(value))
            }
            // `at` is `[]` with a single index, and nothing else.
            "at" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let index: i64 = match &arguments[0] {
                    Object::Int(index) => *index,
                    Object::Float(index) => *index as i64,
                    other => self
                        .coerce_integer_argument(other, position)?
                        .try_into()
                        .unwrap_or(i64::MAX),
                };
                let array = array_rc.borrow();
                let length = array.len() as i64;
                let resolved = if index < 0 { index + length } else { index };
                if resolved < 0 || resolved >= length {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(array[resolved as usize].clone()))
            }
            _ => Ok(None),
        }
    }
}
