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
            Expression::MagicDir { .. } => {
                let dir = self
                    .get_current_file()
                    .and_then(|p| p.parent().map(|d| d.display().to_string()))
                    .unwrap_or_else(|| ".".to_string());
                Ok(Object::String(Rc::new(dir)))
            }

            // ── Closures, grouping ──────────────────────────────────────────
            Expression::Lambda {
                parameters,
                parameter_defaults,
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
                // If captured_vars is None, capture all current scope variables
                // so blocks work correctly across method boundaries (which use
                // isolated scopes).
                if captured.is_empty() && captured_vars.is_none() {
                    captured = self.environment().current_scope_var_refs();
                }
                let block = BlockStatement::with_def_scope(
                    parameters.clone(),
                    parameter_defaults.clone(),
                    body.clone(),
                    captured,
                    self.def_scope_stack.clone(),
                );
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
                // Check for user-defined operator methods on instances. Walk
                // via lookup_method so per-instance singleton-class overrides
                // (used by mspec mocks, among other things) win over the
                // underlying class definition.
                if let (Some(op_name), Object::Instance(_)) =
                    (binary_op_method_name(op), &left_value)
                {
                    if let Some((class, method)) = self.lookup_method(&left_value, op_name)
                        && !method.is_undefined
                    {
                        return self.invoke_method(
                            class,
                            method,
                            left_value.clone(),
                            vec![right_value],
                            *position,
                        );
                    }
                    // Comparable-style fallback: if the class defines `<=>`,
                    // derive `<`, `<=`, `>`, `>=` from its result. (Ruby gets
                    // these from the Comparable mixin; metorex synthesises
                    // them.)
                    if matches!(
                        op,
                        BinaryOp::Less
                            | BinaryOp::LessEqual
                            | BinaryOp::Greater
                            | BinaryOp::GreaterEqual
                    ) && let Some((class, spaceship)) = self.lookup_method(&left_value, "<=>")
                    {
                        let cmp = self.invoke_method(
                            class,
                            spaceship,
                            left_value.clone(),
                            vec![right_value.clone()],
                            *position,
                        )?;
                        if let Object::Int(c) = cmp {
                            let result = match op {
                                BinaryOp::Less => c < 0,
                                BinaryOp::LessEqual => c <= 0,
                                BinaryOp::Greater => c > 0,
                                BinaryOp::GreaterEqual => c >= 0,
                                _ => unreachable!(),
                            };
                            return Ok(Object::Bool(result));
                        }
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
                // `native_fn [args]` — treat as a call with the bracketed array as a single argument.
                // This matches Ruby's `private [:foo, :bar]` which passes an Array to `private`.
                if let Object::NativeFunction(name) = &collection {
                    return self.call_native_function(&name.clone(), vec![key], *position);
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
                    // Thread instances support thread-local storage via
                    // `t[:k]`. Targeted dispatch (gated on the class name)
                    // so the call doesn't recurse through a generic
                    // native-method fallback.
                    if class.name() == "Thread" {
                        let key_str = match &key {
                            Object::Symbol(s) => Some((**s).clone()),
                            Object::String(s) => Some((**s).clone()),
                            _ => None,
                        };
                        if let Some(k) = key_str {
                            let locals = instance_rc.borrow().get_var("__thread_locals").cloned();
                            if let Some(Object::Dict(d)) = locals {
                                return Ok(d.borrow().get(&k).cloned().unwrap_or(Object::Nil));
                            }
                        }
                        return Ok(Object::Nil);
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
                forward_args,
                position,
            } => self.eval_super(arguments, *forward_args, *position),
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
            Expression::BlockArg { expression, .. } => {
                // Outside of an argument list, `&expr` is just `expr`.
                self.evaluate_expression(expression)
            }
            Expression::BeginRescue {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                ..
            } => self.evaluate_begin_value(
                body,
                rescue_clauses,
                else_clause.as_deref(),
                ensure_block.as_deref(),
            ),
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
                        // Own constants first, then (like Ruby's qualified
                        // lookup) the ancestor chain — but not top-level
                        // constants, which a qualified reference must not
                        // reach. Registered autoloads fire on their owner.
                        let entry = self.const_entry_on(&class_rc, name, true, false);
                        let value = match entry {
                            Some((_, Some(v))) => Some(v),
                            Some((owner, None)) => self.try_autoload_constant(&owner, name)?,
                            None => self.try_autoload_constant(&class_rc, name)?,
                        };
                        if let Some(v) = value {
                            // A constant marked private via
                            // `Module#private_constant` only resolves from
                            // inside the owning class's body. Anywhere else
                            // — including methods defined elsewhere that
                            // happen to be invoked — raises NameError.
                            if class_rc.is_private_constant(name) {
                                let inside = self
                                    .def_scope_stack
                                    .iter()
                                    .any(|s| Rc::ptr_eq(s, &class_rc));
                                if !inside {
                                    let msg = format!(
                                        "private constant {}::{} referenced",
                                        class_rc.name(),
                                        name
                                    );
                                    let exc = Object::exception("NameError", msg.clone());
                                    return Err(MetorexError::UncaughtException {
                                        exception: exc,
                                        location: position_to_location(*position),
                                        message: msg,
                                    });
                                }
                            }
                            self.warn_deprecated_constant(&class_rc, name, *position);
                            return Ok(v);
                        }
                        // Uninitialized constants dispatch const_missing —
                        // the default implementation raises NameError.
                        // Autoload's "loaded but didn't define" path lands
                        // here too.
                        self.dispatch_const_missing(&class_rc, name, *position)
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
            Expression::SingletonClass {
                target,
                body,
                position,
            } => self.evaluate_singleton_class_expression(target, body, *position),
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
        BinaryOp::Power => Some("**"),
        BinaryOp::Equal => Some("=="),
        BinaryOp::CaseEqual => Some("==="),
        BinaryOp::NotEqual => Some("!="),
        BinaryOp::Less => Some("<"),
        BinaryOp::Greater => Some(">"),
        BinaryOp::LessEqual => Some("<="),
        BinaryOp::GreaterEqual => Some(">="),
        BinaryOp::Spaceship => Some("<=>"),
        BinaryOp::BitwiseAnd => Some("&"),
        BinaryOp::BitwiseOr => Some("|"),
        BinaryOp::Xor => Some("^"),
        _ => None,
    }
}
