// The visibility a class records for each of its method names.

use super::*;

impl Class {
    /// Set the default visibility for subsequent method definitions in this
    /// class body. Called by bare `private`/`public`/`protected` directives.
    pub fn set_current_visibility(&self, v: impl Into<String>) {
        *self.current_visibility.borrow_mut() = v.into();
    }

    /// Read the current visibility (used by `define_method` to auto-mark new
    /// methods as private when the body has switched to that mode).
    pub fn current_visibility(&self) -> String {
        self.current_visibility.borrow().clone()
    }

    /// Mark this class/module as frozen.
    pub fn freeze(&self) {
        self.frozen.set(true);
    }

    /// Mark this class/module as no longer frozen.
    pub fn thaw(&self) {
        self.frozen.set(false);
    }

    /// Whether this class/module is frozen.
    pub fn is_frozen(&self) -> bool {
        self.frozen.get()
    }

    /// Register `child` as a direct subclass of `self`. Stored weakly so it
    /// can be garbage-collected; `subclasses()` filters out dead refs.
    pub fn add_subclass(&self, child: &Rc<Class>) {
        self.subclasses.borrow_mut().push(Rc::downgrade(child));
    }

    /// Return the live direct subclasses of this class.
    pub fn subclasses(&self) -> Vec<Rc<Class>> {
        let mut subs = self.subclasses.borrow_mut();
        subs.retain(|w| w.strong_count() > 0);
        subs.iter().filter_map(|w| w.upgrade()).collect()
    }

    /// Accessor for the cached singleton-class slot (None until materialized).
    pub fn singleton_class_slot(&self) -> std::cell::Ref<'_, Option<Rc<Class>>> {
        self.singleton_class.borrow()
    }

    /// Install a singleton class on this class. Cached for subsequent access.
    pub fn set_singleton_class(&self, class: Rc<Class>) {
        *self.singleton_class.borrow_mut() = Some(class);
    }

    /// Mark a method name as private on this class, clearing any earlier
    /// `public :name` override so the newer declaration wins.
    pub fn set_method_private(&self, name: impl Into<String>) {
        let name = name.into();
        self.public_overrides.borrow_mut().remove(&name);
        self.protected_method_names.borrow_mut().remove(&name);
        self.private_method_names.borrow_mut().insert(name);
    }

    /// Mark a method name as protected on this class, clearing any earlier
    /// `public :name` override so the newer declaration wins.
    pub fn set_method_protected(&self, name: impl Into<String>) {
        let name = name.into();
        self.public_overrides.borrow_mut().remove(&name);
        self.private_method_names.borrow_mut().remove(&name);
        self.protected_method_names.borrow_mut().insert(name);
    }

    /// Check if a method is marked protected on this class (own table only).
    pub fn is_method_protected(&self, name: &str) -> bool {
        self.protected_method_names.borrow().contains(name)
    }

    /// The protected method names defined directly on this class.
    pub fn protected_method_names(&self) -> Vec<String> {
        let mut names = self
            .protected_method_names
            .borrow()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Whether this class records a visibility of its own for `name`, set by
    /// `private`/`protected`/`public` naming a method it may only inherit.
    pub fn has_visibility_marking(&self, name: &str) -> bool {
        self.private_method_names.borrow().contains(name)
            || self.protected_method_names.borrow().contains(name)
            || self.public_overrides.borrow().contains(name)
    }

    /// The names this class marks a visibility for without defining, which
    /// Ruby reports among its own instance methods.
    pub fn visibility_marked_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .private_method_names
            .borrow()
            .iter()
            .chain(self.protected_method_names.borrow().iter())
            .chain(self.public_overrides.borrow().iter())
            .cloned()
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// Drop any visibility marking for `name`, which is what redefining a
    /// method does when the enclosing body's default visibility is public.
    pub fn clear_method_visibility(&self, name: &str) {
        self.private_method_names.borrow_mut().remove(name);
        self.protected_method_names.borrow_mut().remove(name);
    }

    /// Whether `name` may not be called with an explicit receiver from
    /// outside the class. Private and protected share that restriction.
    pub fn is_method_restricted(&self, name: &str) -> bool {
        self.is_method_private(name) || self.is_method_protected(name)
    }

    /// Mark a method name as public on this class (removes private flag and
    /// records an explicit public override so an inherited private status is
    /// shadowed).
    pub fn set_method_public(&self, name: &str) {
        self.private_method_names.borrow_mut().remove(name);
        self.protected_method_names.borrow_mut().remove(name);
        self.public_overrides.borrow_mut().insert(name.to_string());
    }

    /// Whether this class has an explicit public override for `name` (set via
    /// `public :name`). Used to short-circuit ancestor private checks.
    pub fn has_public_override(&self, name: &str) -> bool {
        self.public_overrides.borrow().contains(name)
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
}
