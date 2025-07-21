// Method struct - represents a class method (bound or unbound)

use crate::ast::Statement;
use crate::callable::Callable;

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
