// Expression-evaluation dispatch: a thin `match` over `Expression` variants
// that delegates each variant to a helper in the sibling modules.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expression};
use crate::error::MetorexError;
use crate::object::{BlockStatement, Object};

use crate::vm::core::VirtualMachine;
use crate::vm::utils::{is_truthy, position_to_location};

impl VirtualMachine {
    /// Dispatch over an expression variant. Each branch is small; large branches
    /// delegate to a helper in `vm/eval/`.
    pub(crate) fn evaluate_expression_inner(
        &mut self,
        expression: &Expression,
    ) -> Result<Object, MetorexError> {
        match expression {
            // ── Literals ────────────────────────────────────────────────────
            Expression::IntLiteral { value, .. } => Ok(Object::Int(*value)),
            Expression::FloatLiteral { value, .. } => Ok(Object::Float(*value)),
            Expression::StringLiteral { value, .. } => Ok(Object::String(Rc::new(value.clone()))),
            Expression::Symbol { value, .. } => Ok(Object::Symbol(Rc::new(value.clone()))),
            Expression::RegexLiteral { pattern, flags, .. } => Ok(Object::Regex(
                Rc::new(pattern.clone()),
                Rc::new(flags.clone()),
            )),
            Expression::BoolLiteral { value, .. } => Ok(Object::Bool(*value)),
            Expression::NilLiteral { .. } => Ok(Object::Nil),
            Expression::InterpolatedString { parts, .. } => self
                .evaluate_interpolated_string(parts)
                .map(|s| Object::String(Rc::new(s))),

            // ── Variables / identifiers ─────────────────────────────────────
            Expression::Identifier { name, position } => self.eval_identifier(name, *position),
            Expression::SelfExpr { position } => self.eval_self(*position),
            Expression::InstanceVariable { name, position } => {
                self.eval_instance_var_read(name, *position)
            }
            Expression::ClassVariable { name, position } => {
                self.eval_class_var_read(name, *position)
            }
            Expression::GlobalVariable { name, .. } => {
                Ok(self.globals().get(name).unwrap_or(Object::Nil))
            }
            Expression::MagicFile { .. } => {
                let path = self
                    .get_current_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(eval)".to_string());
                Ok(Object::String(Rc::new(path)))
            }
            Expression::MagicLine { position, .. } => Ok(Object::Int(position.line as i64)),

            // ── Closures, grouping ──────────────────────────────────────────
            Expression::Lambda {
                parameters,
                body,
                captured_vars,
                ..
            } => {
                let mut captured = HashMap::new();
                if let Some(names) = captured_vars {
                    if names.is_empty() {
                        // Empty vec signals automatic capture of all current scope variables.
                        // This is used for true lambdas (lambda do ... end, arrow syntax).
                        captured = self.environment().current_scope_var_refs();
                    } else {
                        // Explicit list of variables to capture
                        for name in names {
                            if let Some(value_ref) = self.environment().get_ref(name) {
                                captured.insert(name.clone(), value_ref);
                            }
                        }
                    }
                }
                // If captured_vars is None, don't capture anything (regular blocks for .each, etc.)
                let block = BlockStatement::new(parameters.clone(), body.clone(), captured);
                Ok(Object::Block(Rc::new(block)))
            }
            Expression::Grouped { expression, .. } => self.evaluate_expression(expression),

            // ── Operators ───────────────────────────────────────────────────
            Expression::UnaryOp {
                op,
                operand,
                position,
            } => {
                let value = self.evaluate_expression(operand)?;
                self.evaluate_unary_operation(op, value, *position)
            }
            Expression::BinaryOp {
                op,
                left,
                right,
                position,
            } => {
                // Short-circuit evaluation for logical operators and assignment
                match op {
                    BinaryOp::And => {
                        let left_value = self.evaluate_expression(left)?;
                        return if !is_truthy(&left_value) {
                            Ok(left_value)
                        } else {
                            self.evaluate_expression(right)
                        };
                    }
                    BinaryOp::Or => {
                        let left_value = self.evaluate_expression(left)?;
                        return if is_truthy(&left_value) {
                            Ok(left_value)
                        } else {
                            self.evaluate_expression(right)
                        };
                    }
                    BinaryOp::Assign => {
                        let value = self.evaluate_expression(right)?;
                        self.assign_value(left, value.clone())?;
                        return Ok(value);
                    }
                    _ => {}
                }
                let left_value = self.evaluate_expression(left)?;
                let right_value = self.evaluate_expression(right)?;
                // Check for user-defined operator methods on instances
                if let (Some(op_name), Object::Instance(instance_rc)) =
                    (binary_op_method_name(op), &left_value)
                {
                    let class = Rc::clone(&instance_rc.borrow().class);
                    if let Some(method) = class.find_method(op_name) {
                        return self.invoke_method(
                            class,
                            method,
                            left_value.clone(),
                            vec![right_value],
                            *position,
                        );
                    }
                }
                self.evaluate_binary_operation(op, left_value, right_value, *position)
            }

            // ── Collections ─────────────────────────────────────────────────
            Expression::Array { elements, .. } => self.evaluate_array_literal(elements),
            Expression::Dictionary { entries, .. } => self.evaluate_dictionary_literal(entries),
            Expression::Index {
                array,
                index,
                position,
            } => {
                let collection = self.evaluate_expression(array)?;
                let key = self.evaluate_expression(index)?;
                // Block/Lambda [] call syntax: proc[args]
                if let Object::Block(block) = &collection {
                    return block.call(self, vec![key], *position);
                }
                // Check for user-defined [] method on instances
                if let Object::Instance(instance_rc) = &collection {
                    let class = Rc::clone(&instance_rc.borrow().class);
                    if let Some(method) = class.find_method("[]") {
                        return self.invoke_method(
                            class,
                            method,
                            collection.clone(),
                            vec![key],
                            *position,
                        );
                    }
                }
                self.evaluate_index_operation(collection, key, *position)
            }

            // ── Calls ───────────────────────────────────────────────────────
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                trailing_block,
                position,
            } => self.evaluate_method_call(
                receiver,
                method,
                arguments,
                trailing_block.as_ref().map(|b| b.as_ref()),
                *position,
            ),
            Expression::Call {
                callee,
                arguments,
                trailing_block,
                position,
            } => self.eval_call(
                callee,
                arguments,
                trailing_block.as_ref().map(|b| b.as_ref()),
                *position,
            ),
            Expression::Super {
                arguments,
                position,
            } => self.eval_super(arguments, *position),
            Expression::Yield {
                arguments,
                position,
            } => self.eval_yield(arguments, *position),

            // ── Introspection / metaprogramming ─────────────────────────────
            Expression::Defined { expression, .. } => self.eval_defined(expression),

            // ── Other producers ─────────────────────────────────────────────
            Expression::Splat { expression, .. } => {
                // Outside of argument lists, splat evaluates to the array itself
                let value = self.evaluate_expression(expression)?;
                match value {
                    arr @ Object::Array(_) => Ok(arr),
                    other => Ok(Object::Array(Rc::new(RefCell::new(vec![other])))),
                }
            }
            Expression::Range {
                start,
                end,
                exclusive,
                ..
            } => {
                let start_value = self.evaluate_expression(start)?;
                let end_value = self.evaluate_expression(end)?;
                Ok(Object::Range {
                    start: Box::new(start_value),
                    end: Box::new(end_value),
                    exclusive: *exclusive,
                })
            }
            Expression::Case {
                expression,
                cases,
                else_case,
                position,
            } => self.evaluate_case_expression(expression, cases, else_case.as_deref(), *position),
            Expression::ScopeResolution {
                namespace,
                name,
                position,
            } => {
                let ns_value = self.evaluate_expression(namespace)?;
                match ns_value {
                    Object::Class(class_rc) | Object::Module(class_rc) => {
                        class_rc.get_class_var(name).ok_or_else(|| {
                            MetorexError::runtime_error(
                                format!("Uninitialized constant {}::{}", class_rc.name(), name),
                                position_to_location(*position),
                            )
                        })
                    }
                    _ => Err(MetorexError::runtime_error(
                        "'::' scope resolution requires a class or module as namespace".to_string(),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                ..
            } => self.evaluate_if_expression(condition, then_branch, elsif_branches, else_branch),
            Expression::Unless {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.evaluate_unless_expression(condition, then_branch, else_branch),
        }
    }
}

/// Map a binary operator to its operator method name for user-defined dispatch.
fn binary_op_method_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+"),
        BinaryOp::Subtract => Some("-"),
        BinaryOp::Multiply => Some("*"),
        BinaryOp::Divide => Some("/"),
        BinaryOp::Modulo => Some("%"),
        BinaryOp::Equal => Some("=="),
        BinaryOp::NotEqual => Some("!="),
        BinaryOp::Less => Some("<"),
        BinaryOp::Greater => Some(">"),
        BinaryOp::LessEqual => Some("<="),
        BinaryOp::GreaterEqual => Some(">="),
        BinaryOp::Spaceship => Some("<=>"),
        BinaryOp::Xor => Some("^"),
        _ => None,
    }
}
