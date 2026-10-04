// Scope and variable management for Metorex
// This module implements lexical scoping with scope chain traversal

use crate::object::Object;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Represents a single scope in the scope chain
/// Each scope can have a parent scope, forming a chain for variable lookup
#[derive(Debug)]
pub struct Scope {
    /// Variable storage: maps variable names to shared mutable references
    /// This allows closures to mutate captured variables
    variables: HashMap<String, Rc<RefCell<Object>>>,

    /// The names in `variables` in the order they were first bound, which is
    /// the order `local_variables` reports them in.
    order: Vec<String>,

    /// Reference to the parent scope (None for global scope)
    parent: Option<Rc<RefCell<Scope>>>,

    /// Whether this scope is a method boundary. A method body cannot see the
    /// caller's locals, so `local_variables` stops here.
    is_method_boundary: bool,

    /// Names carried in from a Binding, in the order that Binding reported
    /// them. Code run through a Binding names its own locals first and these
    /// after, which is the order Ruby reports them in.
    inherited_names: Vec<String>,

    /// Names bound without the program declaring them, which are not locals
    /// of this scope and so are left out of `local_variables`.
    hidden_names: HashSet<String>,

    /// Names bound ahead of the assignment that introduces them, the way
    /// Ruby's parser reserves a local for the rest of the body. A block
    /// written before that assignment does not close over one of these:
    /// Ruby's local does not exist yet where the block was written.
    hoisted_names: HashSet<String>,

    /// The names a block closed over where it was written, shared with every
    /// call of the block. They are read and written through, after this
    /// scope's own names and before its parent's, and are not locals of this
    /// scope.
    captured: Option<CapturedNames>,
}

/// The names a block closes over, each with the cell it shares with the
/// scope it was written in.
pub type CapturedNames = Rc<HashMap<String, Rc<RefCell<Object>>>>;

impl Scope {
    /// Creates a new scope with no parent (global scope)
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            order: Vec::new(),
            parent: None,
            is_method_boundary: false,
            inherited_names: Vec::new(),
            hidden_names: HashSet::new(),
            hoisted_names: HashSet::new(),
            captured: None,
        }
    }

    /// Creates a new scope with the given parent scope
    pub fn with_parent(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            variables: HashMap::new(),
            order: Vec::new(),
            parent: Some(parent),
            is_method_boundary: false,
            inherited_names: Vec::new(),
            hidden_names: HashSet::new(),
            hoisted_names: HashSet::new(),
            captured: None,
        }
    }

    /// Defines a new variable in the current scope
    /// If the variable already exists in this scope, it will be overwritten
    pub fn define(&mut self, name: String, value: Object) {
        self.hidden_names.remove(&name);
        self.hoisted_names.remove(&name);
        self.remember_order(&name);
        self.variables.insert(name, Rc::new(RefCell::new(value)));
    }

    /// Defines a new variable in the current scope with a shared reference
    /// Used when a closure defines a captured variable
    pub fn define_shared(&mut self, name: String, value: Rc<RefCell<Object>>) {
        self.hidden_names.remove(&name);
        self.hoisted_names.remove(&name);
        self.remember_order(&name);
        self.variables.insert(name, value);
    }

    /// Binds a name carried in from a Binding. It behaves like any other
    /// local, and `local_variables` reports it after the scope's own.
    pub fn define_inherited(&mut self, name: String, value: Rc<RefCell<Object>>) {
        self.hidden_names.remove(&name);
        self.hoisted_names.remove(&name);
        if !self.inherited_names.contains(&name) {
            self.inherited_names.push(name.clone());
        }
        self.remember_order(&name);
        self.variables.insert(name, value);
    }

    /// Read and write through the names a block closed over, without
    /// copying them into this scope.
    pub fn attach_captured(&mut self, captured: CapturedNames) {
        self.captured = Some(captured);
    }

    /// The cell a block closed over for `name`, when this scope runs one.
    fn captured_ref(&self, name: &str) -> Option<&Rc<RefCell<Object>>> {
        self.captured.as_ref().and_then(|held| held.get(name))
    }

    /// Binds a name the program did not declare, so `local_variables` and a
    /// Binding leave it out. The implicit `it` parameter is bound this way.
    pub fn define_hidden(&mut self, name: String, value: Object) {
        self.hoisted_names.remove(&name);
        self.hidden_names.insert(name.clone());
        self.remember_order(&name);
        self.variables.insert(name, Rc::new(RefCell::new(value)));
    }

    /// Reserves `name` for the rest of this scope, the way Ruby's parser
    /// does at the assignment that introduces a local. Reading it answers
    /// nil, and a block written before the assignment does not close over it.
    pub fn hoist(&mut self, name: String) {
        self.remember_order(&name);
        self.hoisted_names.insert(name.clone());
        self.variables
            .insert(name, Rc::new(RefCell::new(Object::Nil)));
    }

    /// Say that the assignment `name` was reserved for has begun, so code
    /// written in its value closes over the name.
    pub fn unhoist(&mut self, name: &str) {
        if self.hoisted_names.remove(name) {
            return;
        }
        if let Some(parent) = &self.parent
            && !self.is_method_boundary
        {
            parent.borrow_mut().unhoist(name);
        }
    }

    /// Whether `name` is only reserved for a later assignment and has not
    /// been assigned yet. Ruby's parser introduces a local at the assignment
    /// that writes it, so a block written above that line does not see one.
    pub fn is_only_hoisted(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            return self.hoisted_names.contains(name);
        }
        if self.captured_ref(name).is_some() {
            return false;
        }
        match &self.parent {
            Some(parent) => parent.borrow().is_only_hoisted(name),
            None => false,
        }
    }

    /// Remove a variable from the current scope only (does not walk parents).
    pub fn undefine(&mut self, name: &str) {
        self.variables.remove(name);
        self.order.retain(|held| held != name);
    }

    /// Records a name the first time this scope binds it.
    fn remember_order(&mut self, name: &str) {
        if !self.variables.contains_key(name) {
            self.order.push(name.to_string());
        }
    }

    /// This scope's names in the order they were first bound.
    fn ordered_names(&self) -> impl Iterator<Item = &String> {
        self.order
            .iter()
            .filter(|name| self.variables.contains_key(*name))
    }

    /// Gets a variable value by traversing the scope chain
    /// Returns None if the variable is not found in any scope
    pub fn get(&self, name: &str) -> Option<Object> {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.borrow().clone());
        }
        if let Some(value_ref) = self.captured_ref(name) {
            return Some(value_ref.borrow().clone());
        }

        // If not found, check the parent scope recursively
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        // Variable not found in any scope
        None
    }

    /// Gets a shared reference to a variable by traversing the scope chain
    /// Used for closure capture to enable mutable closures
    pub fn get_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.clone());
        }
        if let Some(value_ref) = self.captured_ref(name) {
            return Some(value_ref.clone());
        }

        // If not found, check the parent scope recursively
        if let Some(parent) = &self.parent {
            return parent.borrow().get_ref(name);
        }

        // Variable not found in any scope
        None
    }

    /// Sets a variable value by traversing the scope chain
    /// Returns true if the variable was found and updated, false otherwise
    /// This method will NOT create a new variable if it doesn't exist
    pub fn set(&mut self, name: &str, value: Object) -> bool {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            *value_ref.borrow_mut() = value;
            // The assignment the parser reserved the name for has now run,
            // so a block written from here on closes over it.
            self.hoisted_names.remove(name);
            return true;
        }
        if let Some(value_ref) = self.captured_ref(name) {
            *value_ref.borrow_mut() = value;
            return true;
        }

        // A method or block scope does not assign to a name it never bound:
        // Ruby makes that a new local of its own. Only the names it captured
        // reach outward, and those are already bound here as shared cells.
        if self.is_method_boundary {
            return false;
        }

        // If not found, try to set it in the parent scope
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set(name, value);
        }

        // Variable not found in any scope
        false
    }

    /// Gets a variable at a specific depth in the scope chain
    /// depth=0 means current scope, depth=1 means parent, etc.
    /// This is useful for closure resolution where we know the exact depth
    pub fn get_at(&self, depth: usize, name: &str) -> Option<Object> {
        if depth == 0 {
            return self
                .variables
                .get(name)
                .or_else(|| self.captured_ref(name))
                .map(|v| v.borrow().clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get_at(depth - 1, name);
        }

        None
    }

    /// Sets a variable at a specific depth in the scope chain
    /// depth=0 means current scope, depth=1 means parent, etc.
    /// Returns true if successful, false if the depth is invalid or variable doesn't exist
    pub fn set_at(&mut self, depth: usize, name: &str, value: Object) -> bool {
        if depth == 0 {
            if let Some(value_ref) = self.variables.get(name).or_else(|| self.captured_ref(name)) {
                *value_ref.borrow_mut() = value;
                return true;
            }
            return false;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set_at(depth - 1, name, value);
        }

        false
    }

    /// Collects all variables from the entire scope chain
    /// Returns a HashMap with all visible variables (parent scope vars may be shadowed)
    pub fn collect_all_vars(&self) -> HashMap<String, Object> {
        let mut all_vars = HashMap::new();

        // Start from parent and work backwards, so that closer scopes override farther ones
        if let Some(parent) = &self.parent {
            all_vars = parent.borrow().collect_all_vars();
        }
        if let Some(captured) = &self.captured {
            for (name, value_ref) in captured.iter() {
                all_vars.insert(name.clone(), value_ref.borrow().clone());
            }
        }

        // Now add this scope's variables (potentially overriding parent values)
        for (name, value_ref) in &self.variables {
            all_vars.insert(name.clone(), value_ref.borrow().clone());
        }

        all_vars
    }

    /// Names bound in this scope alone, excluding those a block captured.
    pub fn own_variable_names(&self) -> Vec<String> {
        self.ordered_names()
            .filter(|name| !self.hidden_names.contains(*name))
            .cloned()
            .collect()
    }

    /// Names bound in this scope and the enclosing scopes a Ruby local would
    /// be visible from. The walk stops at the root, which holds the builtins,
    /// and at a method boundary, whose locals belong to the method rather
    /// than to the block running inside it.
    pub fn collect_local_variable_names(&self) -> Vec<String> {
        // A scope's own locals come before the ones carried in from a
        // Binding, which is the order Ruby reports them in.
        let mut names: Vec<String> = self
            .own_variable_names()
            .into_iter()
            .filter(|name| !self.inherited_names.contains(name))
            .collect();
        names.extend(
            self.inherited_names
                .iter()
                .filter(|name| self.variables.contains_key(*name))
                .filter(|name| !self.hidden_names.contains(*name))
                .cloned(),
        );
        // A method's own locals end the chain: what encloses the method is
        // not in scope inside it. A block, on the other hand, closes over the
        // scope it was written in, so the walk carries on through it.
        if self.is_method_boundary {
            return names;
        }
        if let Some(parent) = &self.parent {
            let parent_ref = parent.borrow();
            if parent_ref.parent.is_some() {
                names.extend(parent_ref.collect_local_variable_names());
            }
        }
        names
    }

    /// The names a Binding holds. A block closes over the locals of the scope
    /// it was written in, and reading or writing one of those through a
    /// binding reaches the same reference, so a captured name counts here
    /// even though it is not the block's own.
    pub fn collect_binding_variable_names(&self) -> Vec<String> {
        let holds_a_local = |name: &String| {
            !self.hidden_names.contains(name)
                && self
                    .variables
                    .get(name)
                    .is_some_and(|value| !names_a_definition_under(name, &value.borrow()))
        };
        // A scope's own locals come before the ones it can see through the
        // scopes enclosing it, which is the order Ruby reports them in.
        let mut names: Vec<String> = self
            .ordered_names()
            .filter(|name| !self.inherited_names.contains(*name))
            .filter(|name| holds_a_local(name))
            .cloned()
            .collect();
        names.extend(
            self.inherited_names
                .iter()
                .filter(|name| self.variables.contains_key(*name))
                .filter(|name| holds_a_local(name))
                .cloned(),
        );
        if !self.is_method_boundary
            && let Some(parent) = &self.parent
        {
            let parent_ref = parent.borrow();
            if parent_ref.parent.is_some() {
                names.extend(parent_ref.collect_binding_variable_names());
            }
        }
        // A name a block captured is a local of the scope the block was
        // written in. That scope is normally the one the walk above reaches,
        // but a block outliving it keeps the name and nothing else does.
        let mut closed_over: Vec<String> = Vec::new();
        if let Some(captured) = &self.captured {
            for (name, value) in captured.iter() {
                if !self.variables.contains_key(name)
                    && !self.hidden_names.contains(name)
                    && !names_a_definition_under(name, &value.borrow())
                    && !names.contains(name)
                    && !closed_over.contains(name)
                {
                    closed_over.push(name.clone());
                }
            }
        }
        // A capture is taken from a map, so the order it comes back in is
        // not the order the enclosing scope declared the names. Sorting
        // makes what `local_variables` reports the same on every run.
        closed_over.sort();
        names.extend(closed_over);
        names
    }

    /// Mark this scope as a method boundary.
    pub fn mark_method_boundary(&mut self) {
        self.is_method_boundary = true;
    }

    /// The reference this scope holds for `name`, ignoring enclosing scopes.
    pub fn own_var_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        self.variables
            .get(name)
            .or_else(|| self.captured_ref(name))
            .cloned()
    }

    /// Whether this scope is the root of the chain.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Collects all variable references from the entire scope chain
    /// Returns a HashMap with shared references to all visible variables
    /// Used for closure capture to enable mutable closures
    pub fn collect_all_var_refs(&self) -> HashMap<String, Rc<RefCell<Object>>> {
        let mut all_vars = HashMap::new();

        // Start from parent and work backwards, so that closer scopes override
        // farther ones. A method body ends the walk: the scope above it holds
        // the program's own top-level locals, which a block written inside a
        // method does not close over.
        if !self.is_method_boundary
            && let Some(parent) = &self.parent
        {
            all_vars = parent.borrow().collect_all_var_refs();
        }
        if let Some(captured) = &self.captured {
            for (name, value_ref) in captured.iter() {
                all_vars.insert(name.clone(), value_ref.clone());
            }
        }

        // Now add this scope's variables (potentially overriding parent values)
        for (name, value_ref) in &self.variables {
            // A name the program never declared is not one a block closes
            // over: a block reading `it` takes the argument it was handed
            // rather than the `it` of the block around it.
            if self.hoisted_names.contains(name) || self.hidden_names.contains(name) {
                continue;
            }
            all_vars.insert(name.clone(), value_ref.clone());
        }

        all_vars
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether the value `name` holds is a definition rather than a local: a
/// method, or a class or module bound under the constant a `class` or
/// `module` statement names. A lowercase local may hold a class as any
/// other value.
pub fn names_a_definition_under(name: &str, value: &Object) -> bool {
    match value {
        Object::Class(_) | Object::Module(_) => {
            name.starts_with(|first: char| !first.is_lowercase() && first != '_')
        }
        other => names_a_definition(other),
    }
}

/// Whether a binding holds a definition rather than a local: a method, a
/// class, or a module reached by name is not one of a scope's locals.
pub fn names_a_definition(value: &Object) -> bool {
    matches!(
        value,
        Object::Method(_)
            | Object::Class(_)
            | Object::Module(_)
            | Object::NativeFunction(_)
            | Object::CompiledFunction(_)
    )
}
