// Method struct - represents a class method (bound or unbound)

use crate::ast::{Expression, Statement};
use crate::callable::Callable;
use crate::error::SourceLocation;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Object;

/// Method definition (function bound to a class)
#[derive(Debug, Clone)]
pub struct Method {
    /// Name of the method
    pub name: String,
    /// Positional parameter names (excludes &block and keyword params)
    pub parameters: Vec<String>,
    /// Default values for positional parameters, indexed by position
    pub default_parameters: Vec<(usize, Expression)>,
    /// Named keyword parameters: (name, optional_default_expression)
    /// e.g., `def f(name:, age: 10)` → [("name", None), ("age", Some(IntLiteral(10)))]
    pub keyword_parameters: Vec<(String, Option<Expression>)>,
    /// Optional block parameter name (from `&block` syntax)
    pub block_parameter: Option<String>,
    /// Variadic (splat) parameter: (positional_index, name) for `*args`
    pub variadic_param: Option<(usize, String)>,
    /// Method body (AST statements)
    pub body: Vec<Statement>,
    /// Optional receiver (for bound methods)
    pub receiver: Option<Box<Object>>,
    /// Receiver this method must run against no matter where it is installed.
    /// Set by `Method#to_proc`, which produces a Proc permanently attached to
    /// the object the Method was extracted from.
    pub bound_self: Option<Box<Object>>,
    /// Owner of the method (class name or "main" for top-level functions)
    pub owner: Option<String>,
    /// Owning class/module object, when known. Needed by `define_method` to
    /// validate that an UnboundMethod may be rebound onto the target module,
    /// and by `Method#owner` to return the module itself rather than its name.
    pub owner_class: Option<Rc<crate::class::Class>>,
    /// Source location where the method is defined
    pub source_location: Option<SourceLocation>,
    /// Captured closure variables (from define_method blocks)
    pub captured_vars: Option<HashMap<String, Rc<RefCell<Object>>>>,
    /// True if this method was undefined via undef_method (calling it raises an error)
    pub is_undefined: bool,
    /// Lexical class/module nesting captured from the Proc this method was
    /// built from. Restored while the body runs so a nested `def` lands in the
    /// Proc's default definee rather than the receiving class.
    pub captured_def_scope: Vec<Rc<crate::class::Class>>,
    /// True when the body came from a block or Proc handed to `define_method`.
    /// Those bodies follow lambda control flow: `break` and `next` return a
    /// value, and `redo` re-runs the body.
    pub lambda_body: bool,
    /// Refinement modules active (lexically) when this method was defined,
    /// each paired with the snapshot of refined-class names at that moment.
    pub captured_refinements: Vec<(Rc<crate::class::Class>, Vec<String>)>,
    /// The lexically enclosing modules at the point of definition, innermost
    /// first. `Module.nesting` inside the method reports these rather than
    /// the scopes open at the call site.
    pub captured_nesting: Vec<Rc<crate::class::Class>>,
}

impl Method {
    /// Create a new method
    pub fn new(name: String, parameters: Vec<String>, body: Vec<Statement>) -> Self {
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            block_parameter: None,
            variadic_param: None,
            body,
            receiver: None,
            bound_self: None,
            owner: None,
            owner_class: None,
            source_location: None,
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            captured_nesting: Vec::new(),
        }
    }

    /// Create a new method with an owner
    pub fn with_owner(
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
        owner: String,
    ) -> Self {
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            block_parameter: None,
            variadic_param: None,
            body,
            receiver: None,
            bound_self: None,
            owner: Some(owner),
            owner_class: None,
            source_location: None,
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            captured_nesting: Vec::new(),
        }
    }

    /// Create a new method with a source location
    pub fn with_source_location(
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
        source_location: SourceLocation,
    ) -> Self {
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            block_parameter: None,
            variadic_param: None,
            body,
            receiver: None,
            bound_self: None,
            owner: None,
            owner_class: None,
            source_location: Some(source_location),
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            captured_nesting: Vec::new(),
        }
    }

    /// Create a new method with both owner and source location
    pub fn with_owner_and_location(
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
        owner: String,
        source_location: SourceLocation,
    ) -> Self {
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            block_parameter: None,
            variadic_param: None,
            body,
            receiver: None,
            bound_self: None,
            owner: Some(owner),
            owner_class: None,
            source_location: Some(source_location),
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            captured_nesting: Vec::new(),
        }
    }

    /// Create an undefined method sentinel (for undef_method).
    pub fn undefined(name: String) -> Self {
        Self {
            name,
            parameters: vec![],
            default_parameters: vec![],
            keyword_parameters: vec![],
            block_parameter: None,
            variadic_param: None,
            body: vec![],
            receiver: None,
            bound_self: None,
            owner: None,
            owner_class: None,
            source_location: None,
            captured_vars: None,
            is_undefined: true,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            captured_nesting: Vec::new(),
        }
    }

    /// Bind this method to a receiver
    pub fn bind(&self, receiver: Object) -> Self {
        Self {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            default_parameters: self.default_parameters.clone(),
            keyword_parameters: self.keyword_parameters.clone(),
            block_parameter: self.block_parameter.clone(),
            variadic_param: self.variadic_param.clone(),
            body: self.body.clone(),
            receiver: Some(Box::new(receiver)),
            bound_self: self.bound_self.clone(),
            owner: self.owner.clone(),
            owner_class: self.owner_class.clone(),
            source_location: self.source_location.clone(),
            captured_vars: self.captured_vars.clone(),
            is_undefined: self.is_undefined,
            lambda_body: self.lambda_body,
            captured_def_scope: self.captured_def_scope.clone(),
            captured_refinements: self.captured_refinements.clone(),
            captured_nesting: self.captured_nesting.clone(),
        }
    }

    /// Check if this method is bound to a receiver
    pub fn is_bound(&self) -> bool {
        self.receiver.is_some()
    }

    /// Get the receiver if this method is bound
    pub fn receiver(&self) -> Option<&Object> {
        self.receiver.as_deref()
    }
}

/// `Class` has no structural equality, so the owning module is compared by
/// identity while every other field compares structurally.
impl PartialEq for Method {
    fn eq(&self, other: &Self) -> bool {
        let same_owner_class = match (&self.owner_class, &other.owner_class) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };
        let same_refinements = self.captured_refinements.len() == other.captured_refinements.len()
            && self
                .captured_refinements
                .iter()
                .zip(other.captured_refinements.iter())
                .all(|((a_mod, a_names), (b_mod, b_names))| {
                    Rc::ptr_eq(a_mod, b_mod) && a_names == b_names
                });
        same_owner_class
            && same_refinements
            && self.name == other.name
            && self.parameters == other.parameters
            && self.default_parameters == other.default_parameters
            && self.keyword_parameters == other.keyword_parameters
            && self.block_parameter == other.block_parameter
            && self.variadic_param == other.variadic_param
            && self.body == other.body
            && self.receiver == other.receiver
            && self.bound_self == other.bound_self
            && self.owner == other.owner
            && self.source_location == other.source_location
            && self.captured_vars == other.captured_vars
            && self.is_undefined == other.is_undefined
            && self.lambda_body == other.lambda_body
            && self.captured_def_scope.len() == other.captured_def_scope.len()
            && self
                .captured_def_scope
                .iter()
                .zip(other.captured_def_scope.iter())
                .all(|(a, b)| Rc::ptr_eq(a, b))
    }
}

impl Callable for Method {
    fn name(&self) -> &str {
        &self.name
    }

    fn parameters(&self) -> &[String] {
        &self.parameters
    }

    fn body(&self) -> &[Statement] {
        &self.body
    }
}
