//! Runtime class representation for Metorex
//! Handles method tables, inheritance, and instance variable declarations.

use crate::object::{Method, Object};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

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
    /// Class variables, constants, and class-level bookkeeping keyed by name.
    /// Insertion-ordered so `Module#class_variables` reports definition order.
    class_variables: RefCell<IndexMap<String, crate::object::Object>>,
    /// Included modules, in reverse inclusion order (last included = first searched).
    mixins: RefCell<Vec<Rc<Class>>>,
    /// Names of methods whose visibility has been set to private.
    private_method_names: RefCell<HashSet<String>>,
    /// Methods explicitly marked public on this class via `public :name` —
    /// distinct from "no entry" so we can override an inherited private
    /// without adding the method itself to this class's method table.
    public_overrides: RefCell<HashSet<String>>,
    /// Lazily-allocated singleton class attached to *this class object* — the
    /// thing `class << SomeClass; end` opens. Holds class-level (def self.x)
    /// methods once we start tracking them as a real class.
    singleton_class: RefCell<Option<Rc<Class>>>,
    /// Direct subclasses (weak refs to avoid keeping garbage subclasses alive).
    subclasses: RefCell<Vec<Weak<Class>>>,
    /// Frozen flag — once true, mutating methods (alias_method, define_method,
    /// include, etc.) raise FrozenError.
    frozen: std::cell::Cell<bool>,
    /// Current visibility state set by bare `private`/`public`/`protected`
    /// inside the class body. Methods defined while this is `private` are
    /// auto-marked as private. `protected` is treated like `private` for now.
    current_visibility: RefCell<String>,
    /// `autoload :Const, "path"` registry. Maps constant name → unresolved
    /// path string (the value Ruby's `autoload?` returns verbatim).
    autoloads: RefCell<HashMap<String, String>>,
    /// Constant names that *were* registered as autoloads, fired, and
    /// completed loading without actually defining the constant. MRI keeps
    /// these visible in `Module#constants` for a while afterward (so the
    /// name persists as an "unrealized" constant) even though `autoload?`
    /// and `const_defined?` both report nothing. Storing them separately
    /// here keeps `#constants` in sync without resurrecting the autoload.
    unrealized_autoloads: RefCell<HashSet<String>>,
    /// Constant names marked private via `Module#private_constant`. When
    /// any of these is referenced via qualified `Mod::Const` access from
    /// outside the module's lexical scope, MRI raises NameError /private
    /// constant/.
    private_constants: RefCell<HashSet<String>>,
    /// Source location (file, line) of each `autoload :Const, ...` call.
    /// Returned by `Module#const_source_location` when an autoload is
    /// pending or when another thread asks during a load.
    autoload_locations: RefCell<HashMap<String, (String, i64)>>,
    /// Source location (file, line) of each constant assignment that
    /// produced a class_var on this class — `class Foo; X = 1; end`,
    /// `class Foo; class Bar; end; end`, etc. Returned by
    /// `Module#const_source_location` once the constant is bound.
    const_locations: RefCell<HashMap<String, (String, i64)>>,
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
            class_variables: RefCell::new(IndexMap::new()),
            mixins: RefCell::new(Vec::new()),
            private_method_names: RefCell::new(HashSet::new()),
            public_overrides: RefCell::new(HashSet::new()),
            singleton_class: RefCell::new(None),
            subclasses: RefCell::new(Vec::new()),
            frozen: std::cell::Cell::new(false),
            current_visibility: RefCell::new("public".to_string()),
            autoloads: RefCell::new(HashMap::new()),
            unrealized_autoloads: RefCell::new(HashSet::new()),
            private_constants: RefCell::new(HashSet::new()),
            autoload_locations: RefCell::new(HashMap::new()),
            const_locations: RefCell::new(HashMap::new()),
        }
    }

    /// Record where an autoload was registered (for
    /// `Module#const_source_location`).
    pub fn set_autoload_location(&self, name: impl Into<String>, file: String, line: i64) {
        self.autoload_locations
            .borrow_mut()
            .insert(name.into(), (file, line));
    }

    /// Source location of an autoload registration, if any.
    pub fn get_autoload_location(&self, name: &str) -> Option<(String, i64)> {
        self.autoload_locations.borrow().get(name).cloned()
    }

    /// Record where a constant was actually defined (for
    /// `Module#const_source_location`).
    pub fn set_const_location(&self, name: impl Into<String>, file: String, line: i64) {
        self.const_locations
            .borrow_mut()
            .insert(name.into(), (file, line));
    }

    /// Source location of a defined constant on this class, if any.
    pub fn get_const_location(&self, name: &str) -> Option<(String, i64)> {
        self.const_locations.borrow().get(name).cloned()
    }

    /// Forget the source location for a constant. Called by `remove_const` so
    /// a follow-up `autoload` registration's location surfaces through
    /// `Module#const_source_location` instead of the now-removed constant's.
    pub fn remove_const_location(&self, name: &str) {
        self.const_locations.borrow_mut().remove(name);
    }

    /// Mark `name` as a private constant — qualified access from outside
    /// the class raises NameError /private constant/.
    pub fn mark_private_constant(&self, name: impl Into<String>) {
        self.private_constants.borrow_mut().insert(name.into());
    }

    /// Whether `name` is marked private on this class.
    pub fn is_private_constant(&self, name: &str) -> bool {
        self.private_constants.borrow().contains(name)
    }

    /// Remove the private flag from `name` (the inverse of
    /// `mark_private_constant`). Used by `Module#public_constant`.
    pub fn unmark_private_constant(&self, name: &str) {
        self.private_constants.borrow_mut().remove(name);
    }

    /// Mark `name` as an autoload that fired, loaded its file, and didn't
    /// produce the constant. MRI keeps the name in `#constants` afterward.
    pub fn mark_unrealized_autoload(&self, name: impl Into<String>) {
        self.unrealized_autoloads.borrow_mut().insert(name.into());
    }

    /// Names of unrealized autoloads on this class.
    pub fn unrealized_autoload_names(&self) -> Vec<String> {
        self.unrealized_autoloads.borrow().iter().cloned().collect()
    }

    /// Drop the unrealized-autoload bookkeeping for `name`.
    pub fn clear_unrealized_autoload(&self, name: &str) {
        self.unrealized_autoloads.borrow_mut().remove(name);
    }

    /// Register an autoload mapping. `path` is stored verbatim (Ruby's
    /// `autoload?` returns whatever was passed in).
    pub fn set_autoload(&self, name: impl Into<String>, path: impl Into<String>) {
        self.autoloads.borrow_mut().insert(name.into(), path.into());
    }

    /// Return the autoload path registered on *this* class (no recursion).
    pub fn get_autoload(&self, name: &str) -> Option<String> {
        self.autoloads.borrow().get(name).cloned()
    }

    /// Look up an autoload across this class and its ancestor chain
    /// (superclass + included mixins). Mirrors Ruby's `autoload?` recursion.
    pub fn lookup_autoload(&self, name: &str) -> Option<String> {
        if let Some(p) = self.get_autoload(name) {
            return Some(p);
        }
        for mixin in self.mixins.borrow().iter() {
            if let Some(p) = mixin.lookup_autoload(name) {
                return Some(p);
            }
        }
        if let Some(sc) = &self.superclass {
            return sc.lookup_autoload(name);
        }
        None
    }

    /// Remove the autoload entry for `name`, returning the previous path.
    pub fn remove_autoload(&self, name: &str) -> Option<String> {
        self.autoloads.borrow_mut().remove(name)
    }

    /// All autoload-registered names on this class (no recursion). Listed by
    /// `Module#constants` even before the autoload fires, per Ruby semantics.
    pub fn autoload_names(&self) -> Vec<String> {
        self.autoloads.borrow().keys().cloned().collect()
    }

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

    /// Mark a method name as private on this class.
    pub fn set_method_private(&self, name: impl Into<String>) {
        self.private_method_names.borrow_mut().insert(name.into());
    }

    /// Mark a method name as public on this class (removes private flag and
    /// records an explicit public override so an inherited private status is
    /// shadowed).
    pub fn set_method_public(&self, name: &str) {
        self.private_method_names.borrow_mut().remove(name);
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

    /// Return the class name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Ruby-visible name: returns the original (or assigned) name, or the
    /// empty string for truly-anonymous classes. The `name` native method
    /// maps this to `nil` for anonymous classes.
    pub fn ruby_name(&self) -> String {
        if let Some(assigned) = self.assigned_name.borrow().as_ref() {
            return assigned.clone();
        }
        self.name.clone()
    }

    /// Inspect-style label used when an anonymous class needs a printable
    /// identifier — e.g. when it acts as the namespace in `parent::C`, we
    /// synthesise `#<Class:0x<ptr>>::C` for the subclass's name.
    pub fn inspect_name(&self) -> String {
        let rn = self.ruby_name();
        if rn.is_empty() {
            format!("#<Class:0x{:016x}>", self as *const Class as usize)
        } else {
            rn
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
    /// The alias inherits the original method's visibility — if `old_name` is
    /// private anywhere in the lookup chain, `new_name` is marked private on
    /// `self` too (matching MRI's behavior for `alias_method`).
    pub fn alias_method(&self, new_name: &str, old_name: &str) -> bool {
        if let Some(method) = self.find_method(old_name) {
            self.methods
                .borrow_mut()
                .insert(new_name.to_string(), method);
            if self.is_method_private_in_chain(old_name) {
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
        names.sort();
        names
    }

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
            public_overrides: RefCell::new(source.public_overrides.borrow().clone()),
            singleton_class: RefCell::new(None),
            subclasses: RefCell::new(Vec::new()),
            frozen: std::cell::Cell::new(false),
            current_visibility: RefCell::new("public".to_string()),
            autoloads: RefCell::new(source.autoloads.borrow().clone()),
            unrealized_autoloads: RefCell::new(source.unrealized_autoloads.borrow().clone()),
            private_constants: RefCell::new(source.private_constants.borrow().clone()),
            autoload_locations: RefCell::new(source.autoload_locations.borrow().clone()),
            const_locations: RefCell::new(source.const_locations.borrow().clone()),
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
                public_overrides: RefCell::new(src_sc.public_overrides.borrow().clone()),
                singleton_class: RefCell::new(None),
                subclasses: RefCell::new(Vec::new()),
                frozen: std::cell::Cell::new(false),
                current_visibility: RefCell::new("public".to_string()),
                autoloads: RefCell::new(src_sc.autoloads.borrow().clone()),
                unrealized_autoloads: RefCell::new(src_sc.unrealized_autoloads.borrow().clone()),
                private_constants: RefCell::new(src_sc.private_constants.borrow().clone()),
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
            superclass: self.superclass.clone(),
            methods: RefCell::new(self.methods.borrow().clone()),
            instance_variables: RefCell::new(self.instance_variables.borrow().clone()),
            class_variables: RefCell::new(self.class_variables.borrow().clone()),
            mixins: RefCell::new(self.mixins.borrow().clone()),
            private_method_names: RefCell::new(self.private_method_names.borrow().clone()),
            public_overrides: RefCell::new(self.public_overrides.borrow().clone()),
            singleton_class: RefCell::new(self.singleton_class.borrow().clone()),
            subclasses: RefCell::new(self.subclasses.borrow().clone()),
            frozen: std::cell::Cell::new(self.frozen.get()),
            current_visibility: RefCell::new(self.current_visibility.borrow().clone()),
            autoloads: RefCell::new(self.autoloads.borrow().clone()),
            unrealized_autoloads: RefCell::new(self.unrealized_autoloads.borrow().clone()),
            private_constants: RefCell::new(self.private_constants.borrow().clone()),
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
