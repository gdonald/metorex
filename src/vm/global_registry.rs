//! Global object registry for the Metorex virtual machine.
//!
//! This module provides a registry that owns global objects accessible throughout the VM,
//! including built-in classes and singleton values.

use crate::object::Object;
use std::collections::{BTreeSet, HashMap};

/// Registry that owns global objects accessible throughout the VM.
#[derive(Debug, Default)]
pub struct GlobalRegistry {
    objects: HashMap<String, Object>,
    /// The subset of names that are Ruby global variables rather than
    /// constants, classes, or native functions, which share this registry.
    variable_names: BTreeSet<String>,
}

impl GlobalRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a named global object.
    pub fn set(&mut self, name: impl Into<String>, object: Object) {
        self.objects.insert(name.into(), object);
    }

    /// Insert or replace a global *variable*, recording its name so
    /// `global_variables` can report it.
    pub fn set_variable(&mut self, name: impl Into<String>, object: Object) {
        let name = name.into();
        self.variable_names.insert(name.clone());
        self.objects.insert(name, object);
    }

    /// The object a bare constant name stands for. A global variable is held
    /// here under its name with the `$` dropped, so one is never answered as
    /// a constant: `$DEBUG` and `DEBUG` are two different names in Ruby.
    pub fn constant(&self, name: &str) -> Option<Object> {
        if self.variable_names.contains(name) {
            return None;
        }
        self.objects.get(name).cloned()
    }

    /// The global variable names, without their `$` sigil, in sorted order.
    pub fn variable_names(&self) -> impl Iterator<Item = &String> {
        self.variable_names.iter()
    }

    /// Fetch a named global object if present.
    /// Remove a named global object, returning it when present.
    pub fn remove(&mut self, name: &str) -> Option<Object> {
        self.objects.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.objects.get(name).cloned()
    }

    /// Determine whether a name exists in the registry.
    pub fn contains(&self, name: &str) -> bool {
        self.objects.contains_key(name)
    }

    /// Iterator over registered globals (useful for seeding environments).
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Object)> {
        self.objects.iter()
    }
}
