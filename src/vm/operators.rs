//! Operator evaluation functions for the Metorex VM.
//!
//! This module contains the logic for evaluating unary and binary operators including:
//! - Unary operations (+, -)
//! - Binary operations (+, -, *, /, %)
//! - Comparison operations (<, >, <=, >=, ==, !=)

use crate::ast::{BinaryOp, UnaryOp};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

use super::core::VirtualMachine;
use super::errors::{binary_type_error, divide_by_zero_error, unary_type_error};

thread_local! {
    /// Array pairs currently being ordered, so a pair that reaches itself
    /// answers 0 instead of recursing forever.
    static ORDERING: std::cell::RefCell<Vec<(usize, usize)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

impl VirtualMachine {
    /// The method a program of its own put on Integer or Float for an
    /// operator, or None where the numbers answer for themselves.
    fn redefined_number_operator(
        &mut self,
        held: &Object,
        named: &str,
    ) -> Option<(Rc<crate::class::Class>, Rc<crate::object::Method>)> {
        let class = self.builtins().class_of(held);
        let found = class.find_own_method(named)?;
        if found.is_undefined || found.body.is_empty() {
            return None;
        }
        Some((class, found))
    }

    /// The arithmetic a number answers for an operator on its own, reached
    /// without consulting a redefinition. An UnboundMethod cut from Integer
    /// before the program replaced `+` calls through here, so binding it back
    /// runs the original rather than the replacement.
    pub(crate) fn builtin_number_operator(
        &mut self,
        named: &str,
        left: Object,
        right: Object,
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        if !matches!(left, Object::Int(_) | Object::BigInt(_) | Object::Float(_)) {
            return None;
        }
        // Anything but a number on the right is asked to `coerce`, which is
        // the ordinary dispatch rather than the arithmetic underneath.
        if !matches!(right, Object::Int(_) | Object::BigInt(_) | Object::Float(_)) {
            return None;
        }
        match named {
            "+" => Some(self.evaluate_addition(left, right, position)),
            "-" => Some(self.evaluate_numeric_binary(&BinaryOp::Subtract, left, right, position)),
            "*" => Some(self.evaluate_numeric_binary(&BinaryOp::Multiply, left, right, position)),
            "/" => Some(self.evaluate_numeric_binary(&BinaryOp::Divide, left, right, position)),
            "%" => Some(self.evaluate_numeric_binary(&BinaryOp::Modulo, left, right, position)),
            "**" => Some(self.evaluate_numeric_binary(&BinaryOp::Power, left, right, position)),
            _ => None,
        }
    }

    /// Evaluate a unary operation (`+` or `-`).
    pub(crate) fn evaluate_unary_operation(
        &self,
        op: &UnaryOp,
        value: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match op {
            UnaryOp::Plus => match value {
                // Numeric `+x` is a no-op identity.
                Object::Int(_) | Object::BigInt(_) | Object::Float(_) => Ok(value),
                // `+str` asks for a string that changes, so a frozen one and
                // one carrying notice that it will be frozen both answer a
                // copy, and any other answers itself.
                Object::String(ref text) => {
                    if !text.is_frozen() && !text.is_chilled() {
                        return Ok(value.clone());
                    }
                    Ok(Object::String(std::rc::Rc::new(
                        crate::object::StringValue::with_encoding(
                            text.to_text(),
                            text.encoding_name(),
                        ),
                    )))
                }
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Minus => match value {
                // Negating i64::MIN needs the wider type.
                Object::Int(v) => Ok(Object::integer(-num_bigint::BigInt::from(v))),
                Object::BigInt(v) => Ok(Object::integer(-(*v).clone())),
                Object::Float(v) => Ok(Object::Float(-v)),
                // `-str` asks for a string that does not change, so one
                // already frozen answers itself and any other answers a
                // frozen copy.
                Object::String(ref text) => {
                    if text.is_frozen() {
                        text.mark_deduplicated();
                        return Ok(value.clone());
                    }
                    let copy = crate::object::StringValue::with_encoding(
                        text.to_text(),
                        text.encoding_name(),
                    );
                    if text.holds_bytes() {
                        copy.mark_bytes();
                    }
                    copy.mark_deduplicated();
                    Ok(Object::String(std::rc::Rc::new(copy)))
                }
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Not => Ok(Object::Bool(matches!(
                value,
                Object::Bool(false) | Object::Nil
            ))),
        }
    }

    /// Whether `elements` holds a value `eql?` to `wanted`, which is how Ruby
    /// matches for array difference, intersection, and union.
    pub(crate) fn contains_eql(
        &mut self,
        elements: &[Object],
        wanted: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        // The value being looked up is the one asked, so an object of the
        // program's own answers `eql?` for itself. The same object counts
        // whatever its `eql?` says, which is what a hash lookup amounts to.
        for element in elements {
            if same_object(element, wanted) || self.values_eql(wanted, element, position)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

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
            Equal => {
                // A number compared against something that is not one answers
                // by asking that object instead, which is how a class that
                // stands for a number reports equality with one.
                if is_number(&left) && takes_coercion(&right) {
                    let answer =
                        self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
                    return Ok(Object::Bool(answer.is_truthy()));
                }
                // Two patterns are the same when their source and flags are.
                if let (Object::Regex(pattern, flags), Object::Regex(other, other_flags)) =
                    (&left, &right)
                {
                    use crate::vm::native_methods::comparable_flags;
                    return Ok(Object::Bool(
                        pattern == other
                            && comparable_flags(flags) == comparable_flags(other_flags),
                    ));
                }
                // Two hashes compare entry by entry, looking each key up in
                // the other hash the way any key is looked up, which Hash
                // answers from its own method table.
                if matches!((&left, &right), (Object::Dict(_), Object::Dict(_)))
                    && let Some(answer) =
                        self.call_hash_method(&left, "==", std::slice::from_ref(&right), position)?
                {
                    return Ok(Object::Bool(answer.is_truthy()));
                }
                // Two ranges compare by their ends and their exclusivity,
                // which Range answers from its own method table.
                if matches!((&left, &right), (Object::Range { .. }, _)) {
                    let class = self.builtins().class_of(&left);
                    if let Some(answer) = self.call_native_method(
                        &class,
                        &left,
                        "==",
                        std::slice::from_ref(&right),
                        position,
                    )? {
                        return Ok(Object::Bool(answer.is_truthy()));
                    }
                }
                // For instances, dispatch to user-defined == method if present,
                // or to <=> (Comparable protocol) if the class has <=> defined.
                if let Object::Instance(inst_rc) = &left {
                    // Identity shortcut: same object is always ==
                    if let Object::Instance(rhs) = &right
                        && Rc::ptr_eq(inst_rc, rhs)
                    {
                        return Ok(Object::Bool(true));
                    }
                    if let Some((class, method)) = self.lookup_method(&left, "==")
                        && !method.is_undefined
                    {
                        let result = self.invoke_method(
                            class,
                            method,
                            left.clone(),
                            vec![right.clone()],
                            position,
                        )?;
                        return Ok(Object::Bool(result.is_truthy()));
                    }
                    // Rational and Complex answer `==` from their native
                    // tables rather than a method map, and a Complex with no
                    // imaginary part equals the plain number it holds.
                    let instance_class = Rc::clone(&inst_rc.borrow().class);
                    if matches!(instance_class.name(), "Rational" | "Complex")
                        && let Some(result) = self.call_native_method(
                            &instance_class,
                            &left,
                            "==",
                            std::slice::from_ref(&right),
                            position,
                        )?
                    {
                        return Ok(Object::Bool(result.is_truthy()));
                    }
                    // Comparable protocol: if <=> is defined, use it for ==
                    if let Some((cmp_class, cmp_method)) = self.lookup_method(&left, "<=>") {
                        let cmp_obj = self.invoke_method(
                            cmp_class,
                            cmp_method,
                            left.clone(),
                            vec![right.clone()],
                            position,
                        )?;
                        return match cmp_obj {
                            Object::Int(n) => Ok(Object::Bool(n == 0)),
                            Object::Float(f) => Ok(Object::Bool(f == 0.0)),
                            Object::Nil => Ok(Object::Bool(false)),
                            other => {
                                // ArgumentError: <=> must return Integer, Float, or nil
                                let msg = format!(
                                    "comparison of {} with {} failed",
                                    left.type_name(),
                                    other.type_name()
                                );
                                let exc = Object::exception("ArgumentError", msg.clone());
                                Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: msg,
                                })
                            }
                        };
                    }
                }
                // A Process::Status compares by the number it stands for, so
                // it answers `==` against that number as well as against
                // another status.
                if matches!(&left, Object::Instance(held) if held.borrow().class.name() == "Process::Status")
                    && let Some(answer) = self.call_process_status_method(
                        &left,
                        "==",
                        std::slice::from_ref(&right),
                        position,
                    )?
                {
                    return Ok(Object::Bool(answer.is_truthy()));
                }
                // A Set decides equality itself, since it counts anything
                // that says it is one, however that value is built.
                if matches!(left, Object::Set(_))
                    && let Some(answer) = self.call_native_method(
                        &self.builtins().class_of(&left),
                        &left,
                        "==",
                        std::slice::from_ref(&right),
                        position,
                    )?
                {
                    return Ok(Object::Bool(answer.is_truthy()));
                }
                // Two hashes holding objects of the program's own are equal
                // when those objects say so, so each value is asked with `==`
                // rather than compared as data.
                if let (Object::Dict(one), Object::Dict(other)) = (&left, &right) {
                    if Rc::ptr_eq(one, other) {
                        return Ok(Object::Bool(true));
                    }
                    let held: Vec<(String, Object)> = one
                        .borrow()
                        .iter()
                        .filter(|(key, _)| !key.starts_with("__MX_"))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    let against: Vec<(String, Object)> = other
                        .borrow()
                        .iter()
                        .filter(|(key, _)| !key.starts_with("__MX_"))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    if held.iter().chain(against.iter()).any(|(_, value)| {
                        matches!(value, Object::Instance(_))
                            || matches!(value, Object::Array(items) if items.borrow().iter().any(|item| matches!(item, Object::Instance(_))))
                    }) {
                        if held.len() != against.len() {
                            return Ok(Object::Bool(false));
                        }
                        for (key, value) in &held {
                            let Some((_, counterpart)) =
                                against.iter().find(|(other_key, _)| other_key == key)
                            else {
                                return Ok(Object::Bool(false));
                            };
                            let answer = self.evaluate_binary_operation(
                                &BinaryOp::Equal,
                                value.clone(),
                                counterpart.clone(),
                                position,
                            )?;
                            if !answer.is_truthy() {
                                return Ok(Object::Bool(false));
                            }
                        }
                        return Ok(Object::Bool(true));
                    }
                }
                // Two arrays are equal when their elements are, which asks
                // each pair rather than comparing the two as data.
                if matches!((&left, &right), (Object::Array(_), Object::Array(_))) {
                    let same = self.arrays_equal(&left, &right, position)?;
                    return Ok(Object::Bool(same));
                }
                // An object that spells itself as an array answers for the
                // pair, which is how a wrapper around one compares.
                if matches!(left, Object::Array(_))
                    && matches!(right, Object::Instance(_))
                    && crate::vm::native_methods::array_subclass_value(&right).is_none()
                    && self
                        .send_to_object(
                            right.clone(),
                            "respond_to?",
                            vec![Object::symbol("to_ary".to_string())],
                            position,
                        )?
                        .is_truthy()
                {
                    let answer =
                        self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
                    return Ok(Object::Bool(answer.is_truthy()));
                }
                // Two strings are equal when they spell the same thing,
                // whether that is read as bytes or as characters, and their
                // encodings can be compared at all.
                if let (Object::String(text), Object::String(other)) = (&left, &right) {
                    use crate::vm::native_methods::string_methods::{
                        binary_bytes, strings_comparable,
                    };
                    return Ok(Object::Bool(
                        (binary_bytes(text) == binary_bytes(other)
                            || *text.as_str() == *other.as_str())
                            && strings_comparable(text, other),
                    ));
                }
                // Anything else that spells itself as text answers for the
                // pair, which is how a wrapper around a String compares.
                if matches!(left, Object::String(_))
                    && crate::vm::native_methods::string_subclass_value(&right).is_none()
                    && self.responds_to(&right, "to_str")
                {
                    let answer =
                        self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
                    return Ok(Object::Bool(answer.is_truthy()));
                }
                Ok(Object::Bool(left.equals(&right)))
            }
            CaseEqual => {
                // A number's `===` is its `==`, so it too answers by asking an
                // operand that is not a number.
                if is_number(&left) && takes_coercion(&right) {
                    let answer =
                        self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
                    return Ok(Object::Bool(answer.is_truthy()));
                }
                // Regexp === str: whether the pattern matches anywhere. A
                // Symbol matches on its name, and an object that answers
                // `to_str` on the characters it hands over.
                if let Object::Regex(pattern, flags) = &left {
                    let subject = match &right {
                        Object::String(text) | Object::Symbol(text) => {
                            Some(text.as_str().to_string())
                        }
                        other => match crate::vm::native_methods::subject_text(other) {
                            Some(text) => Some(text),
                            None if self.responds_to(other, "to_str") => {
                                match self.send_to_object(
                                    other.clone(),
                                    "to_str",
                                    vec![],
                                    position,
                                )? {
                                    Object::String(text) => Some(text.as_str().to_string()),
                                    _ => None,
                                }
                            }
                            None => None,
                        },
                    };
                    let Some(subject) = subject else {
                        return Ok(Object::Bool(false));
                    };
                    let (pattern, flags) =
                        (pattern.as_str().to_string(), flags.as_str().to_string());
                    let found = self.regexp_match_data(&pattern, &flags, &subject, 0, position)?;
                    return Ok(Object::Bool(found.is_some()));
                }
                // A callable answers for itself: `===` hands the value to
                // the proc and reports whatever it returns, so a proc can
                // stand in for a pattern in a `case`.
                if let Object::Block(block) = &left {
                    return block.call(self, vec![right], position);
                }
                // A Set holds a value or it does not, which is the branch a
                // `case` over one picks.
                if matches!(left, Object::Set(_)) {
                    let class = self.builtins().class_of(&left);
                    if let Some(answer) = self.call_native_method(
                        &class,
                        &left,
                        "include?",
                        std::slice::from_ref(&right),
                        position,
                    )? {
                        return Ok(Object::Bool(answer.is_truthy()));
                    }
                }
                // A Range covers a value the way `cover?` reports, which is
                // what `case` uses to pick a branch.
                if matches!(left, Object::Range { .. }) {
                    let class = self.builtins().class_of(&left);
                    if let Some(answer) = self.call_native_method(
                        &class,
                        &left,
                        "cover?",
                        std::slice::from_ref(&right),
                        position,
                    )? {
                        return Ok(Object::Bool(answer.is_truthy()));
                    }
                }
                // Class/Module === obj: check type membership (Ruby's case
                // equality). Modules also count as the "type test" form so
                // `SomeMod === obj` works the same as `obj.is_a?(SomeMod)`.
                let class_rc_opt = match &left {
                    Object::Class(c) | Object::Module(c) => Some(c),
                    _ => None,
                };
                if let Some(class_rc) = class_rc_opt {
                    if let Object::Exception(exc_ref) = &right {
                        // Check if exception type matches or is a subclass
                        let exc_type = exc_ref.borrow().exception_type.clone();
                        if exc_type == class_rc.name() {
                            return Ok(Object::Bool(true));
                        }
                        // Check the exception class hierarchy. A namespaced
                        // type such as `Errno::EINVAL` is a constant on its
                        // module rather than a top-level name.
                        let carried = exc_ref.borrow().class.clone().map(Object::Class);
                        let resolved = match carried {
                            Some(class) => Some(class),
                            None => match self.globals().get(&exc_type) {
                                Some(class @ Object::Class(_)) => Some(class),
                                _ => self.resolve_qualified_constant(&exc_type),
                            },
                        };
                        if let Some(Object::Class(exc_class)) = resolved {
                            return Ok(Object::Bool(
                                self.builtins().is_subclass_of(&exc_class, class_rc),
                            ));
                        }
                        return Ok(Object::Bool(false));
                    }
                    // A class or a module is an object of Class or of
                    // Module, which is what makes `Module === String` true.
                    if matches!(right, Object::Class(_) | Object::Module(_)) {
                        let meta_name = match &right {
                            Object::Class(_) => "Class",
                            _ => "Module",
                        };
                        if let Some(Object::Class(meta)) = self.globals().get(meta_name)
                            && self.builtins().is_subclass_of(&meta, class_rc)
                        {
                            return Ok(Object::Bool(true));
                        }
                    }
                    // `Left === right` asks whether the right stands as one
                    // of the left, which is `right.is_a?(left)` whatever the
                    // right is. A class is an object too, so `Module ===
                    // String` is true and `String === String` is false.
                    {
                        if crate::builtin_classes::value_class_name(&right)
                            .is_some_and(|name| name == class_rc.name())
                        {
                            return Ok(Object::Bool(true));
                        }
                        let right_class = self.builtins().class_of(&right);
                        if self.builtins().is_subclass_of(&right_class, class_rc) {
                            return Ok(Object::Bool(true));
                        }
                        if let Object::Instance(inst_rc) = &right {
                            let sc_opt = inst_rc.borrow().singleton_class.borrow().clone();
                            if let Some(sc) = sc_opt
                                && self.builtins().is_subclass_of(&sc, class_rc)
                            {
                                return Ok(Object::Bool(true));
                            }
                        }
                        return Ok(Object::Bool(false));
                    }
                }
                // Object#=== is the same object, or whatever #== says. It
                // consults neither #equal? nor #object_id, so a class that
                // overrides those does not change the answer. A class that
                // defines its own `===`, or reaches one through
                // `method_missing`, wins over that default.
                if let Object::Instance(inst_rc) = &left {
                    if let Some((owner, method)) = self.lookup_method(&left, "===")
                        && !method.is_undefined
                    {
                        return self.invoke_method(owner, method, left, vec![right], position);
                    }
                    if let Some((owner, handler)) = self.lookup_method(&left, "method_missing")
                        && !handler.is_undefined
                    {
                        let arguments = vec![Object::symbol("===".to_string()), right];
                        return self.invoke_method(owner, handler, left, arguments, position);
                    }
                    if let Object::Instance(rhs) = &right
                        && Rc::ptr_eq(inst_rc, rhs)
                    {
                        return Ok(Object::Bool(true));
                    }
                    return self.evaluate_binary_operation(&Equal, left, right, position);
                }
                // A String's `===` is its `==`, down to the encodings it
                // refuses to compare.
                if matches!(left, Object::String(_)) {
                    return self.evaluate_binary_operation(&Equal, left, right, position);
                }
                Ok(Object::Bool(left.equals(&right)))
            }
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
            Spaceship => {
                // Object#<=> answers 0 for the same object or when #== says
                // so, and nil otherwise. It never consults #eql?. A class that
                // defines its own #<=> is dispatched before reaching here.
                if let Object::Instance(inst_rc) = &left {
                    // A class of the program's own answers for itself, which
                    // is what a walk over elements reaches here.
                    if let Some((class, method)) = self.lookup_method(&left, "<=>")
                        && !method.is_undefined
                        && !method.body.is_empty()
                    {
                        return self.invoke_method(
                            class,
                            method,
                            left.clone(),
                            vec![right],
                            position,
                        );
                    }
                    if let Object::Instance(rhs) = &right
                        && Rc::ptr_eq(inst_rc, rhs)
                    {
                        return Ok(Object::Int(0));
                    }
                    let equal =
                        self.evaluate_binary_operation(&Equal, left, right.clone(), position)?;
                    return Ok(if equal.is_truthy() {
                        Object::Int(0)
                    } else {
                        Object::Nil
                    });
                }
                // Two arrays order element by element, and each element is
                // sent `<=>` so one of the program's own answers for itself.
                if let Object::Array(left_elements) = &left {
                    let left_elements = Rc::clone(left_elements);
                    if let Some(right_elements) = self.as_array_operand(&right, position)? {
                        return self.order_arrays(&left_elements, &right_elements, position);
                    }
                    return Ok(Object::Nil);
                }
                // An endless Float orders against anything that reports
                // which end it stands at, and against a finite value it is
                // simply greater.
                if let Object::Float(held) = &left
                    && held.is_infinite()
                    && matches!(right, Object::Instance(_))
                    && self.responds_to(&right, "infinite?")
                {
                    let reported =
                        self.send_to_object(right.clone(), "infinite?", Vec::new(), position)?;
                    let theirs = match reported {
                        Object::Int(held) => held,
                        _ => 0,
                    };
                    let mine = if *held > 0.0 { 1 } else { -1 };
                    return Ok(Object::Int(mine.cmp(&theirs) as i64));
                }
                // A String orders against anything that reads as one, then
                // against anything that orders itself, and has no order at
                // all with anything else.
                if matches!(&left, Object::String(_))
                    && let Some(order) = self.order_string_against(&left, &right, position)?
                {
                    return Ok(order);
                }
                let ordered = self.evaluate_spaceship(left.clone(), right.clone(), position)?;
                if !matches!(ordered, Object::Nil) {
                    return Ok(ordered);
                }
                // Object#<=> answers 0 for two values `==` calls the same and
                // nil for everything else, which is what two of a kind with no
                // order of their own fall back on. A value that orders itself
                // has already answered, and asking it again would run an `==`
                // written for an operand of another kind.
                if is_number(&left)
                    || matches!(
                        left,
                        Object::String(_) | Object::Symbol(_) | Object::Array(_)
                    )
                {
                    return Ok(Object::Nil);
                }
                let equal = self.evaluate_binary_operation(&Equal, left, right, position)?;
                Ok(if equal.is_truthy() {
                    Object::Int(0)
                } else {
                    Object::Nil
                })
            }
            BitwiseAnd => match (left, right) {
                // Array intersection, keeping the left operand's order and
                // dropping duplicates.
                (Object::Array(left_items), Object::Array(right_items)) => {
                    // Ruby matches on `hash` and `eql?` here, so an object of
                    // the program's own decides for itself.
                    let right_items = right_items.borrow().clone();
                    let left_items = left_items.borrow().clone();
                    let mut intersection: Vec<Object> = Vec::new();
                    for item in &left_items {
                        if self.contains_eql(&right_items, item, position)?
                            && !self.contains_eql(&intersection, item, position)?
                        {
                            intersection.push(item.clone());
                        }
                    }
                    Ok(Object::array(intersection))
                }
                // nil & x always returns false (Ruby semantics)
                (Object::Nil, _) | (_, Object::Nil) => Ok(Object::Bool(false)),
                (Object::Bool(a), Object::Bool(b)) => Ok(Object::Bool(a & b)),
                (
                    ref left @ (Object::Int(_) | Object::BigInt(_)),
                    ref right @ (Object::Int(_) | Object::BigInt(_)),
                ) => Ok(Object::integer(
                    left.as_big_integer().expect("integer-kinded")
                        & right.as_big_integer().expect("integer-kinded"),
                )),
                (Object::Bool(a), other) => Ok(Object::Bool(a & other.is_truthy())),
                (other, Object::Bool(b)) => Ok(Object::Bool(other.is_truthy() & b)),
                (lhs, rhs) => Err(binary_type_error(BitwiseAnd, &lhs, &rhs, position)),
            },
            BitwiseOr => match (left, right) {
                // Array union, keeping first-seen order and dropping
                // duplicates.
                (Object::Array(left_items), Object::Array(right_items)) => {
                    let mut all = left_items.borrow().clone();
                    all.extend(right_items.borrow().iter().cloned());
                    let mut union: Vec<Object> = Vec::new();
                    for item in &all {
                        if !self.contains_eql(&union, item, position)? {
                            union.push(item.clone());
                        }
                    }
                    Ok(Object::array(union))
                }
                // nil | x returns truthiness of x
                (Object::Nil, other) => Ok(Object::Bool(other.is_truthy())),
                (Object::Bool(a), Object::Bool(b)) => Ok(Object::Bool(a | b)),
                (
                    ref left @ (Object::Int(_) | Object::BigInt(_)),
                    ref right @ (Object::Int(_) | Object::BigInt(_)),
                ) => Ok(Object::integer(
                    left.as_big_integer().expect("integer-kinded")
                        | right.as_big_integer().expect("integer-kinded"),
                )),
                (Object::Bool(a), other) => Ok(Object::Bool(a | other.is_truthy())),
                (other, Object::Bool(b)) => Ok(Object::Bool(other.is_truthy() | b)),
                (lhs, rhs) => Err(binary_type_error(BitwiseOr, &lhs, &rhs, position)),
            },
            Xor => match (left, right) {
                // nil ^ x returns truthiness of x
                (Object::Nil, other) => Ok(Object::Bool(other.is_truthy())),
                (Object::Bool(a), Object::Bool(b)) => Ok(Object::Bool(a ^ b)),
                (
                    ref left @ (Object::Int(_) | Object::BigInt(_)),
                    ref right @ (Object::Int(_) | Object::BigInt(_)),
                ) => Ok(Object::integer(
                    left.as_big_integer().expect("integer-kinded")
                        ^ right.as_big_integer().expect("integer-kinded"),
                )),
                (Object::Bool(a), other) => Ok(Object::Bool(a ^ other.is_truthy())),
                (other, Object::Bool(b)) => Ok(Object::Bool(other.is_truthy() ^ b)),
                (lhs, rhs) => Err(binary_type_error(Xor, &lhs, &rhs, position)),
            },
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

    /// Handle addition across supported operand types.
    pub(crate) fn evaluate_addition(
        &self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match (left, right) {
            (Object::Int(a), Object::Int(b)) => match a.checked_add(b) {
                Some(v) => Ok(Object::Int(v)),
                // Too large for an i64, so carry on in arbitrary precision.
                None => Ok(Object::integer(
                    num_bigint::BigInt::from(a) + num_bigint::BigInt::from(b),
                )),
            },
            (
                ref left @ (Object::Int(_) | Object::BigInt(_)),
                ref right @ (Object::Int(_) | Object::BigInt(_)),
            ) => {
                let (a, b) = (
                    left.as_big_integer().expect("integer-kinded"),
                    right.as_big_integer().expect("integer-kinded"),
                );
                Ok(Object::integer(a + b))
            }
            (Object::BigInt(a), Object::Float(b)) => Ok(Object::Float(big_to_float(&a) + b)),
            (Object::Float(a), Object::BigInt(b)) => Ok(Object::Float(a + big_to_float(&b))),
            (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a + b)),
            (Object::Int(a), Object::Float(b)) => Ok(Object::Float((a as f64) + b)),
            (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a + (b as f64))),
            (Object::String(a), Object::String(b)) => {
                // Where either side stands for the bytes it was read from,
                // the answer does too, and the other side is written out in
                // the bytes its own encoding spells it with.
                let holds_bytes = a.holds_bytes() || b.holds_bytes();
                let joined = if holds_bytes {
                    let mut bytes =
                        crate::vm::native_methods::string_methods::binary_bytes(a.as_ref());
                    bytes.extend(crate::vm::native_methods::string_methods::binary_bytes(
                        b.as_ref(),
                    ));
                    let made = crate::object::StringValue::from_bytes(
                        crate::vm::native_methods::pack_format::bytes_to_string(&bytes).to_string(),
                    );
                    Object::String(Rc::new(made))
                } else {
                    let mut combined = a.as_str().to_string();
                    combined.push_str(&b.as_ref().as_str());
                    Object::string(combined)
                };
                // A run recording where each object was made records this
                // one at the place the two were joined.
                if let Object::String(made) = &joined
                    && let Some(written_at) = self.literal_birthplace(position)
                {
                    made.set_created_at(written_at);
                }
                // The result is written in the receiver's encoding, unless
                // the receiver is empty or nothing but ASCII and the other
                // side is not, where that side's reading carries over.
                if let Object::String(made) = &joined {
                    let plain = |side: &crate::object::StringValue| {
                        side.as_str().is_empty()
                            || (side.as_str().is_ascii() && !side.holds_bytes())
                    };
                    let carried = if plain(a.as_ref()) && !plain(b.as_ref()) {
                        b.as_ref()
                    } else {
                        a.as_ref()
                    };
                    made.set_encoding(carried.encoding_name());
                    if holds_bytes {
                        made.mark_bytes();
                    }
                }
                Ok(joined)
            }
            (Object::Array(a), Object::Array(b)) => {
                let mut combined = a.borrow().clone();
                combined.extend(b.borrow().iter().cloned());
                Ok(Object::Array(Rc::new(std::cell::RefCell::new(combined))))
            }
            (lhs, rhs) => Err(binary_type_error(BinaryOp::Add, &lhs, &rhs, position)),
        }
    }

    /// Evaluate numeric binary operations (`-`, `*`, `/`, `%`).
    pub(crate) fn evaluate_numeric_binary(
        &mut self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // An instance of a String subclass repeats the characters it holds,
        // answering a plain String the way Ruby's does.
        if matches!(op, BinaryOp::Multiply)
            && matches!(left, Object::Instance(_))
            && let Some(text) = crate::vm::native_methods::string_subclass_value(&left)
        {
            return self.evaluate_numeric_binary(op, text, right, position);
        }
        match (left, right) {
            // Array difference: elements of the left array not present in
            // the right one, preserving left order.
            (Object::Array(a), Object::Array(b)) if matches!(op, BinaryOp::Subtract) => {
                let b_items = b.borrow();
                let remaining: Vec<Object> = a
                    .borrow()
                    .iter()
                    .filter(|item| !b_items.iter().any(|other| item.equals(other)))
                    .cloned()
                    .collect();
                Ok(Object::Array(Rc::new(std::cell::RefCell::new(remaining))))
            }
            (
                ref left @ (Object::Int(_) | Object::BigInt(_)),
                ref right @ (Object::Int(_) | Object::BigInt(_)),
            ) => {
                let a = left.as_big_integer().expect("integer-kinded");
                let b = right.as_big_integer().expect("integer-kinded");
                integer_arithmetic(op, a, b, position)
            }
            // An Integer receiver refuses a zero divisor whether or not the
            // divisor is a Float, so the bignum path says so.
            (Object::BigInt(a), Object::Float(b)) if matches!(op, BinaryOp::Modulo) => {
                float_modulo(big_to_float(&a), b, position)
            }
            // A negative base raised to a power that is not a whole number
            // has no real root, so the answer is the principal complex one.
            (Object::BigInt(a), Object::Float(b))
                if matches!(op, BinaryOp::Power)
                    && a.sign() == num_bigint::Sign::Minus
                    && b.fract() != 0.0
                    && b.is_finite() =>
            {
                let magnitude = (-big_to_float(&a)).powf(b);
                let angle = std::f64::consts::PI * b;
                self.make_complex(
                    Object::Float(magnitude * angle.cos()),
                    Object::Float(magnitude * angle.sin()),
                    position,
                )
            }
            (Object::BigInt(a), Object::Float(b)) => {
                float_arithmetic(op, big_to_float(&a), b, position)
            }
            (Object::Float(a), Object::BigInt(b)) => {
                float_arithmetic(op, a, big_to_float(&b), position)
            }
            (Object::Float(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - b)),
                BinaryOp::Multiply => Ok(Object::Float(a * b)),
                // Float division by zero follows IEEE 754 and answers an
                // infinity or NaN. Only Integer / Integer raises.
                BinaryOp::Divide => Ok(Object::Float(a / b)),
                BinaryOp::Modulo => float_modulo(a, b, position),
                // A negative base raised to a power that is not a whole
                // number has no real root, so the answer is the principal
                // complex one.
                BinaryOp::Power if a < 0.0 && b.fract() != 0.0 && b.is_finite() => {
                    let magnitude = (-a).powf(b);
                    let angle = std::f64::consts::PI * b;
                    self.make_complex(
                        Object::Float(magnitude * angle.cos()),
                        Object::Float(magnitude * angle.sin()),
                        position,
                    )
                }
                BinaryOp::Power => Ok(Object::Float(a.powf(b))),
                _ => unreachable!(),
            },
            (Object::Int(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float((a as f64) - b)),
                BinaryOp::Multiply => Ok(Object::Float((a as f64) * b)),
                BinaryOp::Divide => Ok(Object::Float((a as f64) / b)),
                BinaryOp::Modulo => float_modulo(a as f64, b, position),
                // A negative base raised to a power that is not a whole
                // number has no real root, so the answer is the principal
                // complex one.
                BinaryOp::Power if a < 0 && b.fract() != 0.0 && b.is_finite() => {
                    let magnitude = (-(a as f64)).powf(b);
                    let angle = std::f64::consts::PI * b;
                    self.make_complex(
                        Object::Float(magnitude * angle.cos()),
                        Object::Float(magnitude * angle.sin()),
                        position,
                    )
                }
                BinaryOp::Power => Ok(Object::Float((a as f64).powf(b))),
                _ => unreachable!(),
            },
            (Object::Float(a), Object::Int(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - (b as f64))),
                BinaryOp::Multiply => Ok(Object::Float(a * (b as f64))),
                BinaryOp::Divide => Ok(Object::Float(a / (b as f64))),
                BinaryOp::Modulo => float_modulo(a, b as f64, position),
                // `powf` rounds once, where repeated multiplication through
                // `powi` drifts, so `10.0 ** 308` matches `(10 ** 308).to_f`.
                BinaryOp::Power => Ok(Object::Float(a.powf(b as f64))),
                _ => unreachable!(),
            },
            // `"ab" * 3` repeats the string, and a negative count is an
            // ArgumentError rather than an empty string.
            (
                Object::String(text),
                given @ (Object::Int(_)
                | Object::Float(_)
                | Object::BigInt(_)
                | Object::Instance(_)),
            ) if matches!(op, BinaryOp::Multiply) => {
                let count = self.repeat_count(&given, position)?;
                let Ok(count) = usize::try_from(count) else {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative argument",
                        position,
                    ));
                };
                // An empty string repeats to nothing however many times it is
                // asked for, so the width of the count never matters there.
                if text.as_str().is_empty() {
                    return Ok(Object::String(std::rc::Rc::new(
                        crate::object::StringValue::with_encoding(
                            String::new(),
                            text.encoding_name(),
                        ),
                    )));
                }
                if text.as_str().len().checked_mul(count).is_none() {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "argument too big",
                        position,
                    ));
                }
                let made = crate::object::StringValue::with_encoding(
                    text.as_str().repeat(count),
                    text.encoding_name(),
                );
                // A string standing for bytes repeats into one that stands
                // for bytes, so the run it spells is the run repeated.
                if text.holds_bytes() {
                    made.mark_bytes();
                }
                Ok(Object::String(std::rc::Rc::new(made)))
            }
            // `[1, 2] * 3` repeats the array and `[1, 2] * ", "` joins it,
            // which Array answers from its own method table.
            (left, right)
                if matches!(op, BinaryOp::Multiply)
                    && (matches!(left, Object::Array(_))
                        || matches!(
                            crate::vm::native_methods::array_subclass_value(&left),
                            Some(Object::Array(_))
                        )) =>
            {
                match self.call_array_method(&left, "*", std::slice::from_ref(&right), position)? {
                    Some(answered) => Ok(answered),
                    None => Err(binary_type_error(op.clone(), &left, &right, position)),
                }
            }
            (lhs, rhs) => Err(binary_type_error(op.clone(), &lhs, &rhs, position)),
        }
    }

    /// Evaluate comparison operations on numeric operands.
    pub(crate) fn evaluate_comparison(
        &mut self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if matches!(left, Object::Class(_) | Object::Module(_)) {
            return self.evaluate_module_comparison(op, left, right, position);
        }

        // Two exact integers compare exactly, without the rounding a Float
        // conversion would introduce at large magnitudes.
        if matches!(left, Object::Int(_) | Object::BigInt(_))
            && matches!(right, Object::Int(_) | Object::BigInt(_))
        {
            let a = left.as_big_integer().expect("integer-kinded");
            let b = right.as_big_integer().expect("integer-kinded");
            let result = match op {
                BinaryOp::Less => a < b,
                BinaryOp::Greater => a > b,
                BinaryOp::LessEqual => a <= b,
                BinaryOp::GreaterEqual => a >= b,
                _ => unreachable!("caller restricts op to the comparison set"),
            };
            return Ok(Object::Bool(result));
        }

        // A bignum against a Float is ordered exactly, since rounding the
        // bignum to a Float would lose the digits the answer turns on.
        let exact = match (&left, &right) {
            (Object::BigInt(value), Object::Float(float)) => {
                compare_integer_to_float(value, *float)
            }
            (Object::Float(float), Object::BigInt(value)) => {
                compare_integer_to_float(value, *float).map(|order| order.reverse())
            }
            _ => None,
        };
        if matches!(
            (&left, &right),
            (Object::BigInt(_), Object::Float(_)) | (Object::Float(_), Object::BigInt(_))
        ) {
            use std::cmp::Ordering;
            // Nothing compares against NaN, which every ordering reports as
            // false rather than as a failure.
            let result = match exact {
                None => false,
                Some(order) => match op {
                    BinaryOp::Less => order == Ordering::Less,
                    BinaryOp::Greater => order == Ordering::Greater,
                    BinaryOp::LessEqual => order != Ordering::Greater,
                    BinaryOp::GreaterEqual => order != Ordering::Less,
                    _ => unreachable!("caller restricts op to the comparison set"),
                },
            };
            return Ok(Object::Bool(result));
        }

        // Numeric comparisons
        let numeric_result = match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => Some((*a as f64, *b as f64)),
            (Object::Float(a), Object::Float(b)) => Some((*a, *b)),
            (Object::Int(a), Object::Float(b)) => Some((*a as f64, *b)),
            (Object::Float(a), Object::Int(b)) => Some((*a, *b as f64)),
            // String comparison
            (Object::String(a), Object::String(b)) => {
                let result = match op {
                    BinaryOp::Less => **a < **b,
                    BinaryOp::Greater => **a > **b,
                    BinaryOp::LessEqual => **a <= **b,
                    BinaryOp::GreaterEqual => **a >= **b,
                    _ => unreachable!(),
                };
                return Ok(Object::Bool(result));
            }
            _ => None,
        };

        if let Some((lhs, rhs)) = numeric_result {
            let result = match op {
                BinaryOp::Less => lhs < rhs,
                BinaryOp::Greater => lhs > rhs,
                BinaryOp::LessEqual => lhs <= rhs,
                BinaryOp::GreaterEqual => lhs >= rhs,
                _ => unreachable!(),
            };
            return Ok(Object::Bool(result));
        }

        // A user-defined comparison operator wins over the Comparable
        // protocol, the way `==` already dispatches to its own definition.
        if matches!(left, Object::Instance(_))
            && let Some(operator) = comparison_operator_name(op)
            && let Some((class, method)) = self.lookup_method(&left, operator)
            && !method.is_undefined
        {
            return self.invoke_method(class, method, left, vec![right], position);
        }

        // An object with neither the operator nor `<=>` still gets the call
        // offered to `method_missing`, as any other missing method would.
        if matches!(left, Object::Instance(_))
            && let Some(operator) = comparison_operator_name(op)
            && self.lookup_method(&left, "<=>").is_none()
            && let Some((class, method)) = self.lookup_method(&left, "method_missing")
        {
            let name = Object::symbol(operator.to_string());
            return self.invoke_method(class, method, left, vec![name, right], position);
        }

        // For instances, try dispatching to <=> method (Comparable protocol)
        if let Some((class, method)) = self.lookup_method(&left, "<=>") {
            let left_type = left.type_name().to_string();
            let right_type = right.type_name().to_string();
            let cmp_result = self.invoke_method(class, method, left, vec![right], position)?;
            let cmp_value: Option<f64> = match &cmp_result {
                Object::Int(n) => Some(*n as f64),
                Object::Float(f) => Some(*f),
                _ => None,
            };
            if let Some(n) = cmp_value {
                let result = match op {
                    BinaryOp::Less => n < 0.0,
                    BinaryOp::Greater => n > 0.0,
                    BinaryOp::LessEqual => n <= 0.0,
                    BinaryOp::GreaterEqual => n >= 0.0,
                    _ => unreachable!(),
                };
                return Ok(Object::Bool(result));
            }
            // nil or other: raise ArgumentError
            let exc = Object::exception(
                "ArgumentError",
                format!("comparison of {} with {} failed", left_type, right_type),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: "comparison failed".to_string(),
            });
        }

        // Comparison type mismatch is ArgumentError in Ruby, not TypeError.
        // Ruby's format: "comparison of <LeftClass> with <right_value> failed"
        let right_repr = match &right {
            Object::Int(n) => n.to_string(),
            Object::Float(f) => f.to_string(),
            Object::Nil => "nil".to_string(),
            Object::Bool(true) => "true".to_string(),
            Object::Bool(false) => "false".to_string(),
            _ => right.type_name().to_string(),
        };
        let msg = format!(
            "comparison of {} with {} failed",
            left.type_name(),
            right_repr
        );
        Err(MetorexError::UncaughtException {
            exception: Object::exception("ArgumentError", msg.clone()),
            location: position_to_location(position),
            message: msg,
        })
    }

    /// Evaluate `Module#<`, `#<=`, `#>`, and `#>=`. These report ancestry:
    /// true when the relationship holds, false when the opposite relationship
    /// holds, and nil when the two are unrelated. A non class/module argument
    /// raises TypeError.
    fn evaluate_module_comparison(
        &mut self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Object::Class(left_rc) | Object::Module(left_rc)) = &left else {
            unreachable!("caller checks the receiver is a class or module")
        };
        let (Object::Class(right_rc) | Object::Module(right_rc)) = &right else {
            let msg = "compared with non class/module".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", msg.clone()),
                location: position_to_location(position),
                message: msg,
            });
        };

        let left_is_descendant = self.builtins().is_subclass_of(left_rc, right_rc);
        let right_is_descendant = self.builtins().is_subclass_of(right_rc, left_rc);
        let same = left_is_descendant && right_is_descendant;

        let descendant_side = match op {
            BinaryOp::Less | BinaryOp::LessEqual => left_is_descendant,
            BinaryOp::Greater | BinaryOp::GreaterEqual => right_is_descendant,
            _ => unreachable!("only ordering operators reach module comparison"),
        };
        let ancestor_side = match op {
            BinaryOp::Less | BinaryOp::LessEqual => right_is_descendant,
            BinaryOp::Greater | BinaryOp::GreaterEqual => left_is_descendant,
            _ => unreachable!("only ordering operators reach module comparison"),
        };

        if same {
            let inclusive = matches!(op, BinaryOp::LessEqual | BinaryOp::GreaterEqual);
            return Ok(Object::Bool(inclusive));
        }
        if descendant_side {
            return Ok(Object::Bool(true));
        }
        if ancestor_side {
            return Ok(Object::Bool(false));
        }
        Ok(Object::Nil)
    }

    /// The elements of an operand that stands for an array: an Array itself,
    /// an instance of an Array subclass, or an object answering `to_ary`.
    /// A subclass is taken directly, which is why `to_ary` is not called on
    /// one.
    fn as_array_operand(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<Rc<std::cell::RefCell<Vec<Object>>>>, MetorexError> {
        if let Object::Array(elements) = value {
            return Ok(Some(Rc::clone(elements)));
        }
        if let Some(Object::Array(elements)) =
            crate::vm::native_methods::array_subclass_value(value)
        {
            return Ok(Some(elements));
        }
        if !self.responds_to(value, "to_ary") {
            return Ok(None);
        }
        // An error raised inside `to_ary` belongs to the caller, so it travels
        // out rather than reading as an operand that is not an array.
        match self.send_to_object(value.clone(), "to_ary", vec![], position)? {
            Object::Array(elements) => Ok(Some(elements)),
            _ => Ok(None),
        }
    }

    /// Order two arrays element by element, with the shorter one first when
    /// every shared element is equal. A pair already being ordered further up
    /// the stack is taken as equal, so two arrays that reach themselves answer
    /// rather than recursing forever.
    fn order_arrays(
        &mut self,
        left: &Rc<std::cell::RefCell<Vec<Object>>>,
        right: &Rc<std::cell::RefCell<Vec<Object>>>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let pair = (Rc::as_ptr(left) as usize, Rc::as_ptr(right) as usize);
        if pair.0 == pair.1 {
            return Ok(Object::Int(0));
        }
        let entered = ORDERING.with(|active| {
            let mut active = active.borrow_mut();
            if active.contains(&pair) {
                return false;
            }
            active.push(pair);
            true
        });
        if !entered {
            return Ok(Object::Int(0));
        }
        let outcome = self.order_array_elements(left, right, position);
        ORDERING.with(|active| {
            active.borrow_mut().pop();
        });
        outcome
    }

    fn order_array_elements(
        &mut self,
        left: &Rc<std::cell::RefCell<Vec<Object>>>,
        right: &Rc<std::cell::RefCell<Vec<Object>>>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let left_elements = left.borrow().clone();
        let right_elements = right.borrow().clone();
        for (one, other) in left_elements.iter().zip(right_elements.iter()) {
            let order = self.evaluate_binary_operation(
                &BinaryOp::Spaceship,
                one.clone(),
                other.clone(),
                position,
            )?;
            // The first result that is not zero is what the whole comparison
            // answers, whatever kind of value it is.
            match order {
                Object::Int(0) => continue,
                other => return Ok(other),
            }
        }
        Ok(Object::Int(
            left_elements.len().cmp(&right_elements.len()) as i64
        ))
    }

    /// Evaluate the spaceship operator (<=>), returning -1, 0, or 1.
    pub(crate) fn evaluate_spaceship(
        &self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Two exact integers order exactly, whatever their magnitude.
        if matches!(left, Object::Int(_) | Object::BigInt(_))
            && matches!(right, Object::Int(_) | Object::BigInt(_))
        {
            let a = left.as_big_integer().expect("integer-kinded");
            let b = right.as_big_integer().expect("integer-kinded");
            return Ok(Object::Int(a.cmp(&b) as i64));
        }
        // Nothing compares against a value that is not a number, which is
        // what a NaN on either side makes the answer.
        if matches!(&left, Object::Float(held) if held.is_nan())
            || matches!(&right, Object::Float(held) if held.is_nan())
        {
            return Ok(Object::Nil);
        }
        match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a.cmp(b) as i64)),
            // A bignum carries more digits than a Float has, so the two are
            // ordered exactly rather than by rounding the bignum.
            (Object::BigInt(a), Object::Float(b)) => {
                Ok(compare_or_nil(compare_integer_to_float(a, *b)))
            }
            (Object::Float(a), Object::BigInt(b)) => Ok(compare_or_nil(
                compare_integer_to_float(b, *a).map(std::cmp::Ordering::reverse),
            )),
            (Object::Float(a), Object::Float(b)) => Ok(compare_or_nil(a.partial_cmp(b))),
            (Object::Int(a), Object::Float(b)) => {
                let a = *a as f64;
                Ok(compare_or_nil(a.partial_cmp(b)))
            }
            (Object::Float(a), Object::Int(b)) => {
                let b = *b as f64;
                Ok(Object::Int(a.partial_cmp(&b).map_or(0, |o| o as i64)))
            }
            (Object::String(a), Object::String(b)) => Ok(Object::Int(a.cmp(b) as i64)),
            // A Symbol orders by its name, the way its String does.
            (Object::Symbol(a), Object::Symbol(b)) => Ok(Object::Int(a.cmp(b) as i64)),
            // Module#<=>: compares the ancestry relationship of two modules or
            // classes. -1 when the left is a descendant/includer of the right,
            // +1 when it's an ancestor/included-by, 0 when they're the same,
            // and nil when they're unrelated.
            (Object::Class(a) | Object::Module(a), Object::Class(b) | Object::Module(b)) => {
                let left_below_right = self.builtins().is_subclass_of(a, b);
                let right_below_left = self.builtins().is_subclass_of(b, a);
                Ok(match (left_below_right, right_below_left) {
                    (true, true) => Object::Int(0),
                    (true, false) => Object::Int(-1),
                    (false, true) => Object::Int(1),
                    (false, false) => Object::Nil,
                })
            }
            // Module#<=> against a non-module argument returns nil rather than
            // raising.
            (Object::Class(_) | Object::Module(_), _) => Ok(Object::Nil),
            // Two values of unrelated kinds have no ordering, and Ruby reports
            // that as nil rather than as an error. `sort` and friends turn it
            // into the ArgumentError a caller sees.
            _ => {
                let _ = position;
                Ok(Object::Nil)
            }
        }
    }

    /// Ruby's coercion protocol: a number handed an operand it does not know
    /// asks that operand to `coerce` it, then applies the operator to the pair
    /// it answers. An error raised inside `coerce` belongs to the caller, so it
    /// travels out rather than becoming a TypeError here.
    fn coerce_binary_operand(
        &mut self,
        op: &BinaryOp,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // A Rational or Complex asks for coercion too, since its own
        // arithmetic only knows the numbers it can already work with.
        let numeric_left =
            is_number(left) || crate::vm::native_methods::rational_parts(left).is_some();
        if !coerces_its_operand(op) || !numeric_left || !takes_coercion(right) {
            return Ok(None);
        }
        // The operand is asked whether it coerces, the way Ruby asks, so an
        // object that answers `respond_to?` for itself is heard.
        if !self.answers_to(right, "coerce", position)? {
            return Ok(None);
        }
        let pair = self.send_to_object(right.clone(), "coerce", vec![left.clone()], position)?;
        // A `coerce` that answers no pair leaves the operator with nothing to
        // apply, which for an ordering means the two have no order at all.
        let parts = match &pair {
            Object::Array(parts) if parts.borrow().len() == 2 => parts.borrow().clone(),
            // A `coerce` that answers no pair leaves the operator with
            // nothing to apply. Ruby refuses it for a Float, where the
            // comparison is a relational one, and reports no order at all
            // for the rest.
            _ if matches!(op, BinaryOp::Spaceship) && !matches!(left, Object::Float(_)) => {
                return Ok(Some(Object::Nil));
            }
            _ if matches!(op, BinaryOp::Spaceship) => {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "coerce must return [x, y]",
                    position,
                ));
            }
            _ => return Ok(None),
        };
        let name = crate::vm::native_methods::ast_methods::binary_op_str(op);
        self.send_to_object(parts[0].clone(), name, vec![parts[1].clone()], position)
            .map(Some)
    }
}

thread_local! {
    // The pairs of objects an inverse comparison is already running for, so a
    // `<=>` that turns around and asks the other object back is answered with
    // no order at all rather than running forever.
    static INVERSE_COMPARISONS: std::cell::RefCell<Vec<(usize, usize)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

impl VirtualMachine {
    /// How a String orders against another object, or None when the ordering
    /// is left to the general rules.
    fn order_string_against(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(text) = left else {
            return Ok(None);
        };
        // An instance of a String subclass orders by the characters behind it,
        // so a subclass compares equal to the string it was built from.
        let other = match right {
            Object::String(other) => Some(Rc::clone(other)),
            held => match crate::vm::native_methods::string_subclass_value(held) {
                Some(Object::String(other)) => Some(other),
                _ => None,
            },
        };
        if let Some(other) = other {
            return Ok(Some(order_two_strings(text, &other)));
        }
        // Anything that reads as a String is compared as the String it reads
        // as, which is what `to_str` answers.
        if self.responds_to(right, "to_str") {
            let read = self.send_to_object(right.clone(), "to_str", vec![], position)?;
            if let Object::String(other) = read {
                return Ok(Some(order_two_strings(text, &other)));
            }
            return Ok(Some(Object::Nil));
        }
        // Anything that orders itself is asked the other way round, and the
        // answer is turned around to match.
        if !self.responds_to(right, "<=>") {
            return Ok(Some(Object::Nil));
        }
        let pair = (object_identity(left), object_identity(right));
        if INVERSE_COMPARISONS.with(|held| held.borrow().contains(&pair)) {
            return Ok(Some(Object::Nil));
        }
        INVERSE_COMPARISONS.with(|held| held.borrow_mut().push(pair));
        let answered = self.send_to_object(right.clone(), "<=>", vec![left.clone()], position);
        INVERSE_COMPARISONS.with(|held| {
            held.borrow_mut().pop();
        });
        Ok(Some(match answered? {
            Object::Int(order) => Object::Int(-order),
            _ => Object::Nil,
        }))
    }
}

/// The address an object is known by while an inverse comparison runs. A value
/// with no address of its own stands for itself.
fn object_identity(held: &Object) -> usize {
    match held {
        Object::String(text) => Rc::as_ptr(text) as usize,
        Object::Instance(instance) => Rc::as_ptr(instance) as usize,
        Object::Array(elements) => Rc::as_ptr(elements) as usize,
        Object::Dict(entries) => Rc::as_ptr(entries) as usize,
        _ => 0,
    }
}

/// How one string orders against another. Ruby compares the bytes, and two
/// strings whose bytes are the same but whose encodings are not are ordered by
/// where those encodings sit in the list Ruby keeps.
fn order_two_strings(
    left: &Rc<crate::object::StringValue>,
    right: &Rc<crate::object::StringValue>,
) -> Object {
    let left_bytes = crate::vm::native_methods::string_methods::binary_bytes(left);
    let right_bytes = crate::vm::native_methods::string_methods::binary_bytes(right);
    match left_bytes.cmp(&right_bytes) {
        std::cmp::Ordering::Less => return Object::Int(-1),
        std::cmp::Ordering::Greater => return Object::Int(1),
        std::cmp::Ordering::Equal => (),
    }
    let (held, theirs) = (left.encoding_name(), right.encoding_name());
    // Text that is nothing but ASCII reads the same under either encoding, so
    // the two are equal whatever they are tagged with.
    if held == theirs || left_bytes.is_ascii() {
        return Object::Int(0);
    }
    let place = |named: &str| {
        crate::vm::init::ENCODING_NAMES
            .iter()
            .position(|(_, canonical, _)| *canonical == named)
            .unwrap_or(usize::MAX)
    };
    Object::Int(place(&held).cmp(&place(&theirs)) as i64)
}

/// Whether the operator consults `coerce` for an operand it does not know.
/// Equality does not: Ruby answers it by asking the other object instead.
fn coerces_its_operand(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::Power
            | BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::Xor
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual
            | BinaryOp::Spaceship
    )
}

/// Whether this operand is one of the numbers the operators handle directly.
fn is_number(value: &Object) -> bool {
    matches!(value, Object::Int(_) | Object::BigInt(_) | Object::Float(_))
}

/// Whether an operand is a candidate for coercion. Rational and Complex carry
/// their own arithmetic, so they are left to it.
fn takes_coercion(value: &Object) -> bool {
    match value {
        Object::Instance(instance) => {
            !matches!(instance.borrow().class.name(), "Rational" | "Complex")
        }
        _ => false,
    }
}

/// The Ruby method name for an ordering operator.
fn comparison_operator_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Less => Some("<"),
        BinaryOp::Greater => Some(">"),
        BinaryOp::LessEqual => Some("<="),
        BinaryOp::GreaterEqual => Some(">="),
        _ => None,
    }
}

/// The nearest Float to an arbitrary-precision integer, which is what Ruby
/// answers when one meets a Float in arithmetic. A magnitude past the Float
/// range becomes an infinity, as Ruby's does.
pub(crate) fn big_to_float(value: &num_bigint::BigInt) -> f64 {
    use std::str::FromStr;
    f64::from_str(&value.to_string()).unwrap_or(f64::INFINITY)
}

/// Subtract, multiply, divide, modulo, or raise two exact integers.
fn integer_arithmetic(
    op: &BinaryOp,
    left: num_bigint::BigInt,
    right: num_bigint::BigInt,
    position: Position,
) -> Result<Object, MetorexError> {
    use num_bigint::BigInt;
    match op {
        BinaryOp::Subtract => Ok(Object::integer(left - right)),
        BinaryOp::Multiply => Ok(Object::integer(left * right)),
        // Dividing two integers answers an integer, rounded toward negative
        // infinity rather than toward zero, so `-5 / 2` is -3.
        BinaryOp::Divide => {
            if right == BigInt::from(0) {
                return Err(divide_by_zero_error(position));
            }
            Ok(Object::integer(floored_quotient(&left, &right)))
        }
        // The modulus takes the sign of the divisor, which is what pairs with
        // that division: `-5 % 3` is 1.
        BinaryOp::Modulo => {
            if right == BigInt::from(0) {
                return Err(divide_by_zero_error(position));
            }
            Ok(Object::integer(floored_remainder(&left, &right)))
        }
        BinaryOp::Power => {
            // Zero raised to a negative power is a division by zero.
            if left == BigInt::from(0) && right < BigInt::from(0) {
                return Err(divide_by_zero_error(position));
            }
            // One and minus one stay themselves however far they are raised,
            // so the count never has to be built.
            if left == BigInt::from(1) {
                return Ok(Object::Int(1));
            }
            if left == BigInt::from(-1) && right >= BigInt::from(0) {
                let odd = right.bit(0);
                return Ok(Object::Int(if odd { -1 } else { 1 }));
            }
            // A negative exponent has no exact integer result, and one too
            // large to count is past the limit outright.
            let Ok(exponent) = u32::try_from(&right) else {
                if right > BigInt::from(0) {
                    return Err(exponent_too_large(position));
                }
                return Ok(Object::Float(
                    big_to_float(&left).powf(big_to_float(&right)),
                ));
            };
            // The answer takes one bit for each bit of the base, as many
            // times over as the exponent counts. Past the limit metorex
            // builds numbers up to, it is refused rather than attempted.
            let width = (left.bits() as u128).saturating_mul(exponent as u128);
            if width > WIDEST_INTEGER_BITS {
                return Err(exponent_too_large(position));
            }
            Ok(Object::integer(left.pow(exponent)))
        }
        _ => unreachable!("caller restricts op to the arithmetic set"),
    }
}

/// The most bits a number built by raising one to a power may take, which is
/// the room Ruby gives one before refusing to build it at all.
const WIDEST_INTEGER_BITS: u128 = 16 * 1024 * 1024 * 1024;

/// Ruby's ArgumentError for a power whose answer would be too wide to build.
fn exponent_too_large(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", "exponent is too large", position)
}

/// The same set of operations on two Floats.
fn float_arithmetic(
    op: &BinaryOp,
    left: f64,
    right: f64,
    _position: Position,
) -> Result<Object, MetorexError> {
    Ok(Object::Float(match op {
        BinaryOp::Subtract => left - right,
        BinaryOp::Multiply => left * right,
        BinaryOp::Divide => left / right,
        BinaryOp::Modulo => {
            return float_modulo(left, right, _position);
        }
        BinaryOp::Power => left.powf(right),
        _ => unreachable!("caller restricts op to the arithmetic set"),
    }))
}

/// Order an arbitrary-width integer against a Float without rounding the
/// integer, which a bignum does not survive. The Float is split at the decimal
/// point so its whole part is compared exactly and its fraction breaks a tie.
fn compare_integer_to_float(value: &num_bigint::BigInt, float: f64) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    if float.is_nan() {
        return None;
    }
    if float == f64::INFINITY {
        return Some(Ordering::Less);
    }
    if float == f64::NEG_INFINITY {
        return Some(Ordering::Greater);
    }
    let whole = float.trunc();
    let whole: num_bigint::BigInt = format!("{:.0}", whole).parse().ok()?;
    match value.cmp(&whole) {
        Ordering::Equal => (0.0).partial_cmp(&(float - float.trunc())),
        other => Some(other),
    }
}

/// The quotient of two integers rounded toward negative infinity, which is the
/// division Ruby pairs with its modulus.
fn floored_quotient(left: &num_bigint::BigInt, right: &num_bigint::BigInt) -> num_bigint::BigInt {
    let quotient = left / right;
    let remainder = left % right;
    if remainder != num_bigint::BigInt::from(0)
        && (remainder < num_bigint::BigInt::from(0)) != (*right < num_bigint::BigInt::from(0))
    {
        quotient - 1
    } else {
        quotient
    }
}

/// The remainder left by that division, which carries the sign of the divisor.
fn floored_remainder(left: &num_bigint::BigInt, right: &num_bigint::BigInt) -> num_bigint::BigInt {
    let remainder = left % right;
    if remainder != num_bigint::BigInt::from(0)
        && (remainder < num_bigint::BigInt::from(0)) != (*right < num_bigint::BigInt::from(0))
    {
        remainder + right
    } else {
        remainder
    }
}

/// Ruby's `%` on Floats leaves a remainder carrying the sign of the divisor,
/// the same way the integer one does, and refuses a zero divisor rather than
/// answering NaN.
fn float_modulo(left: f64, right: f64, position: Position) -> Result<Object, MetorexError> {
    if right == 0.0 {
        return Err(divide_by_zero_error(position));
    }
    let remainder = left % right;
    if remainder != 0.0 && (remainder < 0.0) != (right < 0.0) {
        return Ok(Object::Float(remainder + right));
    }
    Ok(Object::Float(remainder))
}

/// Whether two values are the same object, which is what `equal?` reports.
fn same_object(left: &Object, right: &Object) -> bool {
    match (left, right) {
        (Object::Instance(one), Object::Instance(other)) => Rc::ptr_eq(one, other),
        (Object::Array(one), Object::Array(other)) => Rc::ptr_eq(one, other),
        (Object::Dict(one), Object::Dict(other)) => Rc::ptr_eq(one, other),
        _ => false,
    }
}

/// The Set method an operator spells, if it names one.
fn set_operator_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add | BinaryOp::BitwiseOr => Some("union"),
        BinaryOp::Subtract => Some("difference"),
        BinaryOp::BitwiseAnd => Some("intersection"),
        BinaryOp::Xor => Some("^"),
        BinaryOp::LessEqual => Some("subset?"),
        BinaryOp::GreaterEqual => Some("superset?"),
        BinaryOp::Less => Some("proper_subset?"),
        BinaryOp::Greater => Some("proper_superset?"),
        BinaryOp::Spaceship => Some("<=>"),
        _ => None,
    }
}

impl VirtualMachine {
    /// How many times `"ab" * count` repeats the string: a Float loses its
    /// fraction, an object gives its `to_int`, and a number too wide to hold
    /// is a RangeError the way Ruby reports one.
    fn repeat_count(
        &mut self,
        count: &Object,
        position: crate::lexer::Position,
    ) -> Result<i64, MetorexError> {
        match count {
            Object::Int(number) => Ok(*number),
            Object::Float(number) => Ok(*number as i64),
            Object::BigInt(_) => Err(crate::vm::errors::simple_exception(
                "RangeError",
                "bignum too big to convert into `long'",
                position,
            )),
            other => match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                Object::Int(number) => Ok(number),
                Object::BigInt(_) => Err(crate::vm::errors::simple_exception(
                    "RangeError",
                    "bignum too big to convert into `long'",
                    position,
                )),
                converted => Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!(
                        "no implicit conversion of {} into Integer",
                        self.builtins().class_of(&converted).name()
                    ),
                    position,
                )),
            },
        }
    }
}

/// One ordering as `<=>` reports it, where an absent ordering is nil.
fn compare_or_nil(order: Option<std::cmp::Ordering>) -> Object {
    match order {
        Some(order) => Object::Int(order as i64),
        None => Object::Nil,
    }
}

/// The name a numeric operator is written under, for looking one up that the
/// program defined of its own.
fn comparison_free_operator_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+"),
        BinaryOp::Subtract => Some("-"),
        BinaryOp::Multiply => Some("*"),
        BinaryOp::Divide => Some("/"),
        BinaryOp::Modulo => Some("%"),
        BinaryOp::Power => Some("**"),
        _ => None,
    }
}
