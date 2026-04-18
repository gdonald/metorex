//! Runtime class representation for Metorex
//! Handles method tables, inheritance, and instance variable declarations.

use crate::object::{Method, Object};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Runtime class definition with method table and inheritance.
#[derive(Debug)]
pub struct Class {
    name: String,
    /// Ruby-visible name, set the first time an anonymous class is assigned
    /// to a constant. `name()` still returns the original (immutable) name —
    /// use `ruby_name()` to pick up the assigned override.
    assigned_name: RefCell<Option<String>>,
    superclass: Option<Rc<Class>>,
    methods: RefCell<HashMap<String, Rc<Method>>>,
    instance_variables: RefCell<HashSet<String>>,
    class_variables: RefCell<HashMap<String, crate::object::Object>>,
    /// Included modules, in reverse inclusion order (last included = first searched).
    mixins: RefCell<Vec<Rc<Class>>>,
    /// Names of methods whose visibility has been set to private.
    private_method_names: RefCell<HashSet<String>>,
    /// Lazily-allocated singleton class attached to *this class object* — the
    /// thing `class << SomeClass; end` opens. Holds class-level (def self.x)
    /// methods once we start tracking them as a real class.
    singleton_class: RefCell<Option<Rc<Class>>>,
}

impl Class {
    /// Create a new class with an optional superclass.
    pub fn new(name: impl Into<String>, superclass: Option<Rc<Class>>) -> Self {
        Self {
            name: name.into(),
            assigned_name: RefCell::new(None),
            superclass,
            methods: RefCell::new(HashMap::new()),
            instance_variables: RefCell::new(HashSet::new()),
            class_variables: RefCell::new(HashMap::new()),
            mixins: RefCell::new(Vec::new()),
            private_method_names: RefCell::new(HashSet::new()),
            singleton_class: RefCell::new(None),
        }
    }

    /// Accessor for the cached singleton-class slot (None until materialized).
    pub fn singleton_class_slot(&self) -> std::cell::Ref<'_, Option<Rc<Class>>> {
        self.singleton_class.borrow()
    }

    /// Install a singleton class on this class. Cached for subsequent access.
    pub fn set_singleton_class(&self, class: Rc<Class>) {
        *self.singleton_class.borrow_mut() = Some(class);
    }

    /// Mark a method name as private on this class.
    pub fn set_method_private(&self, name: impl Into<String>) {
        self.private_method_names.borrow_mut().insert(name.into());
    }

    /// Mark a method name as public on this class (removes private flag).
    pub fn set_method_public(&self, name: &str) {
        self.private_method_names.borrow_mut().remove(name);
    }

    /// Check if a method is marked private on this class (own table only).
    pub fn is_method_private(&self, name: &str) -> bool {
        self.private_method_names.borrow().contains(name)
    }

    /// Return the list of private method names defined directly on this class.
    pub fn private_method_names(&self) -> Vec<String> {
        let mut names = self
            .private_method_names
            .borrow()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Return the class name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Ruby-visible name: returns the original name unless an `assigned_name`
    /// has been set (via `set_assigned_name_if_anonymous`), in which case the
    /// assigned name wins. `String` because we may have to synthesise it.
    pub fn ruby_name(&self) -> String {
        if let Some(assigned) = self.assigned_name.borrow().as_ref() {
            return assigned.clone();
        }
        self.name.clone()
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

    /// Return the superclass if present.
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

    /// Define or replace a method on this class.
    pub fn define_method(&self, name: impl Into<String>, method: Rc<Method>) {
        self.methods.borrow_mut().insert(name.into(), method);
    }

    /// Determine whether this class defines a method (without checking superclasses).
    pub fn has_own_method(&self, name: &str) -> bool {
        self.methods.borrow().contains_key(name)
    }

    /// Add an included module to the mixin chain.
    /// The module is prepended to the list so the most-recently-included module
    /// is searched first (Ruby's MRO).
    pub fn add_mixin(&self, module: Rc<Class>) {
        self.mixins.borrow_mut().insert(0, module);
    }

    /// Snapshot of the mixin chain (most-recently-included first).
    pub fn mixin_chain(&self) -> Vec<Rc<Class>> {
        self.mixins.borrow().clone()
    }

    /// Look up a method by walking the inheritance chain (own → mixins → superclass).
    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        if let Some(method) = self.methods.borrow().get(name) {
            return Some(Rc::clone(method));
        }

        for mixin in self.mixins.borrow().iter() {
            if let Some(method) = mixin.methods.borrow().get(name) {
                return Some(Rc::clone(method));
            }
        }

        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_method(name))
    }

    /// Remove a method defined directly on this class.
    /// Returns true if the method was found and removed, false otherwise.
    pub fn remove_method(&self, name: &str) -> bool {
        self.methods.borrow_mut().remove(name).is_some()
    }

    /// Create an alias for an existing method.
    /// Returns true if the source method was found and aliased, false otherwise.
    pub fn alias_method(&self, new_name: &str, old_name: &str) -> bool {
        if let Some(method) = self.find_method(old_name) {
            self.methods
                .borrow_mut()
                .insert(new_name.to_string(), method);
            true
        } else {
            false
        }
    }

    /// Return a list of method names defined directly on this class.
    pub fn method_names(&self) -> Vec<String> {
        let mut names = self.methods.borrow().keys().cloned().collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Set a class variable on this class.
    pub fn set_class_var(&self, name: impl Into<String>, value: Object) {
        self.class_variables.borrow_mut().insert(name.into(), value);
    }

    /// Retrieve a class variable from this class.
    pub fn get_class_var(&self, name: &str) -> Option<Object> {
        self.class_variables.borrow().get(name).cloned()
    }

    /// List all class-variable names (used by refinement bookkeeping).
    pub fn class_var_names(&self) -> Vec<String> {
        self.class_variables.borrow().keys().cloned().collect()
    }

    /// Remove a class variable/constant by name, returning the previous value
    /// if there was one. Used by `Module#remove_const`.
    pub fn remove_class_var(&self, name: &str) -> Option<crate::object::Object> {
        self.class_variables.borrow_mut().remove(name)
    }

    /// Deep-ish copy for `Class#dup`/`Module#dup`. The result is anonymous
    /// (Ruby's `#name` returns nil until assigned to a constant), carries its
    /// own method/class-var tables, and — critically — gets a fresh singleton
    /// class whose method table is copied from the source's singleton class
    /// (so class-level methods survive the dup, per Ruby semantics).
    pub fn duplicate(source: &Rc<Class>) -> Class {
        let copy = Class {
            name: String::new(),
            assigned_name: RefCell::new(None),
            superclass: source.superclass.clone(),
            methods: RefCell::new(source.methods.borrow().clone()),
            instance_variables: RefCell::new(source.instance_variables.borrow().clone()),
            class_variables: RefCell::new(
                source
                    .class_variables
                    .borrow()
                    .iter()
                    .filter(|(k, _)| {
                        !k.starts_with("__singleton__") && !k.starts_with("__attached__")
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
            mixins: RefCell::new(source.mixins.borrow().clone()),
            private_method_names: RefCell::new(source.private_method_names.borrow().clone()),
            singleton_class: RefCell::new(None),
        };
        if let Some(src_sc) = source.singleton_class.borrow().as_ref() {
            let sc_copy = Rc::new(Class {
                name: format!("#<Class:{}>", copy.name),
                assigned_name: RefCell::new(None),
                superclass: src_sc.superclass.clone(),
                methods: RefCell::new(src_sc.methods.borrow().clone()),
                instance_variables: RefCell::new(src_sc.instance_variables.borrow().clone()),
                class_variables: RefCell::new(
                    src_sc
                        .class_variables
                        .borrow()
                        .iter()
                        .filter(|(k, _)| *k != "__attached__")
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                ),
                mixins: RefCell::new(src_sc.mixins.borrow().clone()),
                private_method_names: RefCell::new(src_sc.private_method_names.borrow().clone()),
                singleton_class: RefCell::new(None),
            });
            *copy.singleton_class.borrow_mut() = Some(sc_copy);
        }
        copy
    }
}

impl Clone for Class {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            assigned_name: RefCell::new(self.assigned_name.borrow().clone()),
            superclass: self.superclass.clone(),
            methods: RefCell::new(self.methods.borrow().clone()),
            instance_variables: RefCell::new(self.instance_variables.borrow().clone()),
            class_variables: RefCell::new(self.class_variables.borrow().clone()),
            mixins: RefCell::new(self.mixins.borrow().clone()),
            private_method_names: RefCell::new(self.private_method_names.borrow().clone()),
            singleton_class: RefCell::new(self.singleton_class.borrow().clone()),
        }
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        let self_super = self.superclass.as_ref().map(Rc::as_ptr);
        let other_super = other.superclass.as_ref().map(Rc::as_ptr);
        if self_super != other_super {
            return false;
        }

        {
            let vars = self.instance_variables.borrow();
            let other_vars = other.instance_variables.borrow();
            if *vars != *other_vars {
                return false;
            }
        }

        let self_methods = self.methods.borrow();
        let other_methods = other.methods.borrow();
        if self_methods.len() != other_methods.len() {
            return false;
        }
        if self.class_variables.borrow().len() != other.class_variables.borrow().len() {
            return false;
        }

        self_methods.iter().all(|(name, method)| {
            other_methods.get(name).is_some_and(|other_method| {
                Rc::ptr_eq(method, other_method) || method == other_method
            })
        }) && {
            let class_vars = self.class_variables.borrow();
            let other_class_vars = other.class_variables.borrow();
            class_vars
                .iter()
                .all(|(name, value)| other_class_vars.get(name) == Some(value))
        }
    }
}

impl Eq for Class {}
