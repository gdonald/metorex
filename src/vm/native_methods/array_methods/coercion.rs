// The elements an argument stands for, and the positions a
// subscript names.

use super::*;

impl VirtualMachine {
    /// The elements of an argument that stands for an array: one as it is, and
    /// anything else through `to_ary`.
    /// One index that must fit a machine word, counted from the end when
    /// negative. A value too large for one is a RangeError, which is what
    /// Ruby raises before it looks at the array.
    pub(crate) fn machine_index(
        &mut self,
        value: &Object,
        length: i64,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let index = match value {
            Object::Int(index) => *index,
            // A Float past the width of a machine word names no index at all.
            Object::Float(index)
                if !(-9.223_372_036_854_776e18..9.223_372_036_854_776e18).contains(index) =>
            {
                let message = format!("float {} out of range of integer", index);
                return Err(crate::vm::errors::simple_exception(
                    "RangeError",
                    &message,
                    position,
                ));
            }
            Object::Float(index) => *index as i64,
            other => {
                let wide = self.coerce_integer_argument(other, position)?;
                let Ok(index) = i64::try_from(&wide) else {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "bignum too big to convert into `long'",
                        position,
                    ));
                };
                index
            }
        };
        Ok(if index < 0 { index + length } else { index })
    }

    /// The start and length a Range names over `length` elements, with a
    /// negative bound counted from the end.
    pub(crate) fn range_bounds(
        &mut self,
        range: &Object,
        length: i64,
        position: Position,
    ) -> Result<(i64, i64), MetorexError> {
        let Object::Range {
            start,
            end,
            exclusive,
            ..
        } = range
        else {
            return Ok((0, 0));
        };
        let first = match start.as_ref() {
            Object::Nil => 0,
            bound => self.machine_index(bound, length, position)?,
        };
        let last = match end.as_ref() {
            Object::Nil => length - 1,
            bound => {
                let resolved = self.machine_index(bound, length, position)?;
                if *exclusive { resolved - 1 } else { resolved }
            }
        };
        Ok((first, last.saturating_sub(first).saturating_add(1).max(0)))
    }

    /// One index into an array of `length` elements, counted from the end when
    /// negative and coerced through `to_int` when it is not already an
    /// Integer.
    pub(crate) fn index_from(
        &mut self,
        value: &Object,
        length: i64,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let index: i64 = match value {
            Object::Int(index) => *index,
            Object::Float(index) => *index as i64,
            other => self
                .coerce_integer_argument(other, position)?
                .try_into()
                .unwrap_or(i64::MAX),
        };
        Ok(if index < 0 { index + length } else { index })
    }

    /// The elements an argument to `zip` stands for: an Array, something that
    /// answers `to_ary`, or failing that anything that can be walked with
    /// `each`. An object with neither is refused the way Ruby refuses it.
    /// The values one `zip` argument lines up against, taking at most
    /// `wanted` of them so an endless walk still ends.
    pub(crate) fn zip_column(
        &mut self,
        value: &Object,
        wanted: usize,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let Object::Array(elements) = value {
            return Ok(elements.borrow().clone());
        }
        if self.answers_to(value, "to_ary", position)?
            && let Object::Array(elements) =
                self.send_to_object(value.clone(), "to_ary", vec![], position)?
        {
            return Ok(elements.borrow().clone());
        }
        if self.answers_to(value, "each", position)? {
            let walk = self.send_to_object(
                value.clone(),
                "to_enum",
                vec![Object::symbol("each".to_string())],
                position,
            )?;
            // `first` stops the walk as soon as it has enough, which is what
            // lets an endless walk be zipped against a finite array.
            let taken =
                self.send_to_object(walk, "first", vec![Object::Int(wanted as i64)], position)?;
            if let Object::Array(elements) = taken {
                return Ok(elements.borrow().clone());
            }
            return Ok(Vec::new());
        }
        let named = match value {
            Object::Bool(true) => "TrueClass".to_string(),
            Object::Bool(false) => "FalseClass".to_string(),
            Object::Nil => "NilClass".to_string(),
            held => self.builtins().class_of(held).ruby_name().to_string(),
        };
        let message = format!("wrong argument type {} (must respond to :each)", named);
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    pub(crate) fn coerce_to_array(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let Object::Array(elements) = value {
            return Ok(elements.borrow().clone());
        }
        // An instance of an Array subclass already is an Array, so its
        // elements are taken directly rather than through `to_ary`.
        if let Some(Object::Array(elements)) =
            crate::vm::native_methods::array_subclass_value(value)
        {
            return Ok(elements.borrow().clone());
        }
        let refuse = |vm: &mut Self| {
            let message = format!(
                "no implicit conversion of {} into Array",
                vm.builtins().class_of(value).name()
            );
            crate::vm::errors::simple_exception("TypeError", &message, position)
        };
        if !self.responds_to(value, "to_ary") {
            return Err(refuse(self));
        }
        match self.send_to_object(value.clone(), "to_ary", vec![], position)? {
            Object::Array(elements) => Ok(elements.borrow().clone()),
            _ => Err(refuse(self)),
        }
    }
}

/// One index into an array, counted from the end when negative, and None when
/// it names no element at all.
pub(crate) fn normalize_index(value: &Object, length: i64) -> Option<usize> {
    let index = match value {
        Object::Int(index) => *index,
        Object::Float(index) => *index as i64,
        _ => return None,
    };
    let index = if index < 0 { index + length } else { index };
    if index < 0 || index >= length {
        return None;
    }
    Some(index as usize)
}
