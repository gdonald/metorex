// Core statement execution logic for the Metorex VM.
// This module handles the dispatcher and basic statement execution.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::errors::*;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::class::Class;
use crate::error::MetorexError;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Evaluate a statement and produce control-flow information for the caller.
    pub(crate) fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
        // Every statement is a `:line` event for whatever is tracing, which
        // costs a check on an empty list when nothing is.
        if !self.tracepoints.is_empty() {
            self.fire_line_event(statement.position())?;
        }
        match statement {
            Statement::Expression {
                expression,
                position,
            } => {
                let result = self.evaluate_expression(expression)?;

                // Ruby-style auto-call: if expression statement evaluates to a Method
                // and the expression is a bare identifier, auto-call it with zero args
                if matches!(expression, Expression::Identifier { .. })
                    && matches!(result, Object::Method(_))
                {
                    let called = self.invoke_callable(result, vec![], *position)?;
                    return Ok(ControlFlow::Value(called));
                }

                // The value travels with the flow, so the last expression in
                // an `if` branch is what the `if` answers.
                Ok(ControlFlow::Value(result))
            }
            Statement::Assignment {
                target,
                value,
                position: _,
            } => {
                // Ruby's parser introduces the local where the assignment is
                // written, ahead of the value, so a block on the right-hand
                // side closes over the name being assigned. That is what
                // lets `walk = -> { walk.call }` call itself.
                if let crate::ast::Expression::Identifier { name, .. } = target {
                    self.environment_mut().unhoist(name);
                }
                let evaluated = match self.conditional_assignment_to_new_constant(target, value) {
                    // `CONST ||= value` where CONST is not defined yet: Ruby
                    // reads the undefined constant as nil rather than raising,
                    // so only the right-hand side is evaluated.
                    Some(right) => self.evaluate_expression(right)?,
                    None => self.evaluate_expression(value)?,
                };
                self.assign_value(target, evaluated.clone())?;
                // An assignment answers the value it assigned, so a block or
                // an `if` branch ending in one has that as its value.
                Ok(ControlFlow::Value(evaluated))
            }
            Statement::MultipleAssignment {
                targets,
                values,
                position: _,
            } => {
                // One value on the right is spread across the targets when it
                // is an Array, and otherwise reaches the first target alone.
                let (answer, source): (Object, Vec<Object>) = if values.len() == 1 {
                    let single = self.evaluate_expression(&values[0])?;
                    let spread = match &single {
                        Object::Array(elements) => elements.borrow().clone(),
                        held => vec![held.clone()],
                    };
                    (single, spread)
                } else {
                    let each = values
                        .iter()
                        .map(|value| self.evaluate_expression(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    (Object::array(each.clone()), each)
                };
                self.spread_into_targets(targets, &source)?;
                // The assignment answers the right-hand side as it was
                // written, which is what `(a, b = 1, 2)` reads back as.
                Ok(ControlFlow::Value(answer))
            }
            Statement::Return { value, position } => {
                let result = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Return {
                    value: result,
                    position: *position,
                })
            }
            Statement::Break { value, position } => {
                let resolved = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Break {
                    value: resolved,
                    position: *position,
                })
            }
            Statement::Continue { value, position } => {
                let resolved = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Continue {
                    value: resolved,
                    position: *position,
                })
            }
            // `retry` inside a rescue body runs the begin body again, which
            // the begin handler does when it sees this.
            Statement::Retry { position } => Ok(ControlFlow::Retry {
                position: *position,
            }),
            Statement::Redo { position } => Ok(ControlFlow::Redo {
                position: *position,
            }),
            Statement::Block {
                statements,
                position: _,
            } => self.execute_block(statements),
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position: _,
            } => self.execute_if(condition, then_branch, elsif_branches, else_branch),
            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                position: _,
            } => self.execute_unless(condition, then_branch, else_branch),
            Statement::While {
                condition,
                body,
                position: _,
            } => self.execute_while(condition, body),
            Statement::DoWhile {
                condition,
                body,
                position: _,
            } => self.execute_do_while(condition, body),
            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => self.execute_for(variable, iterable, body, *position),
            Statement::DeclareLocals { names, .. } => {
                for name in names {
                    if self.environment().get(name).is_none() {
                        self.environment_mut().define(name.clone(), Object::Nil);
                    } else {
                        // A `for` loop names what its body binds so the scope
                        // holding the loop still reads it afterwards, which
                        // means the block the loop runs closes over the name.
                        self.environment_mut().unhoist(name);
                    }
                }
                Ok(ControlFlow::Next)
            }
            Statement::ClassDef {
                name,
                namespace,
                superclass,
                superclass_expression,
                body,
                position,
            } => self.execute_class_def(
                name,
                namespace.as_deref(),
                superclass.as_deref(),
                superclass_expression.as_deref(),
                body,
                *position,
            ),
            // A `def` nested inside a block or a `begin` within a class body
            // reaches here rather than the class-body walk, so it installs on
            // the innermost lexical class the same way that walk would.
            Statement::MethodDef {
                name,
                parameters,
                body,
                is_class_method,
                position,
            } => {
                let Some(target) = self.def_scope_stack.last().cloned() else {
                    return Err(unimplemented_statement_error(statement));
                };
                let _ = (parameters, body, is_class_method);
                self.apply_class_body_statements(
                    &target,
                    std::slice::from_ref(statement),
                    *position,
                )?;
                Ok(ControlFlow::Value(Object::symbol(name.clone())))
            }
            Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                position,
            } => self.execute_begin(body, rescue_clauses, else_clause, ensure_block, *position),
            Statement::Raise {
                exception,
                position,
            } => self.execute_raise(exception, *position),
            Statement::Match {
                expression,
                cases,
                position,
            } => self.execute_match(expression, cases, *position),
            Statement::CaseIn {
                expression,
                cases,
                position,
            } => self.execute_case_in(expression, cases, *position),
            Statement::FunctionDef {
                name,
                parameters,
                body,
                position,
                singleton_class,
            } => self.execute_function_def(
                name,
                parameters,
                body,
                *position,
                singleton_class.as_deref(),
            ),
            Statement::AttrReader { position, .. }
            | Statement::AttrWriter { position, .. }
            | Statement::AttrAccessor { position, .. } => {
                // These are only processed during class definition, not as standalone statements
                Err(MetorexError::runtime_error(
                    "attr_reader, attr_writer, and attr_accessor can only be used inside a class definition",
                    position_to_location(*position),
                ))
            }
            Statement::ModuleDef {
                name,
                namespace,
                body,
                position,
            } => self.execute_module_def(name, namespace.as_deref(), body, *position),
            Statement::Include {
                module_name,
                position,
            } => self.execute_include(module_name, *position),
            Statement::Extend {
                module_name,
                position,
            } => self.execute_extend(module_name, *position),
            Statement::Alias {
                new_name,
                old_name,
                position,
            } => self.execute_alias(new_name, old_name, *position),
        }
    }

    /// Execute statements within a new lexical scope.
    /// Ruby reports a constant given a second value, naming the class or
    /// module the constant is bound on.
    fn warn_already_initialized(
        &mut self,
        owner: &Rc<crate::class::Class>,
        name: &str,
        position: crate::lexer::Position,
    ) {
        let named = owner.ruby_name();
        let named = if named.is_empty() {
            owner.inspect_name()
        } else {
            named
        };
        // A constant at the top level is bound on Object, which Ruby leaves
        // out of the name it reports.
        let message = if named == "Object" {
            format!("warning: already initialized constant {}", name)
        } else {
            format!("warning: already initialized constant {}::{}", named, name)
        };
        self.emit_warning_to_stderr(&message, position);
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        self.environment_mut().push_scope();
        let result = self.execute_statements_internal(statements);
        self.environment_mut().pop_scope();
        result
    }

    /// Core statement execution loop used by program and block execution.
    pub(crate) fn execute_statements_internal(
        &mut self,
        statements: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        let mut last = ControlFlow::Next;
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Next => last = ControlFlow::Next,
                // A statement that produced a value does not end the run; the
                // last one to produce one is what the group answers.
                ControlFlow::Value(value) => last = ControlFlow::Value(value),
                flow => return Ok(flow),
            }
        }
        Ok(last)
    }

    /// Evaluate the value of a constant assignment inside a class or module
    /// body, honoring the `CONST ||= value` form for a constant that has no
    /// value yet.
    pub(crate) fn evaluate_constant_assignment(
        &mut self,
        statement: &Statement,
        value: &Expression,
    ) -> Result<Object, MetorexError> {
        let Statement::Assignment { target, .. } = statement else {
            return self.evaluate_expression(value);
        };
        match self.conditional_assignment_to_new_constant(target, value) {
            Some(right) => self.evaluate_expression(right),
            None => self.evaluate_expression(value),
        }
    }

    /// For `CONST ||= value` where `CONST` is not defined, the right-hand
    /// side of the desugared `CONST = CONST || value`. `None` for every other
    /// assignment, including one to a constant that already has a value.
    fn conditional_assignment_to_new_constant<'a>(
        &mut self,
        target: &Expression,
        value: &'a Expression,
    ) -> Option<&'a Expression> {
        let Expression::BinaryOp {
            op: crate::ast::BinaryOp::Or,
            left,
            right,
            ..
        } = value
        else {
            return None;
        };
        if !constant_target_matches(target, left) {
            return None;
        }
        match self.eval_defined(target) {
            Ok(Object::Nil) => Some(right),
            _ => None,
        }
    }

    /// Hand each value to the target standing in the same place, with the
    /// splat, where there is one, taking everything the others leave.
    pub(crate) fn spread_into_targets(
        &mut self,
        targets: &[Expression],
        source: &[Object],
    ) -> Result<(), MetorexError> {
        let splat_at = targets
            .iter()
            .position(|target| matches!(target, Expression::Splat { .. }));
        match splat_at {
            None => {
                for (index, target) in targets.iter().enumerate() {
                    let value = source.get(index).cloned().unwrap_or(Object::Nil);
                    self.assign_value(target, value)?;
                }
            }
            Some(splat_at) => {
                for (index, target) in targets[..splat_at].iter().enumerate() {
                    let value = source.get(index).cloned().unwrap_or(Object::Nil);
                    self.assign_value(target, value)?;
                }
                // The splat takes what the targets on either side of it do
                // not, which is none at all when there are fewer values than
                // named targets.
                let after = targets.len() - splat_at - 1;
                let taken = source.len().saturating_sub(splat_at + after);
                let collected: Vec<Object> =
                    source.iter().skip(splat_at).take(taken).cloned().collect();
                if let Expression::Splat { expression, .. } = &targets[splat_at] {
                    self.assign_value(expression, Object::array(collected))?;
                }
                for (offset, target) in targets[splat_at + 1..].iter().enumerate() {
                    let value = source
                        .get(splat_at + taken + offset)
                        .cloned()
                        .unwrap_or(Object::Nil);
                    self.assign_value(target, value)?;
                }
            }
        }
        Ok(())
    }

    /// Assign a value to the given target expression.
    pub(crate) fn assign_value(
        &mut self,
        target: &Expression,
        value: Object,
    ) -> Result<(), MetorexError> {
        match target {
            // `(a, b), c = pair, held` names a group of targets, which takes
            // the value apart the way the whole list does.
            Expression::Array { elements, .. } => {
                let source: Vec<Object> = match &value {
                    Object::Array(items) => items.borrow().clone(),
                    held => vec![held.clone()],
                };
                self.spread_into_targets(elements, &source)
            }
            Expression::Identifier { name, position } => {
                // Constant-shaped identifiers (`MyClass = ...`) auto-name
                // anonymous classes/modules on first assignment, matching
                // Ruby's `klass.name` behavior after binding to a constant.
                let is_const = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                // A constant bound inside a class or module body names what
                // it holds under that namespace, so `M::Foo` reports its own
                // path rather than the bare constant name.
                if is_const {
                    match self.def_scope_stack.last().cloned() {
                        Some(enclosing) => name_constant_value(&enclosing, name, &value),
                        None => {
                            if let Object::Class(v) | Object::Module(v) = &value {
                                v.assign_name_recursive(name);
                            }
                        }
                    }
                }
                // Constants assigned in a context that has a lexically
                // enclosing class/module land on that scope's class_var
                // table — matching Ruby's behavior, where `Foo = 1` inside
                // a `module M` (or a lambda defined inside one) resolves
                // later as `M::Foo`. At the top level (no enclosing
                // scope), uppercase assignments go to globals (and
                // Object's class_var) so they survive the require'd
                // file's local environment ending. Both record the
                // assignment site for `Module#const_source_location`.
                let assign_file = self
                    .reported_current_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                if is_const && let Some(enclosing) = self.def_scope_stack.last().cloned() {
                    let is_new = enclosing.get_class_var(name).is_none();
                    if !is_new {
                        self.warn_already_initialized(&enclosing, name, *position);
                    }
                    enclosing.set_class_var(name, value);
                    enclosing.set_const_location(name, assign_file, position.line as i64);
                    if is_new {
                        let owner = Object::Class(Rc::clone(&enclosing));
                        self.trigger_const_added_hook(owner, name, *position)?;
                    }
                    return Ok(());
                }
                if is_const {
                    self.globals_mut().set(name.clone(), value.clone());
                    let mut owner = None;
                    if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                        if object_class.get_class_var(name).is_none() {
                            owner = Some(Object::Class(Rc::clone(&object_class)));
                        } else {
                            self.warn_already_initialized(&object_class, name, *position);
                        }
                        object_class.set_class_var(name, value);
                        object_class.set_const_location(name, assign_file, position.line as i64);
                    }
                    if let Some(owner) = owner {
                        self.trigger_const_added_hook(owner, name, *position)?;
                    }
                    return Ok(());
                }
                if !self.environment_mut().set(name, value.clone()) {
                    self.environment_mut().define(name.clone(), value);
                }
                Ok(())
            }
            // `::Name = value` binds at the top level whatever class or
            // module body the assignment sits in.
            Expression::TopLevelConstant { name, position } => {
                // Naming the module names the anonymous ones it holds, which
                // is how `A::B` reports its path once `A` has a name.
                if let Object::Class(bound) | Object::Module(bound) = &value {
                    bound.assign_name_recursive(name);
                }
                let assign_file = self
                    .reported_current_file()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default();
                self.globals_mut().set(name.clone(), value.clone());
                if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                    object_class.set_class_var(name, value);
                    object_class.set_const_location(name, assign_file, position.line as i64);
                }
                Ok(())
            }
            Expression::InstanceVariable { name, position } => {
                // Instance variables can only be set within a method (where 'self' is defined)
                match self.environment().get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let is_frozen = instance_rc.borrow().frozen;
                        if is_frozen {
                            let receiver = Object::Instance(Rc::clone(&instance_rc));
                            return Err(self.frozen_modification_error(&receiver, *position));
                        }
                        let mut instance = instance_rc.borrow_mut();
                        instance.set_var(name.clone(), value);
                        Ok(())
                    }
                    // An exception carries its own instance variables, so a
                    // user-defined subclass can set state on itself.
                    Some(Object::Exception(details)) => {
                        details
                            .borrow_mut()
                            .instance_vars
                            .insert(name.clone(), value);
                        Ok(())
                    }
                    Some(Object::Class(class)) => {
                        // Module/class-level instance variables stored as class vars
                        class.set_class_var(format!("@{}", name), value);
                        Ok(())
                    }
                    Some(Object::Module(module)) => {
                        module.set_class_var(format!("@{}", name), value);
                        Ok(())
                    }
                    // Immediates (Bool/Int/Float/Symbol/Nil/etc.) and other
                    // non-instance selves are always frozen; assigning an ivar
                    // raises FrozenError to match Ruby.
                    Some(other) => {
                        // The Ruby class name, not the internal tag: Ruby
                        // reports `NilClass`, never `Nil`.
                        let class_name =
                            crate::vm::native_methods::define_method::ruby_class_name(&other);
                        let msg = format!("can't modify frozen {}: {}", class_name, other);
                        let exc = Object::exception("FrozenError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(*position),
                            message: msg,
                        })
                    }
                    None => Err(MetorexError::runtime_error(
                        format!(
                            "Instance variable @{} can only be used within a method",
                            name
                        ),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::ClassVariable { name, position } => {
                // A class variable belongs to the class or module the code
                // was written in, and a write reaches the one furthest up the
                // chain that already holds it.
                if let Some(home) = self.class_variable_home() {
                    Self::class_var_owner(&home, name).set_class_var(name.clone(), value);
                    return Ok(());
                }
                Err(MetorexError::runtime_error(
                    "class variable access from toplevel".to_string(),
                    position_to_location(*position),
                ))
            }
            Expression::Index {
                array,
                index,
                position,
            } => {
                // Evaluate the array/object and index
                let obj = self.evaluate_expression(array)?;
                // `held[*subscripts] = value` spreads the values across the
                // subscript, the same way `held.[]=(*subscripts, value)` does.
                if let Expression::Splat { .. } = index.as_ref() {
                    let mut spread =
                        self.evaluate_arguments(std::slice::from_ref(index.as_ref()))?;
                    spread.push(value);
                    self.send_to_object(obj, "[]=", spread, *position)?;
                    return Ok(());
                }
                let idx = self.evaluate_expression(index)?;

                match obj {
                    // Every array index assignment goes through `[]=`, which
                    // knows the index, start-and-length, and Range forms.
                    Object::Array(_) => {
                        self.send_to_object(obj.clone(), "[]=", vec![idx, value], *position)?;
                        Ok(())
                    }
                    Object::Dict(dict_rc) => {
                        // The environment names and values its variables in
                        // text, so a write to it is read that way first.
                        if self.dict_is_environment(&dict_rc) {
                            let environment = Object::Dict(Rc::clone(&dict_rc));
                            let given = vec![idx, value];
                            self.call_hash_method(&environment, "[]=", &given, *position)?;
                            return Ok(());
                        }
                        // Hash/Dict index assignment. Ruby allows any object
                        // as a key, and a key carrying its own `#hash` takes
                        // the slot that number and `#eql?` place it in.
                        self.hash_store(&dict_rc, &idx, value.clone(), *position)?;
                        self.record_environment_change(&dict_rc, &idx, &value);
                        Ok(())
                    }
                    Object::Instance(instance_rc) => {
                        // An instance of an Array or Hash subclass writes
                        // through the storage it is backed by.
                        let holder = Object::Instance(Rc::clone(&instance_rc));
                        if let Some(backing) =
                            crate::vm::native_methods::array_subclass_value(&holder)
                                .or_else(|| crate::vm::native_methods::hash_subclass_value(&holder))
                        {
                            self.send_to_object(backing, "[]=", vec![idx, value], *position)?;
                            return Ok(());
                        }
                        // Dispatch to user-defined []= method
                        let class = Rc::clone(&instance_rc.borrow().class);
                        if let Some(method) = class.find_method("[]=") {
                            self.invoke_method(
                                class,
                                method,
                                Object::Instance(instance_rc),
                                vec![idx, value],
                                *position,
                            )?;
                            Ok(())
                        } else if let Some(members) =
                            crate::vm::native_methods::struct_members(&class)
                        {
                            self.call_struct_instance_method(
                                &class,
                                &members,
                                &Object::Instance(instance_rc),
                                "[]=",
                                &[idx, value],
                                *position,
                            )?;
                            Ok(())
                        } else if class.name() == "Thread" {
                            // A Thread keeps its locals under `t[:k] = v`,
                            // which the Thread method table writes. Going
                            // through it keeps one reading of the key for the
                            // subscript and the call.
                            self.call_thread_method(
                                &Object::Instance(Rc::clone(&instance_rc)),
                                "[]=",
                                &[idx, value],
                                *position,
                            )?;
                            Ok(())
                        } else {
                            Err(MetorexError::runtime_error(
                                "No []= method defined on instance",
                                position_to_location(*position),
                            ))
                        }
                    }
                    Object::Class(cls) | Object::Module(cls) => {
                        // Dispatch to class/module-level `def self.[]=` (stored
                        // under the `__class__` prefix).
                        let receiver = Object::Class(Rc::clone(&cls));
                        let key = "__class__[]=".to_string();
                        if let Some(method) = cls.find_method(&key) {
                            self.invoke_method(
                                Rc::clone(&cls),
                                method,
                                receiver,
                                vec![idx, value],
                                *position,
                            )?;
                            Ok(())
                        } else if self
                            .call_warning_methods(
                                &cls,
                                "[]=",
                                &[idx.clone(), value.clone()],
                                *position,
                            )?
                            .is_some()
                            || self
                                .call_class_methods(&cls, "[]=", &[idx, value], *position)?
                                .is_some()
                        {
                            Ok(())
                        } else {
                            Err(MetorexError::runtime_error(
                                "No []= method defined on class/module",
                                position_to_location(*position),
                            ))
                        }
                    }
                    // `str[at] = text` writes into the string itself, which
                    // every reference to it then reads.
                    Object::String(_) => {
                        let given = vec![idx, value];
                        self.call_string_mutation(&obj, "[]=", &given, *position)?;
                        Ok(())
                    }
                    // Nil receivers swallow `[]=` as a no-op so spec
                    // fixtures using stub Thread.current (which is Nil)
                    // can run `Thread.current[:in_autoload_rb] = true`
                    // without crashing.
                    Object::Nil => Ok(()),
                    _ => Err(MetorexError::runtime_error(
                        "Cannot index assign on this type",
                        position_to_location(*position),
                    )),
                }
            }
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                position,
                ..
            } => {
                // `values[start, length] = other` parses as a call to `[]`,
                // and assigning to it sends `[]=` with the same subscripts.
                if method == "[]" {
                    let receiver_obj = self.evaluate_expression(receiver)?;
                    let mut subscripts = Vec::with_capacity(arguments.len() + 1);
                    for argument in arguments {
                        subscripts.push(self.evaluate_expression(argument)?);
                    }
                    subscripts.push(value);
                    self.send_to_object(receiver_obj, "[]=", subscripts, *position)?;
                    return Ok(());
                }
                // `held&.name = value` writes only when the receiver is
                // there, which is what the safe call stands for.
                if method == crate::parser::SAFE_CALL
                    && let [Expression::Symbol { value: named, .. }] = arguments.as_slice()
                {
                    let receiver_obj = self.evaluate_expression(receiver)?;
                    if matches!(receiver_obj, Object::Nil) {
                        return Ok(());
                    }
                    self.send_to_object(
                        receiver_obj,
                        &format!("{}=", named),
                        vec![value],
                        *position,
                    )?;
                    return Ok(());
                }
                // Handle setter method calls (e.g., obj.name = value becomes obj.name=(value))
                if arguments.is_empty() {
                    // This is a setter method call: obj.method = value -> obj.method=(value)
                    let setter_method = format!("{}=", method);
                    let receiver_obj = self.evaluate_expression(receiver)?;

                    // An instance of an Array or Hash subclass answers the
                    // writers of the value it is backed by.
                    if let Some(backing) = crate::vm::native_methods::array_subclass_value(
                        &receiver_obj,
                    )
                    .or_else(|| crate::vm::native_methods::hash_subclass_value(&receiver_obj))
                    {
                        let class = self.builtins().class_of(&backing);
                        if self
                            .call_native_method(
                                &class,
                                &backing,
                                &setter_method,
                                std::slice::from_ref(&value),
                                *position,
                            )?
                            .is_some()
                        {
                            return Ok(());
                        }
                    }
                    // Look up the setter method and invoke it
                    match receiver_obj {
                        Object::Instance(instance_rc) => {
                            let (class, method_obj) = {
                                let instance = instance_rc.borrow();
                                let class = instance.class.clone();
                                // A writer written on this one object with
                                // `def obj.name=` answers before the one its
                                // class carries, whether it was recorded on
                                // the instance or on its singleton class.
                                let method_obj = instance
                                    .find_singleton_method(&setter_method)
                                    .or_else(|| {
                                        instance.singleton_class.borrow().as_ref().and_then(
                                            |singleton| singleton.find_method(&setter_method),
                                        )
                                    })
                                    .or_else(|| instance.class.find_method(&setter_method));
                                (class, method_obj)
                            }; // Borrow is dropped here

                            if let Some(method) = method_obj {
                                // Visibility check: an explicit-receiver setter
                                // call (`obj.foo = …`) can only invoke public
                                // methods, mirroring `evaluate_method_call`.
                                let is_explicit_receiver =
                                    !crate::vm::method_lookup::names_self(receiver.as_ref());
                                if is_explicit_receiver
                                    && self.refuses_explicit_receiver(&class, &setter_method)
                                {
                                    let marking = if class.is_method_protected(&setter_method)
                                        && !class.is_method_private(&setter_method)
                                    {
                                        "protected"
                                    } else {
                                        "private"
                                    };
                                    let msg = format!(
                                        "{} method '{}' called for an instance of {}",
                                        marking,
                                        setter_method,
                                        class.name()
                                    );
                                    let exc = Object::exception("NoMethodError", msg.clone());
                                    return Err(MetorexError::UncaughtException {
                                        exception: exc,
                                        location: position_to_location(*position),
                                        message: msg,
                                    });
                                }
                                self.invoke_method(
                                    class,
                                    method,
                                    Object::Instance(Rc::clone(&instance_rc)),
                                    vec![value],
                                    *position,
                                )?;
                                Ok(())
                            } else if instance_rc
                                .borrow()
                                .class
                                .find_method("method_missing")
                                .is_some()
                            {
                                // A class that answers what it was not asked
                                // for decides what a setter with no method
                                // behind it means.
                                let name = Object::symbol(setter_method.clone());
                                self.send_to_object(
                                    Object::Instance(Rc::clone(&instance_rc)),
                                    "method_missing",
                                    vec![name, value],
                                    *position,
                                )?;
                                Ok(())
                            } else {
                                // A setter the instance answers natively, such
                                // as an open file handle's `lineno=`, lives in
                                // the native table rather than the method map.
                                let receiver = Object::Instance(Rc::clone(&instance_rc));
                                let class = self.builtins().class_of(&receiver);
                                if self
                                    .call_native_method(
                                        &class,
                                        &receiver,
                                        &setter_method,
                                        std::slice::from_ref(&value),
                                        *position,
                                    )?
                                    .is_some()
                                {
                                    return Ok(());
                                }
                                Err(MetorexError::runtime_error(
                                    format!("Undefined setter method '{}'", setter_method),
                                    position_to_location(*position),
                                ))
                            }
                        }
                        Object::Class(class_rc) => {
                            // `def self.name=` is stored the way every other
                            // module-level method is, and one written in a
                            // `class << self` body sits on the singleton
                            // class, so both are looked in before a plain
                            // instance method.
                            if let Some(method) = crate::vm::method_lookup::module_level_method(
                                &class_rc,
                                &setter_method,
                            )
                            .or_else(|| {
                                class_rc
                                    .singleton_class_slot()
                                    .clone()
                                    .and_then(|singleton| singleton.find_method(&setter_method))
                            })
                            .or_else(|| class_rc.find_method(&setter_method))
                            {
                                self.invoke_method(
                                    Rc::clone(&class_rc),
                                    method,
                                    Object::Class(class_rc),
                                    vec![value],
                                    *position,
                                )?;
                                Ok(())
                            } else {
                                // A writer the class carries natively, such as
                                // `Encoding.default_internal=`, answers before
                                // the assignment falls back to storing state.
                                if self
                                    .call_class_methods(
                                        &class_rc,
                                        &setter_method,
                                        std::slice::from_ref(&value),
                                        *position,
                                    )?
                                    .is_some()
                                {
                                    return Ok(());
                                }
                                // Store as class variable
                                class_rc.set_class_var(
                                    format!("@{}", setter_method.trim_end_matches('=')),
                                    value,
                                );
                                Ok(())
                            }
                        }
                        Object::Module(module_rc) => {
                            // A writer written in a `class << self` body sits
                            // on the singleton class, which is a third place
                            // an assignment has to look.
                            if let Some(method) = crate::vm::method_lookup::module_level_method(
                                &module_rc,
                                &setter_method,
                            )
                            .or_else(|| {
                                module_rc
                                    .singleton_class_slot()
                                    .clone()
                                    .and_then(|singleton| singleton.find_method(&setter_method))
                            })
                            .or_else(|| module_rc.find_method(&setter_method))
                            {
                                self.invoke_method(
                                    Rc::clone(&module_rc),
                                    method,
                                    Object::Module(module_rc),
                                    vec![value],
                                    *position,
                                )?;
                                Ok(())
                            } else {
                                // A setter the module answers natively, such
                                // as `Process.maxgroups=`, lives in the native
                                // table rather than in the method map.
                                let receiver = Object::Module(Rc::clone(&module_rc));
                                if self
                                    .call_module_methods(
                                        &module_rc,
                                        &receiver,
                                        &setter_method,
                                        std::slice::from_ref(&value),
                                        *position,
                                    )?
                                    .is_some()
                                {
                                    return Ok(());
                                }
                                module_rc.set_class_var(
                                    format!("@{}", setter_method.trim_end_matches('=')),
                                    value,
                                );
                                Ok(())
                            }
                        }
                        // For other receiver types (immediates: Bool/Int/Float/
                        // Symbol/etc., or String), the user's reopened class
                        // (`class TrueClass; …; end`) lives in globals, not in
                        // `builtins.class_of`. Try the global class first so
                        // user-defined accessors fire; fall back to the built-in
                        // class otherwise.
                        other => {
                            let global_class_name = match &other {
                                Object::Bool(true) => Some("TrueClass"),
                                Object::Bool(false) => Some("FalseClass"),
                                Object::Nil => Some("NilClass"),
                                Object::Int(_) => Some("Integer"),
                                Object::Float(_) => Some("Float"),
                                Object::Symbol(_) => Some("Symbol"),
                                _ => None,
                            };
                            let target_class: Option<Rc<crate::class::Class>> = global_class_name
                                .and_then(|name| match self.globals().get(name) {
                                    Some(Object::Class(c)) => Some(c),
                                    _ => None,
                                })
                                .or_else(|| Some(self.builtins().class_of(&other)));

                            if let Some(class) = target_class
                                && let Some(method) = class.find_method(&setter_method)
                            {
                                self.invoke_method(
                                    Rc::clone(&class),
                                    method,
                                    other,
                                    vec![value],
                                    *position,
                                )?;
                                Ok(())
                            } else if let Some((owner, method)) =
                                self.lookup_method(&other, &setter_method)
                            {
                                // An exception carries its own class, which is
                                // where a reopened `attr_accessor` lives.
                                self.invoke_method(owner, method, other, vec![value], *position)?;
                                Ok(())
                            } else {
                                // A setter the class implements natively, such
                                // as the stream methods the String standing in
                                // for STDOUT answers, is reached last.
                                let class = self.builtins().class_of(&other);
                                let handled = self.call_native_method(
                                    &class,
                                    &other,
                                    &setter_method,
                                    std::slice::from_ref(&value),
                                    *position,
                                )?;
                                match handled {
                                    Some(_) => Ok(()),
                                    None => Err(MetorexError::runtime_error(
                                        format!(
                                            "Cannot call setter method '{}' on {}",
                                            setter_method,
                                            other.type_name()
                                        ),
                                        position_to_location(*position),
                                    )),
                                }
                            }
                        }
                    }
                } else {
                    Err(MetorexError::runtime_error(
                        "Cannot assign to method call with arguments",
                        position_to_location(*position),
                    ))
                }
            }
            Expression::GlobalVariable { name, .. } => {
                // A global given a second name writes through to the first.
                let name = self
                    .global_aliases
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone());
                // Ruby has retired the separators a split and a join read
                // when they are given none of their own, and says so once the
                // deprecated category is asked for.
                if matches!(name.as_str(), ";" | ",")
                    && !matches!(value, Object::Nil)
                    && self.warning_category_enabled("deprecated")
                {
                    let message = format!("warning: ${} is deprecated\n", name);
                    self.warn_through_warning_module(
                        message,
                        crate::lexer::Position::new(0, 0, 0),
                    )?;
                }
                self.globals_mut().set_variable(name.clone(), value.clone());
                // A `trace_var` hook on this global runs with the new value.
                let name = name.clone();
                self.fire_global_trace(&name, &value, crate::lexer::Position::new(0, 0, 0))?;
                Ok(())
            }
            // `Ns::Name = value` — assign to a constant on a module/class.
            // If value is an anonymous class/module, also install the Ruby
            // name (Ns::Name) on first assignment.
            Expression::ScopeResolution {
                namespace,
                name,
                position,
            } => {
                let ns = self.evaluate_expression(namespace)?;
                let owner_for_hook = ns.clone();
                match ns {
                    Object::Class(c) | Object::Module(c) => {
                        // MRI emits "already initialized constant Mod::X"
                        // when assigning to a constant that's already
                        // bound — covers both class_var hits and
                        // outstanding autoload registrations. The warning
                        // is unconditional (not gated on $VERBOSE) and
                        // routes through `$stderr` so the mspec
                        // `complain` matcher can capture it.
                        if self.autoload_const_access_depth == 0
                            && (c.get_class_var(name).is_some() || c.get_autoload(name).is_some())
                        {
                            let owner = c.ruby_name();
                            let owner = if owner.is_empty() {
                                c.inspect_name()
                            } else {
                                owner
                            };
                            let msg = format!(
                                "warning: already initialized constant {}::{}",
                                owner, name
                            );
                            self.emit_warning_to_stderr(&msg, *position);
                        }
                        name_constant_value(&c, name, &value);
                        // Explicit `Mod::X = value` outside of an autoload
                        // load supersedes any pending autoload. (The
                        // autoload trigger path manages its own autoload
                        // bookkeeping after the load completes.)
                        c.remove_autoload(name);
                        // Object's constants are top-level constants —
                        // publish so bare references resolve.
                        if c.name() == "Object" {
                            self.globals_mut().set(name.clone(), value.clone());
                        }
                        c.set_class_var(name, value);
                        let assign_file = self
                            .reported_current_file()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default();
                        c.set_const_location(name, assign_file, position.line as i64);
                        self.trigger_const_added_hook(owner_for_hook, name, *position)?;
                        Ok(())
                    }
                    _ => Err(MetorexError::runtime_error(
                        "left of `::=` is not a class/module",
                        position_to_location(*position),
                    )),
                }
            }
            _ => Err(invalid_assignment_target_error(target)),
        }
    }
}

/// Whether `target` is a constant-shaped assignment target that `left` reads,
/// which is the shape `CONST ||= value` desugars to.
fn constant_target_matches(target: &Expression, left: &Expression) -> bool {
    match (target, left) {
        (
            Expression::Identifier { name, .. },
            Expression::Identifier {
                name: left_name, ..
            },
        ) => name == left_name && name.starts_with(|c: char| c.is_ascii_uppercase()),
        (
            Expression::ScopeResolution { name, .. },
            Expression::ScopeResolution {
                name: left_name, ..
            },
        ) => name == left_name,
        _ => false,
    }
}

/// Name an anonymous class or module on binding it to a constant. A named
/// namespace makes the name permanent and cascades into the anonymous modules
/// nested under it; an anonymous one only supplies a temporary name.
pub(crate) fn name_constant_value(namespace: &Rc<Class>, const_name: &str, value: &Object) {
    let (Object::Class(bound) | Object::Module(bound)) = value else {
        return;
    };
    let qualified = format!("{}::{}", namespace.inspect_name(), const_name);
    let namespace_name = namespace.ruby_name();
    if namespace_name.is_empty() || namespace_name.contains("#<") {
        bound.set_assigned_name_if_anonymous(&qualified);
    } else {
        bound.assign_name_recursive(&qualified);
    }
}
