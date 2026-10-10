// Running a `class` body, and the hooks a new class fires.

use super::*;

impl VirtualMachine {
    /// Execute class definition - create a Class object and register it in the environment.
    pub(crate) fn execute_class_def(
        &mut self,
        name: &str,
        namespace_expr: Option<&Expression>,
        superclass_name: Option<&str>,
        superclass_expression: Option<&Expression>,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // `class ::Name` — the parser encodes the leading `::` as a name
        // prefix; it anchors the constant at the top level, bypassing the
        // lexical-scope fallback.
        let (name, top_level) = match name.strip_prefix("::") {
            Some(stripped) => (stripped, true),
            None => (name, false),
        };
        // `class NS::Name` — evaluate NS and use it as the parent scope for
        // the new class's constant binding.
        let parent_scope = if let Some(expr) = namespace_expr {
            match self.evaluate_expression(expr)? {
                Object::Class(c) | Object::Module(c) => {
                    self.refuse_private_reopening(&c, name, position)?;
                    Some(c)
                }
                held => {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!("{} is not a class/module", held.type_name()),
                        position,
                    ));
                }
            }
        } else if top_level {
            None
        } else {
            // Lexical nesting fallback.
            self.constant_home()
        };

        // Resolve superclass if specified. Check enclosing scope's constants first
        // (so `class Bar < Foo` inside `module M` can see `M::Foo`), then the
        // environment, then globals.
        // A superclass written as a call is evaluated here, which is what
        // `class Point < Struct.new(:x)` names.
        let evaluated_parent = match superclass_expression {
            Some(expression) => match self.evaluate_expression(expression)? {
                Object::Class(class) => Some(class),
                other => {
                    let message =
                        format!("superclass must be a Class ({} given)", other.type_name());
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &message,
                        position,
                    ));
                }
            },
            None => None,
        };
        let superclass = if let Some(parent) = evaluated_parent {
            Some(parent)
        } else if let Some(super_name) = superclass_name {
            let resolved = if super_name.contains("::") {
                self.resolve_constant_with_autoload(super_name)?
            } else {
                // Check the explicit parent scope first, then the lexical
                // def-scope chain (so `class ChildA < ParentA` inside
                // `class ContainerA` sees the enclosing module's ParentA),
                // then environment and globals.
                let direct = parent_scope
                    .as_ref()
                    .and_then(|p| p.get_class_var(super_name))
                    .or_else(|| self.resolve_constant_in_scope(super_name));
                let autoloaded = if direct.is_some() {
                    direct
                } else if let Some(parent) = parent_scope.as_ref() {
                    self.try_autoload_constant(parent, super_name)?
                } else {
                    None
                };
                // A name an enclosing module autoloads is read the way the
                // same constant written in an expression here is.
                match autoloaded {
                    Some(found) => Some(found),
                    None => self
                        .evaluate_expression(&crate::ast::Expression::Identifier {
                            name: super_name.to_string(),
                            position,
                        })
                        .ok(),
                }
            };
            match resolved {
                Some(Object::Class(class)) => Some(class),
                Some(held) => {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!("superclass must be a Class ({} given)", held.type_name()),
                        position,
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

        if superclass
            .as_ref()
            .is_some_and(|parent| parent.is_singleton_class())
        {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                "can't make subclass of singleton class",
                position,
            ));
        }

        // Reopen existing class if it exists (Ruby semantics), otherwise create new.
        // Track `is_new` so we know whether to fire the `inherited` hook. If
        // the name is registered as an autoload on the parent scope, fire it
        // first so reopening as a class reuses the loaded definition.
        let existing_class: Option<Rc<Class>> = if let Some(parent) = parent_scope.as_ref() {
            // Object's constants are top-level constants: reopening inside
            // `class ::Object` must find the class registered in globals,
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
                Some(Object::Class(c)) => Some(c),
                Some(_) => return Err(not_a_class_error(name, position)),
                None => None,
            }
        } else if let Some(held) = self.globals().get(name) {
            match held {
                Object::Class(c) => Some(c),
                _ => return Err(not_a_class_error(name, position)),
            }
        } else if let Some(Object::Class(c)) = self.environment().get(name) {
            Some(c)
        } else if let Some(Object::Class(object_class)) = self.globals().get("Object") {
            // A top-level name registered as an autoload is loaded first,
            // so the body reopens the class the file defines.
            match self.try_autoload_constant(&object_class, name)? {
                Some(Object::Class(c)) => Some(c),
                Some(_) => return Err(not_a_class_error(name, position)),
                None => None,
            }
        } else {
            None
        };
        let is_new = existing_class.is_none();
        // Reopening an existing class with a superclass written out names
        // the one it already has, or it is a TypeError.
        if let (Some(existing), Some(expected)) = (&existing_class, &superclass)
            && (superclass_name.is_some() || superclass_expression.is_some())
        {
            let compatible = existing
                .superclass()
                .is_some_and(|held| Rc::ptr_eq(&held, expected));
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
                let new_class = Class::new(full_name, superclass);
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
        self.class_var_cref_stack.push(Some(Rc::clone(&class)));
        // Record the class definition's source location for
        // `Module#const_source_location`. Done here (alongside the
        // eager bind) so the location is available even mid-load.
        // Reopening a class leaves the place it was first defined.
        let class_def_file = self
            .reported_current_file()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let recorded_on = match parent_scope.as_ref() {
            Some(parent) => Some(Rc::clone(parent)),
            // Top-level classes record on Object, the home of top-level
            // constants.
            None => match self.globals().get("Object") {
                Some(Object::Class(object_class)) => Some(object_class),
                _ => None,
            },
        };
        if let Some(home) = recorded_on.filter(|_| is_new) {
            home.set_const_location(name, class_def_file.clone(), position.line as i64);
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
        // `const_added` fires when the constant first becomes defined —
        // before the body runs and before `inherited` (which fires after
        // the body). Reopening does not retrigger it.
        if is_new {
            // A top-level class or module is a constant on Object, which is
            // the hook's receiver there.
            let owner = match parent_scope.as_ref() {
                Some(parent) => Some(Object::Class(Rc::clone(parent))),
                None => self.globals().get("Object"),
            };
            if let Some(owner) = owner {
                self.trigger_const_added_hook(owner, name, position)?;
            }
        }
        // Keyword class bodies get a fresh local scope — `class` is a scope
        // gate in Ruby, so enclosing locals are not visible inside the body.
        self.environment_mut().push_isolated_scope();
        self.environment_mut()
            .define("self".to_string(), Object::Class(Rc::clone(&class)));
        // Ruby reserves every local the body assigns before any of it runs,
        // so one assigned in a branch not taken answers nil.
        for name in crate::ast::collect_assigned_locals(body) {
            self.environment_mut().hoist(name);
        }
        // A trace sees a class body opening and closing, with the class
        // itself as the `self` those two events report.
        self.fire_event("class", position, Vec::new())?;
        let body_result = self.apply_class_body(&class, body, position);
        self.fire_event("end", position, Vec::new())?;
        self.environment_mut().pop_scope();
        self.def_scope_stack.pop();
        self.class_var_cref_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }
        // A class definition answers what its body answered, which is what
        // `eval("class C; :v; end")` hands back.
        let body_value = body_result?;

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

        Ok(ControlFlow::Value(body_value))
    }

    /// Invoke `superclass.inherited(child)` if the hook is defined. Walks the
    /// usual class-method resolution paths (singleton class first, then the
    /// `__class__` fallback), and up the superclass chain so a hook inherited
    /// via `super` is reachable.
    /// Refuse `class NS::Name` or `module NS::Name` naming a private
    /// constant of NS from outside NS's own body, the same as reading it
    /// there would be refused.
    pub(crate) fn refuse_private_reopening(
        &mut self,
        scope: &Rc<Class>,
        name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if scope.is_private_constant(name)
            && scope.get_class_var(name).is_some()
            && !self
                .def_scope_stack
                .iter()
                .any(|open| Rc::ptr_eq(open, scope))
        {
            self.private_constant_refused(scope, scope, name, position)?;
        }
        Ok(())
    }

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

    /// Invoke `owner.const_added(name)` if the hook is defined. `owner` is the
    /// class/module the constant was bound on, passed as the receiver so the
    /// hook body sees it as `self`. The default `Module#const_added` is a
    /// native no-op, so nothing fires unless a user hook exists.
    pub(crate) fn trigger_const_added_hook(
        &mut self,
        owner: Object,
        const_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let owner_rc = match &owner {
            Object::Class(c) | Object::Module(c) => Rc::clone(c),
            _ => return Ok(()),
        };
        // Only class-level definitions count as the hook: `def
        // self.const_added` (stored under the `__class__` prefix) or a
        // method on the singleton-class chain. A plain `def const_added`
        // is an instance method for includers, not a hook.
        let mut found = owner_rc
            .find_method("__class__const_added")
            .map(|m| (Rc::clone(&owner_rc), m));
        if found.is_none() {
            let mut cursor = Some(Rc::clone(&owner_rc));
            while let Some(current) = cursor {
                if let Some(sc) = current.singleton_class_slot().clone()
                    && let Some(m) = sc.find_method("const_added")
                {
                    found = Some((sc, m));
                    break;
                }
                cursor = current.superclass();
            }
        }
        // A reopened `Module` or `Class` carries the hook as an instance
        // method, which is how every module reaches it: `class Module; def
        // const_added(name); end; end` fires for each one. Ordinary method
        // lookup on the owner finds that and skips a class's own instance
        // methods, which are for its includers rather than the hook.
        if found.is_none() {
            found = self.lookup_method(&owner, "const_added");
        }
        if let Some((holder, method)) = found
            && !method.is_undefined
        {
            self.invoke_method(
                holder,
                method,
                owner,
                vec![Object::symbol(const_name.to_string())],
                position,
            )?;
        }
        Ok(())
    }

    /// True when `Module` or `Class` has been reopened with an instance
    /// method named `const_added`, which every class and module then answers.
    pub(crate) fn user_const_added_hook_defined(&self) -> bool {
        ["Class", "Module"].iter().any(|global_name| {
            matches!(
                self.globals().get(global_name),
                Some(Object::Class(global_class)) if global_class.find_method("const_added").is_some()
            )
        })
    }
}

/// The TypeError for `class Name` where the constant already holds something
/// other than a class.
fn not_a_class_error(name: &str, position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("TypeError", &format!("{} is not a class", name), position)
}
