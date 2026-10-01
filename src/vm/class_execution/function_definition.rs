// Running a `def` written outside a class body.

use super::*;

impl VirtualMachine {
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
            .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
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
            .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
            .enumerate()
            .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
            .collect();

        // Create source location from position, naming the file the `def` was
        // read from the way that file names itself. Code handed to `eval`
        // with a name of its own reports that name rather than whatever is
        // running when the location is asked for.
        let mut source_location =
            crate::error::SourceLocation::new(position.line, position.column, position.offset);
        source_location.filename = self
            .reported_current_file()
            .map(|file| file.display().to_string());

        // Extract variadic parameter info
        let variadic_param = parameters
            .iter()
            .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
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
        function.keyword_rest_parameter = parameters
            .iter()
            .find(|p| p.is_keyword)
            .map(|p| p.name.clone());
        function.block_parameter = block_parameter;
        function.variadic_param = variadic_param;
        function.captured_refinements = self.snapshot_active_refinements();
        function.captured_nesting = self.snapshot_lexical_nesting();
        // A method defined outside every class is Object's, and a `def` run
        // while it runs installs on Object too.
        function.definee = self
            .running_method_def_scope()
            .or_else(|| self.def_scope_stack.last().cloned())
            .or_else(|| match self.globals().get("Object") {
                Some(Object::Class(object_class)) => Some(object_class),
                _ => None,
            });
        // A `def` written outside every class belongs to Object, which is the
        // name a backtrace gives it however it is reached.
        if singleton_class.is_none()
            && self.def_scope_stack.is_empty()
            && let Some(Object::Class(object_class)) = self.globals().get("Object")
        {
            function.owner = Some(object_class.name().to_string());
            function.owner_class = Some(object_class);
        }
        let function = Rc::new(function);

        // Singleton method: define on the specific class (e.g., TrueClass)
        // or on a specific instance (`def x.foo` where `x` is an Object).
        if let Some(receiver_name) = singleton_class {
            let sole_instance_receiver =
                receiver_name.strip_prefix(crate::parser::SOLE_INSTANCE_RECEIVER);
            let receiver_name = sole_instance_receiver.unwrap_or(receiver_name);
            // `def @obj.name` reads the instance variable off the current
            // self; `def $stream.name` reads the global.
            let resolved = if let Some(variable) = receiver_name.strip_prefix('@') {
                self.eval_instance_var_read(variable, position).ok()
            } else if let Some(variable) = receiver_name.strip_prefix('$') {
                self.globals().get(variable)
            } else if receiver_name == "self" {
                // At the top level `self` is `main`, which is bound nowhere
                // in the environment and has to be asked for.
                self.eval_self(position).ok()
            } else {
                // A constant receiver is found the way the constant is read
                // anywhere, from the class body the `def` sits in outwards.
                self.environment()
                    .get(receiver_name)
                    .or_else(|| self.resolve_constant_in_scope(receiver_name))
                    .or_else(|| self.globals().get(receiver_name))
            };
            if sole_instance_receiver.is_some() {
                match resolved {
                    Some(Object::Class(target_class)) => {
                        target_class.define_method(name, Rc::clone(&function));
                    }
                    // A `def self.name` written where `self` is an object
                    // rather than a class names that object alone.
                    Some(receiver @ Object::Instance(_)) => {
                        let singleton = self.singleton_class_of(&receiver);
                        singleton.define_method(name, Rc::clone(&function));
                        self.invoke_class_hook(
                            &singleton,
                            "singleton_method_added",
                            name,
                            position,
                        )?;
                    }
                    _ => {}
                }
                return Ok(ControlFlow::Value(Object::symbol(name)));
            }
            match resolved {
                // `def Klass.name` / `def mod.name` defines a singleton
                // method, stored under the same `__class__` convention as
                // `def self.name` inside the body.
                Some(Object::Class(target_class)) | Some(Object::Module(target_class)) => {
                    // A singleton method is written on the singleton class,
                    // so either one being frozen refuses it.
                    let singleton_frozen = target_class
                        .singleton_class_slot()
                        .as_ref()
                        .is_some_and(|singleton| singleton.is_frozen());
                    if singleton_frozen {
                        let receiver = if target_class.is_module() {
                            Object::Module(Rc::clone(&target_class))
                        } else {
                            Object::Class(Rc::clone(&target_class))
                        };
                        return Err(self.frozen_modification_error(&receiver, position));
                    }
                    self.refuse_frozen_definee(&target_class, position)?;
                    target_class.define_method(format!("__class__{}", name), Rc::clone(&function));
                    self.invoke_class_hook(
                        &target_class,
                        "singleton_method_added",
                        name,
                        position,
                    )?;
                }
                // `def obj.name` installs on the object's singleton class,
                // the same place `define_singleton_method` and `class << obj`
                // put one, so `methods`, `undef_method`, and `remove_method`
                // all see it.
                // `def obj.name` installs on any receiver's singleton class,
                // exceptions included.
                Some(
                    receiver @ (Object::Instance(_)
                    | Object::Exception(_)
                    | Object::Array(_)
                    | Object::Dict(_)
                    | Object::Set(_)
                    | Object::String(_)
                    | Object::Regex(_, _)),
                ) => {
                    if self.object_is_frozen(&receiver) {
                        return Err(self.frozen_modification_error(&receiver, position));
                    }
                    let singleton = self.singleton_class_of(&receiver);
                    singleton.define_method(name, Rc::clone(&function));
                    self.invoke_class_hook(&singleton, "singleton_method_added", name, position)?;
                }
                // A number, a symbol, and the like are the same object
                // wherever they turn up, so none of them can carry a method
                // of its own.
                Some(
                    receiver @ (Object::Int(_)
                    | Object::BigInt(_)
                    | Object::Float(_)
                    | Object::Symbol(_)),
                ) => {
                    let message = format!(
                        "can't define singleton method \"{}\" for {}",
                        name,
                        self.builtins().class_of(&receiver).name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                _ => {}
            }
            return Ok(ControlFlow::Value(Object::symbol(name)));
        }

        // Register the function in the environment (for immediate local access)
        self.environment_mut()
            .define(name.to_string(), Object::Method(Rc::clone(&function)));

        // Ruby installs the method on the current default definee: the
        // innermost lexical class/module if there is one, otherwise Object
        // (top-level `def`, globally accessible).
        // A `def` run from a method body installs where that method was
        // defined, and is public whatever visibility the class body left.
        let from_method_body = self.running_method_def_scope();
        if let Some(definee) = from_method_body
            .clone()
            .or_else(|| self.def_scope_stack.last().cloned())
        {
            self.refuse_frozen_definee(&definee, position)?;
            definee.define_method(name, Rc::clone(&function));
            if from_method_body.is_some() {
                definee.clear_method_visibility(name);
            } else {
                apply_current_visibility(&definee, name);
            }
            // A method written at the top level of a wrapped load belongs to
            // the module the load was wrapped in, and a top-level method is
            // private, so it is reached through that module rather than by
            // anything that mixes the module in.
            if self.load_wrap_depth > 0 && self.def_scope_stack.len() == 1 {
                definee.set_method_private(name);
            }
            let hook = Self::method_added_hook_for(&definee);
            self.invoke_class_hook(&definee, hook, name, position)?;
            if module_function_is_active(&definee) {
                self.copy_to_module_function(&definee, name, position)?;
            }
        } else if let Some(Object::Class(object_class)) = self.globals().get("Object") {
            object_class.define_method(name, Rc::clone(&function));
            // A method defined in top-level code is private unless `public`
            // was called there. One defined when a method runs is public.
            let home = self.lexical_home_frame.unwrap_or(self.current_method_frame);
            let in_top_level_code = home == self.toplevel_frame;
            if in_top_level_code && !self.toplevel_public {
                object_class.set_method_private(name);
            } else {
                object_class.clear_method_visibility(name);
            }
        }

        Ok(ControlFlow::Value(Object::symbol(name)))
    }
}
