// The class variables a class holds, and the copies it hands back.

use super::*;

impl Class {
    /// Set a class variable on this class. Setting an uppercase-named
    /// constant clears any "unrealized autoload" bookkeeping for that
    /// name. The autoload entry itself is NOT cleared here — when the
    /// constant is being defined inside a const-access autoload
    /// trigger, leaving the autoload entry alive lets concurrent
    /// `autoload?` queries from other threads still see the path.
    /// `try_autoload_constant` removes the entry after the load
    /// completes; assignments outside of an autoload (e.g. plain
    /// `class M; X = 1; end`) need to drop the autoload too, which is
    /// handled by an explicit `remove_autoload` call where appropriate.
    pub fn set_class_var(&self, name: impl Into<String>, value: Object) {
        let key: String = name.into();
        if key.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            self.unrealized_autoloads.borrow_mut().remove(&key);
        }
        self.class_variables.borrow_mut().insert(key, value);
    }

    /// Retrieve a class variable from this class.
    pub fn get_class_var(&self, name: &str) -> Option<Object> {
        self.class_variables.borrow().get(name).cloned()
    }

    /// List all class-variable names (used by refinement bookkeeping).
    /// What the class holds under its own names, whatever those names are.
    pub fn class_variable_values(&self) -> Vec<Object> {
        self.class_variables.borrow().values().cloned().collect()
    }

    pub fn class_var_names(&self) -> Vec<String> {
        self.class_variables.borrow().keys().cloned().collect()
    }

    /// Names of true class variables (`@@name`) defined directly on this
    /// class/module, in definition order. The shared storage also holds
    /// constants (uppercase keys), class-level instance variables (`@name`),
    /// and internal bookkeeping (`__name__`); those are excluded.
    pub fn own_class_variable_names(&self) -> Vec<String> {
        self.class_variables
            .borrow()
            .keys()
            .filter(|key| {
                let first = key.chars().next();
                !key.starts_with('@')
                    && !key.starts_with("__")
                    && !first.is_some_and(|c| c.is_ascii_uppercase())
            })
            .cloned()
            .collect()
    }

    /// Names of class variables visible from this class, walking included
    /// modules then the superclass (mirroring `lookup_class_var`). Definition
    /// order is preserved and names already seen on a more-derived ancestor
    /// are not repeated.
    pub fn inherited_class_variable_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut seen = HashSet::new();
        self.collect_class_variable_names(&mut names, &mut seen);
        names
    }

    fn collect_class_variable_names(&self, names: &mut Vec<String>, seen: &mut HashSet<String>) {
        for name in self.own_class_variable_names() {
            if seen.insert(name.clone()) {
                names.push(name);
            }
        }
        for mixin in self.mixins.borrow().iter() {
            mixin.collect_class_variable_names(names, seen);
        }
        if let Some(superclass) = self.superclass.as_ref() {
            superclass.collect_class_variable_names(names, seen);
        }
    }

    /// Resolve a class variable across the ancestor chain: this class first,
    /// then its included modules, then its superclass (recursively). Mirrors
    /// Ruby's class-variable lookup, which walks included modules but ignores
    /// extended (singleton) ones.
    pub fn lookup_class_var(&self, name: &str) -> Option<Object> {
        if let Some(value) = self.class_variables.borrow().get(name) {
            return Some(value.clone());
        }
        for mixin in self.mixins.borrow().iter() {
            if let Some(value) = mixin.lookup_class_var(name) {
                return Some(value);
            }
        }
        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.lookup_class_var(name))
    }

    /// Remove a class variable/constant by name, returning the previous value
    /// if there was one. Used by `Module#remove_const`.
    pub fn remove_class_var(&self, name: &str) -> Option<crate::object::Object> {
        self.class_variables.borrow_mut().shift_remove(name)
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
            temporary_name: RefCell::new(None),
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
            prepends: RefCell::new(source.prepends.borrow().clone()),
            module_flag: std::cell::Cell::new(source.module_flag.get()),
            private_method_names: RefCell::new(source.private_method_names.borrow().clone()),
            protected_method_names: RefCell::new(source.protected_method_names.borrow().clone()),
            public_overrides: RefCell::new(source.public_overrides.borrow().clone()),
            singleton_class: RefCell::new(None),
            subclasses: RefCell::new(Vec::new()),
            frozen: std::cell::Cell::new(false),
            current_visibility: RefCell::new("public".to_string()),
            autoloads: RefCell::new(source.autoloads.borrow().clone()),
            unrealized_autoloads: RefCell::new(source.unrealized_autoloads.borrow().clone()),
            private_constants: RefCell::new(source.private_constants.borrow().clone()),
            deprecated_constants: RefCell::new(source.deprecated_constants.borrow().clone()),
            autoload_locations: RefCell::new(source.autoload_locations.borrow().clone()),
            const_locations: RefCell::new(source.const_locations.borrow().clone()),
        };
        if let Some(src_sc) = source.singleton_class.borrow().as_ref() {
            let sc_copy = Rc::new(Class {
                name: format!("#<Class:{}>", copy.name),
                assigned_name: RefCell::new(None),
                temporary_name: RefCell::new(None),
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
                prepends: RefCell::new(src_sc.prepends.borrow().clone()),
                module_flag: std::cell::Cell::new(src_sc.module_flag.get()),
                private_method_names: RefCell::new(src_sc.private_method_names.borrow().clone()),
                protected_method_names: RefCell::new(
                    src_sc.protected_method_names.borrow().clone(),
                ),
                public_overrides: RefCell::new(src_sc.public_overrides.borrow().clone()),
                singleton_class: RefCell::new(None),
                subclasses: RefCell::new(Vec::new()),
                frozen: std::cell::Cell::new(false),
                current_visibility: RefCell::new("public".to_string()),
                autoloads: RefCell::new(src_sc.autoloads.borrow().clone()),
                unrealized_autoloads: RefCell::new(src_sc.unrealized_autoloads.borrow().clone()),
                private_constants: RefCell::new(src_sc.private_constants.borrow().clone()),
                deprecated_constants: RefCell::new(src_sc.deprecated_constants.borrow().clone()),
                autoload_locations: RefCell::new(src_sc.autoload_locations.borrow().clone()),
                const_locations: RefCell::new(src_sc.const_locations.borrow().clone()),
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
            temporary_name: RefCell::new(self.temporary_name.borrow().clone()),
            superclass: self.superclass.clone(),
            methods: RefCell::new(self.methods.borrow().clone()),
            instance_variables: RefCell::new(self.instance_variables.borrow().clone()),
            class_variables: RefCell::new(self.class_variables.borrow().clone()),
            mixins: RefCell::new(self.mixins.borrow().clone()),
            prepends: RefCell::new(self.prepends.borrow().clone()),
            module_flag: std::cell::Cell::new(self.module_flag.get()),
            private_method_names: RefCell::new(self.private_method_names.borrow().clone()),
            protected_method_names: RefCell::new(self.protected_method_names.borrow().clone()),
            public_overrides: RefCell::new(self.public_overrides.borrow().clone()),
            singleton_class: RefCell::new(self.singleton_class.borrow().clone()),
            subclasses: RefCell::new(self.subclasses.borrow().clone()),
            frozen: std::cell::Cell::new(self.frozen.get()),
            current_visibility: RefCell::new(self.current_visibility.borrow().clone()),
            autoloads: RefCell::new(self.autoloads.borrow().clone()),
            unrealized_autoloads: RefCell::new(self.unrealized_autoloads.borrow().clone()),
            private_constants: RefCell::new(self.private_constants.borrow().clone()),
            deprecated_constants: RefCell::new(self.deprecated_constants.borrow().clone()),
            autoload_locations: RefCell::new(self.autoload_locations.borrow().clone()),
            const_locations: RefCell::new(self.const_locations.borrow().clone()),
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
