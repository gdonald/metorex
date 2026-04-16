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
    /// Depth of nested wrapped `load(path, true)` calls. While >0, top-level
    /// `include` is suppressed (Ruby wraps the loaded scope in an anonymous
    /// module so includes don't pollute Object).
    pub(crate) load_wrap_depth: u32,
    /// Depth of user-defined method bodies we're currently inside (lexical
    /// nesting). Reset to 0 when executing a file top-level via load/require.
    pub(crate) user_def_nesting: u32,
    /// Stack of refinement scopes. Each scope holds a list of activated
    /// refinement modules (from `using`) with the snapshot of refined classes
    /// at activation time. Pushed on file load / eval; popped on exit.
    pub(crate) refinement_scopes: Vec<Vec<RefinementEntry>>,
}

/// A single activated refinement: the refinement module and the set of target
/// class names that were refined by it at activation time.
#[derive(Debug, Clone)]
pub struct RefinementEntry {
    pub module: Rc<crate::class::Class>,
    pub classes: std::collections::HashSet<String>,
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
        register_exception_classes(&mut globals);
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
            load_wrap_depth: 0,
            user_def_nesting: 0,
            refinement_scopes: vec![Vec::new()],
        }
    }

    /// Activate a refinement module in the innermost scope.
    pub(crate) fn activate_refinement(&mut self, module: Rc<crate::class::Class>) {
        // Snapshot the set of keyed targets refined at activation time.
        let mut classes = std::collections::HashSet::new();
        for key in module.class_var_names() {
            if key.starts_with("__refine__") {
                classes.insert(key.clone());
            }
        }
        let entry = RefinementEntry { module, classes };
        if let Some(top) = self.refinement_scopes.last_mut() {
            top.push(entry);
        }
    }

    /// Look up a refined method for the given target class (keyed by pointer
    /// + name), scanning active refinement scopes outermost-first.
    pub(crate) fn find_refined_method(
        &self,
        target_key: &str,
        method_name: &str,
    ) -> Option<Rc<crate::object::Method>> {
        for scope in self.refinement_scopes.iter().rev() {
            for entry in scope.iter().rev() {
                if !entry.classes.contains(target_key) {
                    continue;
                }
                if let Some(Object::Class(holder)) = entry.module.get_class_var(target_key)
                    && let Some(m) = holder.find_method(method_name)
                {
                    return Some(m);
                }
            }
        }
        None
    }

    /// Snapshot all currently active refinement entries (for lexical capture
    /// into a method definition).
    pub(crate) fn snapshot_active_refinements(
        &self,
    ) -> Vec<(Rc<crate::class::Class>, Vec<String>)> {
        let mut out = Vec::new();
        for scope in &self.refinement_scopes {
            for entry in scope {
                out.push((
                    Rc::clone(&entry.module),
                    entry.classes.iter().cloned().collect(),
                ));
            }
        }
        out
    }

    pub(crate) fn push_refinement_scope(&mut self) {
        self.refinement_scopes.push(Vec::new());
    }

    pub(crate) fn pop_refinement_scope(&mut self) {
        if self.refinement_scopes.len() > 1 {
            self.refinement_scopes.pop();
        }
    }

    /// True if we are lexically inside a `def` body relative to the current
    /// file-top-level / eval context.
    pub(crate) fn inside_user_method(&self) -> bool {
        self.user_def_nesting > 0
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
