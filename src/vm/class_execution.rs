// Class and function definition execution for the Metorex VM.
// This module handles class and function definition statements.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use std::rc::Rc;

impl VirtualMachine {
    /// Execute class definition - create a Class object and register it in the environment.
    pub(crate) fn execute_class_def(
        &mut self,
        name: &str,
        namespace_expr: Option<&Expression>,
        superclass_name: Option<&str>,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // `class NS::Name` — evaluate NS and use it as the parent scope for
        // the new class's constant binding.
        let parent_scope = if let Some(expr) = namespace_expr {
            match self.evaluate_expression(expr)? {
                Object::Class(c) | Object::Module(c) => Some(c),
                _ => {
                    return Err(MetorexError::runtime_error(
                        "left of `::` in class definition is not a class/module",
                        position_to_location(position),
                    ));
                }
            }
        } else {
            // Lexical nesting fallback.
            self.def_scope_stack.last().cloned()
        };

        // Resolve superclass if specified. Check enclosing scope's constants first
        // (so `class Bar < Foo` inside `module M` can see `M::Foo`), then the
        // environment, then globals.
        let superclass = if let Some(super_name) = superclass_name {
            let resolved = if super_name.contains("::") {
                self.resolve_constant_with_autoload(super_name)?
            } else {
                let direct = parent_scope
                    .as_ref()
                    .and_then(|p| p.get_class_var(super_name))
                    .or_else(|| self.environment().get(super_name))
                    .or_else(|| self.globals().get(super_name));
                if direct.is_some() {
                    direct
                } else if let Some(parent) = parent_scope.as_ref() {
                    self.try_autoload_constant(parent, super_name)?
                } else {
                    None
                }
            };
            match resolved {
                Some(Object::Class(class)) => Some(class),
                Some(_) => {
                    return Err(MetorexError::runtime_error(
                        format!("Superclass '{}' must be a class", super_name),
                        position_to_location(position),
                    ));
                }
                None => {
                    return Err(MetorexError::runtime_error(
                        format!("Undefined superclass '{}'", super_name),
                        position_to_location(position),
                    ));
                }
            }
        } else {
            // Ruby default: classes without an explicit `<` parent inherit from
            // Object, so `class Foo; end` has the full Object → Kernel →
            // BasicObject ancestry. Built-in classes (BasicObject, Object itself)
            // are constructed directly in `init.rs` and don't hit this branch.
            match self.globals().get("Object") {
                Some(Object::Class(object_class)) => Some(object_class),
                _ => None,
            }
        };

        // Reopen existing class if it exists (Ruby semantics), otherwise create new.
        // Track `is_new` so we know whether to fire the `inherited` hook. If
        // the name is registered as an autoload on the parent scope, fire it
        // first so reopening as a class reuses the loaded definition.
        let existing_class: Option<Rc<Class>> = if let Some(parent) = parent_scope.as_ref() {
            let direct = parent.get_class_var(name);
            let resolved = if direct.is_some() {
                direct
            } else {
                self.try_autoload_constant(parent, name)?
            };
            match resolved {
                Some(Object::Class(c)) => Some(c),
                _ => None,
            }
        } else if let Some(Object::Class(c)) = self.globals().get(name) {
            Some(c)
        } else if let Some(Object::Class(c)) = self.environment().get(name) {
            Some(c)
        } else {
            None
        };
        let is_new = existing_class.is_none();
        // Reopening an existing class with an explicit superclass that
        // doesn't lie on the existing ancestor chain is a TypeError. We
        // use a lenient rule (allow if requested superclass is anywhere
        // in the existing chain) instead of MRI's exact-match because the
        // metorex test corpus reopens built-in subclasses with their
        // grandparent — `class FloatDomainError < StandardError` where
        // FloatDomainError actually descends from RangeError. Genuinely
        // unrelated parents (the autoload spec's `Z < ZZ` after a load
        // defined `Z < YY`) still trip the check.
        if let (Some(existing), Some(expected)) = (&existing_class, &superclass)
            && superclass_name.is_some()
        {
            let mut cursor = existing.superclass();
            let mut compatible = false;
            while let Some(ancestor) = cursor {
                if Rc::ptr_eq(&ancestor, expected) {
                    compatible = true;
                    break;
                }
                cursor = ancestor.superclass();
            }
            if !compatible {
                let msg = format!("superclass mismatch for class {}", existing.ruby_name());
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
        }
        let class = match existing_class {
            Some(existing) => existing,
            None => {
                // Compose a Ruby-style qualified name. Named parents get a
                // simple `Outer::Inner`; anonymous parents fall back to their
                // inspect label so e.g. `parent::C` becomes `#<Class:0x..>::C`.
                let full_name = match parent_scope.as_ref() {
                    Some(p) => {
                        let p_name = p.ruby_name();
                        if p_name.is_empty() {
                            format!("{}::{}", p.inspect_name(), name)
                        } else {
                            format!("{}::{}", p_name, name)
                        }
                    }
                    None => name.to_string(),
                };
                let new_class = Rc::new(Class::new(full_name, superclass));
                if let Some(sc) = new_class.superclass() {
                    sc.add_subclass(&new_class);
                }
                new_class
            }
        };

        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define("self".to_string(), Object::Class(Rc::clone(&class)));
        self.def_scope_stack.push(Rc::clone(&class));
        // Record the class definition's source location for
        // `Module#const_source_location`. Done here (alongside the
        // eager bind) so the location is available even mid-load.
        let class_def_file = self
            .get_current_file()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        if let Some(parent) = parent_scope.as_ref() {
            parent.set_const_location(name, class_def_file.clone(), position.line as i64);
        }
        // Eagerly bind the class to its parent / globals BEFORE the body
        // runs. Without this, code in the body (e.g. autoload-triggered
        // checks like `Outer.constants.include?(:Inner)` while the file
        // defining `Outer::Inner` is mid-load) wouldn't see the class
        // until the closing `end`. Repeated at the end is harmless and
        // keeps the existing exit semantics.
        let class_obj_pre = Object::Class(Rc::clone(&class));
        if let Some(parent) = parent_scope.as_ref() {
            parent.set_class_var(name, class_obj_pre);
        } else {
            self.environment_mut()
                .define(name.to_string(), class_obj_pre.clone());
            self.globals_mut().set(name.to_string(), class_obj_pre);
        }
        let body_result = self.apply_class_body(&class, body, position);
        self.def_scope_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }
        body_result?;

        let class_obj = Object::Class(Rc::clone(&class));
        if let Some(parent) = parent_scope {
            parent.set_class_var(name, class_obj);
        } else {
            self.environment_mut()
                .define(name.to_string(), class_obj.clone());
            self.globals_mut().set(name.to_string(), class_obj);
        }

        // Fire `Parent.inherited(Child)` only for fresh subclass definitions
        // (Ruby: reopening doesn't retrigger the hook). The hook runs AFTER
        // the body is processed and the subclass is bound to its name.
        if is_new && let Some(sc) = class.superclass() {
            self.trigger_inherited_hook(&sc, Rc::clone(&class), position)?;
        }

        Ok(ControlFlow::Next)
    }

    /// Invoke `superclass.inherited(child)` if the hook is defined. Walks the
    /// usual class-method resolution paths (singleton class first, then the
    /// `__class__` fallback), and up the superclass chain so a hook inherited
    /// via `super` is reachable.
    pub(crate) fn trigger_inherited_hook(
        &mut self,
        superclass: &Rc<Class>,
        child: Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        let receiver = Object::Class(Rc::clone(superclass));
        if let Some((owner, method)) = self.lookup_method(&receiver, "inherited")
            && !method.is_undefined
        {
            self.invoke_method(
                owner,
                method,
                receiver,
                vec![Object::Class(child)],
                position,
            )?;
        }
        Ok(())
    }

    /// Evaluate a block with `self` bound to the given class/module, executing
    /// the block's body as if it were a class/module definition body.
    pub(crate) fn apply_block_as_class_body(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.apply_block_as_class_body_with_self(
            class,
            block,
            position,
            Object::Class(Rc::clone(class)),
        )
    }

    /// Same as `apply_block_as_class_body` but explicit about the `self` kind
    /// (use `Object::Module(...)` when the receiver should behave as a module).
    pub(crate) fn apply_block_as_class_body_with_self(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
        self_obj: Object,
    ) -> Result<Object, MetorexError> {
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define("self".to_string(), self_obj.clone());
        // Share the outer RefCells so an assignment like `x = self` inside the
        // block mutates the caller's local, matching Ruby's closure semantics
        // for blocks (including `Class.new do ... end`).
        for (name, cell) in block.captured_vars.iter() {
            if self.environment().get(name).is_none() {
                self.environment_mut()
                    .define_shared(name.clone(), cell.clone());
            }
        }
        // Bind the block's first parameter to the class/module, matching Ruby:
        // `Class.new { |c| ... }` and `mod.class_eval { |m| ... }` both yield
        // the class/module as the sole block argument.
        if let Some(first) = block
            .parameters
            .iter()
            .find(|p| !p.starts_with('&') && !p.starts_with('*'))
        {
            let pname = first.trim_start_matches('|').to_string();
            self.environment_mut().define(pname, self_obj);
        }
        self.def_scope_stack.push(Rc::clone(class));
        // A class/module body is not a method context, so `using` is permitted
        // even when this block runs deep inside method calls (e.g. mspec's
        // runner invoking `Class.new do using ...; end`).
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        let result = self.apply_class_body(class, &block.body, position);
        self.user_def_nesting = saved_nesting;
        self.def_scope_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }
        result
    }

    /// Implementation of `Module#class_exec` / `Module#module_exec`: evaluate the
    /// block with class-body semantics (so `def` lands on the receiver) and
    /// `self` bound to the receiver, but bind the block's parameters to the
    /// caller-supplied `args` rather than to the module. Returns the block's
    /// last value.
    pub(crate) fn class_exec_block(
        &mut self,
        class: &Rc<Class>,
        self_obj: Object,
        block: &crate::object::BlockStatement,
        args: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let prev_self = self.environment().get("self");
        self.environment_mut().define("self".to_string(), self_obj);
        for (name, cell) in block.captured_vars.iter() {
            if self.environment().get(name).is_none() {
                self.environment_mut()
                    .define_shared(name.clone(), cell.clone());
            }
        }
        // Bind the block's positional parameters to the supplied arguments.
        let positional: Vec<&String> = block
            .parameters
            .iter()
            .filter(|p| !p.starts_with('&') && !p.starts_with('*'))
            .collect();
        for (i, param) in positional.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(Object::Nil);
            self.environment_mut().define((*param).clone(), value);
        }
        self.def_scope_stack.push(Rc::clone(class));
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        let result = self.apply_class_body(class, &block.body, position);
        self.user_def_nesting = saved_nesting;
        self.def_scope_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }
        result
    }

    /// Shared implementation of `Module#class_eval` / `Module#module_eval` (and
    /// the `Class` variants). Handles both the block form and the string form
    /// (`class_eval(code, filename = "...", lineno = 1)`), returning the value
    /// of the last evaluated statement. `self_obj` is the receiver as it should
    /// appear inside the body (`Object::Class` or `Object::Module`).
    pub(crate) fn class_eval_with_args(
        &mut self,
        class_rc: &Rc<Class>,
        self_obj: Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Block form wins when a block is present.
        if let Some(Object::Block(block)) = self.pending_block.take() {
            if !arguments.is_empty() {
                return Err(self.arg_count_error(
                    format!(
                        "wrong number of arguments (given {}, expected 0)",
                        arguments.len()
                    ),
                    position,
                ));
            }
            return self.apply_block_as_class_body_with_self(class_rc, &block, position, self_obj);
        }

        // String form expects 1..3 positional arguments.
        if arguments.is_empty() {
            return Err(self.arg_count_error(
                "wrong number of arguments (given 0, expected 1..3)".to_string(),
                position,
            ));
        }
        if arguments.len() > 3 {
            return Err(self.arg_count_error(
                format!(
                    "wrong number of arguments (given {}, expected 1..3)",
                    arguments.len()
                ),
                position,
            ));
        }

        let code = self.coerce_eval_string(&arguments[0], position)?;
        let filename = match arguments.get(1) {
            Some(obj) => self.coerce_eval_string(obj, position)?,
            None => {
                let caller = self
                    .get_current_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(eval)".to_string());
                format!("(eval at {}:{})", caller, position.line)
            }
        };
        let lineno = match arguments.get(2) {
            Some(Object::Int(n)) => (*n).max(1) as usize,
            _ => 1,
        };

        let tokens = crate::lexer::Lexer::with_start_line(&code, lineno).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .map_err(|errors| {
                MetorexError::runtime_error(
                    format!(
                        "{}: parse error: {}",
                        filename,
                        errors
                            .iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<_>>()
                            .join("; ")
                    ),
                    position_to_location(position),
                )
            })?;

        // Evaluate the parsed body with `self` bound to the receiver and the
        // receiver pushed as the current definee, so `def` lands on it and
        // constants resolve in its scope. `__FILE__` reflects `filename` for
        // the duration. Refinements active at the call site stay active so
        // `class_eval` can see `using` from the eval scope.
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define("self".to_string(), self_obj.clone());
        // Ruby resolves constants in a string eval using the caller's lexical
        // scope, which for a named receiver includes the namespaces it is
        // nested in. Push those enclosing modules (outermost first) so e.g.
        // `ModuleSpecs::ClassEvalTest.module_eval("Lookup")` finds
        // `ModuleSpecs::Lookup`.
        let mut enclosing_pushed = 0usize;
        let full_name = class_rc.name().to_string();
        if let Some(idx) = full_name.rfind("::") {
            let mut cumulative = String::new();
            for segment in full_name[..idx].split("::") {
                if cumulative.is_empty() {
                    cumulative = segment.to_string();
                } else {
                    cumulative = format!("{}::{}", cumulative, segment);
                }
                if let Some(Object::Module(m) | Object::Class(m)) =
                    self.resolve_constant_in_scope(&cumulative)
                {
                    self.def_scope_stack.push(m);
                    enclosing_pushed += 1;
                }
            }
        }
        self.def_scope_stack.push(Rc::clone(class_rc));
        let prev_file = self.current_file.clone();
        self.current_file = Some(std::path::PathBuf::from(&filename));
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        let result = self.apply_class_body(class_rc, &statements, position);
        self.user_def_nesting = saved_nesting;
        self.current_file = prev_file;
        self.def_scope_stack.pop();
        for _ in 0..enclosing_pushed {
            self.def_scope_stack.pop();
        }
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }
        result
    }

    /// Coerce an eval argument (code string or filename) to a `String`,
    /// invoking `#to_str` for non-strings. Mirrors MRI's error messages:
    /// a missing `#to_str` raises "no implicit conversion of X into String",
    /// and a `#to_str` returning a non-string raises "can't convert X into
    /// String".
    fn coerce_eval_string(
        &mut self,
        arg: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(s) = arg {
            return Ok((**s).clone());
        }
        let class_name = match arg {
            Object::Instance(i) => i.borrow().class.name().to_string(),
            Object::Class(_) | Object::Module(_) => "Module".to_string(),
            other => other.type_name().to_string(),
        };
        if let Some((cls, m)) = self.lookup_method(arg, "to_str")
            && !m.is_undefined
        {
            let result = self.invoke_method(cls, m, arg.clone(), vec![], position)?;
            if let Object::String(s) = result {
                return Ok((*s).clone());
            }
            let msg = format!("can't convert {} into String", class_name);
            let exc = Object::exception("TypeError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        let msg = format!("no implicit conversion of {} into String", class_name);
        let exc = Object::exception("TypeError", msg.clone());
        Err(MetorexError::UncaughtException {
            exception: exc,
            location: position_to_location(position),
            message: msg,
        })
    }

    /// Build an `ArgumentError` with the given message.
    fn arg_count_error(&self, msg: String, position: Position) -> MetorexError {
        let exc = Object::exception("ArgumentError", msg.clone());
        MetorexError::UncaughtException {
            exception: exc,
            location: position_to_location(position),
            message: msg,
        }
    }

    /// Apply a class-body statement list to the given class (also used for
    /// `Class.new { ... }` / anonymous classes). Returns the value of the last
    /// evaluated statement, which `class_eval` / `module_eval` surface as their
    /// result (ordinary class definitions discard it).
    pub(crate) fn apply_class_body(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Reset visibility to public at the start of every class body so
        // reopening doesn't carry over a stale state from a prior body.
        class.set_current_visibility("public");
        let mut last_value = Object::Nil;
        for statement in body {
            // Bare `private` / `public` / `protected` statements (no args)
            // toggle the default visibility for subsequent method defs.
            if let Statement::Expression {
                expression: Expression::Identifier { name, .. },
                ..
            } = statement
                && matches!(name.as_str(), "private" | "public" | "protected")
            {
                class.set_current_visibility(name);
                continue;
            }
            match statement {
                Statement::FunctionDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    singleton_class: None,
                    ..
                } => {
                    let m = build_method_from_params(
                        method_name.clone(),
                        parameters,
                        method_body.clone(),
                        self.snapshot_active_refinements(),
                    );
                    class.define_method(method_name, Rc::new(m));
                    if class.current_visibility() != "public" {
                        class.set_method_private(method_name.clone());
                    }
                    last_value = Object::Symbol(Rc::new(method_name.clone()));
                }
                // `def self.name` inside a `Class.new do ... end` block: the parser
                // emits a FunctionDef (not MethodDef, because `in_class_body` is
                // false inside a do-block). Route it through the same path as
                // `def self.name` in a regular class body — i.e. store as a
                // class-level method under the `__class__` prefix.
                Statement::FunctionDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    singleton_class: Some(receiver),
                    ..
                } if receiver == "self" => {
                    let m = build_method_from_params(
                        method_name.clone(),
                        parameters,
                        method_body.clone(),
                        self.snapshot_active_refinements(),
                    );
                    class.define_method(format!("__class__{}", method_name), Rc::new(m));
                    last_value = Object::Symbol(Rc::new(method_name.clone()));
                }
                Statement::MethodDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    is_class_method,
                    ..
                } => {
                    // Create a Method object
                    let param_names: Vec<String> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .map(|p| p.name.clone())
                        .collect();
                    let keyword_parameters: Vec<(String, Option<crate::ast::Expression>)> =
                        parameters
                            .iter()
                            .filter(|p| p.is_named_keyword)
                            .map(|p| (p.name.clone(), p.default_value.clone()))
                            .collect();
                    let block_parameter = parameters
                        .iter()
                        .find(|p| p.is_block)
                        .map(|p| p.name.clone());
                    let default_params: Vec<(usize, crate::ast::Expression)> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
                        .collect();
                    let variadic_param = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .find(|(_, p)| p.is_variadic)
                        .map(|(i, p)| (i, p.name.clone()));
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.default_parameters = default_params;
                    m.keyword_parameters = keyword_parameters;
                    m.block_parameter = block_parameter;
                    m.variadic_param = variadic_param;
                    m.captured_refinements = self.snapshot_active_refinements();
                    m.owner = Some(class.name().to_string());
                    let method = Rc::new(m);
                    if *is_class_method {
                        // def self.method_name — store as class method with __class__ prefix
                        class.define_method(format!("__class__{}", method_name), method);
                    } else {
                        class.define_method(method_name, method);
                        if class.current_visibility() != "public" {
                            class.set_method_private(method_name.clone());
                        }
                    }
                    last_value = Object::Symbol(Rc::new(method_name.clone()));
                }
                Statement::Assignment {
                    target: Expression::InstanceVariable { name: var_name, .. },
                    value,
                    ..
                } => {
                    // Declare the ivar and bind the class-level value so
                    // `@body = self` in a class body sets a real class-level
                    // instance variable (read back by `class.get_class_var`
                    // with the `@` prefix), not just a declared-but-unset name.
                    class.declare_instance_var(var_name);
                    let initial_value = self.evaluate_expression(value)?;
                    class.set_class_var(format!("@{}", var_name), initial_value);
                }
                Statement::Assignment {
                    target: Expression::ClassVariable { name: var_name, .. },
                    value,
                    ..
                } => {
                    // Class variable initialization (e.g., @@count = 0 in class body)
                    let initial_value = self.evaluate_expression(value)?;
                    class.set_class_var(var_name, initial_value);
                }
                Statement::Expression {
                    expression: Expression::InstanceVariable { name: var_name, .. },
                    ..
                } => {
                    // Instance variable declaration without assignment
                    class.declare_instance_var(var_name);
                }
                Statement::AttrReader { attributes, .. } => {
                    let names = self.resolve_attribute_names(attributes, position)?;
                    let visibility = class.current_visibility();
                    for attr_name in &names {
                        let getter_body = vec![Statement::Return {
                            value: Some(Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let method = Rc::new(Method::new(attr_name.clone(), vec![], getter_body));
                        class.define_method(attr_name, method);
                        if visibility != "public" {
                            class.set_method_private(attr_name.clone());
                        }
                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::AttrWriter { attributes, .. } => {
                    let names = self.resolve_attribute_names(attributes, position)?;
                    let visibility = class.current_visibility();
                    for attr_name in &names {
                        let setter_body = vec![Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: Expression::Identifier {
                                name: "value".to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_name = format!("{}=", attr_name);
                        let method = Rc::new(Method::new(
                            setter_name.clone(),
                            vec!["value".to_string()],
                            setter_body,
                        ));
                        class.define_method(&setter_name, method);
                        if visibility != "public" {
                            class.set_method_private(setter_name);
                        }
                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::AttrAccessor { attributes, .. } => {
                    let names = self.resolve_attribute_names(attributes, position)?;
                    let visibility = class.current_visibility();
                    for attr_name in &names {
                        // Getter
                        let getter_body = vec![Statement::Return {
                            value: Some(Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let getter_method =
                            Rc::new(Method::new(attr_name.clone(), vec![], getter_body));
                        class.define_method(attr_name, getter_method);
                        if visibility != "public" {
                            class.set_method_private(attr_name.clone());
                        }

                        // Setter
                        let setter_body = vec![Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: Expression::Identifier {
                                name: "value".to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_name = format!("{}=", attr_name);
                        let setter_method = Rc::new(Method::new(
                            setter_name.clone(),
                            vec!["value".to_string()],
                            setter_body,
                        ));
                        class.define_method(&setter_name, setter_method);
                        if visibility != "public" {
                            class.set_method_private(setter_name);
                        }

                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::Assignment {
                    target:
                        Expression::Identifier {
                            name: const_name, ..
                        },
                    value,
                    ..
                } if const_name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase()) =>
                {
                    // Constant assignment in class body (e.g., PI = 3.14159).
                    // Lowercase identifiers fall through to the `_` arm and are
                    // treated as normal local-variable assignments.
                    let const_value = self.evaluate_expression(value)?;
                    class.set_class_var(const_name, const_value.clone());
                    last_value = const_value;
                }
                Statement::Include {
                    module_name,
                    position,
                } => {
                    // include ModuleName: dispatch through `append_features`
                    // so user overrides on the module's singleton class fire,
                    // and so cyclic/frozen checks run.
                    let resolved = self.resolve_constant_with_autoload(module_name)?;
                    match resolved {
                        Some(Object::Module(module)) => {
                            self.apply_module_include(class, &module, *position)?;
                        }
                        Some(_) => {
                            return Err(MetorexError::runtime_error(
                                format!("'{}' is not a module", module_name),
                                position_to_location(*position),
                            ));
                        }
                        None => {
                            return Err(MetorexError::runtime_error(
                                format!("Undefined module '{}'", module_name),
                                position_to_location(*position),
                            ));
                        }
                    }
                }
                Statement::Extend {
                    module_name,
                    position,
                } => {
                    // extend ModuleName: add module methods as class-level methods
                    match self.resolve_constant_in_scope(module_name) {
                        Some(Object::Module(module)) => {
                            for method_name in module.method_names() {
                                if let Some(method) = module.find_method(&method_name) {
                                    class.set_class_var(
                                        format!("__ext__{}", method_name),
                                        Object::Method(method),
                                    );
                                }
                            }
                        }
                        Some(_) => {
                            return Err(MetorexError::runtime_error(
                                format!("'{}' is not a module", module_name),
                                position_to_location(*position),
                            ));
                        }
                        None => {
                            return Err(MetorexError::runtime_error(
                                format!("Undefined module '{}'", module_name),
                                position_to_location(*position),
                            ));
                        }
                    }
                }
                Statement::Alias {
                    new_name, old_name, ..
                } => {
                    class.alias_method(new_name, old_name);
                }
                // `class << <target>` inside a class body — open the target's
                // singleton class and apply the inner body there.
                Statement::Expression {
                    expression:
                        Expression::SingletonClass {
                            target,
                            body: inner_body,
                            position: sc_pos,
                        },
                    ..
                } => {
                    self.evaluate_singleton_class_expression(target, inner_body, *sc_pos)?;
                }
                // class << self block — treat inner statements as class-level
                Statement::Block { statements, .. } => {
                    for inner_stmt in statements {
                        if let Statement::AttrAccessor { attributes, .. } = inner_stmt {
                            let names = self.resolve_attribute_names(attributes, position)?;
                            for attr_name in &names {
                                // Getter
                                let getter_body = vec![Statement::Expression {
                                    expression: Expression::InstanceVariable {
                                        name: format!("@{}", attr_name),
                                        position: crate::lexer::Position::default(),
                                    },
                                    position: crate::lexer::Position::default(),
                                }];
                                class.define_method(
                                    attr_name,
                                    Rc::new(Method::new(
                                        attr_name.to_string(),
                                        vec![],
                                        getter_body,
                                    )),
                                );
                                // Setter
                                let setter_body = vec![Statement::Assignment {
                                    target: Expression::InstanceVariable {
                                        name: format!("@{}", attr_name),
                                        position: crate::lexer::Position::default(),
                                    },
                                    value: Expression::Identifier {
                                        name: "value".to_string(),
                                        position: crate::lexer::Position::default(),
                                    },
                                    position: crate::lexer::Position::default(),
                                }];
                                class.define_method(
                                    format!("{}=", attr_name),
                                    Rc::new(Method::new(
                                        format!("{}=", attr_name),
                                        vec!["value".to_string()],
                                        setter_body,
                                    )),
                                );
                            }
                        }
                    }
                }
                _ => {
                    let mut handled = false;
                    // Handle alias_method in class body
                    if let Statement::Expression {
                        expression:
                            Expression::Call {
                                callee,
                                arguments: call_args,
                                ..
                            },
                        ..
                    } = statement
                        && let Expression::Identifier {
                            name: callee_name, ..
                        } = callee.as_ref()
                        && callee_name == "alias_method"
                        && call_args.len() == 2
                    {
                        handled = true;
                        let new_name = match self.evaluate_expression(&call_args[0])? {
                            Object::String(s) => (*s).clone(),
                            Object::Symbol(s) => (*s).clone(),
                            _ => String::new(),
                        };
                        let old_name = match self.evaluate_expression(&call_args[1])? {
                            Object::String(s) => (*s).clone(),
                            Object::Symbol(s) => (*s).clone(),
                            _ => String::new(),
                        };
                        if !new_name.is_empty() && !old_name.is_empty() {
                            class.alias_method(&new_name, &old_name);
                        }
                    }
                    // Handle define_method(:name) { |args| body } calls in class body
                    else if let Statement::Expression {
                        expression:
                            Expression::Call {
                                callee,
                                arguments: call_args,
                                trailing_block: Some(block_expr),
                                ..
                            },
                        ..
                    } = statement
                        && let Expression::Identifier {
                            name: callee_name, ..
                        } = callee.as_ref()
                        && callee_name == "define_method"
                    {
                        handled = true;
                        let method_name_str = if let Some(name_expr) = call_args.first() {
                            match self.evaluate_expression(name_expr)? {
                                Object::String(s) => (*s).clone(),
                                Object::Symbol(s) => (*s).clone(),
                                _ => {
                                    return Err(MetorexError::runtime_error(
                                        "define_method: first argument must be a String or Symbol",
                                        position_to_location(position),
                                    ));
                                }
                            }
                        } else {
                            return Err(MetorexError::runtime_error(
                                "define_method requires at least one argument",
                                position_to_location(position),
                            ));
                        };
                        let block_obj = self.evaluate_expression(block_expr)?;
                        let block = match block_obj {
                            Object::Block(b) => b,
                            _ => {
                                return Err(MetorexError::runtime_error(
                                    "define_method requires a block",
                                    position_to_location(position),
                                ));
                            }
                        };
                        // Split the block's parameter list into regular,
                        // variadic (`*args`), and block (`&blk`) parts so a
                        // `define_method(:m) { |*args, &blk| ... }` (as the
                        // mspec mock framework uses) gets correct arity instead
                        // of treating `*args`/`&blk` as required positionals.
                        let mut regular_params: Vec<String> = Vec::new();
                        let mut variadic_param: Option<(usize, String)> = None;
                        let mut block_parameter: Option<String> = None;
                        for (i, param) in block.parameters.iter().enumerate() {
                            if let Some(name) = param.strip_prefix('*') {
                                variadic_param = Some((i, name.to_string()));
                                regular_params.push(name.to_string());
                            } else if let Some(name) = param.strip_prefix('&') {
                                block_parameter = Some(name.to_string());
                            } else {
                                regular_params.push(param.clone());
                            }
                        }
                        let mut method = Method::new(
                            method_name_str.clone(),
                            regular_params,
                            block.body.clone(),
                        );
                        method.variadic_param = variadic_param;
                        method.block_parameter = block_parameter;
                        // Capture closure: prefer existing captured_vars, otherwise snap current scope
                        method.captured_vars = Some(if block.captured_vars.is_empty() {
                            self.environment().current_scope_var_refs()
                        } else {
                            block.captured_vars.clone()
                        });
                        class.define_method(&method_name_str, Rc::new(method));
                    }
                    // `refine(target) { body }` inside a module body — dispatch
                    // to the module's refine method, preserving the block.
                    else if let Statement::Expression {
                        expression:
                            Expression::Call {
                                callee,
                                arguments: call_args,
                                trailing_block,
                                ..
                            },
                        ..
                    } = statement
                        && let Expression::Identifier {
                            name: callee_name, ..
                        } = callee.as_ref()
                        && callee_name == "refine"
                    {
                        handled = true;
                        self.evaluate_method_call(
                            &Expression::SelfExpr { position },
                            "refine",
                            call_args,
                            trailing_block.as_deref(),
                            position,
                        )?;
                    }
                    // Fall through to general statement execution for arbitrary
                    // expressions and lowercase-identifier assignments (locals,
                    // method calls, @ivars on the class, etc.). Ruby evaluates
                    // any expression inside a class body with `self` bound to
                    // the class; we've already pushed that self in
                    // `apply_block_as_class_body_with_self` / `execute_class_def`.
                    if !handled {
                        match statement {
                            // A bare expression statement (e.g. `1 + 1`, `self`,
                            // a constant reference) is the common tail of a
                            // `class_eval`/`module_eval` string or block — capture
                            // its value so it becomes the eval result.
                            Statement::Expression {
                                expression,
                                position: expr_pos,
                            } => {
                                let v = self.evaluate_expression(expression)?;
                                if matches!(expression, Expression::Identifier { .. })
                                    && matches!(v, Object::Method(_))
                                {
                                    last_value = self.invoke_callable(v, vec![], *expr_pos)?;
                                } else {
                                    last_value = v;
                                }
                            }
                            _ => match self.execute_statement(statement)? {
                                ControlFlow::Value(v) | ControlFlow::Return { value: v, .. } => {
                                    last_value = v;
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }
        Ok(last_value)
    }

    /// Execute function definition - create a Method object and register it in the environment as a function.
    pub(crate) fn execute_function_def(
        &mut self,
        name: &str,
        parameters: &[crate::ast::Parameter],
        body: &[Statement],
        position: crate::lexer::Position,
        singleton_class: Option<&str>,
    ) -> Result<ControlFlow, MetorexError> {
        // Extract positional parameter names (exclude named keyword and block params)
        let param_names: Vec<String> = parameters
            .iter()
            .filter(|p| !p.is_named_keyword && !p.is_block)
            .map(|p| p.name.clone())
            .collect();

        // Extract named keyword parameters
        let keyword_parameters: Vec<(String, Option<crate::ast::Expression>)> = parameters
            .iter()
            .filter(|p| p.is_named_keyword)
            .map(|p| (p.name.clone(), p.default_value.clone()))
            .collect();

        // Extract block parameter name
        let block_parameter = parameters
            .iter()
            .find(|p| p.is_block)
            .map(|p| p.name.clone());

        // Extract positional default values
        let default_parameters: Vec<(usize, crate::ast::Expression)> = parameters
            .iter()
            .filter(|p| !p.is_named_keyword && !p.is_block)
            .enumerate()
            .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
            .collect();

        // Create source location from position
        let source_location =
            crate::error::SourceLocation::new(position.line, position.column, position.offset);

        // Extract variadic parameter info
        let variadic_param = parameters
            .iter()
            .filter(|p| !p.is_named_keyword && !p.is_block)
            .enumerate()
            .find(|(_, p)| p.is_variadic)
            .map(|(i, p)| (i, p.name.clone()));

        // Create a Method object to represent the function
        let mut function = Method::with_source_location(
            name.to_string(),
            param_names,
            body.to_vec(),
            source_location,
        );
        function.default_parameters = default_parameters;
        function.keyword_parameters = keyword_parameters;
        function.block_parameter = block_parameter;
        function.variadic_param = variadic_param;
        function.captured_refinements = self.snapshot_active_refinements();
        let function = Rc::new(function);

        // Singleton method: define on the specific class (e.g., TrueClass)
        // or on a specific instance (`def x.foo` where `x` is an Object).
        if let Some(receiver_name) = singleton_class {
            let resolved = self
                .environment()
                .get(receiver_name)
                .or_else(|| self.globals().get(receiver_name));
            match resolved {
                Some(Object::Class(target_class)) => {
                    target_class.define_method(name, Rc::clone(&function));
                }
                Some(Object::Module(target_mod)) => {
                    target_mod.define_method(name, Rc::clone(&function));
                }
                Some(Object::Instance(inst)) => {
                    inst.borrow()
                        .define_singleton_method(name.to_string(), Rc::clone(&function));
                }
                _ => {}
            }
            return Ok(ControlFlow::Next);
        }

        // Register the function in the environment (for immediate local access)
        self.environment_mut()
            .define(name.to_string(), Object::Method(Rc::clone(&function)));

        // Also register as a method on the global Object class (Ruby semantics:
        // top-level `def` defines a method on Object, globally accessible).
        if let Some(Object::Class(object_class)) = self.globals().get("Object") {
            object_class.define_method(name, Rc::clone(&function));
        }

        Ok(ControlFlow::Next)
    }

    /// Execute module definition - create a Module object and register it.
    pub(crate) fn execute_module_def(
        &mut self,
        name: &str,
        namespace: Option<&crate::ast::Expression>,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // `module NS::Name` — evaluate NS, then install Name on it. The
        // namespace expression must resolve to a Class or Module.
        let explicit_ns: Option<Rc<Class>> = if let Some(expr) = namespace {
            let value = self.evaluate_expression(expr)?;
            match value {
                Object::Class(c) | Object::Module(c) => Some(c),
                other => {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "module namespace must be a Class or Module, got {}",
                            other.type_name()
                        ),
                        position_to_location(position),
                    ));
                }
            }
        } else {
            None
        };

        // If we're lexically nested inside a module/class, resolve the name
        // against the parent's constants first — `module Foo; module Bar; end; end`
        // defines `Foo::Bar`, distinct from any top-level `::Bar` of the same name.
        let parent_scope = explicit_ns
            .clone()
            .or_else(|| self.def_scope_stack.last().cloned());
        // Build a qualified name for fresh modules so `Module#ruby_name`
        // (and warning messages like the autoload "didn't define"
        // emission) report the full `Outer::Inner` path instead of just
        // the leaf.
        let full_name = match parent_scope.as_ref() {
            Some(p) => {
                let p_name = p.ruby_name();
                if p_name.is_empty() {
                    name.to_string()
                } else {
                    format!("{}::{}", p_name, name)
                }
            }
            None => name.to_string(),
        };
        let (module, existing_as_class) = if let Some(parent) = parent_scope.as_ref() {
            let direct = parent.get_class_var(name);
            let resolved = if direct.is_some() {
                direct
            } else {
                self.try_autoload_constant(parent, name)?
            };
            match resolved {
                Some(Object::Module(existing)) => (existing, false),
                Some(Object::Class(existing)) => (existing, true),
                _ => (Rc::new(Class::new(full_name.clone(), None)), false),
            }
        } else if let Some(Object::Module(existing)) = self.globals().get(name) {
            (existing, false)
        } else if let Some(Object::Module(existing)) = self.environment().get(name) {
            (existing, false)
        } else if let Some(Object::Class(existing)) = self.globals().get(name) {
            (existing, true)
        } else if let Some(Object::Class(existing)) = self.environment().get(name) {
            (existing, true)
        } else {
            (Rc::new(Class::new(full_name.clone(), None)), false)
        };

        // Set 'self' to the module for instance variable access in module body
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define("self".to_string(), Object::Module(Rc::clone(&module)));
        self.def_scope_stack.push(Rc::clone(&module));

        // Eagerly publish the (possibly freshly-created) module to its
        // parent / globals BEFORE running the body. Without this, code
        // executed during the body — most notably an autoload-triggered file
        // doing `module Outer; X = ...; end` — would create a brand-new
        // anonymous Outer instead of reopening the one we're currently
        // defining. The same store is repeated at the end after the body
        // completes; doing it both places is harmless and keeps the existing
        // exit semantics.
        let module_obj_pre = if existing_as_class {
            Object::Class(Rc::clone(&module))
        } else {
            Object::Module(Rc::clone(&module))
        };
        if let Some(parent) = parent_scope.as_ref() {
            parent.set_class_var(name, module_obj_pre);
        } else {
            self.environment_mut()
                .define(name.to_string(), module_obj_pre.clone());
            self.globals_mut().set(name.to_string(), module_obj_pre);
        }

        for statement in body {
            match statement {
                Statement::MethodDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    is_class_method,
                    ..
                } => {
                    let param_names: Vec<String> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .map(|p| p.name.clone())
                        .collect();
                    let keyword_parameters: Vec<(String, Option<crate::ast::Expression>)> =
                        parameters
                            .iter()
                            .filter(|p| p.is_named_keyword)
                            .map(|p| (p.name.clone(), p.default_value.clone()))
                            .collect();
                    let block_parameter = parameters
                        .iter()
                        .find(|p| p.is_block)
                        .map(|p| p.name.clone());
                    let default_params: Vec<(usize, crate::ast::Expression)> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
                        .collect();
                    let variadic_param = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .find(|(_, p)| p.is_variadic)
                        .map(|(i, p)| (i, p.name.clone()));
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.default_parameters = default_params;
                    m.keyword_parameters = keyword_parameters;
                    m.block_parameter = block_parameter;
                    m.variadic_param = variadic_param;
                    m.captured_refinements = self.snapshot_active_refinements();
                    m.owner = Some(module.name().to_string());
                    let method = Rc::new(m);
                    if *is_class_method {
                        module.define_method(format!("__class__{}", method_name), method);
                    } else {
                        module.define_method(method_name, method);
                    }
                }
                Statement::Assignment {
                    target:
                        Expression::Identifier {
                            name: const_name, ..
                        },
                    value,
                    ..
                } if const_name.starts_with(|c: char| c.is_uppercase()) => {
                    let const_value = self.evaluate_expression(value)?;
                    module.set_class_var(const_name, const_value);
                }
                // `class << <target>` inside a module body — open the target's
                // singleton class and apply the inner body there.
                Statement::Expression {
                    expression:
                        Expression::SingletonClass {
                            target,
                            body: inner_body,
                            position: sc_pos,
                        },
                    ..
                } => {
                    self.evaluate_singleton_class_expression(target, inner_body, *sc_pos)?;
                }
                // class << self block — process inner statements at module level
                Statement::Block {
                    statements: inner, ..
                } => {
                    for inner_stmt in inner {
                        match inner_stmt {
                            Statement::AttrReader { attributes, .. }
                            | Statement::AttrWriter { attributes, .. }
                            | Statement::AttrAccessor { attributes, .. } => {
                                let names =
                                    self.resolve_attribute_names(attributes, Position::default())?;
                                for attr in &names {
                                    let getter = Method::new(
                                        attr.clone(),
                                        vec![],
                                        vec![Statement::Expression {
                                            expression: Expression::InstanceVariable {
                                                name: attr.clone(),
                                                position: crate::lexer::Position::default(),
                                            },
                                            position: crate::lexer::Position::default(),
                                        }],
                                    );
                                    module.define_method(attr, Rc::new(getter));
                                }
                            }
                            Statement::MethodDef {
                                name: method_name,
                                parameters,
                                body: method_body,
                                ..
                            } => {
                                let param_names: Vec<String> = parameters
                                    .iter()
                                    .filter(|p| !p.is_named_keyword && !p.is_block)
                                    .map(|p| p.name.clone())
                                    .collect();
                                let m = Method::new(
                                    method_name.clone(),
                                    param_names,
                                    method_body.clone(),
                                );
                                module.define_method(method_name, Rc::new(m));
                            }
                            _ => {
                                self.execute_statement(inner_stmt)?;
                            }
                        }
                    }
                }
                // ClassDef inside module — execute_class_def attaches the nested
                // class to the enclosing scope directly via parent_scope detection.
                Statement::ClassDef {
                    name: class_name,
                    namespace,
                    superclass,
                    body: class_body,
                    position: class_pos,
                } => {
                    self.execute_class_def(
                        class_name,
                        namespace.as_deref(),
                        superclass.as_deref(),
                        class_body,
                        *class_pos,
                    )?;
                }
                // Nested module — execute_module_def attaches it to the enclosing scope.
                Statement::ModuleDef {
                    name: mod_name,
                    namespace: mod_ns,
                    body: mod_body,
                    position: mod_pos,
                } => {
                    self.execute_module_def(mod_name, mod_ns.as_deref(), mod_body, *mod_pos)?;
                }
                // Include inside module body: walk the lexical def-scope
                // stack so nested modules can reference sibling constants
                // (e.g. `module C; include P; end` where P is defined in the
                // enclosing module).
                Statement::Include {
                    module_name: inc_name,
                    position: inc_pos,
                } => {
                    let resolved = self.resolve_constant_with_autoload(inc_name)?;
                    if let Some(Object::Module(inc_module)) = resolved {
                        self.apply_module_include(&module, &inc_module, *inc_pos)?;
                    }
                }
                Statement::Alias {
                    new_name, old_name, ..
                } => {
                    module.alias_method(new_name, old_name);
                }
                // Other statements in module body
                _ => {
                    self.execute_statement(statement)?;
                }
            }
        }

        self.def_scope_stack.pop();

        // Restore previous self
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }

        let module_obj = if existing_as_class {
            Object::Class(module)
        } else {
            Object::Module(module)
        };
        if let Some(parent) = parent_scope {
            // Nested module: attach to parent as a constant; do NOT leak into globals,
            // which would clobber same-named builtins (e.g. ::Module, ::Class).
            parent.set_class_var(name, module_obj);
        } else {
            self.environment_mut()
                .define(name.to_string(), module_obj.clone());
            self.globals_mut().set(name.to_string(), module_obj);
        }

        Ok(ControlFlow::Next)
    }

    /// Resolve a bare constant name using Ruby-like lexical scoping: walk
    /// the enclosing class/module stack (innermost first), then check the
    /// local environment, then globals. Used by `include`/`extend` inside
    /// class bodies so nested modules can reference siblings without
    /// fully-qualifying them. Also handles qualified names (`A::B::C`).
    pub(crate) fn resolve_constant_in_scope(&self, name: &str) -> Option<Object> {
        if name.contains("::") {
            let mut parts = name.split("::");
            let head = parts.next()?;
            let mut current = self.resolve_constant_in_scope(head)?;
            for part in parts {
                let class_rc = match &current {
                    Object::Class(c) | Object::Module(c) => Rc::clone(c),
                    _ => return None,
                };
                current = class_rc.get_class_var(part)?;
            }
            return Some(current);
        }
        for enclosing in self.def_scope_stack.iter().rev() {
            if let Some(val) = enclosing.get_class_var(name) {
                return Some(val);
            }
            // A still-being-defined enclosing scope resolves to itself when
            // referenced by its own simple name (e.g. `ModuleSpecs::Alias`
            // inside `module ModuleSpecs; class Allonym; include ... end; end`
            // before ModuleSpecs has been bound in globals).
            if enclosing.name() == name {
                return Some(Object::Module(Rc::clone(enclosing)));
            }
        }
        self.environment()
            .get(name)
            .or_else(|| self.globals().get(name))
    }

    /// Like `resolve_constant_in_scope`, but triggers autoload on any
    /// segment along a qualified `A::B::C` walk where the next part is only
    /// registered as an autoload. Used by class/module statement positions
    /// that should fire pending autoloads (e.g. `class X < A::B::Auto`).
    pub(crate) fn resolve_constant_with_autoload(
        &mut self,
        name: &str,
    ) -> Result<Option<Object>, MetorexError> {
        if name.contains("::") {
            let mut parts = name.split("::");
            let head = match parts.next() {
                Some(h) => h,
                None => return Ok(None),
            };
            let mut current = match self.resolve_constant_with_autoload(head)? {
                Some(v) => v,
                None => return Ok(None),
            };
            for part in parts {
                let class_rc = match &current {
                    Object::Class(c) | Object::Module(c) => Rc::clone(c),
                    _ => return Ok(None),
                };
                if let Some(v) = class_rc.get_class_var(part) {
                    current = v;
                } else if let Some(v) = self.try_autoload_constant(&class_rc, part)? {
                    current = v;
                } else {
                    return Ok(None);
                }
            }
            return Ok(Some(current));
        }
        Ok(self.resolve_constant_in_scope(name))
    }

    /// Execute include at statement level (outside class body).
    /// Top-level `include Mod` adds the module to Object's mixin chain
    /// (Ruby semantics for `include` at the main scope). When called from
    /// inside a block whose def-scope is non-empty (e.g. a lambda invoked
    /// from inside a module body), the innermost class/module receives the
    /// include instead — matching `module M; include X; end` behavior.
    /// Suppressed when running inside `load(path, true)` (wrapped load).
    pub(crate) fn execute_include(
        &mut self,
        module_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        if self.load_wrap_depth > 0 {
            return Ok(ControlFlow::Next);
        }
        let module = self
            .resolve_constant_with_autoload(module_name)?
            .or_else(|| self.resolve_qualified_constant(module_name))
            .or_else(|| self.environment().get(module_name))
            .or_else(|| self.globals().get(module_name));
        let module_rc = match module {
            Some(Object::Module(m)) => m,
            Some(_) => {
                return Err(MetorexError::runtime_error(
                    format!("'{}' is not a module", module_name),
                    position_to_location(position),
                ));
            }
            None => {
                return Err(MetorexError::runtime_error(
                    format!("Undefined module '{}'", module_name),
                    position_to_location(position),
                ));
            }
        };
        // Prefer the innermost lexical class/module if there is one; this
        // makes `include X` inside a lambda invoked from a module body
        // mix into that module rather than into Object.
        if let Some(target) = self.def_scope_stack.last().cloned() {
            self.apply_module_include(&target, &module_rc, position)?;
        } else if let Some(Object::Class(object_class)) = self.globals().get("Object") {
            object_class.add_mixin(module_rc);
        }
        Ok(ControlFlow::Next)
    }

    /// Resolve a `Foo::Bar::Baz` qualified constant by walking from the
    /// outermost name through nested module/class constants.
    fn resolve_qualified_constant(&self, qualified: &str) -> Option<Object> {
        let mut parts = qualified.split("::");
        let head = parts.next()?;
        let mut current = self
            .environment()
            .get(head)
            .or_else(|| self.globals().get(head))?;
        for part in parts {
            let class_rc = match &current {
                Object::Class(c) | Object::Module(c) => Rc::clone(c),
                _ => return None,
            };
            current = class_rc.get_class_var(part)?;
        }
        Some(current)
    }

    /// Execute extend at statement level (outside class body - error).
    pub(crate) fn execute_extend(
        &mut self,
        _module_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        Err(MetorexError::runtime_error(
            "extend can only be used inside a class definition",
            position_to_location(position),
        ))
    }

    /// Mix `module_rc` into `target` with full Ruby semantics: dispatch to a
    /// user-defined `append_features` on the module's singleton class if one
    /// exists; otherwise enforce the default checks (FrozenError when target
    /// is frozen, ArgumentError on a cyclic include) and add the mixin.
    pub(crate) fn apply_module_include(
        &mut self,
        target: &Rc<Class>,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        // `def self.append_features(...)` on the module gets stored under
        // the `__class__` prefix on the module itself (mirroring how the
        // VM caches class-level methods); check that path first.
        if let Some(method) = module_rc.find_method("__class__append_features") {
            self.invoke_method(
                Rc::clone(module_rc),
                method,
                Object::Module(Rc::clone(module_rc)),
                vec![Object::Class(Rc::clone(target))],
                position,
            )?;
            return Ok(());
        }
        if let Some(sc) = module_rc.singleton_class_slot().clone()
            && let Some(method) = sc.find_method("append_features")
        {
            self.invoke_method(
                sc,
                method,
                Object::Module(Rc::clone(module_rc)),
                vec![Object::Class(Rc::clone(target))],
                position,
            )?;
            return Ok(());
        }
        self.default_append_features(target, module_rc, position)
    }

    /// Default `Module#append_features(target)` behavior: validate the target
    /// (frozen check, cyclic include check) then add `module_rc` to its
    /// mixin chain.
    pub(crate) fn default_append_features(
        &mut self,
        target: &Rc<Class>,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if target.is_frozen() {
            let msg = format!("can't modify frozen Module: {}", target.name());
            let exc = Object::exception("FrozenError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        if Rc::ptr_eq(target, module_rc) || module_includes(module_rc, target) {
            let msg = "cyclic include detected".to_string();
            let exc = Object::exception("ArgumentError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        target.add_mixin(Rc::clone(module_rc));
        Ok(())
    }

    /// Execute `alias new_name old_name` outside a class/module body. Ruby
    /// treats this as an alias on Object so it is visible in all instances.
    pub(crate) fn execute_alias(
        &mut self,
        new_name: &str,
        old_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        if let Some(enclosing) = self.def_scope_stack.last() {
            enclosing.alias_method(new_name, old_name);
            return Ok(ControlFlow::Next);
        }
        if let Some(Object::Class(object_class)) = self.globals().get("Object")
            && !object_class.alias_method(new_name, old_name)
        {
            return Err(MetorexError::runtime_error(
                format!("undefined method '{}' for alias", old_name),
                position_to_location(position),
            ));
        }
        Ok(ControlFlow::Next)
    }

    /// Evaluate attribute-name expressions for `attr_reader`/`attr_writer`/
    /// `attr_accessor`. Symbols and strings resolve directly; other values
    /// are coerced via `#to_str`. Mirrors Ruby's `Module#attr_*` semantics.
    pub(crate) fn resolve_attribute_names(
        &mut self,
        attributes: &[Expression],
        position: Position,
    ) -> Result<Vec<String>, MetorexError> {
        let mut names = Vec::with_capacity(attributes.len());
        for attr_expr in attributes {
            let value = self.evaluate_expression(attr_expr)?;
            names.push(self.coerce_method_name(&value, "attr_accessor", position)?);
        }
        Ok(names)
    }
}

/// Whether `needle` appears anywhere in `module_rc`'s ancestor chain
/// (mixins of mixins, plus superclass mixins). Used to detect a cyclic
/// `include`/`append_features` request before the chain is mutated.
fn module_includes(module_rc: &Rc<Class>, needle: &Rc<Class>) -> bool {
    let mut stack: Vec<Rc<Class>> = vec![Rc::clone(module_rc)];
    let mut seen: Vec<*const Class> = Vec::new();
    while let Some(current) = stack.pop() {
        let ptr = Rc::as_ptr(&current);
        if seen.contains(&ptr) {
            continue;
        }
        seen.push(ptr);
        if Rc::ptr_eq(&current, needle) {
            return true;
        }
        for mixin in current.mixin_chain() {
            stack.push(mixin);
        }
        if let Some(sc) = current.superclass() {
            stack.push(sc);
        }
    }
    false
}

/// Build a Method from a parameter list, handling positional, variadic,
/// keyword, block, and default parameters. Shared by the FunctionDef arms in
/// `apply_class_body` (i.e. `def` inside `Class.new do ... end`) so they pick
/// up the same parameter modes as the MethodDef path.
fn build_method_from_params(
    name: String,
    parameters: &[crate::ast::Parameter],
    body: Vec<Statement>,
    refinements: Vec<(Rc<Class>, Vec<String>)>,
) -> Method {
    let param_names: Vec<String> = parameters
        .iter()
        .filter(|p| !p.is_named_keyword && !p.is_block)
        .map(|p| p.name.clone())
        .collect();
    let keyword_parameters: Vec<(String, Option<Expression>)> = parameters
        .iter()
        .filter(|p| p.is_named_keyword)
        .map(|p| (p.name.clone(), p.default_value.clone()))
        .collect();
    let block_parameter = parameters
        .iter()
        .find(|p| p.is_block)
        .map(|p| p.name.clone());
    let default_parameters: Vec<(usize, Expression)> = parameters
        .iter()
        .filter(|p| !p.is_named_keyword && !p.is_block)
        .enumerate()
        .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
        .collect();
    let variadic_param = parameters
        .iter()
        .filter(|p| !p.is_named_keyword && !p.is_block)
        .enumerate()
        .find(|(_, p)| p.is_variadic)
        .map(|(i, p)| (i, p.name.clone()));
    let mut m = Method::new(name, param_names, body);
    m.default_parameters = default_parameters;
    m.keyword_parameters = keyword_parameters;
    m.block_parameter = block_parameter;
    m.variadic_param = variadic_param;
    m.captured_refinements = refinements;
    m
}
