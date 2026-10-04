// The methods a Queue and a SizedQueue answer.

use super::*;

impl VirtualMachine {
    /// Instance-level Queue / SizedQueue methods. metorex runs blocks
    /// synchronously, so blocking-pop semantics aren't useful; `pop` on an
    /// empty queue returns nil rather than blocking. Enough for spec
    /// helpers that wire queues for inter-thread coordination patterns.
    pub(crate) fn call_queue_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        _position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let inst = match receiver {
            Object::Instance(i) => Rc::clone(i),
            _ => return Ok(None),
        };
        let items_obj = inst.borrow().get_var("__queue_items").cloned();
        let items_arr = match items_obj {
            Some(Object::Array(a)) => a,
            _ => return Ok(None),
        };
        match method_name {
            "push" | "<<" | "enq" => {
                // A closed queue takes nothing more, which is how a producer
                // learns the consumers have finished with it.
                if matches!(
                    inst.borrow().get_var("__queue_closed"),
                    Some(Object::Bool(true))
                ) {
                    return Err(crate::vm::errors::simple_exception(
                        "ClosedQueueError",
                        "queue closed",
                        _position,
                    ));
                }
                // A queue made with a limit takes nothing more while it is
                // full, so whoever is putting things on it waits for a reader.
                let limit = match inst.borrow().get_var("__queue_max") {
                    Some(Object::Int(most)) => Some(*most as usize),
                    _ => None,
                };
                let (positional, keywords) = queue_keyword_arguments(arguments);
                let asked_now = positional
                    .get(1)
                    .is_some_and(|given| !matches!(given, Object::Nil | Object::Bool(false)));
                let wait_for = match keywords.get("timeout") {
                    None | Some(Object::Nil) => None,
                    Some(_) if asked_now => {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "can't set a timeout if non_block is enabled",
                            _position,
                        ));
                    }
                    Some(Object::Int(seconds)) => Some(*seconds as f64),
                    Some(Object::Float(seconds)) => Some(*seconds),
                    Some(other) => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!(
                                "no implicit conversion of {} into Float",
                                unconvertible_wording(
                                    other,
                                    self.builtins().class_of(other).name()
                                )
                            ),
                            _position,
                        ));
                    }
                };
                if let Some(most) = limit {
                    if asked_now && items_arr.borrow().len() >= most {
                        return Err(crate::vm::errors::simple_exception(
                            "ThreadError",
                            "queue full",
                            _position,
                        ));
                    }
                    let started = std::time::Instant::now();
                    let waiting_limit = match wait_for {
                        Some(seconds) => std::time::Duration::from_secs_f64(seconds.max(0.0)),
                        None => std::time::Duration::from_secs(2),
                    };
                    let mut counted = false;
                    while items_arr.borrow().len() >= most && started.elapsed() < waiting_limit {
                        // Closing the queue while something waits to put on
                        // it ends the wait, which is how a producer learns it
                        // is done.
                        if matches!(
                            inst.borrow().get_var("__queue_closed"),
                            Some(Object::Bool(true))
                        ) {
                            if counted {
                                let waiting = (queue_waiting_count(&inst) - 1).max(0);
                                inst.borrow_mut()
                                    .set_var("__queue_waiting".to_string(), Object::Int(waiting));
                            }
                            return Err(crate::vm::errors::simple_exception(
                                "ClosedQueueError",
                                "queue closed",
                                _position,
                            ));
                        }
                        if !counted {
                            counted = true;
                            let waiting = queue_waiting_count(&inst) + 1;
                            inst.borrow_mut()
                                .set_var("__queue_waiting".to_string(), Object::Int(waiting));
                        }
                        self.wait_for_other_threads(_position);
                    }
                    if counted {
                        let waiting = (queue_waiting_count(&inst) - 1).max(0);
                        inst.borrow_mut()
                            .set_var("__queue_waiting".to_string(), Object::Int(waiting));
                    }
                    // A limit on how long to wait that passes with the queue
                    // still full answers nothing rather than putting anyway.
                    if wait_for.is_some() && items_arr.borrow().len() >= most {
                        return Ok(Some(Object::Nil));
                    }
                }
                if let Some(item) = positional.first() {
                    items_arr.borrow_mut().push(item.clone());
                }
                self.scheduler_unblock(receiver, _position)?;
                Ok(Some(receiver.clone()))
            }
            "pop" | "deq" | "shift" => {
                let (positional, keywords) = queue_keyword_arguments(arguments);
                let asked_now = positional
                    .first()
                    .is_some_and(|given| !matches!(given, Object::Nil | Object::Bool(false)));
                let limit = match keywords.get("timeout") {
                    None | Some(Object::Nil) => None,
                    Some(_) if asked_now => {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "can't set a timeout if non_block is enabled",
                            _position,
                        ));
                    }
                    Some(Object::Int(seconds)) => Some(*seconds as f64),
                    Some(Object::Float(seconds)) => Some(*seconds),
                    Some(other) => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!(
                                "no implicit conversion of {} into Float",
                                unconvertible_wording(
                                    other,
                                    self.builtins().class_of(other).name()
                                )
                            ),
                            _position,
                        ));
                    }
                };
                // Asking for what is there right now rather than waiting is
                // an error when the queue is empty, however it was closed.
                if asked_now && items_arr.borrow().is_empty() {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "queue empty",
                        _position,
                    ));
                }
                // A fiber that is not blocking waits through its scheduler,
                // which holds it until something is put on the queue or the
                // queue is closed.
                if items_arr.borrow().is_empty()
                    && let Some(scheduler) = self.current_scheduler()
                {
                    let timeout = limit.map_or(Object::Nil, Object::Float);
                    while items_arr.borrow().is_empty()
                        && !matches!(
                            inst.borrow().get_var("__queue_closed"),
                            Some(Object::Bool(true))
                        )
                    {
                        self.scheduler_block(
                            scheduler.clone(),
                            receiver,
                            timeout.clone(),
                            _position,
                        )?;
                        if limit.is_some() {
                            break;
                        }
                    }
                    let taken = if items_arr.borrow().is_empty() {
                        Object::Nil
                    } else {
                        items_arr.borrow_mut().remove(0)
                    };
                    return Ok(Some(taken));
                }
                // Taking from an empty queue waits for something to be put
                // there, so every other waiting thread gets a turn until one
                // of them puts something or none of them can run at all.
                let started = std::time::Instant::now();
                let deadline = started
                    + match limit {
                        Some(seconds) => std::time::Duration::from_secs_f64(seconds.max(0.0)),
                        None => std::time::Duration::from_secs(2),
                    };
                let mut counted = false;
                while items_arr.borrow().is_empty()
                    && !self.pending_threads.is_empty()
                    && std::time::Instant::now() < deadline
                {
                    // Whoever is waiting here is one of the number the queue
                    // reports, for as long as the wait lasts.
                    if !counted {
                        counted = true;
                        let waiting = queue_waiting_count(&inst) + 1;
                        inst.borrow_mut()
                            .set_var("__queue_waiting".to_string(), Object::Int(waiting));
                    }
                    let held = self.pending_threads.len();
                    self.wait_for_other_threads(_position);
                    self.raise_if_thread_killed(_position)?;
                    // Nothing moved and nothing ran for long enough that
                    // nothing ever will. The grace matters because whoever is
                    // waiting on this thread may be the one about to put
                    // something on the queue.
                    if self.pending_threads.len() == held
                        && items_arr.borrow().is_empty()
                        && self.stepping_threads
                        && started.elapsed() >= QUEUE_DEADLOCK_GRACE
                    {
                        break;
                    }
                }
                if counted {
                    let waiting = (queue_waiting_count(&inst) - 1).max(0);
                    inst.borrow_mut()
                        .set_var("__queue_waiting".to_string(), Object::Int(waiting));
                }
                let val = if items_arr.borrow().is_empty() {
                    Object::Nil
                } else {
                    items_arr.borrow_mut().remove(0)
                };
                Ok(Some(val))
            }
            "size" | "length" | "count" => Ok(Some(Object::Int(items_arr.borrow().len() as i64))),
            // How many are waiting for something to be put on the queue.
            "num_waiting" => Ok(Some(Object::Int(queue_waiting_count(&inst)))),
            // How many a SizedQueue holds. Nothing ever waits on one here,
            // but the count it was made with is still what it reports.
            "max" => Ok(inst.borrow().get_var("__queue_max").cloned()),
            "max=" => {
                let Some(held) = arguments.first() else {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        1,
                        0,
                        _position,
                    ));
                };
                let counted = self.queue_capacity_argument(held, _position)?;
                inst.borrow_mut()
                    .set_var("__queue_max".to_string(), Object::Int(counted));
                Ok(Some(Object::Int(counted)))
            }
            "empty?" => Ok(Some(Object::Bool(items_arr.borrow().is_empty()))),
            "clear" => {
                items_arr.borrow_mut().clear();
                Ok(Some(receiver.clone()))
            }
            "close" => {
                inst.borrow_mut()
                    .set_var("__queue_closed".to_string(), Object::Bool(true));
                self.scheduler_unblock_all(receiver, _position)?;
                Ok(Some(receiver.clone()))
            }
            "closed?" => Ok(Some(Object::Bool(matches!(
                inst.borrow().get_var("__queue_closed"),
                Some(Object::Bool(true))
            )))),
            // A queue carries state its own methods change, so Ruby refuses to
            // freeze one at all rather than leaving a half-usable object.
            "freeze" => Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("cannot freeze {}", receiver),
                _position,
            )),
            _ => Ok(None),
        }
    }
}

/// How long a wait on an empty queue keeps going after nothing has moved,
/// before it is taken as a wait nothing will ever satisfy.
const QUEUE_DEADLOCK_GRACE: std::time::Duration = std::time::Duration::from_millis(50);

/// A queue method's positional arguments paired with the keywords it was
/// given, which is where `timeout:` arrives.
fn queue_keyword_arguments(
    arguments: &[Object],
) -> (Vec<Object>, std::collections::HashMap<String, Object>) {
    let mut keywords = std::collections::HashMap::new();
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            for (key, value) in dict.iter() {
                if key.as_str() == "__MX_KWARGS__" {
                    continue;
                }
                keywords.insert(
                    key.strip_prefix(':').unwrap_or(key).to_string(),
                    value.clone(),
                );
            }
            return (arguments[..arguments.len() - 1].to_vec(), keywords);
        }
    }
    (arguments.to_vec(), keywords)
}

/// How many are waiting on a queue for something to be put there.
fn queue_waiting_count(inst: &Rc<std::cell::RefCell<crate::object::Instance>>) -> i64 {
    match inst.borrow().get_var("__queue_waiting") {
        Some(Object::Int(held)) => *held,
        _ => 0,
    }
}
