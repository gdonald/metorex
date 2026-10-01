// `defined?` introspection: returns a description string or nil.

use std::rc::Rc;

use crate::ast::{BinaryOp, Expression, Statement, UnaryOp};
use crate::error::MetorexError;
use crate::object::Object;

use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// Evaluate `defined?(expr)`. Returns a frozen description string for
    /// defined expressions, or `nil` for undefined ones.
    pub(crate) fn eval_defined(&mut self, expression: &Expression) -> Result<Object, MetorexError> {
        Ok(match self.defined_description(expression)? {
            Some(description) => {
                let answer = Object::string(description.to_string());
                if let Object::String(text) = &answer {
                    text.freeze();
                }
                answer
            }
            None => Object::Nil,
        })
    }

    /// What `defined?` says about an expression. A method call and an
    /// operator evaluate their receiver to ask it about the method, and
    /// nothing else is evaluated.
    fn defined_description(
        &mut self,
        expression: &Expression,
    ) -> Result<Option<&'static str>, MetorexError> {
        let described = match expression {
            Expression::NilLiteral { .. } => Some("nil"),
            Expression::BoolLiteral { value: true, .. } => Some("true"),
            Expression::BoolLiteral { value: false, .. } => Some("false"),
            Expression::SelfExpr { .. } => Some("self"),
            Expression::Identifier { name, .. } if name == "self" => Some("self"),
            Expression::Identifier { name, .. } => self.defined_identifier(name),
            Expression::Grouped { expression, .. } => return self.defined_description(expression),
            // A literal Array or Hash is defined when everything written in
            // it is.
            Expression::Array { elements, .. } => {
                self.all_defined(elements.iter())?.then_some("expression")
            }
            Expression::Dictionary { entries, .. } => self
                .all_defined(entries.iter().flat_map(|(key, value)| [key, value]))?
                .then_some("expression"),
            Expression::InstanceVariable { name, .. } => self
                .instance_variable_is_set(name)
                .then_some("instance-variable"),
            Expression::ClassVariable { .. } => self
                .evaluate_expression(expression)
                .is_ok()
                .then_some("class variable"),
            Expression::GlobalVariable { name, .. } => self
                .defined_global(name, expression)?
                .then_some("global-variable"),
            Expression::TopLevelConstant { name, .. } => {
                let private = matches!(self.globals().get("Object"),
                    Some(Object::Class(object_class)) if object_class.is_private_constant(name));
                (!private
                    && (self.object_constant(name).is_some()
                        || self.globals().constant(name).is_some()))
                .then_some("constant")
            }
            Expression::ScopeResolution {
                namespace, name, ..
            } => self.defined_scoped_constant(namespace, name)?,
            // `a && b` and `a || b` are expressions whatever their operands
            // are, and neither is evaluated.
            Expression::BinaryOp {
                op: BinaryOp::And | BinaryOp::Or,
                ..
            } => Some("expression"),
            Expression::BinaryOp {
                op: BinaryOp::Assign,
                ..
            } => Some("assignment"),
            // Any other operator is a method called on the left operand.
            Expression::BinaryOp {
                op, left, right, ..
            } => {
                let name = crate::vm::eval::dispatch::binary_op_method_name(op).unwrap_or("==");
                self.defined_call_on(left, name, std::slice::from_ref(right.as_ref()))?
            }
            Expression::UnaryOp { op, operand, .. } => {
                let name = match op {
                    UnaryOp::Not => "!",
                    UnaryOp::Minus => "-@",
                    UnaryOp::Plus => "+@",
                };
                self.defined_call_on(operand, name, &[])?
            }
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                ..
            } => self.defined_method_call(expression, receiver, method, arguments)?,
            Expression::Call {
                callee, arguments, ..
            } => match callee.as_ref() {
                Expression::Identifier { name, .. } => {
                    if self.all_defined(arguments.iter())? && self.self_answers(name) {
                        Some("method")
                    } else {
                        None
                    }
                }
                _ => Some("expression"),
            },
            Expression::Index { array, index, .. } => {
                self.defined_call_on(array, "[]", std::slice::from_ref(index.as_ref()))?
            }
            Expression::Yield { .. } => self
                .environment()
                .get("__block__")
                .is_some()
                .then_some("yield"),
            Expression::Super { .. } => self.super_method_defined().then_some("super"),
            Expression::BeginRescue { body, .. } if body.len() == 1 => {
                Some(defined_statement(&body[0]))
            }
            // Everything else, a literal, a conditional, a loop, is an
            // expression, and none of it is evaluated.
            _ => Some("expression"),
        };
        Ok(described)
    }

    /// Whether every expression is defined, stopping at the first that is
    /// not.
    fn all_defined<'a>(
        &mut self,
        expressions: impl Iterator<Item = &'a Expression>,
    ) -> Result<bool, MetorexError> {
        for expression in expressions {
            let inner = match expression {
                Expression::Splat { expression, .. }
                | Expression::KeywordSplat { expression, .. }
                | Expression::BlockArg { expression, .. } => expression.as_ref(),
                other => other,
            };
            if self.defined_description(inner)?.is_none() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// A bare name: a local, a constant, or a method `self` answers,
    /// private ones included.
    fn defined_identifier(&mut self, name: &str) -> Option<&'static str> {
        // A class that does not descend from Object never reaches top-level
        // constants, however the program's scope binds them.
        if name.starts_with(char::is_uppercase) && !self.lexical_scope_reaches_top_level() {
            let in_method_nesting = self.method_nesting_stack.last().is_some_and(|nesting| {
                nesting
                    .iter()
                    .any(|scope| scope.get_class_var(name).is_some())
            });
            return (in_method_nesting
                || self
                    .def_scope_stack
                    .iter()
                    .any(|enclosing| enclosing.get_class_var(name).is_some()))
            .then_some("constant");
        }
        if let Some(value) = self.environment().get(name) {
            return Some(match value {
                Object::Class(_) | Object::Module(_) if name.starts_with(char::is_uppercase) => {
                    "constant"
                }
                Object::Method(_) | Object::NativeFunction(_) | Object::CompiledFunction(_) => {
                    "method"
                }
                _ => "local-variable",
            });
        }
        if name.starts_with(char::is_uppercase) {
            return (self.constant_before_object(name).is_some()
                || self.object_constant(name).is_some()
                || self.globals().constant(name).is_some()
                || self.lexically_autoloaded(name))
            .then_some("constant");
        }
        self.self_answers(name).then_some("method")
    }

    /// Whether an enclosing class or module registered an autoload for the
    /// name, which `defined?` reports without running it.
    fn lexically_autoloaded(&mut self, name: &str) -> bool {
        let scopes: Vec<_> = self.def_scope_stack.iter().rev().cloned().collect();
        scopes
            .iter()
            .any(|enclosing| self.effective_autoload(enclosing, name).is_some())
    }

    /// Whether `self` answers `name`, private methods included, as a call
    /// written without a receiver may reach them.
    fn self_answers(&mut self, name: &str) -> bool {
        let receiver = match self.environment().get("self") {
            Some(receiver) => receiver,
            None => match self.eval_self(crate::lexer::Position::new(0, 0, 0)) {
                Ok(receiver) => receiver,
                Err(_) => return self.globals().contains(name),
            },
        };
        if let Some((_, method)) = self.lookup_method(&receiver, name) {
            return !method.is_undefined;
        }
        self.responds_to(&receiver, name)
            || self.method_is_restricted(&receiver, name)
            || matches!(self.globals().get(name), Some(Object::NativeFunction(_)))
    }

    /// Whether an instance variable of `self` has been assigned.
    fn instance_variable_is_set(&self, name: &str) -> bool {
        match self.environment().get("self") {
            Some(Object::Instance(instance)) => instance.borrow().get_var(name).is_some(),
            Some(Object::Class(class) | Object::Module(class)) => {
                class.get_class_var(&format!("@{}", name)).is_some()
            }
            _ => false,
        }
    }

    /// A global is defined once assigned, nil included. `$!` and `$~` always
    /// are, and the ones the last match sets are when that match set them.
    fn defined_global(
        &mut self,
        name: &str,
        expression: &Expression,
    ) -> Result<bool, MetorexError> {
        if matches!(name, "!" | "~") {
            return Ok(true);
        }
        let read_from_last_match = matches!(name, "&" | "`" | "'" | "+")
            || (!name.is_empty() && name.bytes().all(|digit| digit.is_ascii_digit()));
        if read_from_last_match {
            return Ok(!matches!(
                self.evaluate_expression(expression),
                Ok(Object::Nil) | Err(_)
            ));
        }
        let name = self
            .global_aliases
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string());
        Ok(self.globals().get(&name).is_some())
    }

    /// `Ns::Name` is a constant when the namespace is defined, evaluates to
    /// a class or module, and holds the name publicly, without asking
    /// `const_missing` and without running an autoload.
    fn defined_scoped_constant(
        &mut self,
        namespace: &Expression,
        name: &str,
    ) -> Result<Option<&'static str>, MetorexError> {
        if self.defined_description(namespace)?.is_none() {
            return Ok(None);
        }
        let (Ok(Object::Class(scope)) | Ok(Object::Module(scope))) =
            self.evaluate_expression(namespace)
        else {
            return Ok(None);
        };
        let refused = match self.const_entry_on(&scope, name, true, false) {
            Some((owner, _)) => {
                owner.is_private_constant(name)
                    && !self
                        .def_scope_stack
                        .iter()
                        .any(|open| Rc::ptr_eq(open, &owner))
            }
            None => false,
        };
        // Top-level constants are Object's, held with the program's globals.
        let top_level = scope.name() == "Object"
            && (self.object_constant(name).is_some() || self.globals().constant(name).is_some());
        let held = top_level
            || scope.get_class_var(name).is_some()
            || self.effective_autoload(&scope, name).is_some()
            || matches!(
                self.const_entry_on(&scope, name, true, false),
                Some((_, Some(_)))
            );
        Ok((!refused && held).then_some("constant"))
    }

    /// A method call written with a receiver.
    fn defined_method_call(
        &mut self,
        call: &Expression,
        receiver: &Expression,
        method: &str,
        arguments: &[Expression],
    ) -> Result<Option<&'static str>, MetorexError> {
        // A dynamic Regexp, a string literal marked mutable or binary, and
        // `__ENCODING__` are written as calls the parser makes, and each is
        // an expression.
        if matches!(
            method,
            "__literal__" | "__mutable_literal__" | "__binary_literal__"
        ) || is_source_encoding(call)
        {
            return Ok(Some("expression"));
        }
        // `"text".freeze` is read as the one frozen string it stands for, and
        // it is still `freeze` that was written.
        let method = if method == "__frozen_literal__" {
            "freeze"
        } else {
            method
        };
        // `held&.name` asks about `name`.
        let (method, arguments) = match (method, arguments) {
            (crate::parser::SAFE_CALL, [Expression::Symbol { value, .. }, rest @ ..]) => {
                (value.as_str(), rest)
            }
            _ => (method, arguments),
        };
        // `self.name` may reach a private method, the way a bare call does.
        if matches!(receiver, Expression::SelfExpr { .. })
            || matches!(receiver, Expression::Identifier { name, .. } if name == "self")
        {
            return Ok(
                (self.all_defined(arguments.iter())? && self.self_answers(method))
                    .then_some("method"),
            );
        }
        self.defined_call_on(receiver, method, arguments)
    }

    /// A method called on an explicit receiver is defined when the receiver
    /// and arguments are, and the value the receiver evaluates to answers
    /// the method publicly. The receiver is evaluated, and an exception it
    /// raises makes the answer nil.
    fn defined_call_on(
        &mut self,
        receiver: &Expression,
        method: &str,
        arguments: &[Expression],
    ) -> Result<Option<&'static str>, MetorexError> {
        if self.defined_description(receiver)?.is_none() || !self.all_defined(arguments.iter())? {
            return Ok(None);
        }
        let value = match self.evaluate_expression(receiver) {
            Ok(value) => value,
            Err(
                MetorexError::UncaughtException { .. }
                | MetorexError::RuntimeError { .. }
                | MetorexError::TypeError { .. },
            ) => return Ok(None),
            Err(other) => return Err(other),
        };
        Ok(self.answers_publicly(&value, method).then_some("method"))
    }

    /// Whether `value` answers `method` to a caller outside it: a public
    /// method, a protected one when `self` is of the class holding it, or
    /// one `respond_to_missing?` says it answers.
    fn answers_publicly(&mut self, value: &Object, method: &str) -> bool {
        if let Some((owner, found)) = self.lookup_method(value, method) {
            if found.is_undefined || owner.is_method_private(method) {
                return false;
            }
            if owner.is_method_protected(method) {
                let caller = self.environment().get("self").unwrap_or(Object::Nil);
                return self.builtins().class_of(&caller).has_ancestor(&owner)
                    || matches!(&caller, Object::Instance(instance)
                        if instance.borrow().class.has_ancestor(&owner));
            }
            return true;
        }
        if self.responds_to(value, method) {
            return true;
        }
        let position = crate::lexer::Position::new(0, 0, 0);
        if self.lookup_method(value, "respond_to_missing?").is_some() {
            let asked = vec![Object::symbol(method.to_string()), Object::Bool(false)];
            return self
                .send_to_object(value.clone(), "respond_to_missing?", asked, position)
                .is_ok_and(|answer| answer.is_truthy());
        }
        self.send_to_object(
            value.clone(),
            "respond_to?",
            vec![Object::symbol(method.to_string())],
            position,
        )
        .is_ok_and(|answer| answer.is_truthy())
    }
}

/// What `defined?` says about an assignment or a jump written inside it,
/// which it reports on without carrying out. An element assignment is a
/// call to `[]=`, unless it is a compound one.
fn defined_statement(statement: &Statement) -> &'static str {
    match statement {
        Statement::Assignment { target, value, .. } => {
            let element = matches!(target, Expression::Index { .. })
                || matches!(target, Expression::MethodCall { method, .. } if method == "[]");
            let compound = match value {
                Expression::BinaryOp { left, .. } => left.as_ref() == target,
                Expression::MethodCall { receiver, .. } => receiver.as_ref() == target,
                _ => false,
            };
            if element && !compound {
                "method"
            } else {
                "assignment"
            }
        }
        Statement::MultipleAssignment { .. } => "assignment",
        _ => "expression",
    }
}

/// Whether a call is the one the parser writes for `__ENCODING__`, whose
/// receiver, argument, and call all stand at the one place the keyword does.
fn is_source_encoding(call: &Expression) -> bool {
    let Expression::MethodCall {
        receiver,
        method,
        arguments,
        position,
        ..
    } = call
    else {
        return false;
    };
    method == "find"
        && matches!(receiver.as_ref(),
            Expression::Identifier { name, position: at } if name == "Encoding" && at == position)
        && matches!(arguments.as_slice(),
            [Expression::StringLiteral { position: at, .. }] if at == position)
}
