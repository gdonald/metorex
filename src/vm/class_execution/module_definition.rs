// Running a `module` body.

use super::*;

impl VirtualMachine {
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

        // `module ::Name` — the parser encodes the leading `::` as a name
        // prefix; it anchors the constant at the top level, bypassing the
        // lexical-scope fallback.
        let (name, top_level) = match name.strip_prefix("::") {
            Some(stripped) => (stripped, true),
            None => (name, false),
        };
        // If we're lexically nested inside a module/class, resolve the name
        // against the parent's constants first — `module Foo; module Bar; end; end`
        // defines `Foo::Bar`, distinct from any top-level `::Bar` of the same name.
        let parent_scope = if top_level {
            None
        } else {
            explicit_ns
                .clone()
                .or_else(|| self.def_scope_stack.last().cloned())
        };
        // Build a qualified name for fresh modules so `Module#ruby_name`
        // (and warning messages like the autoload "didn't define"
        // emission) report the full `Outer::Inner` path instead of just
        // the leaf.
        // A module nested in an anonymous one carries a temporary name built
        // from the parent's inspect form. It stays anonymous underneath, so
        // naming the parent later renames it too.
        let parent_is_anonymous = parent_scope
            .as_ref()
            .is_some_and(|p| p.ruby_name().is_empty());
        let full_name = match parent_scope.as_ref() {
            Some(p) => format!("{}::{}", p.inspect_name(), name),
            None => name.to_string(),
        };
        let new_module = || {
            let module = Class::new_module(if parent_is_anonymous {
                String::new()
            } else {
                full_name.clone()
            });
            if parent_is_anonymous {
                module.set_assigned_name_if_anonymous(&full_name);
            }
            Rc::new(module)
        };
        let (module, existing_as_class, is_new) = if let Some(parent) = parent_scope.as_ref() {
            // Object's constants are top-level constants: reopening inside
            // `class ::Object` must find the module registered in globals,
            // not create a fresh one.
            let direct = parent.get_class_var(name).or_else(|| {
                if parent.name() == "Object" {
                    self.globals().get(name)
                } else {
                    None
                }
            });
            let resolved = if direct.is_some() {
                direct
            } else {
                self.try_autoload_constant(parent, name)?
            };
            match resolved {
                Some(Object::Module(existing)) => (existing, false, false),
                Some(held @ (Object::Class(_) | Object::Instance(_)))
                | Some(held @ (Object::String(_) | Object::Int(_) | Object::Float(_))) => {
                    return Err(not_a_module(&full_name, &held, position));
                }
                Some(held @ (Object::Bool(_) | Object::Symbol(_) | Object::Array(_))) => {
                    return Err(not_a_module(&full_name, &held, position));
                }
                Some(Object::Nil) => {
                    return Err(not_a_module(&full_name, &Object::Nil, position));
                }
                _ => (new_module(), false, true),
            }
        } else if let Some(Object::Module(existing)) = self.globals().get(name)
            // A module reached only because something including it was mixed
            // into this scope is not a constant of this scope, so the keyword
            // opens a module of its own rather than reopening that one.
            && existing.ruby_name() == full_name
        {
            (existing, false, false)
        } else if let Some(Object::Module(existing)) = self.environment().get(name)
            && existing.ruby_name() == full_name
        {
            (existing, false, false)
        } else if let Some(Object::Class(existing)) = self.globals().get(name) {
            (existing, true, false)
        } else if let Some(Object::Class(existing)) = self.environment().get(name) {
            (existing, true, false)
        } else {
            (new_module(), false, true)
        };

        // Set 'self' to the module for instance variable access in module body
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define("self".to_string(), Object::Module(Rc::clone(&module)));
        self.def_scope_stack.push(Rc::clone(&module));
        self.class_var_cref_stack.push(Some(Rc::clone(&module)));

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
        // Record the definition's source location for
        // `Module#const_source_location`. Top-level modules record on
        // Object, the home of top-level constants.
        if is_new {
            let module_def_file = self
                .reported_current_file()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            if let Some(parent) = parent_scope.as_ref() {
                parent.set_const_location(name, module_def_file, position.line as i64);
            } else if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                object_class.set_const_location(name, module_def_file, position.line as i64);
            }
        }
        // `const_added` fires when the constant first becomes defined —
        // before the body runs. Reopening does not retrigger it.
        if is_new {
            let owner = match parent_scope.as_ref() {
                Some(parent) => Some(Object::Module(Rc::clone(parent))),
                None => self.globals().get("Object"),
            };
            if let Some(owner) = owner {
                self.trigger_const_added_hook(owner, name, position)?;
            }
        }

        // Keyword module bodies get a fresh local scope — `module` is a
        // scope gate in Ruby, so enclosing locals are not visible inside.
        self.environment_mut().push_isolated_scope();
        self.environment_mut()
            .define("self".to_string(), Object::Module(Rc::clone(&module)));

        // Run the body through a helper so scope cleanup below happens even
        // when a statement raises.
        // A module body opens and closes the same way a class body does.
        self.fire_event("class", position, Vec::new())?;
        let body_result = self.execute_module_body(&module, body);
        self.fire_event("end", position, Vec::new())?;

        self.environment_mut().pop_scope();
        self.def_scope_stack.pop();
        self.class_var_cref_stack.pop();

        // Restore previous self
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }
        // A module definition answers what its body answered, which is what
        // `eval("module M; :v; end")` hands back.
        let body_value = body_result?;

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

        Ok(ControlFlow::Value(body_value))
    }

    /// Execute a `module` keyword body's statements against `module`. Split
    /// out of `execute_module_def` so its caller can unwind scope state
    /// regardless of errors.
    fn execute_module_body(
        &mut self,
        module: &Rc<Class>,
        body: &[Statement],
    ) -> Result<Object, MetorexError> {
        // A class variable written in this body belongs to this module, not
        // to whatever block the body happens to be running inside.
        let held_home = std::mem::take(&mut self.class_var_home);
        // Ruby names a module body by the module alone, without the namespace
        // it was written in, which is what a backtrace entry for it says.
        self.call_stack_push(crate::vm::CallFrame::boundary(class_body_label(module)));
        let answer = self.module_body_statements(module, body);
        self.call_stack_pop();
        self.class_var_home = held_home;
        let written_at = body
            .first()
            .map(|statement| statement.position())
            .unwrap_or_else(|| Position::new(0, 0, 0));
        refuse_return_from_a_body(answer, written_at)
    }
}
