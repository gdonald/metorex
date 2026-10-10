// Variable read expressions: SelfExpr, InstanceVariable, ClassVariable, ScopeResolution.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

use crate::vm::core::VirtualMachine;
use crate::vm::errors::undefined_self_error;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Evaluate a bare `self` expression.
    pub(crate) fn eval_self(&self, position: Position) -> Result<Object, MetorexError> {
        if let Some(receiver) = self.environment().get("self") {
            return Ok(receiver);
        }
        // At the top level `self` is `main`, which is what TOPLEVEL_BINDING
        // was built around.
        match self.globals().get("TOPLEVEL_BINDING") {
            Some(Object::Binding(binding)) => binding
                .receiver
                .clone()
                .ok_or_else(|| undefined_self_error(position)),
            _ => Err(undefined_self_error(position)),
        }
    }

    /// Evaluate an instance variable read (`@name`). Falls back to nil for
    /// undefined ivars on instances; reads class-level ivars when self is a
    /// Class or Module.
    pub(crate) fn eval_instance_var_read(
        &self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // At the top level `self` is `main`, which carries the program's own
        // instance variables the way any other object does.
        let held = self.environment().get("self").or_else(|| {
            match self.globals().get("TOPLEVEL_BINDING") {
                Some(Object::Binding(binding)) => binding.receiver.clone(),
                _ => None,
            }
        });
        match held {
            Some(Object::Instance(instance_rc)) => {
                let instance = instance_rc.borrow();
                Ok(instance.get_var(name).cloned().unwrap_or(Object::Nil))
            }
            Some(Object::Exception(details)) => Ok(details
                .borrow()
                .instance_vars
                .get(name)
                .cloned()
                .unwrap_or(Object::Nil)),
            Some(Object::Class(class_rc)) => Ok(class_rc
                .get_class_var(&format!("@{}", name))
                .unwrap_or(Object::Nil)),
            Some(Object::Module(module_rc)) => Ok(module_rc
                .get_class_var(&format!("@{}", name))
                .unwrap_or(Object::Nil)),
            // Immediates (Int/Float/Symbol/Bool/Nil/etc.) and other non-instance
            // selves: ivar reads return nil — matching Ruby, where ivars on
            // immediates default to nil even if no writer ever ran.
            // A collection or a String keeps its instance variables aside,
            // where `instance_variable_set` puts them too.
            Some(other) => Ok(Self::collection_address(&other)
                .and_then(|address| self.collection_variables.get(&address))
                .and_then(|held| held.get(name))
                .cloned()
                .unwrap_or(Object::Nil)),
            None => Err(MetorexError::runtime_error(
                format!(
                    "Instance variable @{} can only be used within a method",
                    name
                ),
                position_to_location(position),
            )),
        }
    }

    /// The class or module a class variable belongs to where the walk
    /// stands: the one the running method was written in, then the one an
    /// `instance_exec` block was written in, then the body being run. A
    /// class variable names what this one and its ancestors hold, whatever
    /// the receiver is.
    pub(crate) fn class_variable_home(&self) -> Option<std::rc::Rc<crate::class::Class>> {
        let cref = match self.class_var_cref_stack.last() {
            // A body being run names its own home, and a block written where
            // no class stood names none.
            Some(held) => held.as_ref()?,
            None => self
                .def_scope_stack
                .last()
                .or_else(|| self.class_var_home.last())?,
        };
        // A singleton class holds no class variables of its own: one written
        // there belongs to the class it stands for.
        if cref.get_class_var("__singleton__").is_some()
            && let Some(Object::Class(attached) | Object::Module(attached)) =
                cref.get_class_var("__attached__")
        {
            return Some(attached);
        }
        Some(std::rc::Rc::clone(cref))
    }

    /// Evaluate a class variable read (`@@name`).
    pub(super) fn eval_class_var_read(
        &self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match self.class_variable_home() {
            Some(home) => {
                let value = Self::inherited_class_var(&home, name)
                    .ok_or_else(|| uninitialized_class_var_error(name, &home, position))?;
                self.refuse_class_variable_in_ractor(
                    name,
                    &Self::class_var_owner(&home, name),
                    position,
                )?;
                Ok(value)
            }
            // With no class or module open, there is nothing for the variable
            // to belong to, which Ruby refuses outright.
            None => Err(MetorexError::runtime_error(
                "class variable access from toplevel".to_string(),
                position_to_location(position),
            )),
        }
    }

    /// A class variable as seen from `class`, which Ruby looks for up the
    /// superclass chain rather than on the one class alone.
    pub(crate) fn inherited_class_var(
        class: &std::rc::Rc<crate::class::Class>,
        name: &str,
    ) -> Option<Object> {
        let mut cursor = Some(std::rc::Rc::clone(class));
        while let Some(current) = cursor {
            if let Some(value) = current.get_class_var(name) {
                return Some(value);
            }
            // A module the class mixes in shares its class variables with it.
            for mixin in current.transitive_mixins() {
                if let Some(value) = mixin.get_class_var(name) {
                    return Some(value);
                }
            }
            cursor = current.superclass();
        }
        None
    }

    /// The class or module a write to `name` belongs to: the one furthest up
    /// the chain that already holds it, or `class` when none does.
    pub(crate) fn class_var_owner(
        class: &std::rc::Rc<crate::class::Class>,
        name: &str,
    ) -> std::rc::Rc<crate::class::Class> {
        let mut owner = None;
        let mut cursor = Some(std::rc::Rc::clone(class));
        while let Some(current) = cursor {
            if current.get_class_var(name).is_some() {
                owner = Some(std::rc::Rc::clone(&current));
            }
            for mixin in current.transitive_mixins() {
                if mixin.get_class_var(name).is_some() {
                    owner = Some(mixin);
                }
            }
            cursor = current.superclass();
        }
        owner.unwrap_or_else(|| std::rc::Rc::clone(class))
    }
}

/// The NameError Ruby raises for a class variable that was never assigned.
fn uninitialized_class_var_error(
    name: &str,
    class: &std::rc::Rc<crate::class::Class>,
    position: Position,
) -> MetorexError {
    let message = format!(
        "uninitialized class variable @@{} in {}",
        name,
        class.inspect_name()
    );
    let exception = Object::exception("NameError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details.name = Some(format!("@@{}", name));
        details.receiver = Some(Box::new(Object::Class(std::rc::Rc::clone(class))));
    }
    MetorexError::UncaughtException {
        exception,
        location: position_to_location(position),
        message,
    }
}
