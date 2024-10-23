// BlockStatement - represents closures/lambdas with captured variables

use crate::ast::Statement;
use crate::callable::Callable;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::vm::VirtualMachine;
use std::collections::HashMap;

use super::Object;

/// Block/lambda/closure with captured variables
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
    /// Parameter names
    pub parameters: Vec<String>,
    /// Block body (AST statements)
    pub body: Vec<Statement>,
    /// Captured variables from outer scope
    pub captured_vars: HashMap<String, Object>,
}

impl BlockStatement {
    /// Create a new block closure
    pub fn new(
        parameters: Vec<String>,
        body: Vec<Statement>,
        captured_vars: HashMap<String, Object>,
    ) -> Self {
        Self {
            parameters,
            body,
            captured_vars,
        }
    }

    /// Get the captured variables
    pub fn captured_vars(&self) -> &HashMap<String, Object> {
        &self.captured_vars
    }

    /// Invoke the block within the provided virtual machine context.
    pub fn call(
        &self,
        vm: &mut VirtualMachine,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        vm.execute_block_callable(self, arguments, position)
    }
}

impl Callable for BlockStatement {
    fn name(&self) -> &str {
        "<block>"
    }

    fn parameters(&self) -> &[String] {
        &self.parameters
    }

    fn body(&self) -> &[Statement] {
        &self.body
    }
}
