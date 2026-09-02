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

/// Placeholder parameter recorded for a block written `{ |a,| }`. The
/// trailing comma is what tells Ruby to destructure a single array argument,
/// so it has to survive parsing; it never binds a name.
pub const TRAILING_COMMA_PARAM: &str = ",";

/// Block/lambda/closure with captured variables
#[derive(Debug, Clone)]
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
    /// The method that lexically encloses this block, as the (callee, defined)
    /// pair `__callee__` and `__method__` report. None for a block created
    /// outside any method.
    pub defining_method: Option<(String, String)>,
    /// True for `-> {}` and `lambda {}`, false for `proc {}` and every
    /// ordinary block. Lambdas check arity strictly; procs pad missing
    /// arguments with nil and drop extras.
    pub is_lambda: bool,
    /// The file the block was written in, which a backtrace entry for a call
    /// made from its body has to name.
    pub source_file: Option<String>,
}

/// Two blocks are the same when they were written the same way. The captured
/// variables are left out: a block can close over itself, so comparing them
/// would not terminate.
impl PartialEq for BlockStatement {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters
            && self.parameter_defaults == other.parameter_defaults
            && self.body == other.body
            && self.defining_method == other.defining_method
            && self.is_lambda == other.is_lambda
    }
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
            defining_method: None,
            is_lambda: false,
            source_file: None,
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
        defining_method: Option<(String, String)>,
        is_lambda: bool,
    ) -> Self {
        Self {
            parameters,
            parameter_defaults,
            body,
            captured_vars,
            captured_def_scope,
            defining_method,
            is_lambda,
            source_file: None,
        }
    }

    /// Get the captured variables
    pub fn captured_vars(&self) -> &HashMap<String, Rc<RefCell<Object>>> {
        &self.captured_vars
    }

    /// True when the parameter list ended in a comma, which makes a single
    /// array argument destructure across the declared parameters.
    pub fn destructures_single_array(&self) -> bool {
        self.parameters.iter().any(|p| p == TRAILING_COMMA_PARAM)
    }

    /// Parameters that actually bind a name, excluding the trailing-comma marker.
    pub fn binding_parameters(&self) -> Vec<String> {
        self.parameters
            .iter()
            .filter(|p| *p != TRAILING_COMMA_PARAM)
            .cloned()
            .collect()
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
