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
                Object::Int(_) | Object::Float(_) => Ok(value),
                // Ruby's `+"str"` returns a mutable copy of the string. We
                // don't track frozenness, so this is just an identity op.
                Object::String(_) => Ok(value),
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Minus => match value {
                Object::Int(v) => Ok(Object::Int(-v)),
                Object::Float(v) => Ok(Object::Float(-v)),
                // Ruby's `-"str"` answers a frozen, deduplicated string.
                // Metorex's strings are already shared and report frozen, so
                // this is an identity op the way `+"str"` is.
                Object::String(_) => Ok(value),
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Not => Ok(Object::Bool(matches!(
                value,
                Object::Bool(false) | Object::Nil
            ))),
        }
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

        match op {
            Add => self.evaluate_addition(left, right, position),
            Modulo if matches!(left, Object::String(_)) => {
                self.evaluate_string_format(left, right, position)
            }
            Subtract | Multiply | Divide | Modulo | Power => {
                self.evaluate_numeric_binary(op, left, right, position)
            }
            Equal => {
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
                Ok(Object::Bool(left.equals(&right)))
            }
            CaseEqual => {
                // Regexp === str: whether the pattern matches anywhere.
                if let Object::Regex(pattern, flags) = &left {
                    // A Regexp matches a Symbol's name as readily as a String.
                    let (Object::String(subject) | Object::Symbol(subject)) = &right else {
                        return Ok(Object::Bool(false));
                    };
                    let source = if flags.contains('i') {
                        format!("(?i){}", pattern)
                    } else {
                        pattern.as_ref().clone()
                    };
                    return Ok(Object::Bool(match regex::Regex::new(&source) {
                        Ok(compiled) => compiled.is_match(subject),
                        Err(_) => false,
                    }));
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
                        // Check exception class hierarchy via globals
                        if let Some(Object::Class(exc_class)) = self.globals().get(&exc_type) {
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
                // overrides those does not change the answer.
                if let Object::Instance(inst_rc) = &left {
                    if let Object::Instance(rhs) = &right
                        && Rc::ptr_eq(inst_rc, rhs)
                    {
                        return Ok(Object::Bool(true));
                    }
                    return self.evaluate_binary_operation(&Equal, left, right, position);
                }
                Ok(Object::Bool(left.equals(&right)))
            }
            NotEqual => Ok(Object::Bool(!left.equals(&right))),
            Less | Greater | LessEqual | GreaterEqual => {
                self.evaluate_comparison(op, left, right, position)
            }
            Spaceship => {
                // Object#<=> answers 0 for the same object or when #== says
                // so, and nil otherwise. It never consults #eql?. A class that
                // defines its own #<=> is dispatched before reaching here.
                if let Object::Instance(inst_rc) = &left {
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
                self.evaluate_spaceship(left, right, position)
            }
            BitwiseAnd => match (left, right) {
                // Array intersection, keeping the left operand's order and
                // dropping duplicates.
                (Object::Array(left_items), Object::Array(right_items)) => {
                    let right_items = right_items.borrow().clone();
                    let mut intersection: Vec<Object> = Vec::new();
                    for item in left_items.borrow().iter() {
                        if right_items.iter().any(|other| other.equals(item))
                            && !intersection.iter().any(|kept| kept.equals(item))
                        {
                            intersection.push(item.clone());
                        }
                    }
                    Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                        intersection,
                    ))))
                }
                // nil & x always returns false (Ruby semantics)
                (Object::Nil, _) | (_, Object::Nil) => Ok(Object::Bool(false)),
                (Object::Bool(a), Object::Bool(b)) => Ok(Object::Bool(a & b)),
                (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a & b)),
                (Object::Bool(a), other) => Ok(Object::Bool(a & other.is_truthy())),
                (other, Object::Bool(b)) => Ok(Object::Bool(other.is_truthy() & b)),
                (lhs, rhs) => Err(binary_type_error(BitwiseAnd, &lhs, &rhs, position)),
            },
            BitwiseOr => match (left, right) {
                // Array union, keeping first-seen order and dropping
                // duplicates.
                (Object::Array(left_items), Object::Array(right_items)) => {
                    let mut union: Vec<Object> = Vec::new();
                    for item in left_items
                        .borrow()
                        .iter()
                        .chain(right_items.borrow().iter())
                    {
                        if !union.iter().any(|kept| kept.equals(item)) {
                            union.push(item.clone());
                        }
                    }
                    Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                        union,
                    ))))
                }
                // nil | x returns truthiness of x
                (Object::Nil, other) => Ok(Object::Bool(other.is_truthy())),
                (Object::Bool(a), Object::Bool(b)) => Ok(Object::Bool(a | b)),
                (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a | b)),
                (Object::Bool(a), other) => Ok(Object::Bool(a | other.is_truthy())),
                (other, Object::Bool(b)) => Ok(Object::Bool(other.is_truthy() | b)),
                (lhs, rhs) => Err(binary_type_error(BitwiseOr, &lhs, &rhs, position)),
            },
            Xor => match (left, right) {
                // nil ^ x returns truthiness of x
                (Object::Nil, other) => Ok(Object::Bool(other.is_truthy())),
                (Object::Bool(a), Object::Bool(b)) => Ok(Object::Bool(a ^ b)),
                (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a ^ b)),
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
                None => Ok(Object::Float((a as f64) + (b as f64))),
            },
            (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a + b)),
            (Object::Int(a), Object::Float(b)) => Ok(Object::Float((a as f64) + b)),
            (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a + (b as f64))),
            (Object::String(a), Object::String(b)) => {
                let mut combined = a.as_ref().clone();
                combined.push_str(b.as_ref());
                Ok(Object::String(Rc::new(combined)))
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
        &self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
            (Object::Int(a), Object::Int(b)) => match op {
                BinaryOp::Subtract => match a.checked_sub(b) {
                    Some(v) => Ok(Object::Int(v)),
                    None => Ok(Object::Float((a as f64) - (b as f64))),
                },
                BinaryOp::Multiply => match a.checked_mul(b) {
                    Some(v) => Ok(Object::Int(v)),
                    None => Ok(Object::Float((a as f64) * (b as f64))),
                },
                BinaryOp::Divide => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else if a % b == 0 {
                        Ok(Object::Int(a / b))
                    } else {
                        Ok(Object::Float((a as f64) / (b as f64)))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Int(a % b))
                    }
                }
                BinaryOp::Power => {
                    if b < 0 {
                        Ok(Object::Float((a as f64).powf(b as f64)))
                    } else {
                        Ok(Object::Int((a as f64).powi(b as i32) as i64))
                    }
                }
                _ => unreachable!(),
            },
            (Object::Float(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - b)),
                BinaryOp::Multiply => Ok(Object::Float(a * b)),
                // Float division by zero follows IEEE 754 and answers an
                // infinity or NaN. Only Integer / Integer raises.
                BinaryOp::Divide => Ok(Object::Float(a / b)),
                BinaryOp::Modulo => Ok(Object::Float(a % b)),
                BinaryOp::Power => Ok(Object::Float(a.powf(b))),
                _ => unreachable!(),
            },
            (Object::Int(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float((a as f64) - b)),
                BinaryOp::Multiply => Ok(Object::Float((a as f64) * b)),
                BinaryOp::Divide => Ok(Object::Float((a as f64) / b)),
                BinaryOp::Modulo => Ok(Object::Float((a as f64) % b)),
                BinaryOp::Power => Ok(Object::Float((a as f64).powf(b))),
                _ => unreachable!(),
            },
            (Object::Float(a), Object::Int(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - (b as f64))),
                BinaryOp::Multiply => Ok(Object::Float(a * (b as f64))),
                BinaryOp::Divide => Ok(Object::Float(a / (b as f64))),
                BinaryOp::Modulo => Ok(Object::Float(a % (b as f64))),
                BinaryOp::Power => Ok(Object::Float(a.powi(b as i32))),
                _ => unreachable!(),
            },
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
            let name = Object::Symbol(Rc::new(operator.to_string()));
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
        let fmt_str = format.as_ref().clone();

        let args: Vec<Object> = match right {
            Object::Array(arr) => arr.borrow().clone(),
            other => vec![other],
        };

        let mut result = String::new();
        let mut arg_idx = 0;
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

                if arg_idx >= args.len() {
                    return Err(MetorexError::runtime_error(
                        "too few arguments for format string".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                }

                let arg = &args[arg_idx];
                arg_idx += 1;

                let formatted = match specifier {
                    's' => {
                        // `%s` renders with `to_s`, so a Symbol loses its
                        // leading colon the way `puts` drops it.
                        let s = match &arg {
                            Object::Symbol(name) => (**name).clone(),
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
                        Object::String(s) => {
                            s.chars().next().map_or(String::new(), |c| c.to_string())
                        }
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

        Ok(Object::String(Rc::new(result)))
    }

    /// Evaluate the spaceship operator (<=>), returning -1, 0, or 1.
    pub(crate) fn evaluate_spaceship(
        &self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a.cmp(b) as i64)),
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
            _ => Err(binary_type_error(
                BinaryOp::Spaceship,
                &left,
                &right,
                position,
            )),
        }
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
