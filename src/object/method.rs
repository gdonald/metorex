// Method struct - represents a class method (bound or unbound)

use crate::ast::Statement;
use crate::callable::Callable;
use crate::error::SourceLocation;

use super::Object;

/// Method definition (function bound to a class)
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    /// Name of the method
    pub name: String,
    /// Parameter names
    pub parameters: Vec<String>,
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
