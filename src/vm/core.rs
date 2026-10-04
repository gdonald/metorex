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
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

use super::init::*;
use super::{CallFrame, GlobalRegistry, Heap};

use crate::builtin_classes::BuiltinClasses;
use crate::environment::Environment;
use crate::object::Object;

/// Core virtual machine responsible for executing Metorex programs.
/// The frame top-level code runs in, so a block written there is told apart
/// from one whose home frame was never recorded.
pub(crate) const TOP_LEVEL_FRAME: u64 = 0;

/// A method known by the file, line, and column it was defined at, and its
/// name.
pub(crate) type MethodDefinitionKey = (Option<(String, usize, usize)>, String);

/// Where a `def` written in a running method's body installs.
#[derive(Clone)]
pub(crate) enum Definee {
    /// The class or module the method was defined in.
    Class(Rc<crate::class::Class>),
    /// The singleton class of the object `instance_exec` is running a block
    /// against, made when a `def` first needs it.
    SingletonOf(Object),
}

pub struct VirtualMachine {
    pub(crate) environment: Environment,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) globals: GlobalRegistry,
    /// The second names globals have been given, each pointing at the one
    /// it stands for. `alias $ERROR_INFO $!` records one here.
    pub(crate) global_aliases: std::collections::HashMap<String, String>,
    /// The globals a C extension defined, read and written through the
    /// functions it gave.
    pub(crate) hooked_globals: std::collections::HashMap<String, crate::vm::capi::HookedGlobal>,
    pub(crate) heap: Rc<RefCell<Heap>>,
    pub(crate) builtins: BuiltinClasses,
    pub(crate) current_file: Option<PathBuf>,

    /// The measurement `Coverage` is running, when one is on.
    pub(crate) coverage: Option<crate::vm::coverage::CoverageRun>,

    /// A line already counted by a statement loop that runs some statements
    /// itself and hands the rest to `execute_statement`, so the one it hands
    /// over is not counted twice.
    pub(crate) coverage_skip_line: Option<usize>,

    /// The last match and the last line read that each suspended fiber left
    /// behind, which it sees again when it is resumed.
    pub(crate) fiber_last_match_and_line: HashMap<usize, (Object, Object)>,
    /// The exception each suspended fiber is handling, with its backtrace,
    /// which `$!` and `$@` read while that fiber runs.
    pub(crate) fiber_errors: HashMap<usize, (Object, Object)>,
    /// The file each suspended fiber was running code from, which `__FILE__`
    /// and backtraces read again when it is resumed.
    pub(crate) fiber_source_files: HashMap<usize, Option<String>>,

    /// Where a number variable too big to name a capture has already been
    /// reported, so each place in the source says so once.
    pub(crate) reported_big_number_variables: std::collections::HashSet<(String, usize, usize)>,
    /// The tracepoints switched on, in the order they were. Empty almost
    /// always, which is what keeps the check on each statement cheap.
    pub(crate) tracepoints: Vec<Object>,
    /// Where the method and block bodies running now were written, innermost
    /// last. Each entry holds the places its code was written within, which
    /// is what a trace aimed at one method or block checks an event against.
    pub(crate) running_code: Vec<crate::vm::tracepoint::RunningCode>,
    /// How many native methods are running inside the innermost Ruby body,
    /// whose own calls are Ruby's C code and fire no `c_call` events.
    pub(crate) native_calls_running: usize,
    /// The file a library metorex carries is read from while it loads, which
    /// is where a method it defines says it was written.
    pub(crate) loading_embedded_library: Option<String>,
    /// Where a constant or a `class` written in a block run as a class body
    /// lands, paired with the depth of `def_scope_stack` it applies at. Ruby
    /// keeps such a block's constants in the scope the block was written in,
    /// None being the top level, while `def` still defines on the class.
    pub(crate) constant_homes: Vec<(usize, Option<Rc<crate::class::Class>>)>,
    /// The line a `:line` event was last fired for, so one statement does not
    /// fire twice and a multi-line expression fires once.
    pub(crate) traced_line: Option<(String, usize)>,
    /// Whether a tracepoint handler is running. Ruby does not call a handler
    /// from inside its own.
    pub(crate) tracing: bool,
    /// The id the next object to be asked for one takes.
    pub(crate) next_object_id: u64,
    /// Pairs of hashes a comparison is part-way through, so a hash holding
    /// itself is answered by the structure around it rather than followed
    /// forever.
    pub(crate) hash_comparisons: Vec<(usize, usize)>,
    /// Collections a hash walk is part-way through, and whether the walk
    /// reached one of them again. Ruby answers a collection that holds itself
    /// from its length alone, so the whole walk collapses once that happens.
    pub(crate) hash_walk: Vec<usize>,
    pub(crate) hash_walk_looped: bool,
    /// Every fiber the program has made, named by its place here. A `Fiber`
    /// object carries that number rather than the coroutine itself.
    pub(crate) fibers: Vec<crate::vm::fibers::FiberState>,
    /// The fibers holding the interpreter, innermost last. A fiber suspends
    /// through the handle the innermost frame carries.
    pub(crate) fiber_frames: Vec<crate::vm::fibers::FiberFrame>,
    /// The fiber the program starts on, which every other one is resumed
    /// from. It is made the first time something asks for it.
    pub(crate) root_fiber: Option<Object>,
    /// The names the fiber a program starts on keeps for itself.
    pub(crate) root_storage: Option<Object>,
    /// The main script's canonical path paired with the path it was named by,
    /// which is what `__FILE__` reports while it is the file running.
    pub(crate) script_path: Option<(PathBuf, PathBuf)>,
    /// Blocks registered by `at_exit`, run in reverse order once the program
    /// is over. One registered while they run goes to the end, so it runs
    /// right after the handler that registered it.
    pub(crate) at_exit_handlers: Vec<Object>,
    /// The places a `BEGIN` block was written, so one reached again while a
    /// program reads its input line by line runs only the first time.
    pub(crate) opened_blocks: std::collections::HashSet<(String, usize, usize)>,
    /// How many frames under the top one a report writes out, or -1 when it
    /// writes every one of them. `--backtrace-limit` settles it.
    pub(crate) backtrace_limit: i64,
    /// Where a class variable written inside `instance_exec` or
    /// `instance_eval` belongs: the class or module the block was written in,
    /// rather than the object it runs against.
    pub(crate) class_var_home: Vec<Rc<crate::class::Class>>,

    /// The class or module a class variable belongs to, one entry per method
    /// body or block body being run. A block written where no class or
    /// module is open records None, which is what makes `@@x` there a read
    /// from the top level.
    pub(crate) class_var_cref_stack: Vec<Option<Rc<crate::class::Class>>>,
    /// The frozen strings `dedup` and `-@` share, keyed by the text and the
    /// encoding it is written in. Two equal strings deduplicate to one object.
    pub(crate) deduped_strings: HashMap<(String, String), Rc<crate::object::StringValue>>,
    /// How many times the program has built an instance of each class, which
    /// is what the object space reports in place of walking a heap.
    pub(crate) allocation_counts: HashMap<usize, i64>,
    /// The listeners and connections a program holds open.
    pub(crate) open_sockets: crate::vm::native_methods::OpenSockets,
    /// The file descriptors an IO object of this program stands over.
    pub(crate) open_streams: crate::vm::native_methods::OpenStreams,
    /// The file whose code is running right now, which differs from
    /// `current_file` inside a method defined in another file.
    pub(crate) current_source_file: Option<String>,
    /// The encoding the source running now is written in, which is what
    /// `__ENCODING__` answers. A file names it in a magic comment, and an
    /// eval takes it from the string it was handed.
    pub(crate) current_source_encoding: Option<String>,
    /// The encoding each loaded file names in its magic comment, against the
    /// spelling the file was named by. A method's body is written in the file
    /// it was defined in, whatever file is calling it.
    pub(crate) file_encodings: HashMap<String, String>,
    /// The spelling each loaded file was named by, against the path its
    /// symlinks resolve to. `__FILE__` and a backtrace name the spelling,
    /// while everything that loads or dedups works from the resolved path.
    pub(crate) reported_files: std::collections::HashMap<std::path::PathBuf, std::path::PathBuf>,
    /// The limits `Timeout.timeout` has open, innermost last: when each one
    /// runs out, and what to raise when it does. A sleep that would run past
    /// the nearest one ends the block instead.
    pub(crate) timeout_limits: Vec<(std::time::Instant, Object, Object)>,
    /// Every object sent to another Ractor with `move: true`, held so its
    /// address names it alone, and those addresses.
    pub(crate) moved_objects: (Vec<Object>, std::collections::HashSet<usize>),
    /// How deep the machine is inside a body-less method stub standing in for
    /// a native one. A method bound explicitly reaches its native body even
    /// on an object whose class answers nothing of Kernel's.
    pub(crate) bound_stub_depth: usize,
    /// The method invocation a block written right now would return from.
    /// A `return` inside a block unwinds to the method that created the
    /// block, so the block records this id and the unwinding stops at the
    /// invocation that matches.
    pub(crate) current_method_frame: Option<u64>,
    /// The Sets a walk is running over, by address. A mutation while one is
    /// open is what Ruby reports as a modification during iteration.
    pub(crate) iterating_sets: Vec<usize>,
    /// The invocations that have not returned yet. A `return` from a block
    /// whose home invocation is gone has nowhere to go, which is what makes
    /// it a LocalJumpError.
    pub(crate) live_frames: Vec<u64>,
    /// The id the next method invocation takes.
    pub(crate) next_method_frame: u64,
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
    /// Whether the waiting threads are already being given a turn, so a
    /// thread that waits inside its own turn does not start the round again.
    pub(crate) stepping_threads: bool,
    /// The fibers a thread's own body runs on. A thread's body is its root
    /// fiber rather than one the program made, so the names it keeps under
    /// `Thread#[]` are the thread's own.
    pub(crate) thread_body_fibers: Vec<usize>,
    /// Every mutex a lock has been taken on, so the locks a thread still
    /// holds can be let go when the thread ends.
    /// Whether the fiber that just handed control back did so because it is
    /// waiting on something rather than because it yielded a value. Waiting
    /// inside a fiber holds up the whole thread, so whoever resumed it waits
    /// too and resumes it again afterwards.
    pub(crate) blocking_in_fiber: bool,
    /// How many locks the thread running now has taken since it was last
    /// given a turn. A thread that takes enough of them hands the turn over,
    /// so one looping on a lock still lets the others run.
    pub(crate) locks_this_turn: usize,
    /// How many statements have run since the turn was last handed over. A
    /// thread that runs enough of them hands the turn over, so one that never
    /// waits on anything still lets the others run.
    pub(crate) statements_this_turn: usize,
    /// What a thread that asked for its exceptions to take the program down
    /// died of, waiting to be raised where the program next waits.
    pub(crate) thread_abort: Option<Object>,
    pub(crate) taken_mutexes: Vec<Rc<std::cell::RefCell<crate::object::Instance>>>,
    /// Stack of canonical paths whose body is *currently executing* via
    /// `execute_file`. The path joins this stack on entry (after the
    /// `$"` mark goes in) and leaves on exit. Distinct from `$"` because
    /// `$"` is added eagerly before the body runs to short-circuit
    /// recursive requires; this stack tells autoload "the constant
    /// hasn't been defined yet because the file is mid-execution, don't
    /// re-load."
    /// How far the lines of the source running now are shifted from the ones
    /// the lexer counted, which is what lets code counted from a line of its
    /// own report the numbers it was given.
    pub(crate) source_line_shift: i64,
    /// The files being loaded right now, each with the thread loading it, so
    /// a thread that asks for a file another is part-way through waits for it.
    pub(crate) loading_paths: Vec<(String, Object)>,
    /// Autoloads currently being loaded, with the thread that initiated
    /// the load. Stored as `(class, name, loading_thread)` triples.
    /// `effective_autoload` consults this list to differentiate the
    /// "loading thread" view (sees the autoload as cleared) from the
    /// "other thread" view (sees the autoload as still active).
    pub(crate) autoload_loading: Vec<(Rc<crate::class::Class>, String, Object)>,
    /// Trailing block passed to the current call (e.g., `foo() do |x| ... end`).
    /// Set before invoke_method/invoke_callable; taken at method body entry.
    pub(crate) pending_block: Option<Object>,
    /// Whether `pending_block` arrived as `&expr` rather than as a literal
    /// block. `Kernel#lambda` rejects a non-lambda proc passed that way.
    pub(crate) pending_block_from_ampersand: bool,
    /// The running flags of the blocks attached to the calls now in
    /// progress, innermost last. Each is cleared when its call returns.
    pub(crate) attached_block_flags: Vec<Rc<std::cell::Cell<bool>>>,
    /// Whether a method defined in top-level code is public, which a bare
    /// `public` or `private` there sets. Each file starts private.
    pub(crate) toplevel_public: bool,
    /// The method frame the running file's top level runs in, which is
    /// the frame of whatever loaded it.
    pub(crate) toplevel_frame: Option<u64>,
    /// For each method invocation running, keyed by its frame, the class or
    /// module the method was defined in and the depth of the def scope
    /// stack when it started. A `def` written in the method's body installs
    /// there, unless a class body opened since is deeper.
    pub(crate) method_definees: HashMap<u64, (usize, Definee)>,
    /// Frames running code their method did not write: a file's top level,
    /// which runs in the frame of the method that loaded it, and a block
    /// `instance_exec` or `class_exec` runs. A `def` there is written in no
    /// method.
    pub(crate) borrowed_frames: Vec<Option<u64>>,
    /// The methods already warned about being passed a block they never
    /// use, known by where each was defined and its name, which warn once
    /// each.
    pub(crate) unused_block_methods: HashSet<MethodDefinitionKey>,
    /// For each block body now running, innermost last, the flag saying
    /// whether its call is still running. None for a lambda, where `break`
    /// ends the lambda itself.
    pub(crate) running_block_breaks: Vec<Option<Rc<std::cell::Cell<bool>>>>,
    /// The object a `&` handed over as the block, where it was already a
    /// callable. `Proc.new(&callable)` answers that same callable.
    pub(crate) pending_block_source: Option<Object>,
    /// The Mersenne Twister `Kernel#rand` draws from, which is the generator
    /// Ruby's own numbers come from, so a seed gives the same sequence here.
    pub(crate) random_words: Vec<u32>,
    /// How far through the words the generator has read.
    pub(crate) random_at: usize,
    /// The seed `srand` last installed, which it answers on the next call.
    pub(crate) random_seed: Object,
    /// True while a FrozenError message is being built. Inspecting the object
    /// can itself try to modify it, and the nested error must not recurse.
    pub(crate) rendering_frozen_error: bool,
    /// The method frame a block opened right now belongs to, while a block
    /// body is running. A block written inside another belongs to the method
    /// the outer one was written in, not to whatever method is running it.
    pub(crate) lexical_home_frame: Option<Option<u64>>,
    /// The scope the main script's own top level runs in. TOPLEVEL_BINDING
    /// stands over it, so the locals it names are whatever that scope holds
    /// when it is asked.
    pub(crate) main_script_scope: Option<Rc<std::cell::RefCell<crate::scope::Scope>>>,
    /// The places a pattern was written on its own as a condition. Ruby says
    /// so about each of them once.
    pub(crate) regexp_conditions: HashSet<(String, usize, usize, usize)>,
    /// The value a `case` is matching, with what it answered when it was
    /// asked for its elements. One `case` asks its own subject once, and
    /// every clause in it reads that; a value nested inside a pattern is
    /// asked on its own.
    pub(crate) deconstructed_values: Vec<(Object, Option<Vec<Object>>)>,
    /// Why the pattern being tried failed, which a `case` holding a single
    /// `in` clause names in the error it raises. The first failure recorded
    /// is the innermost one, which is the one Ruby reports.
    pub(crate) pattern_failure: Option<crate::vm::pattern_matching::PatternFailure>,
    /// The receiver a Kernel function was written with, which `Kernel.eval`
    /// and `obj.send(:eval, ...)` run the code against in place of the self
    /// in force where the call was made.
    pub(crate) kernel_function_receiver: Option<Object>,
    /// How many lambda bodies are running, so a `return` carried out of an
    /// eval can tell whether a lambda is there to return from.
    pub(crate) lambda_body_depth: usize,
    /// Whether the program asked for the place each object was made to be
    /// recorded, which `objspace/trace` and `trace_object_allocations` do.
    pub(crate) tracing_allocations: bool,
    /// How many `trace_object_allocations_start` calls are still open.
    pub(crate) allocation_tracing_depth: usize,
    /// Where each object made while tracing was on was made, by address.
    pub(crate) allocation_sites:
        HashMap<usize, (WeakTarget, crate::vm::allocation_sites::AllocationSite)>,
    /// Whether `--debug-frozen-string-literal` was written, which has every
    /// string literal remember where it was written so a refused change can
    /// name the place.
    pub(crate) debug_frozen_string_literal: bool,
    /// Collections frozen by `freeze`, keyed by the address they live at.
    /// An Array, Hash, or Set has nowhere of its own to record the flag. The
    /// value keeps the collection alive, so its address cannot be recycled by
    /// a later one and read back as frozen.
    pub(crate) frozen_collections: HashMap<usize, Object>,

    /// Patterns built by `Regexp.new`, which a program may still change. A
    /// pattern written as a literal is frozen where it stands. Each entry
    /// holds its pattern weakly, so a literal made later at the same address
    /// is not read as the built one that was freed.
    pub(crate) built_patterns: HashMap<usize, std::rc::Weak<String>>,
    /// The sets `compare_by_identity` was called on, against the address each
    /// one lives at. A Set has nowhere of its own to record the flag, and the
    /// value keeps the set alive so a later one cannot take its address and
    /// read back as comparing by identity.
    pub(crate) identity_sets: HashMap<usize, Object>,
    /// The strings `pack` wrote a pointer to, against the pointer it wrote.
    /// `unpack` with 'P' or 'p' reads a string back out of here, which is how
    /// a packed pointer keeps naming what it was given.
    pub(crate) packed_pointers: HashMap<u64, Object>,
    /// The ranges `dup` made, which are not frozen the way a range a program
    /// wrote is. The mark a range carries names it here, and the value keeps
    /// that mark alive so a later one cannot take its address.
    pub(crate) thawed_ranges: HashMap<usize, Rc<()>>,
    /// The mark each range written with literal ends carries, against where
    /// it was written. Ruby builds such a range once, so every run of the
    /// line answers the same object.
    pub(crate) written_ranges: HashMap<(String, usize, usize), Rc<()>>,
    /// Where the `require` or `load` now running was written, which is what a
    /// warning raised at the top of the loaded file names as its caller.
    pub(crate) load_call_site: Option<(String, crate::lexer::Position)>,
    /// The encoding of the source string each `Regexp.new` pattern was built
    /// from, recorded against the pattern's address since a Regexp carries
    /// its source as plain text.
    /// Each entry holds the pattern weakly, so a pattern built later at the
    /// same address is not read as the one that was freed.
    pub(crate) pattern_encodings: HashMap<usize, (std::rc::Weak<String>, String)>,
    /// The encodings C set on patterns with `rb_enc_associate`, which a
    /// pattern reports whatever it holds.
    pub(crate) forced_pattern_encodings: HashMap<usize, (std::rc::Weak<String>, String)>,
    /// The instance variables set on an Array, Hash, or Set. A collection has
    /// nowhere of its own to keep them, so the VM records them against the
    /// address it lives at.
    pub(crate) collection_variables: HashMap<usize, HashMap<String, Object>>,
    /// The objects `collection_variables` holds variables for, kept so that
    /// none is freed and its address given to a new object, which would then
    /// read as carrying the old one's variables.
    pub(crate) collection_variable_owners: HashMap<usize, Object>,
    /// Handlers `Signal.trap` installed, keyed by signal name without its
    /// `SIG` prefix. A String value names a built-in disposition; anything
    /// else is a callable `Process.kill` runs in place of raising.
    pub(crate) signal_handlers: HashMap<String, Object>,
    /// Hooks `trace_var` registered, keyed by global name without its `$`.
    /// Each runs with the new value whenever that global is assigned.
    pub(crate) traced_globals: HashMap<String, Vec<Object>>,
    /// Names the environment already held once the builtins were seeded.
    /// `local_variables` reports the names a program bound, not these.
    pub(crate) seeded_global_names: HashSet<String>,
    /// Depth of nested wrapped `load(path, true)` calls. While >0, top-level
    /// `include` is suppressed (Ruby wraps the loaded scope in an anonymous
    /// module so includes don't pollute Object).
    pub(crate) load_wrap_depth: u32,
    /// The module a wrapped `load` runs its file inside, which its constants
    /// and top-level methods land on instead of Object.
    pub(crate) load_wrap_module: Option<Rc<crate::class::Class>>,
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
    /// The patterns written with `o`, kept under the site each was written
    /// at so the same one is answered every time that line is reached.
    pub(crate) patterns_built_once: std::collections::HashMap<String, Object>,
    /// Whether the flip-flop written at each place is on. A range written
    /// where a condition goes holds its own state between turns.
    pub(crate) flip_flops: std::collections::HashMap<(String, usize, usize, usize), bool>,
    /// How long a match of each pattern written with a limit of its own may
    /// take. A pattern written with `timeout: nil` is held here as None,
    /// which says it takes as long as it takes whatever the class names.
    pub(crate) pattern_timeouts: std::collections::HashMap<String, Option<std::time::Duration>>,
    /// The binding of the method body that just finished, which a trace
    /// reading `binding` off a `return` event is handed. Captured before the
    /// body's scope is popped, since the locals are gone after that.
    pub(crate) traced_binding: Option<Object>,
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
    /// The methods whose bodies are running, innermost last. A bare `super`
    /// reads the parameters of the one on top as they stand.
    pub(crate) method_running_stack: Vec<Rc<crate::object::Method>>,
    /// What each WeakRef handle points at, indexed by the handle.
    pub(crate) weak_references: Vec<WeakTarget>,
    /// The libraries metorex carries that have run, each named by where its
    /// text sits, so a second name for one does not run it again.
    pub(crate) embedded_sources_run: std::collections::HashSet<usize>,
    /// The module each running method was defined in, innermost last. A
    /// `super` starts its walk from here, which is the only way to place a
    /// method defined in an anonymous module.
    pub(crate) method_owner_stack: Vec<Option<Rc<crate::class::Class>>>,
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
    /// Hash literals already reported for a duplicated key, by source file and
    /// position. Ruby names a duplicate once for the literal as written, so a
    /// literal inside a loop is reported the once.
    pub(crate) reported_duplicate_keys: std::collections::HashSet<(String, usize, usize)>,
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
        register_builtin_modules(&mut globals, &builtins);
        // After the singletons, which create the canonical Object that every
        // exception class descends from.
        register_singletons(&mut globals);
        register_exception_classes(&mut globals);
        register_special_globals(&mut globals);
        register_native_functions(&mut globals);

        seed_environment_with_globals(&mut environment, &globals);
        let seeded_global_names = environment.current_scope_vars().into_keys().collect();

        let mut vm = Self {
            // `$-0` is `$/` under the flag's name, and so on for the others
            // the command line spells with a dash.
            hooked_globals: std::collections::HashMap::new(),
            global_aliases: [
                ("-0", "/"),
                ("-v", "VERBOSE"),
                ("-w", "VERBOSE"),
                ("-d", "DEBUG"),
                ("-I", ":"),
            ]
            .into_iter()
            .map(|(alias, original)| (alias.to_string(), original.to_string()))
            .collect(),
            environment,
            call_stack: Vec::new(),
            globals,
            heap: Rc::new(RefCell::new(Heap::default())),
            builtins,
            current_file: None,
            coverage: None,
            coverage_skip_line: None,
            fiber_last_match_and_line: HashMap::new(),
            fiber_errors: HashMap::new(),
            fiber_source_files: HashMap::new(),
            reported_big_number_variables: std::collections::HashSet::new(),
            tracepoints: Vec::new(),
            running_code: Vec::new(),
            native_calls_running: 0,
            loading_embedded_library: None,
            constant_homes: Vec::new(),
            traced_line: None,
            tracing: false,
            next_object_id: 1,
            hash_comparisons: Vec::new(),
            hash_walk: Vec::new(),
            hash_walk_looped: false,
            fibers: Vec::new(),
            fiber_frames: Vec::new(),
            root_fiber: None,
            root_storage: None,
            script_path: None,
            at_exit_handlers: Vec::new(),
            opened_blocks: std::collections::HashSet::new(),
            backtrace_limit: -1,
            class_var_home: Vec::new(),
            class_var_cref_stack: Vec::new(),
            deduped_strings: HashMap::new(),
            allocation_counts: HashMap::new(),
            open_sockets: Default::default(),
            open_streams: Default::default(),
            current_source_file: None,
            current_source_encoding: None,
            file_encodings: HashMap::new(),
            reported_files: std::collections::HashMap::new(),
            timeout_limits: Vec::new(),
            moved_objects: (Vec::new(), std::collections::HashSet::new()),
            bound_stub_depth: 0,
            current_method_frame: Some(TOP_LEVEL_FRAME),
            iterating_sets: Vec::new(),
            live_frames: vec![TOP_LEVEL_FRAME],
            next_method_frame: TOP_LEVEL_FRAME + 1,
            loaded_files: HashSet::new(),
            autoload_const_access_depth: 0,
            thread_current_stack: Vec::new(),
            pending_threads: Vec::new(),
            stepping_threads: false,
            thread_body_fibers: Vec::new(),
            blocking_in_fiber: false,
            locks_this_turn: 0,
            statements_this_turn: 0,
            thread_abort: None,
            taken_mutexes: Vec::new(),
            source_line_shift: 0,
            loading_paths: Vec::new(),
            autoload_loading: Vec::new(),
            pending_block: None,
            pending_block_from_ampersand: false,
            attached_block_flags: Vec::new(),
            toplevel_public: false,
            toplevel_frame: Some(TOP_LEVEL_FRAME),
            method_definees: HashMap::new(),
            borrowed_frames: Vec::new(),
            unused_block_methods: HashSet::new(),
            running_block_breaks: Vec::new(),
            pending_block_source: None,
            random_words: Vec::new(),
            random_at: 0,
            random_seed: Object::Int(seed_from_clock() as i64),
            rendering_frozen_error: false,
            lexical_home_frame: None,
            main_script_scope: None,
            regexp_conditions: HashSet::new(),
            deconstructed_values: Vec::new(),
            pattern_failure: None,
            kernel_function_receiver: None,
            lambda_body_depth: 0,
            tracing_allocations: false,
            allocation_tracing_depth: 0,
            allocation_sites: HashMap::new(),
            debug_frozen_string_literal: false,
            frozen_collections: HashMap::new(),
            built_patterns: HashMap::new(),
            identity_sets: HashMap::new(),
            packed_pointers: HashMap::new(),
            thawed_ranges: HashMap::new(),
            written_ranges: HashMap::new(),
            load_call_site: None,
            pattern_encodings: HashMap::new(),
            forced_pattern_encodings: HashMap::new(),
            collection_variables: HashMap::new(),
            collection_variable_owners: HashMap::new(),
            // The interpreter answers for an interrupt itself, so that one
            // reads as written for from the start while every other signal
            // is still the operating system's to answer.
            signal_handlers: HashMap::from([
                ("INT".to_string(), Object::string("DEFAULT".to_string())),
                // A broken pipe is set aside at the start, which is what
                // makes a write to one an error the program hears about
                // rather than the end of it.
                ("PIPE".to_string(), Object::Nil),
            ]),
            traced_globals: HashMap::new(),
            seeded_global_names,
            load_wrap_depth: 0,
            load_wrap_module: None,
            user_def_nesting: 0,
            refinement_scopes: vec![Vec::new()],
            def_scope_stack: Vec::new(),
            patterns_built_once: std::collections::HashMap::new(),
            flip_flops: std::collections::HashMap::new(),
            pattern_timeouts: std::collections::HashMap::new(),
            traced_binding: None,
            primitive_singleton_classes: std::collections::HashMap::new(),
            method_arg_stack: Vec::new(),
            method_running_stack: Vec::new(),
            weak_references: Vec::new(),
            embedded_sources_run: std::collections::HashSet::new(),
            method_owner_stack: Vec::new(),
            method_nesting_stack: Vec::new(),
            catch_tags: Vec::new(),
            autoload_reload_depth: 0,
            reported_duplicate_keys: std::collections::HashSet::new(),
        };
        // Ruby always has an ARGV, so a program that reads or replaces it
        // works whether or not a caller has handed one over.
        vm.set_argv(Vec::new());
        vm.load_prelude();
        vm.open_standard_streams();
        // The prelude's own class and module names are part of the core
        // library, not locals the program declared, so `local_variables` and
        // friends have to keep skipping them.
        vm.seeded_global_names
            .extend(vm.environment.current_scope_vars().into_keys());
        // The program runs in a scope of its own beneath the one holding the
        // core library, so a method, which sees only that root scope, does
        // not see the program's top-level locals, and a block closes over
        // the program's locals without the core library's names.
        vm.environment.push_isolated_scope();
        vm
    }

    /// Record the exception a rescue clause is handling. Ruby exposes it as
    /// `$!`, and its backtrace as `$@`.
    /// Give `raised` the exception it followed as its `#cause`. Ruby sets it
    /// once, and never to the exception itself.
    pub(crate) fn record_cause(raised: &Object, cause: &Object) {
        let (Object::Exception(raised_ref), Object::Exception(cause_ref)) = (raised, cause) else {
            return;
        };
        if Rc::ptr_eq(raised_ref, cause_ref)
            || raised_ref.borrow().cause.is_some()
            || raised_ref.borrow().cause_settled
        {
            return;
        }
        raised_ref.borrow_mut().cause = Some(Box::new(cause.clone()));
    }

    /// Put `$!` back to what it was before a rescue clause ran. A clause
    /// that handled an exception leaves the one it interrupted in place, so
    /// an outer handler still names what it is handling.
    pub(crate) fn restore_current_exception(&mut self, exception: Object) {
        let backtrace = match &exception {
            Object::Exception(details) => details
                .borrow()
                .backtrace
                .as_ref()
                .map(|trace| {
                    let entries: Vec<Object> = trace
                        .iter()
                        .map(|line| Object::string(line.clone()))
                        .collect();
                    Object::Array(Rc::new(RefCell::new(entries)))
                })
                .unwrap_or(Object::Nil),
            _ => Object::Nil,
        };
        self.environment()
            .global_scope()
            .borrow_mut()
            .define("$!".to_string(), exception.clone());
        self.globals_mut().set_variable("!", exception);
        self.globals_mut().set_variable("@", backtrace);
    }

    /// Say where an exception the interpreter raised came from, for one that
    /// carries no place of its own. The backtrace is built from it, so a
    /// NameError raised by a name lookup names the line that read the name.
    pub(crate) fn note_exception_location(
        &self,
        exception: &Object,
        location: &crate::error::SourceLocation,
    ) {
        if location.line == 0 {
            return;
        }
        if let Object::Exception(details) = exception {
            let mut held = details.borrow_mut();
            if held.location.is_none() && held.backtrace.is_none() {
                let named = location
                    .filename
                    .clone()
                    .or_else(|| self.current_source_file.clone())
                    .or_else(|| {
                        self.current_file
                            .as_ref()
                            .map(|file| file.display().to_string())
                    })
                    .unwrap_or_else(|| "script".to_string());
                held.location = Some(crate::object::SourceLocation::new(
                    named,
                    location.line,
                    location.column,
                ));
            }
        }
    }

    pub(crate) fn set_current_exception(&mut self, exception: Object) {
        // The word that a thread or fiber is being stopped is not the
        // program's to see, so it never becomes `$!`.
        if let Object::Exception(details) = &exception
            && details.borrow().exception_type == crate::vm::fibers::FIBER_KILLED
        {
            return;
        }
        // An exception reaching a rescue clause takes the one that was active
        // as its `#cause`, which is how an error raised inside a rescue body
        // records what it followed. Set once, and never to itself.
        if let Object::Exception(details) = &exception
            && details.borrow().cause.is_none()
            && !details.borrow().cause_settled
            && let Some(active) = self.globals().get("!")
            && let Object::Exception(active_ref) = &active
            && !Rc::ptr_eq(details, active_ref)
        {
            details.borrow_mut().cause = Some(Box::new(active.clone()));
        }
        // An exception raised by the interpreter itself carries no backtrace
        // of its own, and one being handled has to report where it came from,
        // so the stack it was raised on is recorded here.
        let exception = match &exception {
            Object::Exception(details) if details.borrow().backtrace.is_none() => {
                let position = details
                    .borrow()
                    .location
                    .as_ref()
                    .map(|held| crate::lexer::Position::new(held.line, held.column, 0))
                    .unwrap_or_else(|| crate::lexer::Position::new(0, 0, 0));
                self.add_stack_trace_to_exception(exception.clone(), position)
            }
            _ => exception,
        };
        self.environment()
            .global_scope()
            .borrow_mut()
            .define("$!".to_string(), exception.clone());
        let backtrace = match &exception {
            Object::Exception(details) => details
                .borrow()
                .backtrace
                .as_ref()
                .map(|trace| {
                    let entries: Vec<Object> = trace
                        .iter()
                        .map(|line| Object::string(line.clone()))
                        .collect();
                    Object::Array(Rc::new(RefCell::new(entries)))
                })
                .unwrap_or(Object::Nil),
            _ => Object::Nil,
        };
        self.globals_mut().set_variable("!", exception);
        self.globals_mut().set_variable("@", backtrace);
    }

    /// A `SourceLocation` for `position`, tagged with the file being run so
    /// `Method#source_location` can name it.
    pub(crate) fn source_location_for(
        &self,
        position: crate::lexer::Position,
    ) -> crate::error::SourceLocation {
        let mut location =
            crate::error::SourceLocation::new(position.line, position.column, position.offset);
        // A `def` belongs to the file it was written in, which is not always
        // the file being run: a block from another file runs with that file
        // still current.
        location.filename = self
            .loading_embedded_library
            .clone()
            .or_else(|| self.current_source_file.clone())
            .or_else(|| {
                self.reported_current_file()
                    .map(|file| file.display().to_string())
            });
        location
    }

    /// Activate a refinement module in the innermost scope.
    pub(crate) fn activate_refinement(&mut self, module: Rc<crate::class::Class>) {
        let entries = Self::refinement_entries_for(&module);
        if let Some(top) = self.refinement_scopes.last_mut() {
            top.extend(entries);
        }
    }

    /// The refinement entries a module carries right now, including those it
    /// picks up from the modules it includes. Read at activation rather than
    /// held from an earlier point, so every refinement in a module is visible
    /// to the others no matter which was declared first.
    pub(crate) fn refinement_entries_for(module: &Rc<crate::class::Class>) -> Vec<RefinementEntry> {
        let mut entries = Vec::new();
        // What a module includes is recorded first and the module itself
        // last, since the search runs back through them and a refinement the
        // module writes stands ahead of one it takes in.
        let mut sources: Vec<Rc<crate::class::Class>> = module.transitive_mixins();
        sources.push(Rc::clone(module));
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
        entries
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
        let elements: Vec<Object> = args.into_iter().map(Object::string).collect();
        let argv = Object::Array(Rc::new(RefCell::new(elements)));
        self.globals.set("ARGV", argv.clone());
        // `$*` is the same list under the name Ruby's own punctuation gives
        // it, which `$ARGV` reads through.
        self.globals.set_variable("*", argv.clone());
        self.environment
            .global_scope()
            .borrow_mut()
            .define("ARGV".to_string(), argv);
        self.seeded_global_names.insert("ARGV".to_string());
    }

    /// Run a closure with a new call frame pushed onto the stack.
    pub fn with_call_frame<F, R>(&mut self, mut frame: CallFrame, action: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        frame.entered_from(self.environment.current_scope());
        self.call_stack.push(frame);
        let result = action(self);
        self.call_stack.pop();
        result
    }

    /// Push a frame that stays until it is popped, for scopes whose body is
    /// run by a loop rather than a single closure.
    pub(crate) fn call_stack_push(&mut self, mut frame: CallFrame) {
        frame.entered_from(self.environment.current_scope());
        self.call_stack.push(frame);
    }

    /// How many blocks deep a block about to run sits. One written straight
    /// in a method or a file is one deep, and one written inside another
    /// block is one deeper than that.
    pub(crate) fn block_nesting_depth(&self) -> u32 {
        match self.call_stack.last() {
            Some(frame) if frame.block_depth() > 0 => frame.block_depth() + 1,
            _ => 1,
        }
    }

    /// The scope a block written here belongs to. A block written inside
    /// another belongs to the same scope that one does, whatever the block
    /// is later called from, so the scope travels along the frames rather
    /// than being looked up on the stack.
    pub(crate) fn enclosing_scope_label(&self) -> String {
        match self.call_stack.last() {
            Some(frame) if frame.block_depth() > 0 => frame
                .written_in()
                .map(|held| held.to_string())
                .unwrap_or_else(|| "<main>".to_string()),
            Some(frame) => frame.name().to_string(),
            None => "<main>".to_string(),
        }
    }

    /// Pop the frame `call_stack_push` added.
    pub(crate) fn call_stack_pop(&mut self) {
        self.call_stack.pop();
    }

    /// The (callee, defined) names of the method currently running, or None
    /// at file or class-body scope. Block frames report the method that
    /// lexically encloses them rather than whichever method called them.
    /// The class path and the name of the method enclosing what is running,
    /// as `ObjectSpace.allocation_class_path` and `allocation_method_id`
    /// report them. A singleton method has no class path.
    pub(crate) fn enclosing_method_owner(&self) -> Option<(Option<String>, String)> {
        use crate::vm::FrameKind;
        for frame in self.call_stack.iter().rev() {
            match frame.kind() {
                FrameKind::Block => continue,
                FrameKind::Boundary => return None,
                FrameKind::Method { defined, .. } => {
                    return Some((frame.owner_path().map(str::to_string), defined.clone()));
                }
            }
        }
        None
    }

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

/// A starting seed drawn from the clock, so a fresh process draws a different
/// sequence than the last one.
pub(crate) fn seed_from_clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15)
        | 1
}

/// A method carries the class it was defined in, and a class carries its
/// methods, so the two hold each other up and neither is freed when the
/// machine that built them goes. A run that builds many machines, as the test
/// suite does, would keep every one of them.
///
/// Letting go of a machine walks the classes it can reach and has each drop
/// what points back at it, so the graph comes apart.
impl Drop for VirtualMachine {
    fn drop(&mut self) {
        let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut waiting: Vec<Rc<crate::class::Class>> = Vec::new();

        for class in self.builtins.all_classes().into_values() {
            waiting.push(class);
        }
        for (_, held) in self.globals.iter() {
            if let Object::Class(class) | Object::Module(class) = held {
                waiting.push(Rc::clone(class));
            }
        }

        let mut found: Vec<Rc<crate::class::Class>> = Vec::new();
        while let Some(class) = waiting.pop() {
            if !seen.insert(Rc::as_ptr(&class) as usize) {
                continue;
            }
            if let Some(parent) = class.superclass() {
                waiting.push(parent);
            }
            waiting.extend(class.mixin_chain());
            waiting.extend(class.prepend_chain());
            if let Some(singleton) = class.singleton_class_slot().clone() {
                waiting.push(singleton);
            }
            for held in class.class_variable_values() {
                if let Object::Class(inner) | Object::Module(inner) = held {
                    waiting.push(inner);
                }
            }
            found.push(class);
        }

        for class in found {
            class.tear_down();
        }
    }
}

/// An object a WeakRef points at without keeping it alive. A value Ruby
/// never frees, such as an Integer or a Symbol, is held as it is.
pub(crate) enum WeakTarget {
    Instance(std::rc::Weak<RefCell<crate::object::Instance>>),
    Array(std::rc::Weak<RefCell<Vec<Object>>>),
    Dict(std::rc::Weak<RefCell<indexmap::IndexMap<String, Object>>>),
    String(std::rc::Weak<crate::object::StringValue>),
    Held(Object),
}

impl WeakTarget {
    pub(crate) fn of(object: &Object) -> Self {
        match object {
            Object::Instance(held) => WeakTarget::Instance(Rc::downgrade(held)),
            Object::Array(held) => WeakTarget::Array(Rc::downgrade(held)),
            Object::Dict(held) => WeakTarget::Dict(Rc::downgrade(held)),
            Object::String(held) => WeakTarget::String(Rc::downgrade(held)),
            other => WeakTarget::Held(other.clone()),
        }
    }

    /// The object, when something else still holds it.
    pub(crate) fn reach(&self) -> Option<Object> {
        match self {
            WeakTarget::Instance(held) => held.upgrade().map(Object::Instance),
            WeakTarget::Array(held) => held.upgrade().map(Object::Array),
            WeakTarget::Dict(held) => held.upgrade().map(Object::Dict),
            WeakTarget::String(held) => held.upgrade().map(Object::String),
            WeakTarget::Held(object) => Some(object.clone()),
        }
    }
}
