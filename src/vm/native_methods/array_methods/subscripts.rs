// The run of elements a subscript or an arithmetic sequence names.

use super::*;

impl VirtualMachine {
    /// The run of elements a `slice!` subscript names: where it starts, how
    /// wide it is, and whether it named one element rather than a run. None
    /// when the subscript reaches past the array, which answers nil.
    pub(crate) fn slice_span(
        &mut self,
        arguments: &[Object],
        size: usize,
        position: Position,
    ) -> Result<Option<(usize, usize, bool)>, MetorexError> {
        let total = size as i64;
        match arguments.len() {
            1 => {
                let Some(Object::Range {
                    start,
                    end,
                    exclusive,
                    ..
                }) = crate::vm::native_methods::as_range(&arguments[0])
                else {
                    let index = self.coerce_integer_argument(&arguments[0], position)?;
                    let at = i64::try_from(&index).unwrap_or(i64::MAX);
                    let at = if at < 0 { at + total } else { at };
                    if at < 0 || at >= total {
                        return Ok(None);
                    }
                    return Ok(Some((at as usize, 1, true)));
                };
                let opening = match start.as_ref() {
                    Object::Nil => 0,
                    held => {
                        let asked = self.coerce_integer_argument(held, position)?;
                        let asked = i64::try_from(&asked).unwrap_or(i64::MAX);
                        if asked < 0 { asked + total } else { asked }
                    }
                };
                if opening < 0 || opening > total {
                    return Ok(None);
                }
                let closing = match end.as_ref() {
                    Object::Nil => total - 1,
                    held => {
                        let asked = self.coerce_integer_argument(held, position)?;
                        let asked = i64::try_from(&asked).unwrap_or(i64::MAX);
                        let placed = if asked < 0 { asked + total } else { asked };
                        if exclusive { placed - 1 } else { placed }
                    }
                };
                let width = (closing - opening + 1).clamp(0, total - opening);
                Ok(Some((opening as usize, width as usize, false)))
            }
            2 => {
                let asked = self.coerce_integer_argument(&arguments[0], position)?;
                let wanted = self.coerce_integer_argument(&arguments[1], position)?;
                let at = i64::try_from(&asked).unwrap_or(i64::MAX);
                let wanted = i64::try_from(&wanted).unwrap_or(i64::MAX);
                if wanted < 0 {
                    return Ok(None);
                }
                let at = if at < 0 { at + total } else { at };
                if at < 0 || at > total {
                    return Ok(None);
                }
                Ok(Some((at as usize, wanted.min(total - at) as usize, false)))
            }
            given => Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                given,
                position,
            )),
        }
    }
}

impl VirtualMachine {
    /// The run of elements an arithmetic sequence names, or None when the
    /// subscript is not one.
    pub(crate) fn sequence_slice(
        &mut self,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        value: &Object,
        total: i64,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if !self.is_arithmetic_sequence(value) {
            return Ok(None);
        }
        let step = match self.send_to_object(value.clone(), "step", vec![], position)? {
            Object::Int(held) => held,
            other => self.machine_index(&other, 0, position)?,
        };
        if step == 0 {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "step can't be 0",
                position,
            ));
        }
        let opening = self.send_to_object(value.clone(), "begin", vec![], position)?;
        let closing = self.send_to_object(value.clone(), "end", vec![], position)?;
        let excluded = matches!(
            self.send_to_object(value.clone(), "exclude_end?", vec![], position)?,
            Object::Bool(true)
        );
        // A sequence stepping by one reads exactly the way the range it was
        // built from reads, down to answering nil for a start past the end.
        if step == 1 {
            let span = Object::Range {
                start: Box::new(opening),
                end: Box::new(closing),
                exclusive: excluded,
                mark: std::rc::Rc::new(()),
            };
            let (start, width) = self.range_bounds(&span, total, position)?;
            if start < 0 || start > total {
                return Ok(Some(Object::Nil));
            }
            let held = array_rc.borrow();
            let end = start.saturating_add(width).min(total);
            return Ok(Some(Object::array(
                held[start as usize..end.max(start) as usize].to_vec(),
            )));
        }
        let resolved = |vm: &mut Self, held: &Object| -> Result<Option<i64>, MetorexError> {
            match held {
                Object::Nil => Ok(None),
                other => {
                    let counted = vm.machine_index(other, 0, position)?;
                    Ok(Some(if counted < 0 {
                        counted + total
                    } else {
                        counted
                    }))
                }
            }
        };
        let (low, high) = if step > 0 {
            let low = resolved(self, &opening)?.unwrap_or(0);
            let high = match resolved(self, &closing)? {
                Some(held) => held - i64::from(excluded),
                None => total - 1,
            };
            (low, high)
        } else {
            // A sequence running downward reads the same span the other way
            // about, so the bounds trade places.
            let closing_held = resolved(self, &closing)?.map(|held| held + i64::from(excluded));
            let low = closing_held.unwrap_or(0);
            let high = match resolved(self, &opening)? {
                Some(held) if closing_held.is_none() => held.min(total - 1),
                Some(held) => held,
                None => total - 1,
            };
            (low, high)
        };
        if low < 0 || low > total || high - low >= total {
            let shown = self.send_to_object(value.clone(), "inspect", vec![], position)?;
            let message = format!("{} out of range", shown);
            return Err(crate::vm::errors::simple_exception(
                "RangeError",
                &message,
                position,
            ));
        }
        let top = high.min(total - 1);
        let mut taken = Vec::new();
        if top >= low {
            let held = array_rc.borrow();
            if step > 0 {
                let mut at = low;
                while at <= top {
                    taken.push(held[at as usize].clone());
                    at += step;
                }
            } else {
                let mut at = top;
                while at >= low {
                    taken.push(held[at as usize].clone());
                    at += step;
                }
            }
        }
        Ok(Some(Object::array(taken)))
    }

    /// Whether a value is an arithmetic sequence, which names a run of the
    /// array with a gap between the elements it takes.
    fn is_arithmetic_sequence(&self, value: &Object) -> bool {
        let Object::Instance(instance) = value else {
            return false;
        };
        let mut cursor = Some(Rc::clone(&instance.borrow().class));
        while let Some(class) = cursor {
            if class.name() == "Enumerator::ArithmeticSequence" {
                return true;
            }
            cursor = class.superclass();
        }
        false
    }
}
