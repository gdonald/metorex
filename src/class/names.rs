// The name a class answers to, whether it was given one or not.

use super::*;

impl Class {
    /// Return the class name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Ruby-visible name: returns the original (or assigned) name, or the
    /// empty string for truly-anonymous classes. The `name` native method
    /// maps this to `nil` for anonymous classes.
    pub fn ruby_name(&self) -> String {
        if let Some(temporary) = self.temporary_name.borrow().as_ref() {
            return temporary.clone();
        }
        if let Some(assigned) = self.assigned_name.borrow().as_ref() {
            return assigned.clone();
        }
        self.name.clone()
    }

    /// Inspect-style label used when an anonymous class needs a printable
    /// identifier — e.g. when it acts as the namespace in `parent::C`, we
    /// synthesize `#<Class:0x<ptr>>::C` for the subclass's name.
    pub fn inspect_name(&self) -> String {
        // A refinement always displays by its label, whatever it is named.
        if let Some(crate::object::Object::String(label)) =
            self.get_class_var(crate::vm::REFINEMENT_LABEL_KEY)
        {
            return format!("#<refinement:{}>", label);
        }
        let rn = self.ruby_name();
        if rn.is_empty() {
            format!(
                "#<{}:0x{:016x}>",
                self.kind_name(),
                self as *const Class as usize
            )
        } else {
            rn
        }
    }

    /// Set or clear the name from `Module#set_temporary_name`, cascading into
    /// the anonymous modules nested under it so they follow the new name.
    /// Clearing also drops the derived `#<Module:0x…>::N` form, leaving the
    /// module with no name at all.
    pub fn set_temporary_name(&self, name: Option<String>) {
        *self.temporary_name.borrow_mut() = name.clone();
        if name.is_none() {
            let derived = matches!(
                self.assigned_name.borrow().as_ref(),
                Some(assigned) if assigned.contains("#<") || assigned.starts_with("::")
            );
            if derived {
                *self.assigned_name.borrow_mut() = None;
            }
        }
        for (key, value) in self.class_variables.borrow().iter() {
            if !key.chars().next().is_some_and(|c| c.is_uppercase()) {
                continue;
            }
            let (crate::object::Object::Class(nested) | crate::object::Object::Module(nested)) =
                value
            else {
                continue;
            };
            if nested.has_permanent_name() {
                continue;
            }
            nested.set_temporary_name(name.as_ref().map(|n| format!("{}::{}", n, key)));
        }
    }

    /// Whether this object has a permanent name: one from its definition, or
    /// one assigned by binding it to a constant under a named namespace.
    pub fn has_permanent_name(&self) -> bool {
        if !self.name.is_empty() {
            return true;
        }
        match self.assigned_name.borrow().as_ref() {
            Some(assigned) => !assigned.contains("#<") && !assigned.starts_with("::"),
            None => false,
        }
    }

    /// Install a Ruby-visible name on an anonymous class. No-op if the class
    /// already has a name (either original or previously assigned).
    pub fn set_assigned_name_if_anonymous(&self, name: &str) {
        if !self.name.is_empty() {
            return;
        }
        let mut slot = self.assigned_name.borrow_mut();
        if slot.is_none() {
            *slot = Some(name.to_string());
        }
    }

    /// Assign a Ruby-visible name to a previously-anonymous module and
    /// cascade into anonymous modules bound to its constants, mirroring
    /// Ruby's naming of nested anonymous modules once the root gains a
    /// permanent name. Placeholder names (`::B`, `#<Class:0x…>::B`) are
    /// replaced; definition names and previously-assigned permanent names
    /// win.
    pub fn assign_name_recursive(&self, new_name: &str) {
        if !self.name.is_empty() {
            return;
        }
        // A temporary name gives way to a permanent one.
        let current = match self.temporary_name.borrow().as_ref() {
            Some(_) => String::new(),
            None => self.ruby_name(),
        };
        if !current.is_empty() && !current.starts_with("::") && !current.contains("#<") {
            return;
        }
        // A permanent name supersedes any temporary one.
        *self.temporary_name.borrow_mut() = None;
        *self.assigned_name.borrow_mut() = Some(new_name.to_string());
        for (key, val) in self.class_variables.borrow().iter() {
            if !key.chars().next().is_some_and(|c| c.is_uppercase()) {
                continue;
            }
            if let crate::object::Object::Class(c) | crate::object::Object::Module(c) = val {
                c.assign_name_recursive(&format!("{}::{}", new_name, key));
            }
        }
    }

    /// Return the superclass if present.
    /// Let go of everything this class holds that points back at it. A
    /// method carries the class it was defined in, and a class carries its
    /// methods, so the two keep each other alive. Nothing reads a class after
    /// this, and it is what lets the whole graph be freed.
    pub fn tear_down(&self) {
        super::methods::method_state_changed();
        self.methods.borrow_mut().clear();
        self.class_variables.borrow_mut().clear();
        self.mixins.borrow_mut().clear();
        self.prepends.borrow_mut().clear();
        *self.singleton_class.borrow_mut() = None;
    }

    pub fn superclass(&self) -> Option<Rc<Class>> {
        self.superclass.as_ref().map(Rc::clone)
    }

    /// Declare a new instance variable on this class.
    pub fn declare_instance_var(&self, name: impl Into<String>) {
        self.instance_variables.borrow_mut().insert(name.into());
    }

    /// Check if this class (or a superclass) declares the given instance variable.
    pub fn has_instance_var(&self, name: &str) -> bool {
        if self.instance_variables.borrow().contains(name) {
            return true;
        }

        self.superclass
            .as_ref()
            .is_some_and(|superclass| superclass.has_instance_var(name))
    }

    /// Return the list of instance variable names defined directly on this class.
    pub fn instance_variables(&self) -> Vec<String> {
        let mut vars = self
            .instance_variables
            .borrow()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        vars.sort();
        vars
    }
}
