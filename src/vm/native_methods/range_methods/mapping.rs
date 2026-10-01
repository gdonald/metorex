// The values a range builds, and the ones at either end.

use super::*;

impl VirtualMachine {
    /// The values a range builds, and the ones at either end.
    pub(crate) fn call_range_mapping_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Range {
            start,
            end,
            exclusive,
            ..
        } = receiver
        else {
            return Ok(None);
        };
        match method_name {
            "map" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let pending = self.pending_block.take();
                let block = match pending {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            "map requires a block",
                            position_to_location(position),
                        ));
                    }
                };

                match (start.as_ref(), end.as_ref()) {
                    (Object::Int(start_val), Object::Int(end_val)) => {
                        let end_inclusive = if *exclusive { *end_val - 1 } else { *end_val };

                        let mut results = Vec::new();
                        for i in *start_val..=end_inclusive {
                            let args = vec![Object::Int(i)];
                            let value = self.execute_block_body(&block, args)?;
                            results.push(value);
                        }
                        Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
                    }
                    // Any other range maps over the values it walks.
                    _ => {
                        let elements = self.range_elements(start, end, *exclusive, position)?;
                        let mut results = Vec::with_capacity(elements.len());
                        for element in elements {
                            results.push(self.execute_block_body(&block, vec![element])?);
                        }
                        Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
                    }
                }
            }
            // `first` and `last` answer the endpoint, or that many values from
            // the front or the back when given a count.
            "begin" => Ok(Some((**start).clone())),
            "end" => Ok(Some((**end).clone())),
            "first" | "last" => {
                if arguments.is_empty() {
                    // The open side of a range has no first or last value.
                    if method_name == "first" && matches!(start.as_ref(), Object::Nil) {
                        return Err(crate::vm::errors::simple_exception(
                            "RangeError",
                            "cannot get the first element of beginless range",
                            position,
                        ));
                    }
                    if method_name == "last" && matches!(end.as_ref(), Object::Nil) {
                        return Err(crate::vm::errors::simple_exception(
                            "RangeError",
                            "cannot get the last element of endless range",
                            position,
                        ));
                    }
                    return Ok(Some(if method_name == "first" {
                        (**start).clone()
                    } else {
                        (**end).clone()
                    }));
                }
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let wanted: i64 = self
                    .coerce_integer_argument(&arguments[0], position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                if wanted < 0 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative array size (or size too big)",
                        position,
                    ));
                }
                if method_name == "last" && matches!(end.as_ref(), Object::Nil) {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the last element of endless range",
                        position,
                    ));
                }
                // A range counting up from an Integer with no end has no list
                // to collect, so the wanted values are counted out directly.
                if method_name == "first"
                    && let Some(counted) = endless_int_start(start, end)
                {
                    return Ok(Some(Object::array(
                        (0..wanted)
                            .map(|step| Object::Int(counted + step))
                            .collect(),
                    )));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                let taken = (wanted as usize).min(elements.len());
                Ok(Some(Object::array(if method_name == "first" {
                    elements[..taken].to_vec()
                } else {
                    elements[elements.len() - taken..].to_vec()
                })))
            }
            _ => Ok(None),
        }
    }
}
