// Holding a value between two bounds, and the singleton methods an
// object carries.

use super::*;

impl VirtualMachine {
    /// Holding a value between two bounds, and the singleton methods an
    /// object carries.
    pub(crate) fn call_object_clamping_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "clamp" => {
                // Comparable#clamp — 1 or 2 args, Range, or beginless/endless ranges
                let (min, max) = if arguments.len() == 1 {
                    // Range argument
                    match &arguments[0] {
                        Object::Range {
                            start,
                            end,
                            exclusive,
                            ..
                        } => {
                            // Exclusive range is an error — except when end is nil
                            // (endless exclusive range like `x...` is allowed).
                            if *exclusive && !matches!(**end, Object::Nil) {
                                let exc = Object::exception(
                                    "ArgumentError",
                                    "cannot clamp with an exclusive range".to_string(),
                                );
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: "cannot clamp with an exclusive range".to_string(),
                                });
                            }
                            ((**start).clone(), (**end).clone())
                        }
                        _ => {
                            return Err(method_argument_error(
                                method_name,
                                1,
                                arguments.len(),
                                position,
                            ));
                        }
                    }
                } else if arguments.len() == 2 {
                    (arguments[0].clone(), arguments[1].clone())
                } else {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                };

                // Verify min <= max when both are non-nil (also raise if incomparable)
                if !matches!(min, Object::Nil) && !matches!(max, Object::Nil) {
                    let cmp = self.dispatch_spaceship(&min, &max, position)?;
                    match cmp {
                        Some(n) if n > 0 => {
                            let exc = Object::exception(
                                "ArgumentError",
                                "min argument must be smaller than max argument".to_string(),
                            );
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: "min argument must be smaller than max argument"
                                    .to_string(),
                            });
                        }
                        None => {
                            let exc =
                                Object::exception("ArgumentError", "comparison failed".to_string());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: "comparison failed".to_string(),
                            });
                        }
                        _ => {}
                    }
                }

                // Clamp: return min if self < min, max if self > max, else self
                if !matches!(min, Object::Nil) {
                    let cmp = self.dispatch_spaceship(receiver, &min, position)?;
                    if matches!(cmp, Some(n) if n < 0) {
                        return Ok(Some(min));
                    }
                }
                if !matches!(max, Object::Nil) {
                    let cmp = self.dispatch_spaceship(receiver, &max, position)?;
                    if matches!(cmp, Some(n) if n > 0) {
                        return Ok(Some(max));
                    }
                }
                Ok(Some(receiver.clone()))
            }
            "between?" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // between?(min, max) returns true if min <= self <= max
                // Try <=> dispatch
                let cmp_min = self.dispatch_spaceship(receiver, &arguments[0], position)?;
                let cmp_max = self.dispatch_spaceship(receiver, &arguments[1], position)?;
                let result = match (cmp_min, cmp_max) {
                    (Some(a), Some(b)) => a >= 0 && b <= 0,
                    _ => false,
                };
                Ok(Some(Object::Bool(result)))
            }
            // Kernel#singleton_class. nil, true, and false each have exactly
            // one instance, so their singleton class is the class itself.
            // Immediates that are not objects at all cannot have one.
            // Kernel#singleton_methods(all = true) — the names in the
            // object's singleton layer. `all` adds what its ancestors'
            // singleton classes supply.
            "singleton_methods" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let include_ancestors = !matches!(
                    arguments.first(),
                    Some(Object::Bool(false)) | Some(Object::Nil)
                );
                Ok(Some(self.singleton_method_names_with_ancestors(
                    receiver,
                    include_ancestors,
                )))
            }
            // `IO#reopen` points a stream at another place, and Ruby gives
            // the object a singleton class of its own again when it does. It
            // also takes the class of the stream it was pointed at.
            "__fresh_singleton_class__" => {
                if let Object::Instance(inst_rc) = receiver {
                    if let Some(Object::Instance(other_rc)) = arguments.first() {
                        let adopted = other_rc.borrow().class.clone();
                        inst_rc.borrow_mut().class = adopted;
                    }
                    let held = inst_rc.borrow();
                    *held.singleton_class.borrow_mut() = None;
                    held.singleton_methods.borrow_mut().clear();
                }
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }
}
