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
                // Class === obj: check type membership (Ruby's case equality)
                if let Object::Class(class_rc) = &left {
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
                    if !matches!(right, Object::Class(_) | Object::Module(_)) {
                        let right_class = self.builtins().class_of(&right);
                        return Ok(Object::Bool(
                            self.builtins().is_subclass_of(&right_class, class_rc),
                        ));
                    }
                }
                // Fallback: === behaves like ==
                Ok(Object::Bool(left.equals(&right)))
            }
            NotEqual => Ok(Object::Bool(!left.equals(&right))),
            Less | Greater | LessEqual | GreaterEqual => {
                self.evaluate_comparison(op, left, right, position)
            }
            Spaceship => self.evaluate_spaceship(left, right, position),
            BitwiseAnd => match (left, right) {
                // nil & x always returns false (Ruby semantics)
                (Object::Nil, _) | (_, Object::Nil) => Ok(Object::Bool(false)),
                (Object::Bool(a), Object::Bool(b)) => Ok(Object::Bool(a & b)),
                (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a & b)),
                (Object::Bool(a), other) => Ok(Object::Bool(a & other.is_truthy())),
                (other, Object::Bool(b)) => Ok(Object::Bool(other.is_truthy() & b)),
                (lhs, rhs) => Err(binary_type_error(BitwiseAnd, &lhs, &rhs, position)),
            },
            BitwiseOr => match (left, right) {
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
                BinaryOp::Divide => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a / b))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a % b))
                    }
                }
                BinaryOp::Power => Ok(Object::Float(a.powf(b))),
                _ => unreachable!(),
            },
            (Object::Int(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float((a as f64) - b)),
                BinaryOp::Multiply => Ok(Object::Float((a as f64) * b)),
                BinaryOp::Divide => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float((a as f64) / b))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float((a as f64) % b))
                    }
                }
                BinaryOp::Power => Ok(Object::Float((a as f64).powf(b))),
                _ => unreachable!(),
            },
            (Object::Float(a), Object::Int(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - (b as f64))),
                BinaryOp::Multiply => Ok(Object::Float(a * (b as f64))),
                BinaryOp::Divide => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a / (b as f64)))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a % (b as f64)))
                    }
                }
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
        let fmt_str = match &left {
            Object::String(s) => s.as_ref().clone(),
            _ => unreachable!("caller guarantees left is String"),
        };

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
                        let s = format!("{}", arg);
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
            _ => Err(binary_type_error(
                BinaryOp::Spaceship,
                &left,
                &right,
                position,
            )),
        }
    }
}
