// Core statement execution logic for the Metorex VM.
// This module handles the dispatcher and basic statement execution.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::errors::*;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Evaluate a statement and produce control-flow information for the caller.
    pub(crate) fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
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
                    self.invoke_callable(result, vec![], *position)?;
                    return Ok(ControlFlow::Next);
                }

                Ok(ControlFlow::Next)
            }
            Statement::Assignment {
                target,
                value,
                position: _,
            } => {
                let evaluated = self.evaluate_expression(value)?;
                self.assign_value(target, evaluated)?;
                Ok(ControlFlow::Next)
            }
            Statement::MultipleAssignment {
                targets,
                values,
                position: _,
            } => {
                if values.len() == 1 {
                    // Single value on right side — try to splat an array
                    let evaluated = self.evaluate_expression(&values[0])?;
                    match evaluated {
                        Object::Array(arr) => {
                            let elements = arr.borrow();
                            for (i, target) in targets.iter().enumerate() {
                                let val = elements.get(i).cloned().unwrap_or(Object::Nil);
                                self.assign_value(target, val)?;
                            }
                        }
                        _ => {
                            // Assign first target, rest get nil
                            for (i, target) in targets.iter().enumerate() {
                                let val = if i == 0 {
                                    evaluated.clone()
                                } else {
                                    Object::Nil
                                };
                                self.assign_value(target, val)?;
                            }
                        }
                    }
                } else {
                    // Multiple values — evaluate all RHS first, then assign
                    let evaluated: Vec<Object> = values
                        .iter()
                        .map(|v| self.evaluate_expression(v))
                        .collect::<Result<Vec<_>, _>>()?;
                    for (i, target) in targets.iter().enumerate() {
                        let val = evaluated.get(i).cloned().unwrap_or(Object::Nil);
                        self.assign_value(target, val)?;
                    }
                }
                Ok(ControlFlow::Next)
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
            Statement::Continue { position } => Ok(ControlFlow::Continue {
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
            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => self.execute_for(variable, iterable, body, *position),
            Statement::ClassDef {
                name,
                namespace,
                superclass,
                body,
                position,
            } => self.execute_class_def(
                name,
                namespace.as_deref(),
                superclass.as_deref(),
                body,
                *position,
            ),
            Statement::MethodDef { .. } => {
                // MethodDef should only appear inside ClassDef bodies, not at top level
                Err(unimplemented_statement_error(statement))
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
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Next => continue,
                flow => return Ok(flow),
            }
        }
        Ok(ControlFlow::Next)
    }

    /// Assign a value to the given target expression.
    pub(crate) fn assign_value(
        &mut self,
        target: &Expression,
        value: Object,
    ) -> Result<(), MetorexError> {
        match target {
            Expression::Identifier { name, .. } => {
                // Constant-shaped identifiers (`MyClass = ...`) auto-name
                // anonymous classes/modules on first assignment, matching
                // Ruby's `klass.name` behavior after binding to a constant.
                let is_const = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                if is_const && let Object::Class(v) | Object::Module(v) = &value {
                    v.set_assigned_name_if_anonymous(name);
                }
                // Constants assigned in a context that has a lexically
                // enclosing class/module land on that scope's class_var
                // table — matching Ruby's behavior, where `Foo = 1` inside
                // a `module M` (or a lambda defined inside one) resolves
                // later as `M::Foo`. At the top level (no enclosing
                // scope), uppercase assignments go to globals (and
                // Object's class_var) so they survive the require'd
                // file's local environment ending.
                if is_const && let Some(enclosing) = self.def_scope_stack.last() {
                    enclosing.set_class_var(name, value);
                    return Ok(());
                }
                if is_const {
                    self.globals_mut().set(name.clone(), value.clone());
                    if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                        object_class.set_class_var(name, value);
                    }
                    return Ok(());
                }
                if !self.environment_mut().set(name, value.clone()) {
                    self.environment_mut().define(name.clone(), value);
                }
                Ok(())
            }
            Expression::InstanceVariable { name, position } => {
                // Instance variables can only be set within a method (where 'self' is defined)
                match self.environment().get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let is_frozen = instance_rc.borrow().frozen;
                        if is_frozen {
                            let class_name = instance_rc.borrow().class.name().to_string();
                            let msg = format!("can't modify frozen {}", class_name);
                            let exc = Object::exception("FrozenError", msg.clone());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(*position),
                                message: msg,
                            });
                        }
                        let mut instance = instance_rc.borrow_mut();
                        instance.set_var(name.clone(), value);
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
                        let msg = format!("can't modify frozen {}: {}", other.type_name(), other);
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
                // Class variables can only be set within a method or class context
                // For now, we'll look for 'self' to get the class
                match self.environment().get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let instance = instance_rc.borrow();
                        instance.class.set_class_var(name.clone(), value);
                        Ok(())
                    }
                    Some(Object::Class(class)) | Some(Object::Module(class)) => {
                        // A class variable assigned in a singleton class body
                        // (`class << self`) attaches to the lexical enclosing
                        // class, not the singleton itself. Redirect to the
                        // attached object when it is a class or module.
                        if class.get_class_var("__singleton__").is_some()
                            && let Some(Object::Class(attached) | Object::Module(attached)) =
                                class.get_class_var("__attached__")
                        {
                            attached.set_class_var(name.clone(), value);
                        } else {
                            class.set_class_var(name.clone(), value);
                        }
                        Ok(())
                    }
                    Some(_) => Err(MetorexError::runtime_error(
                        format!("Cannot set class variable @@{} in this context", name),
                        position_to_location(*position),
                    )),
                    None => Err(MetorexError::runtime_error(
                        format!(
                            "Class variable @@{} can only be used within a class or method",
                            name
                        ),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::Index {
                array,
                index,
                position,
            } => {
                // Evaluate the array/object and index
                let obj = self.evaluate_expression(array)?;
                let idx = self.evaluate_expression(index)?;

                match obj {
                    Object::Array(array_rc) => {
                        // Array index assignment
                        if let Object::Int(i) = idx {
                            let mut array = array_rc.borrow_mut();
                            let len = array.len() as i64;
                            let actual_index = if i < 0 { len + i } else { i };

                            if actual_index < 0 || actual_index >= len {
                                return Err(MetorexError::runtime_error(
                                    format!("Array index out of bounds: {}", i),
                                    position_to_location(*position),
                                ));
                            }
                            array[actual_index as usize] = value;
                            Ok(())
                        } else {
                            Err(MetorexError::runtime_error(
                                "Array index must be an integer",
                                position_to_location(*position),
                            ))
                        }
                    }
                    Object::Dict(dict_rc) => {
                        // Hash/Dict index assignment — Ruby allows any object as a key
                        let key_str =
                            crate::vm::utils::object_to_dict_key(&idx).unwrap_or_default();
                        let is_primitive = crate::vm::utils::is_primitive_key(&idx);
                        let mut dict = dict_rc.borrow_mut();
                        // Store non-primitive key objects in a sentinel sub-map
                        if !is_primitive {
                            let key_objs_key = "__MX_KEY_OBJECTS__".to_string();
                            let mut key_objs = match dict.get(&key_objs_key) {
                                Some(Object::Dict(d)) => d.borrow().clone(),
                                _ => std::collections::HashMap::new(),
                            };
                            key_objs.insert(key_str.clone(), idx.clone());
                            dict.insert(
                                key_objs_key,
                                Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(key_objs))),
                            );
                        }
                        dict.insert(key_str, value);
                        Ok(())
                    }
                    Object::Instance(instance_rc) => {
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
                        } else if class.name() == "Thread" {
                            // Thread-local storage: `t[:k] = v`. Targeted
                            // here to avoid recursing through a general
                            // native-method fallback.
                            let key_str = match &idx {
                                Object::Symbol(s) => Some((**s).clone()),
                                Object::String(s) => Some((**s).clone()),
                                _ => None,
                            };
                            if let Some(k) = key_str {
                                let existing =
                                    instance_rc.borrow().get_var("__thread_locals").cloned();
                                let dict = match existing {
                                    Some(Object::Dict(d)) => d,
                                    _ => {
                                        let d = Rc::new(std::cell::RefCell::new(
                                            std::collections::HashMap::new(),
                                        ));
                                        instance_rc.borrow_mut().set_var(
                                            "__thread_locals".to_string(),
                                            Object::Dict(Rc::clone(&d)),
                                        );
                                        d
                                    }
                                };
                                dict.borrow_mut().insert(k, value);
                            }
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
                        } else {
                            Err(MetorexError::runtime_error(
                                "No []= method defined on class/module",
                                position_to_location(*position),
                            ))
                        }
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
                // Handle setter method calls (e.g., obj.name = value becomes obj.name=(value))
                if arguments.is_empty() {
                    // This is a setter method call: obj.method = value -> obj.method=(value)
                    let setter_method = format!("{}=", method);
                    let receiver_obj = self.evaluate_expression(receiver)?;

                    // Look up the setter method and invoke it
                    match receiver_obj {
                        Object::Instance(instance_rc) => {
                            let (class, method_obj) = {
                                let instance = instance_rc.borrow();
                                let class = instance.class.clone();
                                let method_obj = instance.class.find_method(&setter_method);
                                (class, method_obj)
                            }; // Borrow is dropped here

                            if let Some(method) = method_obj {
                                // Visibility check: an explicit-receiver setter
                                // call (`obj.foo = …`) can only invoke public
                                // methods, mirroring `evaluate_method_call`.
                                let is_explicit_receiver =
                                    !matches!(receiver.as_ref(), Expression::SelfExpr { .. });
                                if is_explicit_receiver && class.is_method_private(&setter_method) {
                                    let msg = format!(
                                        "private method '{}' called for an instance of {}",
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
                            } else {
                                Err(MetorexError::runtime_error(
                                    format!("Undefined setter method '{}'", setter_method),
                                    position_to_location(*position),
                                ))
                            }
                        }
                        Object::Class(class_rc) => {
                            if let Some(method) = class_rc.find_method(&setter_method) {
                                self.invoke_method(
                                    Rc::clone(&class_rc),
                                    method,
                                    Object::Class(class_rc),
                                    vec![value],
                                    *position,
                                )?;
                                Ok(())
                            } else {
                                // Store as class variable
                                class_rc.set_class_var(
                                    format!("@{}", setter_method.trim_end_matches('=')),
                                    value,
                                );
                                Ok(())
                            }
                        }
                        Object::Module(module_rc) => {
                            if let Some(method) = module_rc.find_method(&setter_method) {
                                self.invoke_method(
                                    Rc::clone(&module_rc),
                                    method,
                                    Object::Module(module_rc),
                                    vec![value],
                                    *position,
                                )?;
                                Ok(())
                            } else {
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
                            } else {
                                Err(MetorexError::runtime_error(
                                    format!(
                                        "Cannot call setter method '{}' on {}",
                                        setter_method,
                                        other.type_name()
                                    ),
                                    position_to_location(*position),
                                ))
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
                self.globals_mut().set(name.clone(), value);
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
                        if let Object::Class(v) | Object::Module(v) = &value {
                            let qualified = format!("{}::{}", c.ruby_name(), name);
                            v.set_assigned_name_if_anonymous(&qualified);
                        }
                        // Explicit `Mod::X = value` outside of an autoload
                        // load supersedes any pending autoload. (The
                        // autoload trigger path manages its own autoload
                        // bookkeeping after the load completes.)
                        c.remove_autoload(name);
                        c.set_class_var(name, value);
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
