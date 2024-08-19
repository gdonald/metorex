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
                Object::Int(_) | Object::Float(_) => Ok(value),
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Minus => match value {
                Object::Int(v) => Ok(Object::Int(-v)),
                Object::Float(v) => Ok(Object::Float(-v)),
                _ => Err(unary_type_error(op, &value, position)),
            },
        }
    }

    /// Evaluate a binary operation across runtime values.
    pub(crate) fn evaluate_binary_operation(
        &self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;

        match op {
            Add => self.evaluate_addition(left, right, position),
            Subtract | Multiply | Divide | Modulo => {
                self.evaluate_numeric_binary(op, left, right, position)
            }
            Equal => Ok(Object::Bool(left.equals(&right))),
            NotEqual => Ok(Object::Bool(!left.equals(&right))),
            Less | Greater | LessEqual | GreaterEqual => {
                self.evaluate_comparison(op, left, right, position)
            }
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
            (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a + b)),
            (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a + b)),
            (Object::Int(a), Object::Float(b)) => Ok(Object::Float((a as f64) + b)),
            (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a + (b as f64))),
            (Object::String(a), Object::String(b)) => {
                let mut combined = a.as_ref().clone();
                combined.push_str(b.as_ref());
                Ok(Object::String(Rc::new(combined)))
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
                BinaryOp::Subtract => Ok(Object::Int(a - b)),
                BinaryOp::Multiply => Ok(Object::Int(a * b)),
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
                _ => unreachable!(),
            },
            (lhs, rhs) => Err(binary_type_error(op.clone(), &lhs, &rhs, position)),
        }
    }

    /// Evaluate comparison operations on numeric operands.
    pub(crate) fn evaluate_comparison(
        &self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (lhs, rhs) = match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => (*a as f64, *b as f64),
            (Object::Float(a), Object::Float(b)) => (*a, *b),
            (Object::Int(a), Object::Float(b)) => (*a as f64, *b),
            (Object::Float(a), Object::Int(b)) => (*a, *b as f64),
            (lhs, rhs) => {
                return Err(binary_type_error(op.clone(), lhs, rhs, position));
            }
        };

        let result = match op {
            BinaryOp::Less => lhs < rhs,
            BinaryOp::Greater => lhs > rhs,
            BinaryOp::LessEqual => lhs <= rhs,
            BinaryOp::GreaterEqual => lhs >= rhs,
            _ => unreachable!(),
        };

        Ok(Object::Bool(result))
    }
}
