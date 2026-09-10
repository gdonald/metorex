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

/// A number in exponent notation: one digit, a point, `precision` digits, and
/// the signed power of ten written with at least two figures.
fn exponent_notation(value: f64, precision: usize) -> String {
    let written = format!("{:.precision$e}", value);
    match written.split_once('e') {
        Some((mantissa, power)) => {
            let (sign, digits) = match power.strip_prefix('-') {
                Some(rest) => ("-", rest),
                None => ("+", power),
            };
            format!("{mantissa}e{sign}{:0>2}", digits)
        }
        None => written,
    }
}

impl VirtualMachine {
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
                // `+str` asks for a string that changes, so a frozen one
                // answers a copy and any other answers itself.
                Object::String(ref text) => {
                    if !text.is_frozen() {
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
                        return Ok(value.clone());
                    }
                    let copy = crate::object::StringValue::with_encoding(
                        text.to_text(),
                        text.encoding_name(),
                    );
                    copy.freeze();
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
            Subtract | BitwiseAnd | BitwiseOr
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
            Add => self.evaluate_addition(left, right, position),
            Modulo if matches!(left, Object::String(_)) => {
                self.evaluate_string_format(left, right, position)
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
                // Two arrays holding objects of the program's own are equal
                // when those objects say so, so each pair is asked with `==`
                // rather than compared as data.
                if let (Object::Array(one), Object::Array(other)) = (&left, &right) {
                    if Rc::ptr_eq(one, other) {
                        return Ok(Object::Bool(true));
                    }
                    let held = one.borrow().clone();
                    let against = other.borrow().clone();
                    if held
                        .iter()
                        .chain(against.iter())
                        .any(|item| matches!(item, Object::Instance(_)))
                    {
                        if held.len() != against.len() {
                            return Ok(Object::Bool(false));
                        }
                        for (item, counterpart) in held.iter().zip(against.iter()) {
                            let answer = self.evaluate_binary_operation(
                                &BinaryOp::Equal,
                                item.clone(),
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
                    // For non-class/module RHS, check the receiver's class
                    // chain. Also include the singleton class of Instances so
                    // `Module === obj.extend(Module)` returns true.
                    if !matches!(right, Object::Class(_) | Object::Module(_)) {
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
                self.evaluate_spaceship(left, right, position)
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
                let mut combined = a.as_str().to_string();
                combined.push_str(&b.as_ref().as_str());
                Ok(Object::string(combined))
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
                Ok(Object::String(std::rc::Rc::new(
                    crate::object::StringValue::with_encoding(
                        text.as_str().repeat(count),
                        text.encoding_name(),
                    ),
                )))
            }
            // `[1, 2] * 3` repeats the array, and `[1, 2] * ", "` joins it
            // with that separator.
            (Object::Array(elements), Object::Int(count)) if matches!(op, BinaryOp::Multiply) => {
                let Ok(count) = usize::try_from(count) else {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative argument",
                        position,
                    ));
                };
                let source = elements.borrow().clone();
                let mut repeated = Vec::with_capacity(source.len() * count);
                for _ in 0..count {
                    repeated.extend(source.iter().cloned());
                }
                Ok(Object::array(repeated))
            }
            (Object::Array(elements), Object::String(separator))
                if matches!(op, BinaryOp::Multiply) =>
            {
                let joined = elements
                    .borrow()
                    .iter()
                    .map(|element| format!("{}", element))
                    .collect::<Vec<_>>()
                    .join(&*separator.as_str());
                Ok(Object::string(joined))
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

    /// Evaluate Ruby-style String `%` formatting (`"hello %s" % "world"`).
    ///
    /// When the right operand is an Array, each element is consumed in order by
    /// successive format specifiers. Otherwise the single value is used for the
    /// first (and only expected) specifier.
    /// The value a `%{name}` or `%<name>` reference names, which must be a
    /// key of the Hash the format was given.
    fn format_keyword_value(
        &self,
        right: &Object,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Object::Dict(entries) = right
            && let Some(value) = entries.borrow().get(&format!(":{}", name))
        {
            return Ok(value.clone());
        }
        let message = format!("key<{}> not found", name);
        Err(MetorexError::UncaughtException {
            exception: Object::exception("KeyError", message.clone()),
            location: position_to_location(position),
            message,
        })
    }

    pub(crate) fn evaluate_string_format(
        &self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Object::String(format) = &left else {
            let message = format!("no implicit conversion of {} into String", left.type_name());
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: position_to_location(position),
                message,
            });
        };
        let fmt_str = format.as_str().to_string();

        let args: Vec<Object> = match &right {
            Object::Array(arr) => arr.borrow().clone(),
            // A keyword Hash names its values rather than filling positions.
            Object::Dict(_) => Vec::new(),
            other => vec![(*other).clone()],
        };

        let mut result = String::new();
        let mut arg_idx = 0;
        let mut named: Option<Object> = None;
        let chars: Vec<char> = fmt_str.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '%' {
                i += 1;
                if i >= chars.len() {
                    result.push('%');
                    break;
                }

                // Literal %%
                if chars[i] == '%' {
                    result.push('%');
                    i += 1;
                    continue;
                }

                // `%{name}` and `%<name>s` read a keyword from the Hash on
                // the right rather than taking the next positional argument.
                if chars[i] == '{' || chars[i] == '<' {
                    let closing = if chars[i] == '{' { '}' } else { '>' };
                    let mut name = String::new();
                    i += 1;
                    while i < chars.len() && chars[i] != closing {
                        name.push(chars[i]);
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1;
                    }
                    let value = self.format_keyword_value(&right, &name, position)?;
                    if closing == '}' {
                        result.push_str(&format!("{}", value));
                        continue;
                    }
                    // `%<name>` carries on into the specifier that follows,
                    // formatting the value it named.
                    named = Some(value);
                }

                // Parse optional flags: -, +, 0, space
                let mut left_align = false;
                let mut plus_sign = false;
                let mut zero_pad = false;
                let mut space_sign = false;
                loop {
                    if i >= chars.len() {
                        break;
                    }
                    match chars[i] {
                        '-' => {
                            left_align = true;
                            i += 1;
                        }
                        '+' => {
                            plus_sign = true;
                            i += 1;
                        }
                        '0' => {
                            zero_pad = true;
                            i += 1;
                        }
                        ' ' => {
                            space_sign = true;
                            i += 1;
                        }
                        _ => break,
                    }
                }

                // Parse optional width
                let mut width: Option<usize> = None;
                if i < chars.len() && chars[i].is_ascii_digit() {
                    let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    width = Some(
                        chars[start..i]
                            .iter()
                            .collect::<String>()
                            .parse()
                            .unwrap_or(0),
                    );
                }

                // Parse optional precision (.N)
                let mut precision: Option<usize> = None;
                if i < chars.len() && chars[i] == '.' {
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    precision = Some(
                        chars[start..i]
                            .iter()
                            .collect::<String>()
                            .parse()
                            .unwrap_or(0),
                    );
                }

                if i >= chars.len() {
                    return Err(MetorexError::runtime_error(
                        "incomplete format specifier in String#%".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                }

                let specifier = chars[i];
                i += 1;

                // A `%<name>` prefix already chose the value; otherwise the
                // next positional argument fills the specifier.
                let taken = named.take();
                if taken.is_none() && arg_idx >= args.len() {
                    return Err(MetorexError::runtime_error(
                        "too few arguments for format string".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let arg = match &taken {
                    Some(value) => value,
                    None => {
                        let value = &args[arg_idx];
                        arg_idx += 1;
                        value
                    }
                };

                let formatted = match specifier {
                    's' => {
                        // `%s` renders with `to_s`, so a Symbol loses its
                        // leading colon the way `puts` drops it and nil
                        // renders as nothing at all.
                        let s = match &arg {
                            Object::Symbol(name) => name.as_str().to_string(),
                            Object::Nil => String::new(),
                            other => format!("{}", other),
                        };
                        if let Some(prec) = precision {
                            s[..s.len().min(prec)].to_string()
                        } else {
                            s
                        }
                    }
                    'd' | 'i' => match arg {
                        Object::Int(n) => {
                            if plus_sign && *n >= 0 {
                                format!("+{}", n)
                            } else if space_sign && *n >= 0 {
                                format!(" {}", n)
                            } else {
                                format!("{}", n)
                            }
                        }
                        Object::Float(f) => {
                            let n = *f as i64;
                            if plus_sign && n >= 0 {
                                format!("+{}", n)
                            } else if space_sign && n >= 0 {
                                format!(" {}", n)
                            } else {
                                format!("{}", n)
                            }
                        }
                        _ => format!("{}", arg),
                    },
                    'f' => {
                        let val = match arg {
                            Object::Float(f) => *f,
                            Object::Int(n) => *n as f64,
                            _ => {
                                return Err(MetorexError::runtime_error(
                                    format!(
                                        "%%f requires numeric argument, got {}",
                                        arg.type_name()
                                    ),
                                    crate::vm::utils::position_to_location(position),
                                ));
                            }
                        };
                        let prec = precision.unwrap_or(6);
                        if plus_sign && val >= 0.0 {
                            format!("+{:.prec$}", val)
                        } else if space_sign && val >= 0.0 {
                            format!(" {:.prec$}", val)
                        } else {
                            format!("{:.prec$}", val)
                        }
                    }
                    // `%e` and `%E` write a number in exponent notation, with
                    // one digit before the point and a signed two-digit power.
                    'e' | 'E' => {
                        let val = match arg {
                            Object::Float(held) => *held,
                            Object::Int(held) => *held as f64,
                            Object::BigInt(held) => held.to_string().parse().unwrap_or(0.0),
                            _ => {
                                return Err(MetorexError::runtime_error(
                                    format!(
                                        "%%{} requires numeric argument, got {}",
                                        specifier,
                                        arg.type_name()
                                    ),
                                    crate::vm::utils::position_to_location(position),
                                ));
                            }
                        };
                        let written = exponent_notation(val, precision.unwrap_or(6));
                        let written = if specifier == 'E' {
                            written.to_uppercase()
                        } else {
                            written
                        };
                        if plus_sign && val >= 0.0 {
                            format!("+{}", written)
                        } else if space_sign && val >= 0.0 {
                            format!(" {}", written)
                        } else {
                            written
                        }
                    }
                    'x' => match arg {
                        Object::Int(n) => format!("{:x}", n),
                        _ => format!("{}", arg),
                    },
                    'X' => match arg {
                        Object::Int(n) => format!("{:X}", n),
                        _ => format!("{}", arg),
                    },
                    'o' => match arg {
                        Object::Int(n) => format!("{:o}", n),
                        _ => format!("{}", arg),
                    },
                    'b' => match arg {
                        Object::Int(n) => format!("{:b}", n),
                        _ => format!("{}", arg),
                    },
                    'p' => match arg {
                        Object::String(s) => format!("\"{}\"", s),
                        Object::Nil => "nil".to_string(),
                        other => format!("{}", other),
                    },
                    'c' => match arg {
                        Object::Int(n) => {
                            if let Some(ch) = char::from_u32(*n as u32) {
                                ch.to_string()
                            } else {
                                format!("{}", n)
                            }
                        }
                        Object::String(s) => s
                            .as_str()
                            .chars()
                            .next()
                            .map_or(String::new(), |c| c.to_string()),
                        _ => format!("{}", arg),
                    },
                    other => {
                        return Err(MetorexError::runtime_error(
                            format!("unknown format specifier '%{}'", other),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };

                // Apply width and alignment
                if let Some(w) = width {
                    if left_align {
                        result.push_str(&format!("{:<w$}", formatted));
                    } else if zero_pad
                        && matches!(specifier, 'd' | 'i' | 'f' | 'x' | 'X' | 'o' | 'b')
                    {
                        result.push_str(&format!("{:0>w$}", formatted));
                    } else {
                        result.push_str(&format!("{:>w$}", formatted));
                    }
                } else {
                    result.push_str(&formatted);
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        // With `$VERBOSE` on, Ruby points out arguments the format never
        // reached. A keyword Hash names its values, so leaving one of those
        // unused is not a mistake and is not counted here.
        if arg_idx < args.len() && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true)))
        {
            eprintln!("warning: too many arguments for format string");
        }
        Ok(Object::string(result))
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
        match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a.cmp(b) as i64)),
            // A bignum carries more digits than a Float has, so the two are
            // ordered exactly rather than by rounding the bignum.
            (Object::BigInt(a), Object::Float(b)) => Ok(Object::Int(
                compare_integer_to_float(a, *b).map_or(0, |order| order as i64),
            )),
            (Object::Float(a), Object::BigInt(b)) => Ok(Object::Int(
                compare_integer_to_float(b, *a).map_or(0, |order| order.reverse() as i64),
            )),
            (Object::Float(a), Object::Float(b)) => {
                Ok(Object::Int(a.partial_cmp(b).map_or(0, |o| o as i64)))
            }
            (Object::Int(a), Object::Float(b)) => {
                let a = *a as f64;
                Ok(Object::Int(a.partial_cmp(b).map_or(0, |o| o as i64)))
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
        if !self.responds_to(right, "coerce") {
            return Ok(None);
        }
        let pair = self.send_to_object(right.clone(), "coerce", vec![left.clone()], position)?;
        // A `coerce` that answers no pair leaves the operator with nothing to
        // apply, which for an ordering means the two have no order at all.
        let parts = match &pair {
            Object::Array(parts) if parts.borrow().len() == 2 => parts.borrow().clone(),
            _ if matches!(op, BinaryOp::Spaceship) => return Ok(Some(Object::Nil)),
            _ => return Ok(None),
        };
        let name = crate::vm::native_methods::ast_methods::binary_op_str(op);
        self.send_to_object(parts[0].clone(), name, vec![parts[1].clone()], position)
            .map(Some)
    }
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
            // A negative exponent has no exact integer result.
            let Ok(exponent) = u32::try_from(&right) else {
                return Ok(Object::Float(
                    big_to_float(&left).powf(big_to_float(&right)),
                ));
            };
            Ok(Object::integer(left.pow(exponent)))
        }
        _ => unreachable!("caller restricts op to the arithmetic set"),
    }
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
