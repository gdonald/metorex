// Method struct - represents a class method (bound or unbound)

use crate::ast::{Expression, Statement};
use crate::callable::Callable;
use crate::error::SourceLocation;

use super::Object;

/// Method definition (function bound to a class)
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    /// Name of the method
    pub name: String,
    /// Positional parameter names (excludes &block and keyword params)
    pub parameters: Vec<String>,
    /// Named keyword parameters: (name, optional_default_expression)
    /// e.g., `def f(name:, age: 10)` → [("name", None), ("age", Some(IntLiteral(10)))]
    pub keyword_parameters: Vec<(String, Option<Expression>)>,
    /// Optional block parameter name (from `&block` syntax)
    pub block_parameter: Option<String>,
    /// Method body (AST statements)
    pub body: Vec<Statement>,
    /// Optional receiver (for bound methods)
    pub receiver: Option<Box<Object>>,
    /// Owner of the method (class name or "main" for top-level functions)
    pub owner: Option<String>,
    /// Source location where the method is defined
    pub source_location: Option<SourceLocation>,
}

impl Method {
    /// Create a new method
    pub fn new(name: String, parameters: Vec<String>, body: Vec<Statement>) -> Self {
        Self {
            name,
            parameters,
            keyword_parameters: vec![],
            block_parameter: None,
            body,
            receiver: None,
            owner: None,
            source_location: None,
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
            keyword_parameters: vec![],
            block_parameter: None,
            body,
            receiver: None,
            owner: Some(owner),
            source_location: None,
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
            keyword_parameters: vec![],
            block_parameter: None,
            body,
            receiver: None,
            owner: None,
            source_location: Some(source_location),
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
            keyword_parameters: vec![],
            block_parameter: None,
            body,
            receiver: None,
            owner: Some(owner),
            source_location: Some(source_location),
        }
    }

    /// Bind this method to a receiver
    pub fn bind(&self, receiver: Object) -> Self {
        Self {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            keyword_parameters: self.keyword_parameters.clone(),
            block_parameter: self.block_parameter.clone(),
            body: self.body.clone(),
            receiver: Some(Box::new(receiver)),
            owner: self.owner.clone(),
            source_location: self.source_location.clone(),
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
