// The method table, the mixin chains, and the lookups over them.

use super::*;

impl Class {
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

    /// Add a prepended module, ahead of the class's own method table.
    pub fn add_prepend(&self, module: Rc<Class>) {
        self.prepends.borrow_mut().insert(0, module);
    }

    /// Snapshot of the prepend chain (most-recently-prepended first).
    pub fn prepend_chain(&self) -> Vec<Rc<Class>> {
        self.prepends.borrow().clone()
    }

    /// The prepend chain expanded through each prepended module's own
    /// prepends and includes, in lookup order.
    pub fn transitive_prepends(&self) -> Vec<Rc<Class>> {
        let mut chain = Vec::new();
        let mut seen: Vec<*const Class> = Vec::new();
        for prepended in self.prepend_chain() {
            let ptr = Rc::as_ptr(&prepended);
            if seen.contains(&ptr) {
                continue;
            }
            seen.push(ptr);
            for nested in prepended.transitive_prepends() {
                let nested_ptr = Rc::as_ptr(&nested);
                if !seen.contains(&nested_ptr) {
                    seen.push(nested_ptr);
                    chain.push(nested);
                }
            }
            chain.push(Rc::clone(&prepended));
            for mixin in prepended.transitive_mixins() {
                let mixin_ptr = Rc::as_ptr(&mixin);
                if !seen.contains(&mixin_ptr) {
                    seen.push(mixin_ptr);
                    chain.push(mixin);
                }
            }
        }
        chain
    }

    /// True when `module` has already been prepended here.
    pub fn has_prepend(&self, module: &Rc<Class>) -> bool {
        self.prepends
            .borrow()
            .iter()
            .any(|prepended| Rc::ptr_eq(prepended, module))
    }

    /// The mixin chain expanded through each module's own mixins, in method
    /// and constant lookup order. A module included into an already-included
    /// module shows up here without the outer class being touched.
    pub fn transitive_mixins(&self) -> Vec<Rc<Class>> {
        let mut chain = Vec::new();
        let mut seen: Vec<*const Class> = Vec::new();
        let mut pending: Vec<Rc<Class>> = self.mixin_chain();
        while !pending.is_empty() {
            let module = pending.remove(0);
            let ptr = Rc::as_ptr(&module);
            if seen.contains(&ptr) {
                continue;
            }
            seen.push(ptr);
            let nested = module.mixin_chain();
            chain.push(module);
            pending.splice(0..0, nested);
        }
        chain
    }

    /// Look up a method in this class's own table, without consulting mixins
    /// or the superclass chain.
    pub fn find_own_method(&self, name: &str) -> Option<Rc<Method>> {
        self.methods.borrow().get(name).map(Rc::clone)
    }

    /// A method written `def self.name` lives in the class's own table under
    /// a prefix rather than in the singleton class. Reading it back here lets
    /// the singleton class answer for it the way Ruby's does.
    pub fn find_attached_class_method(&self, name: &str) -> Option<Rc<Method>> {
        if !self.is_singleton_class() || name.starts_with("__class__") {
            return None;
        }
        match self.get_class_var("__attached__") {
            Some(crate::object::Object::Class(attached))
            | Some(crate::object::Object::Module(attached)) => {
                attached.find_own_method(&format!("__class__{}", name))
            }
            _ => None,
        }
    }

    /// Look up a method by walking the inheritance chain (own → mixins → superclass).
    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        for prepended in self.prepends.borrow().iter() {
            if let Some(method) = prepended.find_method(name) {
                return Some(method);
            }
        }
        if let Some(method) = self.methods.borrow().get(name) {
            return Some(Rc::clone(method));
        }
        if let Some(method) = self.find_attached_class_method(name) {
            return Some(method);
        }

        for mixin in self.mixins.borrow().iter() {
            if let Some(method) = mixin.find_method(name) {
                return Some(method);
            }
        }

        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_method(name))
    }

    /// Look for a method along the superclass chain alone, skipping the
    /// modules mixed in along the way. A module's own methods are not
    /// inherited by whatever includes it, so a class method has to be found
    /// this way rather than through the full ancestry.
    pub fn find_inherited_method(&self, name: &str) -> Option<Rc<Method>> {
        if let Some(method) = self.methods.borrow().get(name) {
            return Some(Rc::clone(method));
        }
        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_inherited_method(name))
    }

    /// Look up a method the same way `find_method` does, returning the module
    /// that actually defines it alongside the method itself.
    pub fn find_method_with_owner(self: &Rc<Class>, name: &str) -> Option<(Rc<Class>, Rc<Method>)> {
        for prepended in self.prepends.borrow().iter() {
            if let Some(found) = prepended.find_method_with_owner(name) {
                return Some(found);
            }
        }
        if let Some(method) = self.methods.borrow().get(name) {
            return Some((Rc::clone(self), Rc::clone(method)));
        }
        if let Some(method) = self.find_attached_class_method(name) {
            return Some((Rc::clone(self), method));
        }

        for mixin in self.mixins.borrow().iter() {
            if let Some(found) = mixin.find_method_with_owner(name) {
                return Some(found);
            }
        }

        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_method_with_owner(name))
    }

    /// True when this class object is a singleton class (created by
    /// `class << obj` or `obj.singleton_class`).
    pub fn is_singleton_class(&self) -> bool {
        self.get_class_var("__singleton__").is_some()
    }

    /// True when `other` appears in this module's ancestry (itself, its
    /// mixins, or any superclass and that superclass's mixins).
    pub fn has_ancestor(self: &Rc<Class>, other: &Rc<Class>) -> bool {
        if Rc::ptr_eq(self, other) {
            return true;
        }
        for prepended in self.prepends.borrow().iter() {
            if prepended.has_ancestor(other) {
                return true;
            }
        }
        for mixin in self.mixins.borrow().iter() {
            if mixin.has_ancestor(other) {
                return true;
            }
        }
        self.superclass
            .as_ref()
            .is_some_and(|superclass| superclass.has_ancestor(other))
    }

    /// Remove a method defined directly on this class.
    /// Returns true if the method was found and removed, false otherwise.
    pub fn remove_method(&self, name: &str) -> bool {
        self.methods.borrow_mut().remove(name).is_some()
    }

    /// Create an alias for an existing method.
    /// Returns true if the source method was found and aliased, false otherwise.
    /// The alias inherits the original method's visibility — if `old_name` is
    /// private anywhere in the lookup chain, `new_name` is marked private on
    /// `self` too (matching MRI's behavior for `alias_method`).
    pub fn alias_method(self: &Rc<Class>, new_name: &str, old_name: &str) -> bool {
        if let Some(method) = self.find_method(old_name) {
            let mut aliased = (*method).clone();
            aliased.original_name = Some(
                method
                    .original_name
                    .clone()
                    .unwrap_or_else(|| method.name.clone()),
            );
            aliased.name = new_name.to_string();
            aliased.alias_origin = method
                .alias_origin
                .clone()
                .or_else(|| method.owner_class.clone());
            // The alias belongs to the class that made it, even when the
            // method it copies came from a prepended or included module.
            aliased.owner_class = Some(Rc::clone(self));
            aliased.owner = Some(self.ruby_name());
            self.methods
                .borrow_mut()
                .insert(new_name.to_string(), std::rc::Rc::new(aliased));
            if self.is_method_protected_in_chain(old_name) {
                self.set_method_protected(new_name.to_string());
            } else if self.is_method_private_in_chain(old_name) {
                self.set_method_private(new_name.to_string());
            }
            return true;
        }
        // Singleton class of a class/module: `def self.x` methods live on
        // the attached class under the `__class__` prefix, not on the
        // singleton class itself. Alias from there so e.g. mspec's mock
        // installer can save a class method aside.
        if self.get_class_var("__singleton__").is_some()
            && let Some(Object::Class(attached) | Object::Module(attached)) =
                self.get_class_var("__attached__")
            && let Some(method) = attached.find_method(&format!("__class__{}", old_name))
        {
            self.methods
                .borrow_mut()
                .insert(new_name.to_string(), method);
            return true;
        }
        false
    }

    /// Check whether a method is marked protected anywhere on this class or
    /// its ancestor chain. An explicit `public :name` override on a closer
    /// class shadows a more distant protected marking.
    fn is_method_protected_in_chain(&self, name: &str) -> bool {
        if self.has_public_override(name) {
            return false;
        }
        if self.is_method_protected(name) {
            return true;
        }
        for mixin in self.mixins.borrow().iter() {
            if mixin.has_public_override(name) {
                return false;
            }
            if mixin.is_method_protected(name) {
                return true;
            }
        }
        self.superclass
            .as_ref()
            .is_some_and(|sc| sc.is_method_protected_in_chain(name))
    }

    /// Check whether a method is marked private anywhere on this class or its
    /// ancestor chain (mixins + superclasses). An explicit `public :name`
    /// override on a closer class shadows a more distant private marking.
    fn is_method_private_in_chain(&self, name: &str) -> bool {
        if self.has_public_override(name) {
            return false;
        }
        if self.is_method_private(name) {
            return true;
        }
        for mixin in self.mixins.borrow().iter() {
            if mixin.has_public_override(name) {
                return false;
            }
            if mixin.is_method_private(name) {
                return true;
            }
        }
        if let Some(sc) = &self.superclass {
            return sc.is_method_private_in_chain(name);
        }
        false
    }

    /// Return a list of method names defined directly on this class.
    pub fn method_names(&self) -> Vec<String> {
        let mut names = self.methods.borrow().keys().cloned().collect::<Vec<_>>();
        // A singleton class answers for the `def self.name` methods its
        // attached class holds under a prefix.
        if self.is_singleton_class()
            && let Some(
                crate::object::Object::Class(attached) | crate::object::Object::Module(attached),
            ) = self.get_class_var("__attached__")
        {
            for held in attached.methods.borrow().keys() {
                if let Some(plain) = held.strip_prefix("__class__")
                    && !names.iter().any(|name| name == plain)
                {
                    names.push(plain.to_string());
                }
            }
        }
        names.sort();
        names
    }
}
