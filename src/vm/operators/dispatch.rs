// Which operator a binary expression spells, and the operand types each
// one reads.

use super::*;

impl VirtualMachine {
    /// Evaluate a binary operation across runtime values.
    pub(crate) fn evaluate_binary_operation(
        &mut self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;

        // A Complex carries its own arithmetic too, and a real number on the
        // left of one is promoted so that arithmetic runs.
        if let Some(name) = crate::vm::eval::binary_op_method_name(op) {
            if crate::vm::native_methods::complex_parts(&left).is_some()
                && let Some(result) =
                    self.call_complex_method(&left, name, std::slice::from_ref(&right), position)?
            {
                return Ok(result);
            }
            if crate::vm::native_methods::complex_parts(&right).is_some()
                && is_number(&left)
                && let Some(result) = {
                    let promoted = self.make_complex(left.clone(), Object::Int(0), position)?;
                    self.call_complex_method(
                        &promoted,
                        name,
                        std::slice::from_ref(&right),
                        position,
                    )?
                }
            {
                return Ok(result);
            }
        }

        // A Rational carries its own arithmetic, which runs whichever side it
        // sits on.
        if let Some(name) = crate::vm::eval::binary_op_method_name(op)
            && crate::vm::native_methods::rational_parts(&left).is_some()
            && let Some(result) =
                self.call_rational_method(&left, name, std::slice::from_ref(&right), position)?
        {
            return Ok(result);
        }
        // `2.0 ** (1/3r)` raises to a fractional power, which no exact
        // Rational answer stands for, so the exponent is read as a Float and
        // the Float arithmetic runs.
        if matches!(op, BinaryOp::Power)
            && matches!(left, Object::Float(_))
            && crate::vm::native_methods::rational_parts(&right).is_some()
        {
            let exponent = self.send_to_object(right.clone(), "to_f", vec![], position)?;
            if let Object::Float(exponent) = exponent {
                return self.evaluate_numeric_binary(op, left, Object::Float(exponent), position);
            }
        }
        // A Rational on the right promotes the number on the left, so the
        // Rational's own exact arithmetic runs rather than a Float one.
        // An integer raised to a Rational answers exactly where the exponent
        // names a whole number, and reads the exponent as a Float where it
        // does not.
        if matches!(op, BinaryOp::Power)
            && matches!(left, Object::Int(_) | Object::BigInt(_))
            && let Some((numerator, denominator)) =
                crate::vm::native_methods::rational_parts(&right)
        {
            if denominator != num_bigint::BigInt::from(1) {
                let exponent = self.send_to_object(right.clone(), "to_f", vec![], position)?;
                if let Object::Float(exponent) = exponent {
                    return self.evaluate_numeric_binary(
                        op,
                        left,
                        Object::Float(exponent),
                        position,
                    );
                }
            }
            // A negative exponent answers the reciprocal, which a Rational
            // holds exactly.
            let negative = numerator < num_bigint::BigInt::from(0);
            let raised = self.evaluate_numeric_binary(
                op,
                left,
                Object::integer(if negative { -numerator } else { numerator }),
                position,
            )?;
            let Some(whole) = raised.as_big_integer() else {
                return Ok(raised);
            };
            return if negative {
                self.make_rational(num_bigint::BigInt::from(1), whole, position)
            } else {
                self.make_rational(whole, num_bigint::BigInt::from(1), position)
            };
        }
        if let (Some(name), Object::Int(_) | Object::BigInt(_) | Object::Float(_)) =
            (crate::vm::eval::binary_op_method_name(op), &left)
            && crate::vm::native_methods::rational_parts(&right).is_some()
        {
            let promoted = self.promote_to_rational(&left, position)?;
            if let Some(result) =
                self.call_rational_method(&promoted, name, std::slice::from_ref(&right), position)?
            {
                return Ok(result);
            }
        }

        // Ruby hands an operand it does not know to the other side's `coerce`,
        // which answers the pair to apply the operator to instead.
        if let Some(coerced) = self.coerce_binary_operand(op, &left, &right, position)? {
            return Ok(coerced);
        }

        // A Set answers the algebra operators from its own method table, so
        // `a - b`, `a | b`, and the comparisons read the same as the named
        // methods do.
        if matches!(left, Object::Set(_))
            && let Some(name) = set_operator_name(op)
        {
            let class = self.builtins().class_of(&left);
            if let Some(result) = self.call_native_method(
                &class,
                &left,
                name,
                std::slice::from_ref(&right),
                position,
            )? {
                return Ok(result);
            }
        }

        match op {
            // `"a" + other` puts the right operand through `to_str`, which
            // String answers from its own method table.
            Add if matches!(left, Object::String(_)) && !matches!(right, Object::String(_)) => {
                match self.call_string_method(&left, "+", std::slice::from_ref(&right), position)? {
                    Some(answered) => Ok(answered),
                    None => Err(binary_type_error(BinaryOp::Add, &left, &right, position)),
                }
            }
            // `[1] + other` puts the right operand through `to_ary`, so an
            // object standing for an array concatenates the way one does.
            Add if matches!(left, Object::Array(_)) && !matches!(right, Object::Array(_)) => {
                let Some(elements) = self.as_array_operand(&right, position)? else {
                    return Err(binary_type_error(BinaryOp::Add, &left, &right, position));
                };
                self.evaluate_addition(left, Object::Array(elements), position)
            }
            Add if crate::vm::native_methods::array_subclass_value(&left).is_some() => {
                let elements = self
                    .as_array_operand(&left, position)?
                    .expect("an Array subclass instance stands for an array");
                // Recurse so the right operand goes through the same
                // conversion, which is how two subclass instances add.
                self.evaluate_binary_operation(
                    &BinaryOp::Add,
                    Object::Array(elements),
                    right,
                    position,
                )
            }
            // `-`, `&`, and `|` between arrays put the right operand through
            // `to_ary` too, the same way `+` does.
            Subtract | BitwiseAnd | BitwiseOr
                if matches!(left, Object::Array(_)) && !matches!(right, Object::Array(_)) =>
            {
                let Some(elements) = self.as_array_operand(&right, position)? else {
                    return Err(binary_type_error(op.clone(), &left, &right, position));
                };
                self.evaluate_binary_operation(op, left, Object::Array(elements), position)
            }
            Subtract | BitwiseAnd | BitwiseOr | Multiply
                if crate::vm::native_methods::array_subclass_value(&left).is_some() =>
            {
                let elements = self
                    .as_array_operand(&left, position)?
                    .expect("an Array subclass instance stands for an array");
                self.evaluate_binary_operation(op, Object::Array(elements), right, position)
            }
            // Array difference keeps what the right operand does not hold,
            // matching on `eql?` the way Ruby does.
            Subtract if matches!((&left, &right), (Object::Array(_), Object::Array(_))) => {
                let (Object::Array(left_items), Object::Array(right_items)) = (&left, &right)
                else {
                    unreachable!("both operands are arrays")
                };
                let left_items = left_items.borrow().clone();
                let right_items = right_items.borrow().clone();
                let mut remaining = Vec::new();
                for item in &left_items {
                    if !self.contains_eql(&right_items, item, position)? {
                        remaining.push(item.clone());
                    }
                }
                Ok(Object::array(remaining))
            }
            // An operator the program redefined on Integer or Float answers
            // for it, whatever the numbers underneath would have done.
            Add | Subtract | Multiply | Divide | Modulo | Power
                if matches!(left, Object::Int(_) | Object::BigInt(_) | Object::Float(_))
                    && let Some(named) = comparison_free_operator_name(op)
                    && let Some((owner, method)) = self.redefined_number_operator(&left, named) =>
            {
                self.invoke_method(owner, method, left, vec![right], position)
            }
            Add => self.evaluate_addition(left, right, position),
            Modulo if matches!(left, Object::String(_)) => {
                self.evaluate_string_format(left, right, position)
            }
            // An instance of a String subclass writes a format the same way,
            // answering a plain String the way Ruby's does.
            Modulo
                if matches!(
                    crate::vm::native_methods::string_subclass_value(&left),
                    Some(Object::String(_))
                ) =>
            {
                let held = crate::vm::native_methods::string_subclass_value(&left)
                    .expect("a string behind the subclass");
                self.evaluate_string_format(held, right, position)
            }
            // `(1..10) % 2` is Range#%, which answers the same sequence
            // `step` does.
            Modulo if matches!(left, Object::Range { .. }) => {
                self.send_to_object(left, "%", vec![right], position)
            }
            Subtract | Multiply | Divide | Modulo | Power => {
                self.evaluate_numeric_binary(op, left, right, position)
            }
            Equal => self.evaluate_equality(left, right, position),
            CaseEqual => self.evaluate_case_equality(left, right, position),
            // Ruby defines `!=` as the negation of `==`, so a class that
            // defines only `==` gets both.
            NotEqual => {
                let equal = self.evaluate_binary_operation(&Equal, left, right, position)?;
                Ok(Object::Bool(!crate::vm::utils::is_truthy(&equal)))
            }
            // Two hashes compare by containment rather than by order, which
            // Hash answers from its own method table.
            Less | Greater | LessEqual | GreaterEqual if matches!(left, Object::Dict(_)) => {
                let name = match op {
                    Less => "<",
                    Greater => ">",
                    LessEqual => "<=",
                    _ => ">=",
                };
                let class = self.builtins().class_of(&left);
                match self.call_native_method(
                    &class,
                    &left,
                    name,
                    std::slice::from_ref(&right),
                    position,
                )? {
                    Some(answer) => Ok(answer),
                    None => self.evaluate_comparison(op, left, right, position),
                }
            }
            Less | Greater | LessEqual | GreaterEqual => {
                self.evaluate_comparison(op, left, right, position)
            }
            Spaceship => self.evaluate_ordering(left, right, position),
            BitwiseAnd => self.evaluate_bitwise_and(left, right, position),
            BitwiseOr => self.evaluate_bitwise_or(left, right, position),
            Xor => self.evaluate_exclusive_or(left, right, position),
            And | Or => Err(MetorexError::internal_error(format!(
                "Logical operation '{:?}' should be handled by short-circuit evaluation",
                op
            ))),
            Assign | AddAssign | SubtractAssign | MultiplyAssign | DivideAssign => {
                Err(MetorexError::internal_error(format!(
                    "Assignment operation '{:?}' should be handled by statement execution",
                    op
                )))
            }
        }
    }
}
