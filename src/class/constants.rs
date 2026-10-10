// The constants a class holds, and the autoloads registered on it.

use super::*;

impl Class {
    /// Make a class with an optional superclass, recorded among the objects
    /// still alive.
    pub fn new(name: impl Into<String>, superclass: Option<Rc<Class>>) -> Rc<Self> {
        let made = Rc::new(Self::unrecorded(name, superclass));
        crate::object::live::record_class(&made);
        made
    }

    fn unrecorded(name: impl Into<String>, superclass: Option<Rc<Class>>) -> Self {
        Self {
            name: name.into(),
            assigned_name: RefCell::new(None),
            temporary_name: RefCell::new(None),
            superclass,
            methods: RefCell::new(HashMap::new()),
            instance_variables: RefCell::new(HashSet::new()),
            class_variables: RefCell::new(IndexMap::new()),
            mixins: RefCell::new(Vec::new()),
            prepends: RefCell::new(Vec::new()),
            private_method_names: RefCell::new(HashSet::new()),
            protected_method_names: RefCell::new(HashSet::new()),
            public_overrides: RefCell::new(HashSet::new()),
            singleton_class: RefCell::new(None),
            subclasses: RefCell::new(Vec::new()),
            module_flag: std::cell::Cell::new(false),
            frozen: std::cell::Cell::new(false),
            current_visibility: RefCell::new("public".to_string()),
            autoloads: RefCell::new(HashMap::new()),
            unrealized_autoloads: RefCell::new(HashSet::new()),
            private_constants: RefCell::new(HashSet::new()),
            deprecated_constants: RefCell::new(HashSet::new()),
            autoload_locations: RefCell::new(HashMap::new()),
            const_locations: RefCell::new(HashMap::new()),
            method_cache: RefCell::new(HashMap::new()),
            method_walks: RefCell::new(HashMap::new()),
        }
    }

    /// Create a module: a class object that reports itself as a module.
    pub fn new_module(name: impl Into<String>) -> Rc<Self> {
        let module = Self::new(name, None);
        module.module_flag.set(true);
        module
    }

    /// Whether this object is a module rather than a class.
    pub fn is_module(&self) -> bool {
        self.module_flag.get()
    }

    /// The word Ruby uses for this object in error messages.
    pub fn kind_name(&self) -> &'static str {
        if self.is_module() { "Module" } else { "Class" }
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

    /// Mark `name` as deprecated — reading it warns but still returns the
    /// value.
    pub fn mark_deprecated_constant(&self, name: impl Into<String>) {
        self.deprecated_constants.borrow_mut().insert(name.into());
    }

    /// Whether `name` is marked deprecated on this class.
    pub fn is_deprecated_constant(&self, name: &str) -> bool {
        self.deprecated_constants.borrow().contains(name)
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
}
