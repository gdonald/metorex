// Virtual machine core structure for the Metorex AST interpreter.
//
// This module defines the runtime scaffolding (struct, constructor, getters,
// call-stack helpers). The execution loop and expression evaluator live in
// sibling modules:
//   - vm/loading.rs:  file loading, `require`, `execute_file`
//   - vm/program.rs:  `execute_program`, `evaluate_arguments`, `evaluate_expression`
//   - vm/eval/*:      per-variant expression evaluation helpers + dispatch
// Operator/statement/method-call helpers live in their own existing modules.

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

use super::init::*;
use super::{CallFrame, GlobalRegistry, Heap};

use crate::builtin_classes::BuiltinClasses;
use crate::environment::Environment;
use crate::object::Object;

/// Core virtual machine responsible for executing Metorex programs.
pub struct VirtualMachine {
    pub(crate) environment: Environment,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) globals: GlobalRegistry,
    pub(crate) heap: Rc<RefCell<Heap>>,
    pub(crate) builtins: BuiltinClasses,
    pub(crate) current_file: Option<PathBuf>,
    pub(crate) loaded_files: HashSet<PathBuf>,
    /// Trailing block passed to the current call (e.g., `foo() do |x| ... end`).
    /// Set before invoke_method/invoke_callable; taken at method body entry.
    pub(crate) pending_block: Option<Object>,
}

impl VirtualMachine {
    /// Construct a new virtual machine instance with all built-ins registered.
    pub fn new() -> Self {
        let mut environment = Environment::new();
        let builtins = BuiltinClasses::new();

        initialize_builtin_methods(&builtins);

        let mut globals = GlobalRegistry::new();
        register_builtin_classes(&mut globals, &builtins);
        register_builtin_modules(&mut globals);
        register_singletons(&mut globals);
        register_special_globals(&mut globals);
        register_native_functions(&mut globals);

        seed_environment_with_globals(&mut environment, &globals);

        Self {
            environment,
            call_stack: Vec::new(),
            globals,
            heap: Rc::new(RefCell::new(Heap::default())),
            builtins,
            current_file: None,
            loaded_files: HashSet::new(),
            pending_block: None,
        }
    }

    /// Access the environment.
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    /// Mutably access the environment (used by the interpreter).
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    /// Access the registered built-in classes.
    pub fn builtins(&self) -> &BuiltinClasses {
        &self.builtins
    }

    /// Access the global registry.
    pub fn globals(&self) -> &GlobalRegistry {
        &self.globals
    }

    /// Mutably access the global registry.
    pub fn globals_mut(&mut self) -> &mut GlobalRegistry {
        &mut self.globals
    }

    /// Borrow the heap allocator.
    pub fn heap(&self) -> Rc<RefCell<Heap>> {
        Rc::clone(&self.heap)
    }

    /// Set the ARGV global with script arguments.
    pub fn set_argv(&mut self, args: Vec<String>) {
        let elements: Vec<Object> = args
            .into_iter()
            .map(|s| Object::String(Rc::new(s)))
            .collect();
        let argv = Object::Array(Rc::new(RefCell::new(elements)));
        self.globals.set("ARGV", argv.clone());
        self.environment.define("ARGV".to_string(), argv);
    }

    /// Run a closure with a new call frame pushed onto the stack.
    pub fn with_call_frame<F, R>(&mut self, frame: CallFrame, action: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.call_stack.push(frame);
        let result = action(self);
        self.call_stack.pop();
        result
    }

    /// Inspect the current call stack (top is last element).
    pub fn call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    /// Get the name of the current method being executed (from the top of the call stack).
    pub(crate) fn get_current_method_name(&self) -> Option<&str> {
        self.call_stack.last().map(|frame| frame.name())
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}
