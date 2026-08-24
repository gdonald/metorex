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
    /// Depth counter that tracks whether we're inside a const-access
    /// autoload trigger. While >0, `effective_autoload` skips moving the
    /// cleared name to `unrealized_autoloads` — that bookkeeping only
    /// applies to the direct-require path.
    pub(crate) autoload_const_access_depth: u32,
    /// Stack of currently-executing Thread instances. The top of the stack
    /// is what `Thread.current` returns; pushed at the start of a
    /// `Thread.new { }` block's deferred execution (`.value`/`.join`) and
    /// popped on exit. When empty, `Thread.current` returns Nil.
    pub(crate) thread_current_stack: Vec<Object>,
    /// Threads created via `Thread.new` whose block hasn't been run yet
    /// (no `.value`/`.join` call). When `Queue#pop` is invoked on an
    /// empty queue, we drain this list and run their blocks — a
    /// coroutine-style hack so a `Queue.pop` that "waits" for another
    /// thread's `Queue.push` actually gets unblocked under our
    /// synchronous Thread model. Threads remove themselves on first
    /// `.value`/`.join` (whichever runs first).
    pub(crate) pending_threads: Vec<Object>,
    /// Stack of canonical paths whose body is *currently executing* via
    /// `execute_file`. The path joins this stack on entry (after the
    /// `$"` mark goes in) and leaves on exit. Distinct from `$"` because
    /// `$"` is added eagerly before the body runs to short-circuit
    /// recursive requires; this stack tells autoload "the constant
    /// hasn't been defined yet because the file is mid-execution, don't
    /// re-load."
    pub(crate) loading_paths: Vec<String>,
    /// Autoloads currently being loaded, with the thread that initiated
    /// the load. Stored as `(class, name, loading_thread)` triples.
    /// `effective_autoload` consults this list to differentiate the
    /// "loading thread" view (sees the autoload as cleared) from the
    /// "other thread" view (sees the autoload as still active).
    pub(crate) autoload_loading: Vec<(Rc<crate::class::Class>, String, Object)>,
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
    /// Stack of lexically enclosing class/module definitions. Used to route
    /// nested `class`/`module` declarations to the enclosing scope (so
    /// `module Foo; class Bar; end; end` defines `Foo::Bar`, not `::Bar`).
    /// Pushed on entering a `class`/`module` body; popped on exit. Does NOT
    /// track method call receivers — only lexical nesting.
    pub(crate) def_scope_stack: Vec<Rc<crate::class::Class>>,
    /// Lazily-populated singleton classes for value-kind receivers that
    /// have no per-object storage (Nil / true / false / Int / Float /
    /// Symbol / String). Keyed by a stable string tag so every lookup for
    /// the same receiver returns the same Class instance.
    pub(crate) primitive_singleton_classes:
        std::collections::HashMap<String, Rc<crate::class::Class>>,
    /// Stack of positional arguments captured for each active method
    /// invocation. `super` (bare form) reads the top entry to forward args
    /// to the parent method; pushed by invoke_method, popped on return.
    pub(crate) method_arg_stack: Vec<Vec<crate::object::Object>>,
    /// The lexical nesting captured by each method currently on the call
    /// stack, so `Module.nesting` inside a body reports the definition site.
    pub(crate) method_nesting_stack: Vec<Vec<Rc<crate::class::Class>>>,
    /// Tags of the `catch` blocks currently running, innermost last. `throw`
    /// consults them so a tag nothing is catching raises rather than unwinding
    /// past the whole program.
    pub(crate) catch_tags: Vec<crate::object::Object>,
    /// Depth of autoload-driven re-runs of an already-required file. A
    /// constant assignment during one is repeating work Ruby would not have
    /// repeated, so its "already initialized" warning is an artifact.
    pub(crate) autoload_reload_depth: usize,
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
            autoload_const_access_depth: 0,
            thread_current_stack: Vec::new(),
            pending_threads: Vec::new(),
            loading_paths: Vec::new(),
            autoload_loading: Vec::new(),
            pending_block: None,
            load_wrap_depth: 0,
            user_def_nesting: 0,
            refinement_scopes: vec![Vec::new()],
            def_scope_stack: Vec::new(),
            primitive_singleton_classes: std::collections::HashMap::new(),
            method_arg_stack: Vec::new(),
            method_nesting_stack: Vec::new(),
            catch_tags: Vec::new(),
            autoload_reload_depth: 0,
        }
    }

    /// Activate a refinement module in the innermost scope.
    pub(crate) fn activate_refinement(&mut self, module: Rc<crate::class::Class>) {
        // Snapshot the keyed targets refined at activation time, including
        // those the module picks up from the modules it includes.
        let mut entries = Vec::new();
        let mut sources = vec![Rc::clone(&module)];
        sources.extend(module.transitive_mixins());
        for source in sources {
            let classes: std::collections::HashSet<String> = source
                .class_var_names()
                .into_iter()
                .filter(|key| key.starts_with(crate::vm::REFINEMENT_KEY_PREFIX))
                .collect();
            if !classes.is_empty() {
                entries.push(RefinementEntry {
                    module: source,
                    classes,
                });
            }
        }
        if let Some(top) = self.refinement_scopes.last_mut() {
            top.extend(entries);
        }
    }

    /// The refinement modules active in the innermost scope, which is what
    /// `Module.used_refinements` reports.
    pub(crate) fn active_refinements(&self) -> Vec<Object> {
        let mut refinements = Vec::new();
        let Some(scope) = self.refinement_scopes.last() else {
            return refinements;
        };
        for entry in scope {
            for key in &entry.classes {
                if let Some(Object::Class(holder) | Object::Module(holder)) =
                    entry.module.get_class_var(key)
                {
                    refinements.push(Object::Module(holder));
                }
            }
        }
        refinements
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
                if let Some(Object::Class(holder) | Object::Module(holder)) =
                    entry.module.get_class_var(target_key)
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
    /// The lexically enclosing modules at this point, innermost first. Used
    /// to give each method the `Module.nesting` in force where it was defined.
    pub(crate) fn snapshot_lexical_nesting(&self) -> Vec<Rc<crate::class::Class>> {
        self.def_scope_stack.iter().rev().map(Rc::clone).collect()
    }

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

    /// Push a frame that stays until it is popped, for scopes whose body is
    /// run by a loop rather than a single closure.
    pub(crate) fn call_stack_push(&mut self, frame: CallFrame) {
        self.call_stack.push(frame);
    }

    /// Pop the frame `call_stack_push` added.
    pub(crate) fn call_stack_pop(&mut self) {
        self.call_stack.pop();
    }

    /// The (callee, defined) names of the method currently running, or None
    /// at file or class-body scope. Block frames report the method that
    /// lexically encloses them rather than whichever method called them.
    pub(crate) fn enclosing_method_names(&self) -> Option<(String, String)> {
        use crate::vm::FrameKind;
        for frame in self.call_stack.iter().rev() {
            match frame.kind() {
                FrameKind::Block => continue,
                FrameKind::Boundary => return None,
                FrameKind::Method { callee, defined } => {
                    return Some((callee.clone(), defined.clone()));
                }
            }
        }
        None
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
