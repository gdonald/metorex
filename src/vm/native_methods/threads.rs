// The methods a Thread answers, and the block it runs.

use super::*;

use crate::vm::ractors::RACTOR_VAR;

impl VirtualMachine {
    /// Instance-level Thread methods. The "thread" runs synchronously when
    /// `value`/`join` is called for the first time.
    /// The name a thread-local is kept under. Ruby takes a String or a
    /// Symbol, asks anything else for `to_str`, and refuses what answers
    /// none.
    fn thread_local_name(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match given {
            Object::Symbol(name) | Object::String(name) => Ok(name.as_str().to_string()),
            other if self.responds_to(other, "to_str") => {
                match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                    Object::String(name) => Ok(name.as_str().to_string()),
                    answered => Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!("{} is not a symbol nor a string", answered),
                        position,
                    )),
                }
            }
            other => {
                let rendered = match self.send_to_object(other.clone(), "inspect", vec![], position)
                {
                    Ok(Object::String(text)) => text.as_str().to_string(),
                    _ => crate::vm::native_methods::array_methods::inspect_element(other),
                };
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("{rendered} is not a symbol nor a string"),
                    position,
                ))
            }
        }
    }

    pub(crate) fn call_thread_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let inst = match receiver {
            Object::Instance(i) => Rc::clone(i),
            _ => return Ok(None),
        };
        match method_name {
            "__ractor__" => Ok(Some(
                inst.borrow()
                    .get_var(RACTOR_VAR)
                    .cloned()
                    .unwrap_or(Object::Nil),
            )),
            "__ractor__=" => {
                let ractor = arguments.first().cloned().unwrap_or(Object::Nil);
                inst.borrow_mut()
                    .set_var(RACTOR_VAR.to_string(), ractor.clone());
                Ok(Some(ractor))
            }
            "value" | "join" => {
                if !arguments.is_empty() && method_name == "value" {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // `join` takes a limit on how long to wait, and answers nil
                // when the thread is still going once it has passed.
                let limit = match arguments.first() {
                    None | Some(Object::Nil) => None,
                    Some(given) => Some(std::time::Duration::from_secs_f64(
                        self.float_value_of(given, position)?.max(0.0),
                    )),
                };
                let cached = inst.borrow().get_var("__thread_value").cloned();
                if let Some(val) = cached {
                    inst.borrow_mut()
                        .set_var("__thread_reaped".to_string(), Object::Bool(true));
                    // A thread that died of an exception hands it to whoever
                    // waits on it, however long after the fact.
                    let died_of = inst.borrow().get_var("__thread_error").cloned();
                    if let Some(held @ Object::Exception(_)) = died_of {
                        self.call_native_function("raise", vec![held], position)?;
                    }
                    return Ok(Some(if method_name == "join" {
                        receiver.clone()
                    } else {
                        val
                    }));
                }
                // A fiber that is not blocking waits through its scheduler,
                // which holds it until the thread ends or the limit passes.
                if let Some(scheduler) = self.current_scheduler() {
                    let timeout = arguments.first().cloned().unwrap_or(Object::Nil);
                    while inst.borrow().get_var("__thread_value").is_none() {
                        self.scheduler_block(
                            scheduler.clone(),
                            receiver,
                            timeout.clone(),
                            position,
                        )?;
                        if limit.is_some() {
                            break;
                        }
                    }
                    if inst.borrow().get_var("__thread_value").is_none() {
                        return Ok(Some(Object::Nil));
                    }
                    return self.call_thread_method(receiver, method_name, &[], position);
                }
                // Waiting on a thread runs it, a step at a time, so a
                // thread of its own waiting on something else still gets a
                // turn. A thread starts with no child of its own behind it,
                // which is what `Process.last_status` reports there.
                let held_status = self.take_last_status();
                let waited = self.run_thread_within(receiver, limit, position);
                self.restore_last_status(held_status);
                let Some(value) = waited? else {
                    return Ok(Some(Object::Nil));
                };
                inst.borrow_mut()
                    .set_var("__thread_value".to_string(), value.clone());
                inst.borrow_mut()
                    .set_var("__thread_reaped".to_string(), Object::Bool(true));
                Ok(Some(if method_name == "join" {
                    receiver.clone()
                } else {
                    value
                }))
            }
            // Thread-local storage: `t[:k]` and `t[:k] = v`. Backed by an
            // ivar Hash on the Thread instance.
            "[]" | "thread_variable_get" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key_str = self.thread_local_name(&arguments[0], position)?;
                let store = self.thread_local_store(method_name, receiver);
                let locals = inst.borrow().get_var(&store).cloned();
                if let Some(Object::Dict(held)) = locals {
                    return Ok(Some(
                        held.borrow().get(&key_str).cloned().unwrap_or(Object::Nil),
                    ));
                }
                Ok(Some(Object::Nil))
            }
            "key?" | "thread_variable?" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key_str = self.thread_local_name(&arguments[0], position)?;
                let store = self.thread_local_store(method_name, receiver);
                let locals = inst.borrow().get_var(&store).cloned();
                let held = matches!(locals, Some(Object::Dict(ref names)) if names.borrow().contains_key(&key_str));
                Ok(Some(Object::Bool(held)))
            }
            "keys" | "thread_variables" => {
                if !arguments.is_empty() {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let store = self.thread_local_store(method_name, receiver);
                let locals = inst.borrow().get_var(&store).cloned();
                let Some(Object::Dict(held)) = locals else {
                    return Ok(Some(Object::array(Vec::new())));
                };
                let named: Vec<Object> = held
                    .borrow()
                    .keys()
                    .map(|name| Object::symbol(name.clone()))
                    .collect();
                Ok(Some(Object::array(named)))
            }
            "[]=" | "thread_variable_set" => {
                if self.object_is_frozen(receiver) {
                    return Err(crate::vm::errors::simple_exception(
                        "FrozenError",
                        "can't modify frozen thread locals",
                        position,
                    ));
                }
                if arguments.len() != 2 {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let key_str = self.thread_local_name(&arguments[0], position)?;
                let store = self.thread_local_store(method_name, receiver);
                let existing = inst.borrow().get_var(&store).cloned();
                let dict = match existing {
                    Some(Object::Dict(held)) => held,
                    _ => {
                        let held = Rc::new(std::cell::RefCell::new(indexmap::IndexMap::new()));
                        inst.borrow_mut()
                            .set_var(store.clone(), Object::Dict(Rc::clone(&held)));
                        held
                    }
                };
                // A thread variable set to nil is gone rather than held as
                // nil, so the thread stops naming it among its variables.
                if method_name == "thread_variable_set" && matches!(arguments[1], Object::Nil) {
                    dict.borrow_mut().shift_remove(&key_str);
                } else {
                    dict.borrow_mut().insert(key_str, arguments[1].clone());
                }
                Ok(Some(arguments[1].clone()))
            }
            // A thread runs when it is joined, so one that has not been is
            // still alive. `kill` marks it finished without running it.
            "alive?" => Ok(Some(Object::Bool(
                inst.borrow().get_var("__thread_value").is_none(),
            ))),
            "kill" | "exit" | "terminate" => {
                // A thread stopped part-way answers nil for its status and
                // for its value.
                inst.borrow_mut()
                    .set_var("__thread_killed".to_string(), Object::Bool(true));
                // A thread waiting on something is woken so it unwinds where
                // it waits, running the `ensure` blocks it is inside.
                inst.borrow_mut()
                    .set_var("__thread_waiting".to_string(), Object::Bool(false));
                // A thread stopping itself unwinds from here rather than
                // carrying on, so the rest of its block does not run.
                self.raise_if_thread_killed(position)?;
                // A thread part-way through still has `ensure` blocks to run,
                // so it keeps its turn and is left with no value until it
                // unwinds. One that never started has nothing to unwind.
                let started = matches!(
                    inst.borrow().get_var("__thread_fiber"),
                    Some(Object::Int(_))
                );
                if !started {
                    self.pending_threads.retain(|thread| {
                        if let (Object::Instance(pending), Object::Instance(target)) =
                            (thread, receiver)
                        {
                            !Rc::ptr_eq(pending, target)
                        } else {
                            true
                        }
                    });
                    inst.borrow_mut()
                        .set_var("__thread_value".to_string(), Object::Nil);
                    inst.borrow_mut()
                        .set_var("__thread_reaped".to_string(), Object::Bool(true));
                }
                Ok(Some(receiver.clone()))
            }
            // Ruby names a thread by its address, where its body was
            // written, and what it is doing, in bytes rather than characters.
            "inspect" | "to_s" => {
                let address = Rc::as_ptr(&inst) as usize;
                let doing = match self.call_thread_method(receiver, "status", &[], position)? {
                    Some(Object::String(word)) => word.as_str().to_string(),
                    _ => "dead".to_string(),
                };
                let written_at = match (
                    inst.borrow().get_var("__thread_source"),
                    inst.borrow().get_var("__thread_line"),
                ) {
                    (Some(Object::String(file)), Some(Object::Int(line))) => {
                        Some(format!("{}:{}", file.as_str(), line))
                    }
                    _ => None,
                };
                Ok(Some(Object::binary_string(match written_at {
                    Some(place) => format!("#<Thread:0x{address:016x} {place} {doing}>"),
                    None => format!("#<Thread:0x{address:016x} {doing}>"),
                })))
            }
            // `Thread#initialize` is what a subclass reaches through
            // `super`, and the block it is given is what the thread runs.
            "initialize" => {
                let Some(block) = self.pending_block.take() else {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "must be called with a block",
                        position,
                    ));
                };
                if inst.borrow().get_var("__thread_block").is_some() {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "already initialized thread",
                        position,
                    ));
                }
                self.give_thread_a_body(&inst, block, arguments);
                Ok(Some(Object::Nil))
            }
            // A thread's name, which is nothing until one is given.
            "name" => Ok(Some(
                inst.borrow()
                    .get_var("__thread_name")
                    .cloned()
                    .unwrap_or(Object::Nil),
            )),
            "name=" => {
                let given = arguments.first().cloned().unwrap_or(Object::Nil);
                let named = match given {
                    Object::Nil => Object::Nil,
                    Object::String(_) => given,
                    other if self.responds_to(&other, "to_str") => {
                        self.send_to_object(other, "to_str", vec![], position)?
                    }
                    other => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!(
                                "no implicit conversion of {} into String",
                                self.builtins().class_of(&other).name()
                            ),
                            position,
                        ));
                    }
                };
                if let Object::String(text) = &named
                    && text.as_str().contains('\0')
                {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "string contains null byte",
                        position,
                    ));
                }
                inst.borrow_mut()
                    .set_var("__thread_name".to_string(), named.clone());
                Ok(Some(named))
            }
            // How eagerly a thread is given turns. Metorex gives every thread
            // the same turn, so the number is remembered and nothing more.
            "priority" => Ok(Some(
                inst.borrow()
                    .get_var("__thread_priority")
                    .cloned()
                    .unwrap_or(Object::Int(0)),
            )),
            "priority=" => {
                let given = arguments.first().cloned().unwrap_or(Object::Int(0));
                let Object::Int(wanted) = given else {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "priority must be an Integer",
                        position,
                    ));
                };
                inst.borrow_mut().set_var(
                    "__thread_priority".to_string(),
                    Object::Int(wanted.clamp(PRIORITY_FLOOR, PRIORITY_CEILING)),
                );
                Ok(Some(given))
            }
            // Where the thread stands, as a backtrace reads. A thread that
            // has finished has none, which Ruby reports as nil.
            // The same places `backtrace` names, as Location objects. Only
            // the thread running now can be asked, since the places another
            // thread stands are not objects until it is running.
            "backtrace_locations" => {
                if inst.borrow().get_var("__thread_value").is_some() {
                    return Ok(Some(Object::Nil));
                }
                let mut places = self.caller_location_objects(position);
                // The innermost place is the call to `backtrace_locations`
                // itself, which sits where the caller's own innermost place
                // does and is named for the method rather than the caller.
                if let Some(Object::Instance(innermost)) = places.first() {
                    let here = crate::object::Instance::new(self.backtrace_location_class());
                    for named in ["lineno", "path", "absolute_path"] {
                        let held = innermost.borrow().get_var(named).cloned();
                        if let Some(value) = held {
                            here.borrow_mut().set_var(named.to_string(), value);
                        }
                    }
                    here.borrow_mut().set_var(
                        "label".to_string(),
                        Object::string("Thread#backtrace_locations"),
                    );
                    places.insert(0, Object::Instance(here));
                }
                let Some((skip, length)) = self.caller_slice_bounds(arguments, places.len(), 0)
                else {
                    return Ok(Some(Object::Nil));
                };
                let mut kept: Vec<Object> = places.into_iter().skip(skip).collect();
                if let Some(length) = length {
                    kept.truncate(length);
                }
                Ok(Some(Object::array(kept)))
            }
            "backtrace" => {
                if inst.borrow().get_var("__thread_value").is_some() {
                    return Ok(Some(Object::Nil));
                }
                let current = self.running_thread();
                let running = matches!(
                    (&current, receiver),
                    (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b)
                );
                let lines = if running {
                    self.own_backtrace_lines(position)
                } else {
                    self.thread_backtrace_lines(receiver).unwrap_or_default()
                };
                let Some((skip, length)) = self.caller_slice_bounds(arguments, lines.len(), 0)
                else {
                    return Ok(Some(Object::Nil));
                };
                let mut kept: Vec<String> = lines.into_iter().skip(skip).collect();
                if let Some(length) = length {
                    kept.truncate(length);
                }
                Ok(Some(Object::array(
                    kept.into_iter().map(Object::string).collect(),
                )))
            }
            // A thread is handed an exception to raise where it left off, so
            // one waiting inside `sleep` wakes and raises it there.
            // The exception a `raise` with these arguments would raise,
            // built without raising it, so a thread can be handed one.
            // The cause is settled here, from what the caller is handling,
            // so the thread that raises it does not take its own.
            "__build_raised__" => {
                // A backtrace named in the call, or one the exception already
                // carries, stays. Any other is recorded where the thread
                // raises it, not here.
                let keeps_backtrace = matches!(arguments.get(2), Some(Object::Array(_)))
                    || matches!(arguments.first(), Some(Object::Exception(given)) if given.borrow().backtrace.is_some());
                let built = self.build_raise_exception(arguments, position)?;
                if !keeps_backtrace && let Object::Exception(details) = &built {
                    let mut details = details.borrow_mut();
                    details.backtrace = None;
                    details.backtrace_sites = None;
                    details.backtrace_array = None;
                    details.backtrace_locations_array = None;
                }
                if let Object::Exception(details) = &built
                    && !details.borrow().cause_settled
                {
                    if details.borrow().cause.is_none()
                        && let Some(active @ Object::Exception(_)) = self.globals().get("!")
                    {
                        crate::vm::VirtualMachine::record_cause(&built, &active);
                    }
                    details.borrow_mut().cause_settled = true;
                }
                Ok(Some(built))
            }
            "__raise_later__" => {
                let handed = arguments.to_vec();
                inst.borrow_mut()
                    .set_var("__thread_raise".to_string(), Object::array(handed));
                inst.borrow_mut()
                    .set_var("__thread_waiting".to_string(), Object::Bool(false));
                Ok(Some(Object::Nil))
            }
            // Waking a thread that has already finished is an error; one
            // that has not run yet wakes to no effect, since it runs when it
            // is joined.
            "wakeup" | "run" => {
                if matches!(
                    inst.borrow().get_var("__thread_reaped"),
                    Some(Object::Bool(true))
                ) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "killed thread",
                        position,
                    ));
                }
                // Waking a thread takes it out of the wait it was in, so it
                // is ready to run rather than asleep.
                inst.borrow_mut()
                    .set_var("__thread_waiting".to_string(), Object::Bool(false));
                Ok(Some(receiver.clone()))
            }
            // The number the operating system knows the thread by. Only the
            // thread running now has one; the rest have not been handed to
            // the system at all.
            "native_thread_id" => {
                let running = match self.thread_current_stack.last() {
                    Some(current) => current.clone(),
                    None => self.globals().get("__Thread_main").unwrap_or(Object::Nil),
                };
                let is_running = matches!(
                    (&running, receiver),
                    (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b)
                );
                if !is_running {
                    return Ok(Some(Object::Nil));
                }
                // SAFETY: `getpid` reads a number and touches nothing else.
                Ok(Some(Object::Int(unsafe { libc::getpid() } as i64)))
            }
            // A thread that has not had a turn, or that is waiting for
            // something, is stopped; one running now is not.
            // A thread is stopped when it is asleep or when it is over.
            "stop?" => {
                let doing = self.call_thread_method(receiver, "status", &[], position)?;
                Ok(Some(Object::Bool(match doing {
                    Some(Object::String(word)) => &*word.as_str() == "sleep",
                    _ => true,
                })))
            }
            // Ruby reports "run" for the thread running now, "sleep" for one
            // waiting on something, false for one whose block ran out, and
            // nil for one stopped or killed part-way.
            // Ruby reports a thread that ended by itself or was killed as
            // false, one that ended on an exception as nil, and a live one by
            // what it is doing. A thread part-way through being stopped is
            // aborting unless it is waiting, which reads as asleep.
            "status" => {
                if inst.borrow().get_var("__thread_error").is_some() {
                    return Ok(Some(Object::Nil));
                }
                if inst.borrow().get_var("__thread_value").is_some() {
                    return Ok(Some(Object::Bool(false)));
                }
                let waiting = matches!(
                    inst.borrow().get_var("__thread_waiting"),
                    Some(Object::Bool(true))
                );
                if waiting {
                    return Ok(Some(Object::string("sleep")));
                }
                if matches!(
                    inst.borrow().get_var("__thread_killed"),
                    Some(Object::Bool(true))
                ) || matches!(
                    inst.borrow().get_var("__thread_dying"),
                    Some(Object::Bool(true))
                ) {
                    return Ok(Some(Object::string("aborting")));
                }
                Ok(Some(Object::string("run")))
            }
            _ => Ok(None),
        }
    }

    /// The fiber a lock is held under. A thread's own body is its root fiber
    /// rather than one the program made, so a lock taken there is the
    /// thread's rather than a fiber's.
    pub(crate) fn lock_holding_fiber(&self) -> i64 {
        let held = self.fiber_current_handle();
        if self.thread_body_fibers.contains(&held) {
            return -1;
        }
        held as i64
    }

    /// Record the thread and fiber running now as the ones holding the lock.
    pub(crate) fn mark_mutex_held(
        &mut self,
        inst: &Rc<std::cell::RefCell<crate::object::Instance>>,
    ) {
        let holder = self.running_thread();
        let fiber = self.lock_holding_fiber();
        {
            let mut held = inst.borrow_mut();
            held.set_var("__mutex_locked".to_string(), Object::Bool(true));
            held.set_var("__mutex_thread".to_string(), holder);
            held.set_var("__mutex_fiber".to_string(), Object::Int(fiber));
        }
        if !self.taken_mutexes.iter().any(|seen| Rc::ptr_eq(seen, inst)) {
            self.taken_mutexes.push(Rc::clone(inst));
        }
    }

    /// Whether the thread and fiber running now are the ones holding the
    /// lock. Ruby holds a lock per fiber, so a fiber the holder started does
    /// not own it.
    pub(crate) fn mutex_is_owned(
        &mut self,
        inst: &Rc<std::cell::RefCell<crate::object::Instance>>,
    ) -> bool {
        let fiber = self.lock_holding_fiber();
        if !matches!(inst.borrow().get_var("__mutex_fiber"), Some(Object::Int(held)) if *held == fiber)
        {
            return false;
        }
        self.mutex_held_by_this_thread(inst)
    }

    /// Whether the thread running now holds the lock, under any of its fibers.
    pub(crate) fn mutex_held_by_this_thread(
        &mut self,
        inst: &Rc<std::cell::RefCell<crate::object::Instance>>,
    ) -> bool {
        let holder = inst.borrow().get_var("__mutex_thread").cloned();
        let running = self.running_thread();
        matches!(
            (holder, running),
            (Some(Object::Instance(a)), Object::Instance(b)) if Rc::ptr_eq(&a, &b)
        )
    }

    /// The Thread whose block is running, which is the main one outside any.
    pub(crate) fn running_thread(&mut self) -> Object {
        if let Some(current) = self.thread_current_stack.last() {
            return current.clone();
        }
        if let Some(main) = self.globals().get("__Thread_main") {
            return main;
        }
        // Outside every thread block the thread running is the main one,
        // which is made the first time anything asks after it.
        let Some(Object::Class(thread_class)) = self.globals().get("Thread") else {
            return Object::Nil;
        };
        let made = crate::object::Instance::new(Rc::clone(&thread_class));
        let main = Object::Instance(made);
        self.globals_mut().set("__Thread_main", main.clone());
        main
    }

    /// What `Thread#raise` handed the thread running now, taken so it is
    /// raised once and no more.
    pub(crate) fn exception_handed_to_thread(&mut self) -> Option<Vec<Object>> {
        let Some(Object::Instance(running)) = self.thread_current_stack.last().cloned() else {
            return None;
        };
        let handed = running.borrow().get_var("__thread_raise").cloned();
        let Some(Object::Array(values)) = handed else {
            return None;
        };
        running
            .borrow_mut()
            .set_var("__thread_raise".to_string(), Object::Nil);
        let taken = values.borrow().clone();
        Some(taken)
    }

    /// Give a thread the block it runs and the values that block is handed,
    /// and note where the block was written so `inspect` can name it.
    pub(crate) fn give_thread_a_body(
        &mut self,
        thread: &Rc<std::cell::RefCell<crate::object::Instance>>,
        block: Object,
        arguments: &[Object],
    ) {
        let written_at = match &block {
            Object::Block(body) => match (&body.source_file, body.opened_at) {
                (Some(file), Some(line)) => Some((file.clone(), line)),
                _ => None,
            },
            _ => None,
        };
        let mut held = thread.borrow_mut();
        held.set_var("__thread_block".to_string(), block);
        held.set_var(
            "__thread_args".to_string(),
            Object::array(arguments.to_vec()),
        );
        if let Some((file, line)) = written_at {
            held.set_var("__thread_source".to_string(), Object::string(file));
            held.set_var("__thread_line".to_string(), Object::Int(line as i64));
        }
    }
}
