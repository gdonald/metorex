// Environment stack for managing lexical scopes in Metorex
// This module implements a stack-based scope management system

use crate::object::Object;
use crate::scope::Scope;
use std::cell::RefCell;
use std::rc::Rc;

/// Represents the environment with a stack of scopes
/// The environment manages the scope chain and tracks the current depth
#[derive(Debug)]
pub struct Environment {
    /// Stack of scopes, with the top being the current scope
    scopes: Vec<Rc<RefCell<Scope>>>,

    /// Current depth in the scope stack (0 = global scope)
    depth: usize,
}

impl Environment {
    /// Creates a new environment with a global scope
    pub fn new() -> Self {
        let global_scope = Rc::new(RefCell::new(Scope::new()));
        Environment {
            scopes: vec![global_scope],
            depth: 0,
        }
    }

    /// An environment standing where `scope` is current, for reading the
    /// locals of a frame further out than the running one.
    pub fn viewing(&self, scope: Rc<RefCell<Scope>>) -> Self {
        let global = self.global_scope();
        let scopes = if Rc::ptr_eq(&global, &scope) {
            vec![global]
        } else {
            vec![global, scope]
        };
        Environment {
            depth: scopes.len() - 1,
            scopes,
        }
    }

    /// Pushes a new scope onto the stack
    /// The new scope's parent will be the current top scope
    pub fn push_scope(&mut self) {
        let parent = self.scopes.last().unwrap().clone();
        let new_scope = Rc::new(RefCell::new(Scope::with_parent(parent)));
        self.scopes.push(new_scope);
        self.depth += 1;
    }

    /// Pushes an isolated scope (method boundary). The parent is the global
    /// scope, not the caller's scope, so local variable assignment inside a
    /// method body cannot accidentally modify the caller's variables.
    pub fn push_isolated_scope(&mut self) {
        let global = self.scopes[0].clone();
        let new_scope = Rc::new(RefCell::new(Scope::with_parent(global)));
        new_scope.borrow_mut().mark_method_boundary();
        self.scopes.push(new_scope);
        self.depth += 1;
    }

    /// Pops the current scope from the stack
    /// Returns the popped scope, or None if we're at the global scope
    /// Note: The global scope can never be popped
    pub fn pop_scope(&mut self) -> Option<Rc<RefCell<Scope>>> {
        if self.scopes.len() <= 1 {
            // Cannot pop the global scope
            return None;
        }

        self.depth -= 1;
        self.scopes.pop()
    }

    /// Returns a reference to the current (top) scope
    pub fn current_scope(&self) -> Rc<RefCell<Scope>> {
        self.scopes.last().unwrap().clone()
    }

    /// Returns the current scope depth
    /// 0 = global scope, 1 = first nested scope, etc.
    pub fn current_depth(&self) -> usize {
        self.depth
    }

    /// Returns a reference to the global scope
    pub fn global_scope(&self) -> Rc<RefCell<Scope>> {
        self.scopes[0].clone()
    }

    /// Defines a variable in the current scope
    pub fn define(&mut self, name: String, value: Object) {
        self.current_scope().borrow_mut().define(name, value);
    }

    /// Reserves a name for the rest of the current scope ahead of the
    /// assignment that introduces it, the way Ruby's parser does.
    pub fn hoist(&mut self, name: String) {
        self.current_scope().borrow_mut().hoist(name);
    }

    /// Whether an assignment to `name` in the body being entered introduces a
    /// local. A name nothing has bound does, and so does one that only a
    /// builtin method of the same name answers to: `p = 5` makes `p` a local
    /// of that scope even though `Kernel#p` shares the name.
    pub fn assignment_introduces_a_local(&self, name: &str) -> bool {
        match self.get(name) {
            None => true,
            Some(value) => {
                crate::scope::names_a_definition(&value) && self.resolves_to_root_binding(name)
            }
        }
    }

    /// Whether `name` is only reserved for an assignment further down, which
    /// means no local of that name is in scope where execution stands.
    pub fn name_is_only_hoisted(&self, name: &str) -> bool {
        self.current_scope().borrow().is_only_hoisted(name)
    }

    /// Binds a name the program did not declare, which `local_variables`
    /// and a Binding leave out.
    pub fn define_hidden(&mut self, name: String, value: Object) {
        self.current_scope().borrow_mut().define_hidden(name, value);
    }

    /// Say that the assignment a hoisted name was reserved for has begun,
    /// so a block written in its value closes over the name.
    pub fn unhoist(&mut self, name: &str) {
        self.current_scope().borrow_mut().unhoist(name);
    }

    /// Remove a variable from the current scope (does not touch parent scopes).
    pub fn undefine(&mut self, name: &str) {
        self.current_scope().borrow_mut().undefine(name);
    }

    /// Gets a variable value by traversing the scope chain from the current scope
    pub fn get(&self, name: &str) -> Option<Object> {
        self.current_scope().borrow().get(name)
    }

    /// Sets a variable value by traversing the scope chain from the current scope
    /// Returns true if the variable was found and updated, false otherwise
    pub fn set(&mut self, name: &str, value: Object) -> bool {
        self.current_scope().borrow_mut().set(name, value)
    }

    /// Gets a variable at a specific depth relative to the current scope
    pub fn get_at(&self, depth: usize, name: &str) -> Option<Object> {
        self.current_scope().borrow().get_at(depth, name)
    }

    /// Sets a variable at a specific depth relative to the current scope
    pub fn set_at(&mut self, depth: usize, name: &str, value: Object) -> bool {
        self.current_scope().borrow_mut().set_at(depth, name, value)
    }

    /// Collects all variables from the current scope chain
    /// This is used for lambda closure capture
    pub fn current_scope_vars(&self) -> std::collections::HashMap<String, Object> {
        self.current_scope().borrow().collect_all_vars()
    }

    /// Collects all variable references from the current scope chain
    /// This is used for lambda closure capture with mutable closures
    pub fn current_scope_var_refs(
        &self,
    ) -> indexmap::IndexMap<String, std::rc::Rc<std::cell::RefCell<Object>>> {
        self.current_scope().borrow().collect_all_var_refs()
    }

    /// Names bound as local variables where execution currently sits. At the
    /// top level that is the root scope's own names; anywhere else it is the
    /// chain up to but not including the root, which holds the builtins.
    pub fn local_variable_names(&self) -> Vec<String> {
        let current = self.current_scope();
        let scope = current.borrow();
        if scope.is_root() {
            scope.own_variable_names()
        } else {
            scope.collect_local_variable_names()
        }
    }

    /// The names a Binding captured here holds, which take in the locals a
    /// block closed over as well as its own.
    pub fn binding_variable_names(&self) -> Vec<String> {
        let current = self.current_scope();
        let scope = current.borrow();
        if scope.is_root() {
            scope.own_variable_names()
        } else {
            scope.collect_binding_variable_names()
        }
    }

    /// Whether `name` resolves to the very reference the root scope holds.
    /// A block captures enclosing names by reference, so a builtin or a
    /// top-level local reaches a block's own scope as the same reference
    /// rather than as a local of its own.
    pub fn resolves_to_root_binding(&self, name: &str) -> bool {
        if self.current_scope().borrow().is_root() {
            return false;
        }
        let Some(root_ref) = self.scopes[0].borrow().own_var_ref(name) else {
            return false;
        };
        self.get_ref(name)
            .is_some_and(|current| std::rc::Rc::ptr_eq(&current, &root_ref))
    }

    /// Defines a variable in the current scope with a shared reference
    /// Used when a closure defines a captured variable
    pub fn define_shared(&mut self, name: String, value: std::rc::Rc<std::cell::RefCell<Object>>) {
        self.current_scope().borrow_mut().define_shared(name, value);
    }

    /// Defines a name a block captured from its definition site. It resolves
    /// like any other variable but is not reported as a local of this scope.
    /// Binds a name carried in from a Binding in the current scope.
    pub fn define_inherited(
        &mut self,
        name: String,
        value: std::rc::Rc<std::cell::RefCell<Object>>,
    ) {
        self.current_scope()
            .borrow_mut()
            .define_inherited(name, value);
    }

    /// Read and write through the names a block closed over in the current
    /// scope, without copying them into it.
    pub fn attach_reserved(&mut self, reserved: std::rc::Rc<Vec<String>>) {
        self.current_scope().borrow_mut().attach_reserved(reserved);
    }

    pub fn attach_captured(&mut self, captured: crate::scope::CapturedNames) {
        self.current_scope().borrow_mut().attach_captured(captured);
    }

    /// Gets a shared reference to a variable
    pub fn get_ref(&self, name: &str) -> Option<std::rc::Rc<std::cell::RefCell<Object>>> {
        self.current_scope().borrow().get_ref(name)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
