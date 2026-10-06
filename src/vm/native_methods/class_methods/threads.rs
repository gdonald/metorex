// Threads, fibers and the objects they hand values through.

use super::*;
use crate::vm::ractors::RACTOR_VAR;

/// Where `Thread.report_on_exception=` keeps what it was set to.
const REPORT_ON_EXCEPTION_GLOBAL: &str = "__thread_report_on_exception";

impl VirtualMachine {
    /// Threads, fibers, queues and mutexes, along with the calendar helpers
    /// Time is written against.
    pub(crate) fn call_thread_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // Time is written in Ruby against a few calendar helpers the C
        // library carries out, so a local time follows the operating
        // system's zone rules.
        if class_rc.name() == "Time"
            && let Some(answered) =
                self.call_time_class_methods(method_name, arguments, position)?
        {
            return Ok(Answered(answered));
        }
        // Queue.new / SizedQueue.new — synchronous FIFO stub. The instance
        // carries an Array under `__queue_items`; SizedQueue ignores its
        // capacity argument (we never block).
        // Mutex.new / ConditionVariable.new — single-threaded stubs (no shared
        // state needed; the synchronize/wait/broadcast methods are no-ops).
        if method_name == "new"
            && (class_rc.name() == "Thread::Mutex"
                || class_rc.name() == "Thread::ConditionVariable")
        {
            use crate::object::Instance;
            let inst_rc = Instance::new(Rc::clone(class_rc));
            return Ok(Answered(Object::Instance(inst_rc)));
        }
        if method_name == "new"
            && (class_rc.name() == "Thread::Queue" || class_rc.name() == "Thread::SizedQueue")
        {
            use crate::object::Instance;
            let inst_rc = Instance::new(Rc::clone(class_rc));
            inst_rc.borrow_mut().set_var(
                "__queue_items".to_string(),
                Object::Array(Rc::new(std::cell::RefCell::new(Vec::new()))),
            );
            // `Queue.new(enumerable)` starts the queue off with what the
            // enumerable holds, in the order it holds them.
            if class_rc.name() == "Thread::Queue"
                && let Some(held) = arguments.first()
            {
                let seeded = self.queue_seed_argument(held, position)?;
                inst_rc
                    .borrow_mut()
                    .set_var("__queue_items".to_string(), Object::array(seeded));
            }
            // `SizedQueue.new(n)` says how many the queue holds, which it
            // reports whether or not anything ever waits on it.
            if class_rc.name() == "Thread::SizedQueue" {
                let counted = match arguments.first() {
                    Some(held) => self.queue_capacity_argument(held, position)?,
                    None => {
                        return Err(method_argument_error("new", 1, 0, position));
                    }
                };
                inst_rc
                    .borrow_mut()
                    .set_var("__queue_max".to_string(), Object::Int(counted));
            }
            return Ok(Answered(Object::Instance(inst_rc)));
        }
        // Thread.new captures the block; we run it lazily on `value` so that
        // serialized "concurrent" specs (which set a shared flag between
        // construction and value-collection) still observe the flag change.
        // Newly-constructed threads land on `pending_threads` so an empty
        // `Queue#pop` (which would block in real Ruby) can drain them and
        // make forward progress.
        if matches!(method_name, "new" | "start" | "fork")
            && class_named_in_chain(class_rc, "Thread")
        {
            use crate::object::Instance;
            let block = self.pending_block.take().unwrap_or(Object::Nil);
            let inst_rc = Instance::new(Rc::clone(class_rc));
            let obj = Object::Instance(Rc::clone(&inst_rc));
            // A thread belongs to the Ractor of the thread that made it.
            if let Object::Instance(maker) = self.running_thread()
                && let Some(ractor) = maker.borrow().get_var(RACTOR_VAR).cloned()
            {
                inst_rc.borrow_mut().set_var(RACTOR_VAR.to_string(), ractor);
            }
            // A subclass may write its own `initialize`, and what it hands to
            // `super` is what the thread runs. `start` and `fork` never go
            // through it, which is what tells them apart from `new`.
            let own_initialize = match self.lookup_method(&obj, "initialize") {
                Some((defined_in, method)) if method_name == "new" => match defined_in.name() {
                    "Thread" | "Object" | "BasicObject" => None,
                    _ => Some((defined_in, method)),
                },
                _ => None,
            };
            match own_initialize {
                Some((defined_in, method)) => {
                    self.pending_block = match block {
                        Object::Nil => None,
                        held => Some(held),
                    };
                    self.invoke_method(
                        defined_in,
                        method,
                        obj.clone(),
                        arguments.to_vec(),
                        position,
                    )?;
                }
                None => {
                    // A thread has to be given something to run. `new`
                    // reports that as a ThreadError, where `start` and `fork`
                    // report it the way any method missing its block does.
                    if matches!(block, Object::Nil) {
                        return Err(crate::vm::errors::simple_exception(
                            if method_name == "new" {
                                "ThreadError"
                            } else {
                                "ArgumentError"
                            },
                            "must be called with a block",
                            position,
                        ));
                    }
                    self.give_thread_a_body(&inst_rc, block, arguments);
                }
            }
            if inst_rc.borrow().get_var("__thread_block").is_none() {
                return Err(crate::vm::errors::simple_exception(
                    "ThreadError",
                    "uninitialized thread - check `Thread#initialize'",
                    position,
                ));
            }
            self.pending_threads.push(obj.clone());
            return Ok(Answered(obj));
        }
        // `Fiber.new { ... }` makes a fiber the block runs on when it is
        // first resumed.
        if method_name == "new" && class_named_in_chain(class_rc, "Fiber") {
            use crate::object::Instance;
            let Some(Object::Block(block)) = self.pending_block.take() else {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "tried to create a Fiber without a block",
                    position,
                ));
            };
            // `Fiber.new(blocking: true)` asks for a fiber that blocks when
            // it waits rather than handing control to a scheduler.
            let blocking = match arguments.last() {
                Some(Object::Dict(options)) => options
                    .borrow()
                    .get(":blocking")
                    .is_some_and(|held| held.is_truthy()),
                _ => false,
            };
            // `storage:` names what the fiber keeps for itself. Without it
            // the fiber inherits what the one making it held.
            let named = match arguments.last() {
                Some(Object::Dict(options)) => options.borrow().get(":storage").cloned(),
                _ => None,
            };
            let storage = match named {
                // A fiber inherits a copy of what the one making it keeps, so
                // writing a name in the new fiber leaves the old one alone.
                None => {
                    let holder = self.fiber_current_handle();
                    match self.fiber_storage_if_held(holder) {
                        Some(Object::Dict(held)) => {
                            let copied = held.borrow().clone();
                            Some(Object::Dict(Rc::new(std::cell::RefCell::new(copied))))
                        }
                        other => other,
                    }
                }
                Some(Object::Nil) => None,
                Some(held) => {
                    self.check_fiber_storage(&held, position)?;
                    Some(held)
                }
            };
            let handle = self.fiber_create(block, blocking, storage);
            let inst_rc = Instance::new(Rc::clone(class_rc));
            inst_rc
                .borrow_mut()
                .set_var("__fiber__".to_string(), Object::Int(handle as i64));
            return Ok(Answered(Object::Instance(inst_rc)));
        }
        if class_named_in_chain(class_rc, "Fiber") {
            match method_name {
                // `Fiber.yield` suspends the fiber holding the interpreter,
                // handing its arguments to whoever resumed it.
                "yield" => {
                    let handed = crate::vm::fibers::passing_value(arguments.to_vec());
                    let current = self.fiber_current_handle();
                    if let Some(state) = self.fibers.get_mut(current) {
                        state.yielding = true;
                    }
                    let given = self.fiber_suspend(handed, position)?;
                    return Ok(Answered(crate::vm::fibers::passing_value(given)));
                }
                "current" => {
                    return Ok(Answered(self.fiber_current()));
                }
                // The scheduler a fiber that is not blocking hands its
                // waiting over to. Setting nil takes it away again.
                "set_scheduler" if arguments.len() == 1 => {
                    self.set_thread_scheduler(arguments[0].clone(), position)?;
                    return Ok(Answered(arguments[0].clone()));
                }
                "scheduler" => {
                    return Ok(Answered(self.thread_scheduler().unwrap_or(Object::Nil)));
                }
                "current_scheduler" => {
                    return Ok(Answered(self.current_scheduler().unwrap_or(Object::Nil)));
                }
                // `Fiber.schedule` asks the thread's scheduler for a fiber
                // running the block.
                "schedule" => {
                    let Some(scheduler) = self.thread_scheduler() else {
                        return Err(crate::vm::errors::simple_exception(
                            "RuntimeError",
                            "No scheduler is available!",
                            position,
                        ));
                    };
                    // The block stays pending, so the scheduler's `fiber`
                    // is the method handed it.
                    return Ok(Answered(self.send_to_object(
                        scheduler,
                        "fiber",
                        arguments.to_vec(),
                        position,
                    )?));
                }
                // `Fiber.blocking { |f| ... }` runs the block with the
                // running fiber blocking, and puts back what it was after.
                "blocking" if self.pending_block.is_some() => {
                    let Some(Object::Block(block)) = self.pending_block.take() else {
                        return Ok(Answered(Object::Nil));
                    };
                    let held = self.fiber_current_handle();
                    let was = self.fiber_set_blocking(held, true);
                    let current = self.fiber_current();
                    let answered = self.execute_block_body(&block, vec![current]);
                    self.fiber_set_blocking(held, was);
                    return answered.map(Answered);
                }
                // `Fiber[:name]` reads what the running fiber keeps under
                // that name, and `Fiber[:name] = held` writes it.
                "[]" if arguments.len() == 1 => {
                    let named = self.fiber_storage_name(&arguments[0], position)?;
                    let running = self.fiber_current_handle();
                    let Some(Object::Dict(held)) = self.fiber_storage_if_held(running) else {
                        return Ok(Answered(Object::Nil));
                    };
                    let found = held.borrow().get(&named).cloned();
                    return Ok(Answered(found.unwrap_or(Object::Nil)));
                }
                "[]=" if arguments.len() == 2 => {
                    let named = self.fiber_storage_name(&arguments[0], position)?;
                    let running = self.fiber_current_handle();
                    let Object::Dict(held) = self.fiber_storage(running) else {
                        return Ok(Answered(arguments[1].clone()));
                    };
                    // A name given nil is dropped rather than kept as nil.
                    if matches!(arguments[1], Object::Nil) {
                        held.borrow_mut().shift_remove(&named);
                    } else {
                        held.borrow_mut().insert(named, arguments[1].clone());
                    }
                    return Ok(Answered(arguments[1].clone()));
                }
                // Ruby answers the number of the blocking level here rather
                // than a flag, and false where nothing is blocking.
                "blocking?" => {
                    let held = self.fiber_current_handle();
                    return Ok(Answered(if self.fiber_is_blocking(held) {
                        Object::Int(1)
                    } else {
                        Object::Bool(false)
                    }));
                }
                _ => {}
            }
        }
        // Thread.pass / Thread.current / Thread.report_on_exception= — minimal
        // stubs sufficient for fixture and spec helpers.
        if class_rc.name() == "Thread" {
            match method_name {
                // `Thread.pass` hands the turn over, which is what lets a
                // thread waiting on another make its own progress.
                "pass" => {
                    self.pass_to_other_threads(position)?;
                    return Ok(Answered(Object::Nil));
                }
                // Hand control over, answering whether there was anything to
                // hand it to. A wait that nothing else can end stops here
                // rather than turning forever.
                "__hand_over__" => {
                    if !self.other_threads_are_waiting() {
                        return Ok(Answered(Object::Bool(false)));
                    }
                    self.wait_for_other_threads(position);
                    // A thread stopped or handed an exception while it waited
                    // takes it here, which is where the wait ends.
                    self.deliver_thread_interrupts(true, position)?;
                    return Ok(Answered(Object::Bool(true)));
                }
                // `Thread.kill` stops the thread it is handed, the way that
                // thread's own `kill` does.
                "kill" | "exit" => {
                    let target = match arguments.first() {
                        Some(named) => named.clone(),
                        None => self.running_thread(),
                    };
                    return self
                        .call_thread_method(&target, "kill", &[], position)
                        .map(nested_answer);
                }
                // The threads that have not finished, which is what
                // `Thread.list` reports.
                "list" => {
                    // Asking which threads there are gives each of them a
                    // turn, so a loop watching the list is what lets them run.
                    if !self.running_a_thread_body() && !self.stepping_threads {
                        self.step_pending_threads(position);
                    }
                    let mut living = vec![self.running_thread()];
                    let main = self.globals().get("__Thread_main").unwrap_or(Object::Nil);
                    if !matches!(main, Object::Nil)
                        && !living.iter().any(|held| same_object(held, &main))
                    {
                        living.push(main);
                    }
                    for thread in self.pending_threads.clone() {
                        let over = matches!(&thread, Object::Instance(held)
                            if held.borrow().get_var("__thread_value").is_some());
                        if !over && !living.iter().any(|held| same_object(held, &thread)) {
                            living.push(thread);
                        }
                    }
                    return Ok(Answered(Object::array(living)));
                }
                // `Thread.stop` puts the thread running now to sleep until
                // something wakes it.
                "stop" => {
                    if !self.running_a_thread_body() && self.living_threads().is_empty() {
                        return Err(crate::vm::errors::simple_exception(
                            "ThreadError",
                            "stopping only thread\n\tnote: use sleep to stop forever",
                            position,
                        ));
                    }
                    self.sleep_until_woken(true, position)?;
                    return Ok(Answered(Object::Nil));
                }
                // Thread.current returns the innermost Thread instance whose
                // block is being executed, or Nil at the top level. Used by
                // spec fixtures that thread-local-store via
                // `Thread.current[:k] = v` and read it back via the thread
                // instance after `.value`/`.join`. `Thread.main` is the same
                // shape but conceptually the program's root thread; we don't
                // distinguish, so it returns the same value.
                // Outside any thread block the running thread is the main
                // one, which is a Thread like any other.
                "current" | "main" => {
                    // `current` is the thread whose block is running, where
                    // `main` is always the one the program started on.
                    if method_name == "current"
                        && let Some(current) = self.thread_current_stack.last()
                    {
                        return Ok(Answered(current.clone()));
                    }
                    if let Some(main) = self.globals().get("__Thread_main") {
                        return Ok(Answered(main));
                    }
                    let instance = crate::object::Instance::new(Rc::clone(class_rc));
                    let main = Object::Instance(instance);
                    self.globals_mut().set("__Thread_main", main.clone());
                    return Ok(Answered(main));
                }
                // Whether a thread made from now on reports dying of an
                // exception, which is on until the program turns it off.
                "report_on_exception" => {
                    let held = self.globals().get(REPORT_ON_EXCEPTION_GLOBAL);
                    return Ok(Answered(Object::Bool(
                        held.is_none_or(|wanted| wanted.is_truthy()),
                    )));
                }
                "report_on_exception=" => {
                    let wanted = arguments.first().cloned().unwrap_or(Object::Nil);
                    self.globals_mut()
                        .set(REPORT_ON_EXCEPTION_GLOBAL, Object::Bool(wanted.is_truthy()));
                    return Ok(Answered(wanted));
                }
                "respond_to?" => {
                    if let Some(arg) = arguments.first() {
                        let n = match arg {
                            Object::String(s) => s.as_str().to_string(),
                            Object::Symbol(s) => s.as_str().to_string(),
                            _ => return Ok(Answered(Object::Bool(false))),
                        };
                        let known = matches!(
                            n.as_str(),
                            "report_on_exception=" | "pass" | "current" | "new"
                        );
                        return Ok(Answered(Object::Bool(known)));
                    }
                }
                _ => {}
            }
        }
        Ok(Unclaimed)
    }
}
