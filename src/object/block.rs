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
/// Marks a `|**nil|` declaration, which says the block takes no keyword
/// arguments at all. It names nothing and binds nothing.
pub const NO_KEYWORDS_PARAM: &str = "**nil";
/// Marks a `|(a, b)|` group, whose names follow separated by commas. One
/// array argument spreads across them.
pub const DESTRUCTURED_GROUP_PREFIX: &str = "(";

/// Marks a name written after the `;` in a block's parameter list. It is a
/// local of the block rather than a parameter, so it starts as nil and takes
/// no argument.
pub const BLOCK_LOCAL_PREFIX: &str = ";";
/// Marks a `|name:|` keyword parameter, which takes its value from the
/// keyword arguments rather than by position.
pub const KEYWORD_PARAM_PREFIX: &str = ":";

/// The parameter a block takes because its body reads a bare `it`. It is
/// spelled apart from an ordinary `it` so the block can tell the implicit
/// parameter from a name of its own: the implicit one is not a local the
/// block declared, and it gives way to an `it` already in scope.
pub const IMPLICIT_IT_PARAM: &str = "~it";

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
    /// The method invocation this block was written inside. A `return` in the
    /// body unwinds to that invocation, however many other methods the block
    /// travels through first. None for a block created outside any method.
    pub home_frame: Option<u64>,
    /// The line the block was opened on, which is where `source_location`
    /// says it was written even when its body starts further down.
    pub opened_at: Option<usize>,
    /// The name a callable built from a Symbol stands for, which is what it
    /// says of itself in place of a file and a line.
    pub from_symbol: Option<String>,
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
            home_frame: None,
            opened_at: None,
            from_symbol: None,
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
            home_frame: None,
            opened_at: None,
            from_symbol: None,
        }
    }

    /// Get the captured variables
    pub fn captured_vars(&self) -> &HashMap<String, Rc<RefCell<Object>>> {
        &self.captured_vars
    }

    /// True when a single array argument spreads across the parameters, which
    /// a block does when it declares more than one of them or when its
    /// parameter list ended in a comma.
    pub fn destructures_single_array(&self) -> bool {
        // A lambda takes its arguments the way a method does, so a lone array
        // stays one argument rather than spreading across the parameters.
        if self.is_lambda {
            return false;
        }
        if self
            .parameters
            .iter()
            .any(|name| name == TRAILING_COMMA_PARAM)
        {
            return true;
        }
        self.parameters
            .iter()
            .filter(|name| !name.starts_with('&') && !name.starts_with(KEYWORD_PARAM_PREFIX))
            .count()
            > 1
    }

    /// Parameters that actually bind a name, excluding the trailing-comma marker.
    pub fn binding_parameters(&self) -> Vec<String> {
        self.parameters
            .iter()
            .filter(|p| {
                *p != TRAILING_COMMA_PARAM
                    && *p != NO_KEYWORDS_PARAM
                    && !p.starts_with(BLOCK_LOCAL_PREFIX)
            })
            .cloned()
            .collect()
    }

    /// The names written after the `;`, which are locals of the block rather
    /// than parameters.
    pub fn block_locals(&self) -> Vec<String> {
        self.parameters
            .iter()
            .filter_map(|p| p.strip_prefix(BLOCK_LOCAL_PREFIX))
            .map(|name| name.to_string())
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
