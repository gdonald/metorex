// The methods a ConditionVariable answers.

use super::*;

impl VirtualMachine {
    /// The threads lined up on a condition variable, made the first time one
    /// waits.
    fn condition_variable_line(
        &mut self,
        inst: &Rc<std::cell::RefCell<crate::object::Instance>>,
    ) -> Rc<std::cell::RefCell<Vec<Object>>> {
        if let Some(Object::Array(line)) = inst.borrow().get_var("__cv_waiters") {
            return Rc::clone(line);
        }
        let line = Rc::new(std::cell::RefCell::new(Vec::new()));
        inst.borrow_mut()
            .set_var("__cv_waiters".to_string(), Object::Array(Rc::clone(&line)));
        line
    }

    /// ConditionVariable instance methods. A waiter joins the line and then
    /// sleeps on the object it was handed, the way Ruby leaves the sleeping
    /// to `Mutex#sleep`. `signal` wakes the thread at the head of the line
    /// and `broadcast` wakes every one of them.
    pub(crate) fn call_condition_variable_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(inst) = receiver else {
            return Ok(None);
        };
        let inst = Rc::clone(inst);
        match method_name {
            "wait" => {
                let sleeper = arguments.first().cloned().unwrap_or(Object::Nil);
                let limit = arguments.get(1).cloned().unwrap_or(Object::Nil);
                let waiter = self.running_thread();
                self.condition_variable_line(&inst)
                    .borrow_mut()
                    .push(waiter);
                let slept = match self.lookup_method(&sleeper, "sleep") {
                    Some((class, method)) => {
                        self.invoke_method(class, method, sleeper.clone(), vec![limit], position)
                    }
                    None => self
                        .call_mutex_method(&sleeper, "sleep", &[limit], position)
                        .map(|answer| answer.unwrap_or(Object::Nil)),
                };
                let line = self.condition_variable_line(&inst);
                let running = self.running_thread();
                let mut waiting = line.borrow_mut();
                if let Some(place) = waiting.iter().position(|held| same_thread(held, &running)) {
                    waiting.remove(place);
                }
                drop(waiting);
                slept?;
                Ok(Some(receiver.clone()))
            }
            "signal" | "broadcast" => {
                let line = self.condition_variable_line(&inst);
                let mut waiting = line.borrow_mut();
                let serving = if method_name == "signal" {
                    1.min(waiting.len())
                } else {
                    waiting.len()
                };
                let woken: Vec<Object> = waiting.drain(..serving).collect();
                drop(waiting);
                for thread in woken {
                    if let Object::Instance(held) = thread {
                        held.borrow_mut()
                            .set_var("__thread_waiting".to_string(), Object::Bool(false));
                    }
                }
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }
}
