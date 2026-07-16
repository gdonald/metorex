// BlockStatement - represents closures/lambdas with captured variables

use crate::ast::Statement;
use crate::callable::Callable;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::vm::VirtualMachine;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Object;

/// Block/lambda/closure with captured variables
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
    /// Parameter names
    pub parameters: Vec<String>,
    /// Default values for optional parameters, keyed by index into
    /// `parameters` (e.g. `{ |a, b = 1| }` records `(1, <1>)`).
    pub parameter_defaults: Vec<(usize, crate::ast::Expression)>,
    /// Block body (AST statements)
    pub body: Vec<Statement>,
    /// Captured variables from outer scope (shared mutable references)
    pub captured_vars: HashMap<String, Rc<RefCell<Object>>>,
    /// Lexical class/module nesting at the moment the block was defined.
    /// Restored during invocation so a bare `Foo = 1` inside the body lands
    /// on the same enclosing module that an unbroken straight-line statement
    /// would have hit.
    pub captured_def_scope: Vec<Rc<Class>>,
}

impl BlockStatement {
    /// Create a new block closure
    pub fn new(
        parameters: Vec<String>,
        body: Vec<Statement>,
        captured_vars: HashMap<String, Rc<RefCell<Object>>>,
    ) -> Self {
        Self {
            parameters,
            parameter_defaults: Vec::new(),
            body,
            captured_vars,
            captured_def_scope: Vec::new(),
        }
    }

    /// Create a new block closure with a captured lexical scope. Used by
    /// the Lambda evaluator so the block remembers which class/module it
    /// was lexically inside.
    pub fn with_def_scope(
        parameters: Vec<String>,
        parameter_defaults: Vec<(usize, crate::ast::Expression)>,
        body: Vec<Statement>,
        captured_vars: HashMap<String, Rc<RefCell<Object>>>,
        captured_def_scope: Vec<Rc<Class>>,
    ) -> Self {
        Self {
            parameters,
            parameter_defaults,
            body,
            captured_vars,
            captured_def_scope,
        }
    }

    /// Get the captured variables
    pub fn captured_vars(&self) -> &HashMap<String, Rc<RefCell<Object>>> {
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
