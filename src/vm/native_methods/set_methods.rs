//! Native method implementations for the Set class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Object, ObjectHash};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the Set class.
    pub(crate) fn call_set_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Set(set_rc) = receiver else {
            return Ok(None);
        };
        match method_name {
            "add" | "insert" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let hash = ObjectHash::from_object(&arguments[0]).ok_or_else(|| {
                    MetorexError::runtime_error(
                        format!(
                            "Cannot add {} to set (not hashable)",
                            arguments[0].type_name()
                        ),
                        position_to_location(position),
                    )
                })?;
                set_rc.borrow_mut().insert(hash);
                Ok(Some(receiver.clone()))
            }
            "remove" | "delete" | "delete?" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let hash = ObjectHash::from_object(&arguments[0]).ok_or_else(|| {
                    MetorexError::runtime_error(
                        format!(
                            "Cannot remove {} from set (not hashable)",
                            arguments[0].type_name()
                        ),
                        position_to_location(position),
                    )
                })?;
                let removed = set_rc.borrow_mut().shift_remove(&hash);
                // `delete` answers the set either way, while `delete?` says
                // nil when there was nothing to remove. `remove` reports
                // whether it removed anything.
                Ok(Some(match method_name {
                    "remove" => Object::Bool(removed),
                    "delete?" if !removed => Object::Nil,
                    _ => receiver.clone(),
                }))
            }
            "contains?" | "include?" | "has?" | "===" | "member?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(self.set_holds(
                    set_rc,
                    &arguments[0],
                    position,
                )?)))
            }
            "size" | "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(set_rc.borrow().len() as i64)))
            }
            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(set_rc.borrow().is_empty())))
            }
            "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements: Vec<Object> = set_rc
                    .borrow()
                    .iter()
                    .map(|held| held.value.clone())
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(elements)))))
            }
            "union" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.set_operand(&arguments[0], method_name, position)?;
                let other = &other;
                let result: indexmap::IndexSet<ObjectHash> =
                    set_rc.borrow().union(&other.borrow()).cloned().collect();
                Ok(Some(Object::Set(Rc::new(RefCell::new(result)))))
            }
            "intersection" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.set_operand(&arguments[0], method_name, position)?;
                let other = &other;
                let result: indexmap::IndexSet<ObjectHash> = set_rc
                    .borrow()
                    .intersection(&other.borrow())
                    .cloned()
                    .collect();
                Ok(Some(Object::Set(Rc::new(RefCell::new(result)))))
            }
            "difference" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.set_operand(&arguments[0], method_name, position)?;
                let other = &other;
                let result: indexmap::IndexSet<ObjectHash> = set_rc
                    .borrow()
                    .difference(&other.borrow())
                    .cloned()
                    .collect();
                Ok(Some(Object::Set(Rc::new(RefCell::new(result)))))
            }
            "each" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => block,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        let size = set_rc.borrow().len() as i64;
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                vec![],
                                Some(size),
                                position,
                            )
                            .map(Some);
                    }
                };
                let elements: Vec<ObjectHash> = set_rc.borrow().iter().cloned().collect();
                self.iterating_sets.push(Rc::as_ptr(set_rc) as usize);
                let walk = (|vm: &mut Self| -> Result<(), MetorexError> {
                    for elem in elements {
                        let args = vec![elem.value.clone()];
                        match vm.execute_block_with_control_flow(&block, args)? {
                            super::super::ControlFlow::Next
                            | super::super::ControlFlow::Value(_)
                            | super::super::ControlFlow::Redo { .. }
                            | super::super::ControlFlow::Continue { .. } => {
                                continue;
                            }
                            super::super::ControlFlow::Break { .. } => break,
                            super::super::ControlFlow::Return { value, position } => {
                                return Err(MetorexError::NonLocalReturn {
                                    value,
                                    location: super::super::utils::position_to_location(position),
                                    home_frame: block.home_frame,
                                });
                            }
                            // The exception the block raised carries on as
                            // itself, so a rescue naming its class catches it.
                            super::super::ControlFlow::Exception {
                                exception,
                                position,
                            } => {
                                let message = super::super::utils::format_exception(&exception);
                                return Err(MetorexError::UncaughtException {
                                    exception,
                                    location: super::super::utils::position_to_location(position),
                                    message,
                                });
                            }
                        }
                    }
                    Ok(())
                })(self);
                self.iterating_sets.pop();
                walk?;
                Ok(Some(receiver.clone()))
            }
            "<<" | "add?" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let held = element_of(&arguments[0], position)?;
                let added = set_rc.borrow_mut().insert(held);
                if method_name == "add?" && !added {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(receiver.clone()))
            }
            "clear" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                set_rc.borrow_mut().clear();
                Ok(Some(receiver.clone()))
            }
            "merge" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                for argument in arguments {
                    for element in self.enumerable_elements(argument, method_name, position)? {
                        let held = element_of(&element, position)?;
                        set_rc.borrow_mut().insert(held);
                    }
                }
                Ok(Some(receiver.clone()))
            }
            "subtract" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for element in self.enumerable_elements(&arguments[0], method_name, position)? {
                    let held = element_of(&element, position)?;
                    set_rc.borrow_mut().shift_remove(&held);
                }
                Ok(Some(receiver.clone()))
            }
            // The operator spellings of the set algebra.
            "|" | "+" => self.call_set_method(receiver, "union", arguments, position),
            "&" => self.call_set_method(receiver, "intersection", arguments, position),
            "-" => self.call_set_method(receiver, "difference", arguments, position),
            "^" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.set_operand(&arguments[0], method_name, position)?;
                let result: indexmap::IndexSet<ObjectHash> = other
                    .borrow()
                    .symmetric_difference(&set_rc.borrow())
                    .cloned()
                    .collect();
                Ok(Some(Object::Set(Rc::new(RefCell::new(result)))))
            }
            "subset?" | "<=" | "superset?" | ">=" | "proper_subset?" | "<" | "proper_superset?"
            | ">" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Some(theirs) = self.set_like_elements(&arguments[0], position)? else {
                    let message = "value must be a set".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                };
                let mine = set_rc.borrow();
                let contained = mine.iter().all(|held| theirs.contains(held));
                let contains = theirs.iter().all(|held| mine.contains(held));
                let answer = match method_name {
                    "subset?" | "<=" => contained,
                    "superset?" | ">=" => contains,
                    "proper_subset?" | "<" => contained && mine.len() < theirs.len(),
                    _ => contains && mine.len() > theirs.len(),
                };
                Ok(Some(Object::Bool(answer)))
            }
            // `<=>` reports containment: 0 when the sets are equal, -1 for a
            // proper subset, +1 for a proper superset, and nil when neither
            // holds.
            "<=>" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Some(Object::Set(other_rc)) = arguments.first() else {
                    return Ok(Some(Object::Nil));
                };
                let mine = set_rc.borrow();
                let theirs = other_rc.borrow();
                let contained = mine.iter().all(|held| theirs.contains(held));
                let contains = theirs.iter().all(|held| mine.contains(held));
                Ok(Some(match (contained, contains) {
                    (true, true) => Object::Int(0),
                    (true, false) => Object::Int(-1),
                    (false, true) => Object::Int(1),
                    (false, false) => Object::Nil,
                }))
            }
            "disjoint?" | "intersect?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.set_operand(&arguments[0], method_name, position)?;
                let shares = set_rc
                    .borrow()
                    .iter()
                    .any(|held| other.borrow().contains(held));
                Ok(Some(Object::Bool(if method_name == "disjoint?" {
                    !shares
                } else {
                    shares
                })))
            }
            "==" | "eql?" => {
                let Some(other) = arguments.first() else {
                    return Ok(Some(Object::Bool(false)));
                };
                let Some(theirs) = self.set_like_elements(other, position)? else {
                    return Ok(Some(Object::Bool(false)));
                };
                let mine = set_rc.borrow();
                let same =
                    mine.len() == theirs.len() && mine.iter().all(|held| theirs.contains(held));
                Ok(Some(Object::Bool(same)))
            }
            "dup" | "clone" | "to_set" => {
                let copy: indexmap::IndexSet<ObjectHash> = set_rc.borrow().clone();
                Ok(Some(Object::Set(Rc::new(RefCell::new(copy)))))
            }
            // The in-place filters, which keep or drop by what the block
            // answers and hand back the set itself.
            "delete_if" | "keep_if" | "select!" | "filter!" | "reject!" | "map!" | "collect!" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .build_enumerator(receiver.clone(), method_name, vec![], None, position)
                        .map(Some);
                };
                let elements: Vec<ObjectHash> = set_rc.borrow().iter().cloned().collect();
                let mut kept: indexmap::IndexSet<ObjectHash> = indexmap::IndexSet::new();
                let mut changed = false;
                for held in elements {
                    let answer =
                        self.execute_block_callable(&block, vec![held.value.clone()], position)?;
                    match method_name {
                        "map!" | "collect!" => {
                            let mapped = element_of(&answer, position)?;
                            changed |= mapped != held;
                            kept.insert(mapped);
                        }
                        "reject!" | "delete_if" => {
                            if answer.is_truthy() {
                                changed = true;
                            } else {
                                kept.insert(held);
                            }
                        }
                        _ => {
                            if answer.is_truthy() {
                                kept.insert(held);
                            } else {
                                changed = true;
                            }
                        }
                    }
                }
                *set_rc.borrow_mut() = kept;
                // The bang forms answer nil when they changed nothing.
                if matches!(method_name, "select!" | "filter!" | "reject!") && !changed {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(receiver.clone()))
            }
            "join" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let array = self.send_to_object(receiver.clone(), "to_a", vec![], position)?;
                let class = self.builtins().class_of(&array);
                self.call_native_method(&class, &array, "join", arguments, position)
            }
            "replace" => {
                self.refuse_mutation_during_iteration(set_rc, method_name, position)?;
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let replacement = self.set_operand(&arguments[0], method_name, position)?;
                let elements = replacement.borrow().clone();
                *set_rc.borrow_mut() = elements;
                Ok(Some(receiver.clone()))
            }
            "inspect" | "to_s" => {
                let elements: Vec<Object> = set_rc
                    .borrow()
                    .iter()
                    .map(|held| held.value.clone())
                    .collect();
                let rendered = crate::object::render_guarded(Rc::as_ptr(set_rc) as usize, || {
                    elements
                        .iter()
                        .map(super::array_methods::inspect_element)
                        .collect::<Vec<String>>()
                        .join(", ")
                });
                Ok(Some(Object::string(match rendered {
                    Some(body) => format!("Set[{}]", body),
                    None => "Set[...]".to_string(),
                })))
            }
            "hash" => {
                let mut total: i64 = 0;
                for held in set_rc.borrow().iter() {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash(&held.hash_value, &mut hasher);
                    total ^= std::hash::Hasher::finish(&hasher) as i64;
                }
                Ok(Some(Object::Int(total)))
            }
            _ => Ok(None),
        }
    }

    /// A Set cannot be changed while a walk over it is open, which is what
    /// Ruby reports as a modification during iteration.
    fn refuse_mutation_during_iteration(
        &self,
        set_rc: &Rc<RefCell<indexmap::IndexSet<ObjectHash>>>,
        caller: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if !self.iterating_sets.contains(&(Rc::as_ptr(set_rc) as usize)) {
            return Ok(());
        }
        let message = format!("can't add to a set during iteration ({})", caller);
        Err(MetorexError::UncaughtException {
            exception: Object::exception("RuntimeError", message.clone()),
            location: position_to_location(position),
            message,
        })
    }

    /// Whether the set holds `wanted`. An element the rendering does not
    /// match is still the same element when its `hash` and `eql?` say so,
    /// which is what Ruby's Set membership asks.
    fn set_holds(
        &mut self,
        set_rc: &Rc<RefCell<indexmap::IndexSet<ObjectHash>>>,
        wanted: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if let Some(rendered) = ObjectHash::from_object(wanted)
            && set_rc.borrow().contains(&rendered)
        {
            return Ok(true);
        }
        if !matches!(wanted, Object::Instance(_)) {
            return Ok(false);
        }
        let wanted_hash = self.send_to_object(wanted.clone(), "hash", vec![], position)?;
        let held: Vec<Object> = set_rc
            .borrow()
            .iter()
            .filter(|held| matches!(held.value, Object::Instance(_)))
            .map(|held| held.value.clone())
            .collect();
        for candidate in held {
            let candidate_hash =
                self.send_to_object(candidate.clone(), "hash", vec![], position)?;
            if !candidate_hash.equals(&wanted_hash) {
                continue;
            }
            let same =
                self.send_to_object(wanted.clone(), "eql?", vec![candidate.clone()], position)?;
            if same.is_truthy() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The other operand of a set operation, which may be any Enumerable.
    fn set_operand(
        &mut self,
        argument: &Object,
        caller: &str,
        position: Position,
    ) -> Result<Rc<RefCell<indexmap::IndexSet<ObjectHash>>>, MetorexError> {
        if let Object::Set(other) = argument {
            return Ok(Rc::clone(other));
        }
        let mut collected = indexmap::IndexSet::new();
        for element in self.enumerable_elements(argument, caller, position)? {
            collected.insert(element_of(&element, position)?);
        }
        Ok(Rc::new(RefCell::new(collected)))
    }

    /// The elements of an operand, which has to answer `to_a`.
    fn enumerable_elements(
        &mut self,
        argument: &Object,
        caller: &str,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let walked = match argument {
            Object::Array(elements) => return Ok(elements.borrow().clone()),
            Object::Set(elements) => {
                return Ok(elements
                    .borrow()
                    .iter()
                    .map(|held| held.value.clone())
                    .collect());
            }
            other if self.responds_to(other, "to_a") => {
                self.send_to_object(other.clone(), "to_a", vec![], position)?
            }
            _ => {
                let message = format!("value must be enumerable ({})", caller);
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            }
        };
        match walked {
            Object::Array(elements) => Ok(elements.borrow().clone()),
            _ => Ok(Vec::new()),
        }
    }
}

/// The set element an object stands for, or an error when it has no stable
/// rendering to be told apart by.
fn element_of(object: &Object, position: Position) -> Result<ObjectHash, MetorexError> {
    ObjectHash::from_object(object).ok_or_else(|| {
        MetorexError::runtime_error(
            format!("Cannot add {} to set (not hashable)", object.type_name()),
            position_to_location(position),
        )
    })
}

impl VirtualMachine {
    /// The elements of a value that stands for a set. A Set answers its own,
    /// and so does anything whose `is_a?(Set)` says it is one, which is how
    /// Ruby lets a set-like object be compared against a real Set.
    fn set_like_elements(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<indexmap::IndexSet<ObjectHash>>, MetorexError> {
        if let Object::Set(held) = value {
            return Ok(Some(held.borrow().clone()));
        }
        let Some(set_class) = self.globals().get("Set") else {
            return Ok(None);
        };
        let answer = self.send_to_object(value.clone(), "is_a?", vec![set_class], position)?;
        if !answer.is_truthy() {
            return Ok(None);
        }
        let listed = self.send_to_object(value.clone(), "to_a", vec![], position)?;
        let Object::Array(elements) = listed else {
            return Ok(None);
        };
        let mut collected = indexmap::IndexSet::new();
        for element in elements.borrow().iter() {
            collected.insert(element_of(element, position)?);
        }
        Ok(Some(collected))
    }
}
