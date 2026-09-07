//! Native method implementations for the Hash class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

/// Sentinel key for storing the default proc on hashes created with Hash.new { ... }
const DEFAULT_PROC_KEY: &str = "__MX_DEFAULT_PROC__";
/// The value `Hash.new(default)` answers for a key the hash has no entry for.
const DEFAULT_VALUE_KEY: &str = "__MX_DEFAULT__";
/// Sentinel key for storing original non-primitive key objects
const KEY_OBJECTS_KEY: &str = "__MX_KEY_OBJECTS__";
/// Sentinel marking a hash that `compare_by_identity` was called on.
const BY_IDENTITY_KEY: &str = "__MX_BY_IDENTITY__";

/// Check if a key is an internal sentinel key
fn is_internal_key(key: &str) -> bool {
    key == DEFAULT_PROC_KEY
        || key == DEFAULT_VALUE_KEY
        || key == BY_IDENTITY_KEY
        || key == "__MX_KWARGS__"
        || key == KEY_OBJECTS_KEY
}

/// Reconstruct the original key Object from its string key, using the sentinel
/// __MX_KEY_OBJECTS__ sub-map if present for non-primitive keys.
fn reconstruct_key(dict: &indexmap::IndexMap<String, Object>, key_str: &str) -> Object {
    if let Some(Object::Dict(key_objs)) = dict.get(KEY_OBJECTS_KEY)
        && let Some(obj) = key_objs.borrow().get(key_str)
    {
        return obj.clone();
    }
    crate::vm::utils::dict_key_to_object(key_str)
}

impl VirtualMachine {
    /// Execute native methods for the Hash class.
    pub(crate) fn call_hash_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Dict(dict_rc) = receiver else {
            return Ok(None);
        };
        match method_name {
            // No-op stub: metorex doesn't distinguish identity from equality
            // for hash keys, so `compare_by_identity` just returns the receiver.
            "compare_by_identity" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                dict_rc
                    .borrow_mut()
                    .insert(BY_IDENTITY_KEY.to_string(), Object::Bool(true));
                Ok(Some(receiver.clone()))
            }
            "compare_by_identity?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(
                    dict_rc.borrow().contains_key(BY_IDENTITY_KEY),
                )))
            }
            "keys" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let keys: Vec<Object> = dict
                    .keys()
                    .filter(|k| !is_internal_key(k))
                    .map(|k| reconstruct_key(&dict, k))
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(keys)))))
            }
            "values" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let values: Vec<Object> = dict
                    .iter()
                    .filter(|(k, _)| !is_internal_key(k))
                    .map(|(_, v)| v.clone())
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(values)))))
            }
            "has_key?" | "key?" | "include?" | "member?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                Ok(Some(Object::Bool(found.is_some())))
            }
            "entries" | "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let entries: Vec<Object> = dict
                    .iter()
                    .filter(|(k, _)| !is_internal_key(k))
                    .map(|(k, v)| {
                        Object::Array(Rc::new(RefCell::new(vec![
                            reconstruct_key(&dict, k),
                            v.clone(),
                        ])))
                    })
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(entries)))))
            }
            "delete" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                let Some(key_str) = found else {
                    // A key the hash has no entry for is handed to the block,
                    // which decides what `delete` answers.
                    return match block {
                        Some(block) => self
                            .execute_block_callable(&block, vec![arguments[0].clone()], position)
                            .map(Some),
                        None => Ok(Some(Object::Nil)),
                    };
                };
                let mut dict = dict_rc.borrow_mut();
                let removed = dict.shift_remove(&key_str).unwrap_or(Object::Nil);
                // Also remove from key objects sentinel if present
                if let Some(Object::Dict(key_objs)) = dict.get(KEY_OBJECTS_KEY) {
                    key_objs.borrow_mut().shift_remove(&key_str);
                }
                Ok(Some(removed))
            }
            // The value and the block a hash answers with for a key it has no
            // entry for, and the writers that set them.
            "default" => {
                // `default(key)` runs the default proc for that key, while
                // `default` on its own answers the stored value.
                let proc = dict_rc.borrow().get(DEFAULT_PROC_KEY).cloned();
                if let (Some(key), Some(Object::Block(block))) = (arguments.first(), proc) {
                    return self
                        .execute_block_callable(
                            &block,
                            vec![receiver.clone(), key.clone()],
                            position,
                        )
                        .map(Some);
                }
                Ok(Some(
                    dict_rc
                        .borrow()
                        .get(DEFAULT_VALUE_KEY)
                        .cloned()
                        .unwrap_or(Object::Nil),
                ))
            }
            "default=" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                // Setting a default value clears the default proc, since a
                // hash answers with one or the other.
                let mut dict = dict_rc.borrow_mut();
                dict.shift_remove(DEFAULT_PROC_KEY);
                dict.insert(DEFAULT_VALUE_KEY.to_string(), arguments[0].clone());
                drop(dict);
                Ok(Some(arguments[0].clone()))
            }
            "default_proc" => Ok(Some(
                dict_rc
                    .borrow()
                    .get(DEFAULT_PROC_KEY)
                    .cloned()
                    .unwrap_or(Object::Nil),
            )),
            "default_proc=" => {
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::Nil => {
                        dict_rc.borrow_mut().shift_remove(DEFAULT_PROC_KEY);
                    }
                    other => {
                        dict_rc
                            .borrow_mut()
                            .insert(DEFAULT_PROC_KEY.to_string(), other.clone());
                    }
                }
                Ok(Some(arguments[0].clone()))
            }
            "length" | "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let count = dict.keys().filter(|k| !is_internal_key(k)).count();
                Ok(Some(Object::Int(count as i64)))
            }
            "[]" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(self.evaluate_index_operation(
                    receiver.clone(),
                    arguments[0].clone(),
                    position,
                )?))
            }
            // `dig(key, *rest)` — fetch `key`, then keep digging into the
            // result. A missing key answers nil without visiting `rest`.
            "dig" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let Some(key_str) = crate::vm::utils::object_to_dict_key(&arguments[0]) else {
                    return Ok(Some(Object::Nil));
                };
                let found = dict_rc.borrow().get(&key_str).cloned();
                let Some(value) = found else {
                    return Ok(Some(Object::Nil));
                };
                if arguments.len() == 1 {
                    return Ok(Some(value));
                }
                if matches!(value, Object::Nil) {
                    return Ok(Some(Object::Nil));
                }
                self.dig_into(&value, &arguments[1..], position).map(Some)
            }
            "get" | "fetch" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 2),
                        arguments.len(),
                        position,
                    ));
                }
                // The block is set aside first, since looking the key up may
                // run code of the program's own that would otherwise take it.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                if let Some(key) = found {
                    let value = dict_rc.borrow().get(&key).cloned();
                    if let Some(value) = value {
                        if block.is_some() && arguments.len() == 2 {
                            self.warn_hash_block_supersedes(position)?;
                        }
                        return Ok(Some(value));
                    }
                }
                // A block wins over a default value, and Ruby says so.
                if let Some(block) = block {
                    if arguments.len() == 2 {
                        self.warn_hash_block_supersedes(position)?;
                    }
                    return self
                        .execute_block_callable(&block, vec![arguments[0].clone()], position)
                        .map(Some);
                }
                if arguments.len() == 2 {
                    return Ok(Some(arguments[1].clone()));
                }
                if method_name == "get" {
                    return Ok(Some(Object::Nil));
                }
                let rendered =
                    crate::vm::native_methods::array_methods::inspect_element(&arguments[0]);
                let message = format!("key not found: {}", rendered);
                let error = crate::vm::errors::simple_exception("KeyError", &message, position);
                Err(self.with_key_error_details(error, receiver, &arguments[0]))
            }
            // `merge` answers a new hash and `merge!`/`update` change the
            // receiver. Each argument is put through `to_hash`, and a block
            // decides what a key both hashes hold ends up with.
            "merge" | "merge!" | "update" => {
                let in_place = method_name != "merge";
                if in_place && self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut merged = dict_rc.borrow().clone();
                for argument in arguments {
                    let other = match argument {
                        Object::Dict(_) => argument.clone(),
                        other if self.responds_to(other, "to_hash") => {
                            self.send_to_object(other.clone(), "to_hash", vec![], position)?
                        }
                        other => {
                            let message = format!(
                                "no implicit conversion of {} into Hash",
                                self.builtins().class_of(other).ruby_name()
                            );
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &message,
                                position,
                            ));
                        }
                    };
                    let Object::Dict(other_rc) = &other else {
                        continue;
                    };
                    let incoming: Vec<(String, Object)> = other_rc
                        .borrow()
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    for (key, value) in incoming {
                        if is_internal_key(&key) {
                            continue;
                        }
                        let settled = match (&block, merged.get(&key).cloned()) {
                            (Some(block), Some(existing)) => {
                                let key_object = reconstruct_key(&merged, &key);
                                self.execute_block_callable(
                                    block,
                                    vec![key_object, existing, value.clone()],
                                    position,
                                )?
                            }
                            _ => value.clone(),
                        };
                        merged.insert(key, settled);
                    }
                    // The key objects the other hash recorded travel with it.
                    if let Some(Object::Dict(other_keys)) = other_rc.borrow().get(KEY_OBJECTS_KEY) {
                        let mut ours = match merged.get(KEY_OBJECTS_KEY) {
                            Some(Object::Dict(existing)) => existing.borrow().clone(),
                            _ => indexmap::IndexMap::new(),
                        };
                        for (key, object) in other_keys.borrow().iter() {
                            ours.insert(key.clone(), object.clone());
                        }
                        merged.insert(
                            KEY_OBJECTS_KEY.to_string(),
                            Object::Dict(Rc::new(RefCell::new(ours))),
                        );
                    }
                }
                if !in_place {
                    return Ok(Some(carry_identity(dict_rc, merged)));
                }
                *dict_rc.borrow_mut() = merged;
                Ok(Some(receiver.clone()))
            }
            // `each_pair` is Ruby's alias for `each`.
            // `map` yields each pair and collects what the block answers,
            // which is an Array rather than a Hash.
            "map" | "collect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Err(MetorexError::runtime_error(
                        format!("{} requires a block", method_name),
                        position_to_location(position),
                    ));
                };
                let dict = dict_rc.borrow();
                let entries: Vec<(Object, Object)> = dict
                    .iter()
                    .filter(|(key, _)| !is_internal_key(key))
                    .map(|(key, value)| (reconstruct_key(&dict, key), value.clone()))
                    .collect();
                drop(dict);
                let mut mapped = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    mapped.push(self.execute_block_callable(&block, vec![key, value], position)?);
                }
                Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    mapped,
                )))))
            }
            "each" | "each_pair" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let entries = self.hash_pairs(dict_rc);
                for (key, value) in entries {
                    // Ruby yields one `[key, value]` array, which a block of
                    // two parameters spreads across them.
                    let args = vec![Object::array(vec![key, value])];
                    match self.execute_block_with_control_flow(&block, args)? {
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
                        super::super::ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::UncaughtException {
                                exception: exception.clone(),
                                location: super::super::utils::position_to_location(position),
                                message: super::super::utils::format_exception(&exception),
                            });
                        }
                    }
                }
                Ok(Some(receiver.clone()))
            }
            // The size and emptiness of the entries, leaving the sentinels
            // the hash keeps for its default and its key objects out.
            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let empty = dict_rc.borrow().keys().all(|key| is_internal_key(key));
                Ok(Some(Object::Bool(empty)))
            }
            // `has_value?` compares with `==`, which is what Ruby uses for
            // values as against the `eql?` it uses for keys.
            "has_value?" | "value?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for value in self.hash_values(dict_rc) {
                    if self.elements_equal(&value, &arguments[0], position)? {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                Ok(Some(Object::Bool(false)))
            }
            // `key(value)` answers the first key holding that value.
            "key" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for (key, value) in self.hash_pairs(dict_rc) {
                    if self.elements_equal(&value, &arguments[0], position)? {
                        return Ok(Some(key));
                    }
                }
                Ok(Some(Object::Nil))
            }
            "each_key" | "each_value" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                for (key, value) in self.hash_pairs(dict_rc) {
                    let yielded = if method_name == "each_key" {
                        key
                    } else {
                        value
                    };
                    self.execute_block_callable(&block, vec![yielded], position)?;
                }
                Ok(Some(receiver.clone()))
            }
            "invert" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut inverted = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let rendered = crate::vm::utils::object_to_dict_key(&value).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&value) {
                        remember_key_object(&mut inverted, &rendered, &value);
                    }
                    inverted.insert(rendered, key);
                }
                // The values become the keys, so the identity setting the old
                // keys were matched under does not carry over.
                Ok(Some(Object::Dict(Rc::new(RefCell::new(inverted)))))
            }
            // `to_h` without a block answers the hash itself, and `to_hash`
            // always does.
            "to_hash" => Ok(Some(receiver.clone())),
            "store" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                self.hash_store(dict_rc, &arguments[0], arguments[1].clone());
                self.record_environment_change(dict_rc, &arguments[0], &arguments[1]);
                Ok(Some(arguments[1].clone()))
            }
            "clear" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let kept: Vec<(String, Object)> = dict_rc
                    .borrow()
                    .iter()
                    .filter(|(key, _)| is_internal_key(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                let mut dict = dict_rc.borrow_mut();
                dict.clear();
                for (key, value) in kept {
                    dict.insert(key, value);
                }
                drop(dict);
                Ok(Some(receiver.clone()))
            }
            // `compact` drops the pairs whose value is nil, and `compact!`
            // does it in place, answering nil when there was none to drop.
            "compact!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let pairs = self.hash_pairs(dict_rc);
                let dropped: Vec<String> = pairs
                    .iter()
                    .filter(|(_, value)| matches!(value, Object::Nil))
                    .map(|(key, _)| crate::vm::utils::object_to_dict_key(key).unwrap_or_default())
                    .collect();
                if dropped.is_empty() {
                    return Ok(Some(Object::Nil));
                }
                let mut dict = dict_rc.borrow_mut();
                for key in dropped {
                    dict.shift_remove(&key);
                }
                drop(dict);
                Ok(Some(receiver.clone()))
            }
            "compact" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut kept = indexmap::IndexMap::new();
                // `compact` keeps the default and the default proc, which the
                // other derived hashes leave behind.
                for sentinel in [DEFAULT_VALUE_KEY, DEFAULT_PROC_KEY] {
                    if let Some(value) = dict_rc.borrow().get(sentinel) {
                        kept.insert(sentinel.to_string(), value.clone());
                    }
                }
                for (key, value) in self.hash_pairs(dict_rc) {
                    if matches!(value, Object::Nil) {
                        continue;
                    }
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut kept, &rendered, &key);
                    }
                    kept.insert(rendered, value);
                }
                Ok(Some(carry_identity(dict_rc, kept)))
            }
            // `except` drops the named keys, and `slice` keeps only them.
            "except" | "slice" => {
                let mut named = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    named.push(crate::vm::utils::object_to_dict_key(argument).unwrap_or_default());
                }
                let pairs = self.hash_pairs(dict_rc);
                let mut kept = indexmap::IndexMap::new();
                // `slice` answers the pairs in the order they were asked for,
                // while `except` keeps the hash's own order.
                if method_name == "slice" {
                    for wanted in &named {
                        let Some((key, value)) = pairs.iter().find(|(key, _)| {
                            &crate::vm::utils::object_to_dict_key(key).unwrap_or_default() == wanted
                        }) else {
                            continue;
                        };
                        if !crate::vm::utils::is_primitive_key(key) {
                            remember_key_object(&mut kept, wanted, key);
                        }
                        kept.insert(wanted.clone(), value.clone());
                    }
                    return Ok(Some(carry_identity(dict_rc, kept)));
                }
                for (key, value) in pairs {
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if named.contains(&rendered) {
                        continue;
                    }
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut kept, &rendered, &key);
                    }
                    kept.insert(rendered, value);
                }
                Ok(Some(carry_identity(dict_rc, kept)))
            }
            // `values_at` answers the values the named keys hold, and
            // `fetch_values` raises for a key the hash has no entry for.
            "values_at" | "fetch_values" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut picked = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let rendered =
                        crate::vm::utils::object_to_dict_key(argument).unwrap_or_default();
                    let found = dict_rc.borrow().get(&rendered).cloned();
                    match found {
                        Some(value) => picked.push(value),
                        None if method_name == "values_at" => picked.push(Object::Nil),
                        None => match &block {
                            Some(block) => {
                                let block = Rc::clone(block);
                                picked.push(self.execute_block_callable(
                                    &block,
                                    vec![argument.clone()],
                                    position,
                                )?);
                            }
                            None => {
                                let message = format!(
                                    "key not found: {}",
                                    crate::vm::native_methods::array_methods::inspect_element(
                                        argument
                                    )
                                );
                                let error = crate::vm::errors::simple_exception(
                                    "KeyError", &message, position,
                                );
                                return Err(self.with_key_error_details(error, receiver, argument));
                            }
                        },
                    }
                }
                Ok(Some(Object::array(picked)))
            }
            // `transform_values` and `transform_keys` rebuild the hash with
            // what the block answers for each value or key.
            "transform_values" | "transform_keys" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let transforming_values = method_name == "transform_values";
                let mut built = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let subject = if transforming_values {
                        value.clone()
                    } else {
                        key.clone()
                    };
                    let answered = self.execute_block_callable(&block, vec![subject], position)?;
                    let (new_key, new_value) = if transforming_values {
                        (key, answered)
                    } else {
                        (answered, value)
                    };
                    let rendered =
                        crate::vm::utils::object_to_dict_key(&new_key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&new_key) {
                        remember_key_object(&mut built, &rendered, &new_key);
                    }
                    built.insert(rendered, new_value);
                }
                Ok(Some(carry_identity(dict_rc, built)))
            }
            // `select` and `reject` answer a new hash, while `keep_if` and
            // `delete_if` change the receiver and answer it.
            "select" | "filter" | "reject" | "keep_if" | "delete_if" | "select!" | "filter!"
            | "reject!" => {
                let in_place = matches!(
                    method_name,
                    "keep_if" | "delete_if" | "select!" | "filter!" | "reject!"
                );
                // Without a block it answers an Enumerator first, which Ruby
                // allows even on a frozen hash.
                if in_place
                    && matches!(self.pending_block, Some(Object::Block(_)))
                    && self.object_is_frozen(receiver)
                {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let rejecting = matches!(method_name, "reject" | "delete_if" | "reject!");
                let mut kept = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let verdict = self.execute_block_callable(
                        &block,
                        vec![key.clone(), value.clone()],
                        position,
                    )?;
                    if verdict.is_truthy() == rejecting {
                        continue;
                    }
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut kept, &rendered, &key);
                    }
                    kept.insert(rendered, value);
                }
                if !in_place {
                    return Ok(Some(carry_identity(dict_rc, kept)));
                }
                let before = dict_rc
                    .borrow()
                    .keys()
                    .filter(|key| !is_internal_key(key))
                    .count();
                let changed = kept.len() != before;
                let sentinels: Vec<(String, Object)> = dict_rc
                    .borrow()
                    .iter()
                    .filter(|(key, _)| is_internal_key(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                let mut dict = dict_rc.borrow_mut();
                dict.clear();
                for (key, value) in sentinels.into_iter().chain(kept) {
                    dict.insert(key, value);
                }
                drop(dict);
                // The bang forms answer nil when nothing changed, while
                // `keep_if` and `delete_if` always answer the hash.
                let bang = matches!(method_name, "select!" | "filter!" | "reject!");
                Ok(Some(if bang && !changed {
                    Object::Nil
                } else {
                    receiver.clone()
                }))
            }
            // Each key and value renders through its own `inspect`, so an
            // object that defines one is shown the way it asks to be. A hash
            // that reaches itself prints `{...}` rather than recursing.
            "inspect" | "to_s" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let address = Rc::as_ptr(dict_rc) as usize;
                if crate::object::rendering_in_progress(address) {
                    return Ok(Some(Object::string("{...}")));
                }
                crate::object::begin_rendering(address);
                let mut parts = Vec::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let rendered = self.render_pair(&key, &value, position);
                    match rendered {
                        Ok(rendered) => parts.push(rendered),
                        Err(error) => {
                            crate::object::end_rendering();
                            return Err(error);
                        }
                    }
                }
                crate::object::end_rendering();
                Ok(Some(Object::string(format!("{{{}}}", parts.join(", ")))))
            }
            // `[]=` writes one entry, the way `hash[key] = value` does.
            "[]=" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                self.hash_store(dict_rc, &arguments[0], arguments[1].clone());
                self.record_environment_change(dict_rc, &arguments[0], &arguments[1]);
                Ok(Some(arguments[1].clone()))
            }
            // `to_h` answers the hash itself, or what the block makes of
            // each pair.
            "to_h" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Ok(Some(receiver.clone()));
                };
                let mut built = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let produced =
                        self.execute_block_callable(&block, vec![key, value], position)?;
                    let (new_key, new_value) = self.pair_from_block_result(produced, position)?;
                    let rendered =
                        crate::vm::utils::object_to_dict_key(&new_key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&new_key) {
                        remember_key_object(&mut built, &rendered, &new_key);
                    }
                    built.insert(rendered, new_value);
                }
                Ok(Some(Object::Dict(Rc::new(RefCell::new(built)))))
            }
            // `replace` takes on the entries of another hash, which it asks
            // for with `to_hash`.
            "replace" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let other = match &arguments[0] {
                    Object::Dict(_) => arguments[0].clone(),
                    other if self.responds_to(other, "to_hash") => {
                        self.send_to_object(other.clone(), "to_hash", vec![], position)?
                    }
                    other => {
                        let message = format!(
                            "no implicit conversion of {} into Hash",
                            self.builtins().class_of(other).ruby_name()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                };
                let Object::Dict(other_rc) = &other else {
                    return Ok(Some(receiver.clone()));
                };
                // A Hash written as `replace(c: -1)` arrives carrying the
                // parser's keyword marker, which is not one of its entries.
                let mut incoming = other_rc.borrow().clone();
                incoming.shift_remove(crate::vm::param_binding::KWARGS_MARKER);
                *dict_rc.borrow_mut() = incoming;
                Ok(Some(receiver.clone()))
            }
            // The bang forms of the transforms change the receiver.
            "transform_values!" | "transform_keys!" => {
                if self.object_is_frozen(receiver)
                    && matches!(self.pending_block, Some(Object::Block(_)))
                {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let transforming_values = method_name == "transform_values!";
                let mut built = indexmap::IndexMap::new();
                // A `break` out of the block keeps what was transformed so
                // far and leaves the rest of the entries as they were.
                let mut broke_with = None;
                let pairs = self.hash_pairs(dict_rc);
                let mut walked = pairs.iter();
                for (key, value) in walked.by_ref() {
                    let (key, value) = (key.clone(), value.clone());
                    let subject = if transforming_values {
                        value.clone()
                    } else {
                        key.clone()
                    };
                    let answered =
                        match self.execute_block_callable(&block, vec![subject], position) {
                            Ok(answered) => answered,
                            Err(MetorexError::BlockBreak {
                                value: broke_value, ..
                            }) => {
                                broke_with = Some(broke_value);
                                keep_pair(&mut built, key, value);
                                break;
                            }
                            Err(error) => return Err(error),
                        };
                    let (new_key, new_value) = if transforming_values {
                        (key, answered)
                    } else {
                        (answered, value)
                    };
                    keep_pair(&mut built, new_key, new_value);
                }
                if broke_with.is_some() {
                    for (key, value) in walked {
                        keep_pair(&mut built, key.clone(), value.clone());
                    }
                }
                let sentinels: Vec<(String, Object)> = dict_rc
                    .borrow()
                    .iter()
                    .filter(|(key, _)| is_internal_key(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                let mut dict = dict_rc.borrow_mut();
                dict.clear();
                for (key, value) in sentinels.into_iter().chain(built) {
                    dict.insert(key, value);
                }
                drop(dict);
                Ok(Some(broke_with.unwrap_or_else(|| receiver.clone())))
            }
            // `assoc(key)` and `rassoc(value)` answer the matching pair.
            "assoc" | "rassoc" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let matching_key = method_name == "assoc";
                for (key, value) in self.hash_pairs(dict_rc) {
                    let candidate = if matching_key { &key } else { &value };
                    if self.elements_equal(candidate, &arguments[0], position)? {
                        return Ok(Some(Object::array(vec![key, value])));
                    }
                }
                Ok(Some(Object::Nil))
            }
            // `flatten` answers the pairs one after another, flattening a
            // level deeper when asked.
            "flatten" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let depth: i64 = match arguments.first() {
                    None => 1,
                    Some(Object::Int(level)) => *level,
                    Some(other) => self
                        .coerce_integer_argument(other, position)?
                        .try_into()
                        .unwrap_or(1),
                };
                let mut rows = Vec::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    rows.push(Object::array(vec![key, value]));
                }
                let flat = Object::array(rows);
                self.send_to_object(flat, "flatten", vec![Object::Int(depth)], position)
                    .map(Some)
            }
            // `sort` orders the pairs the way an array of them sorts.
            "sort" => {
                let mut rows = Vec::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    rows.push(Object::array(vec![key, value]));
                }
                let pairs = Object::array(rows);
                if let Some(block) = self.pending_block.take() {
                    self.pending_block = Some(block);
                }
                self.send_to_object(pairs, "sort", vec![], position)
                    .map(Some)
            }
            // `shift` removes the first pair and answers it.
            "shift" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let first = self.hash_pairs(dict_rc).into_iter().next();
                let Some((key, value)) = first else {
                    return Ok(Some(Object::Nil));
                };
                let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                dict_rc.borrow_mut().shift_remove(&rendered);
                Ok(Some(Object::array(vec![key, value])))
            }
            // `deconstruct_keys` answers the hash itself, whatever keys the
            // pattern asked for.
            "deconstruct_keys" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(receiver.clone()))
            }
            // `any?` answers whether the block holds for a pair, or whether
            // the hash holds anything at all.
            "any?" | "none?" | "all?" | "one?" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                // A pattern argument is matched with `===` against the
                // `[key, value]` pair, and it wins over a block.
                let pattern = arguments.first().cloned();
                if pattern.is_some() && block.is_some() {
                    self.emit_warning_to_stderr("warning: given block not used", position);
                }
                let pairs = self.hash_pairs(dict_rc);
                if pattern.is_none() && block.is_none() {
                    let any = !pairs.is_empty();
                    return Ok(Some(Object::Bool(match method_name {
                        "any?" => any,
                        "none?" => !any,
                        "one?" => pairs.len() == 1,
                        _ => true,
                    })));
                }
                let mut answers = Vec::with_capacity(pairs.len());
                for (key, value) in pairs {
                    let verdict = match &pattern {
                        Some(pattern) => self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::CaseEqual,
                            pattern.clone(),
                            Object::array(vec![key, value]),
                            position,
                        )?,
                        None => {
                            let block = block.as_ref().expect("a block or a pattern was given");
                            self.execute_block_callable(block, vec![key, value], position)?
                        }
                    };
                    answers.push(verdict.is_truthy());
                }
                Ok(Some(Object::Bool(match method_name {
                    "any?" => answers.iter().any(|held| *held),
                    "none?" => !answers.iter().any(|held| *held),
                    "one?" => answers.iter().filter(|held| **held).count() == 1,
                    _ => answers.iter().all(|held| *held),
                })))
            }
            // `<`, `<=`, `>`, and `>=` compare hashes by containment.
            "<" | "<=" | ">" | ">=" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // An operand that is not a Hash is asked for one, which is
                // what `to_hash` answers.
                let other = match &arguments[0] {
                    Object::Dict(_) => arguments[0].clone(),
                    other if self.responds_to(other, "to_hash") => {
                        self.send_to_object(other.clone(), "to_hash", vec![], position)?
                    }
                    other => {
                        let message = format!(
                            "no implicit conversion of {} into Hash",
                            self.builtins().class_of(other).ruby_name()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                };
                let Object::Dict(other_rc) = &other else {
                    let message = format!(
                        "no implicit conversion of {} into Hash",
                        self.builtins().class_of(&arguments[0]).ruby_name()
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &message,
                        position,
                    ));
                };
                let ours = self.hash_pairs(dict_rc);
                let theirs = self.hash_pairs(other_rc);
                let (smaller, larger) = match method_name {
                    "<" | "<=" => (&ours, &theirs),
                    _ => (&theirs, &ours),
                };
                let mut contained = true;
                for (key, value) in smaller {
                    let rendered = crate::vm::utils::object_to_dict_key(key).unwrap_or_default();
                    let found = larger
                        .iter()
                        .find(|(other_key, _)| {
                            crate::vm::utils::object_to_dict_key(other_key).unwrap_or_default()
                                == rendered
                        })
                        .map(|(_, other_value)| other_value.clone());
                    match found {
                        Some(other_value) => {
                            if !self.elements_equal(value, &other_value, position)? {
                                contained = false;
                                break;
                            }
                        }
                        None => {
                            contained = false;
                            break;
                        }
                    }
                }
                let strict = matches!(method_name, "<" | ">");
                let same_size = ours.len() == theirs.len();
                Ok(Some(Object::Bool(contained && !(strict && same_size))))
            }
            _ => Ok(None),
        }
    }
}

/// A hash key that is not a primitive is recorded in the sentinel sub-map, so
/// the original object comes back when the hash is walked.
fn remember_key_object(
    pairs: &mut indexmap::IndexMap<String, Object>,
    rendered: &str,
    key: &Object,
) {
    let mut objects = match pairs.get(KEY_OBJECTS_KEY) {
        Some(Object::Dict(existing)) => existing.borrow().clone(),
        _ => indexmap::IndexMap::new(),
    };
    objects.insert(rendered.to_string(), key.clone());
    pairs.insert(
        KEY_OBJECTS_KEY.to_string(),
        Object::Dict(Rc::new(RefCell::new(objects))),
    );
}

impl VirtualMachine {
    /// The key and value of every entry, with the sentinels left out.
    fn hash_pairs(
        &self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
    ) -> Vec<(Object, Object)> {
        let dict = dict_rc.borrow();
        dict.iter()
            .filter(|(key, _)| !is_internal_key(key))
            .map(|(key, value)| (reconstruct_key(&dict, key), value.clone()))
            .collect()
    }

    /// The value of every entry, with the sentinels left out.
    fn hash_values(
        &self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
    ) -> Vec<Object> {
        dict_rc
            .borrow()
            .iter()
            .filter(|(key, _)| !is_internal_key(key))
            .map(|(_, value)| value.clone())
            .collect()
    }

    /// Store one entry, recording the key object when it is not a primitive.
    fn hash_store(
        &self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        key: &Object,
        value: Object,
    ) {
        let rendered = crate::vm::utils::object_to_dict_key(key).unwrap_or_default();
        let mut dict = dict_rc.borrow_mut();
        if !crate::vm::utils::is_primitive_key(key) {
            remember_key_object(&mut dict, &rendered, key);
        }
        dict.insert(rendered, value);
    }
}

impl VirtualMachine {
    /// One `key => value` entry as `inspect` shows it. A Symbol key reads as
    /// `name: value`, the way Ruby prints one.
    fn render_pair(
        &mut self,
        key: &Object,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let rendered_value = self.get_inspect_representation(value, position)?;
        if let Object::Symbol(name) = key
            && name
                .chars()
                .next()
                .is_some_and(|first| first.is_alphabetic() || first == '_')
            && name.chars().all(|letter| {
                letter.is_alphanumeric() || letter == '_' || letter == '?' || letter == '!'
            })
        {
            return Ok(format!("{}: {}", name, rendered_value));
        }
        let rendered_key = self.get_inspect_representation(key, position)?;
        Ok(format!("{} => {}", rendered_key, rendered_value))
    }
}

impl VirtualMachine {
    /// The stored key string matching `wanted`, or None when the hash holds
    /// no entry for it. A key that is not a primitive is matched the way Ruby
    /// matches one: same `hash`, then `eql?`.
    pub(crate) fn hash_find_key(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        wanted: &Object,
        position: Position,
    ) -> Result<Option<String>, MetorexError> {
        let rendered = crate::vm::utils::object_to_dict_key(wanted).unwrap_or_default();
        if dict_rc.borrow().contains_key(&rendered) {
            return Ok(Some(rendered));
        }
        if crate::vm::utils::is_primitive_key(wanted) {
            return Ok(None);
        }
        let wanted_hash = self.send_to_object(wanted.clone(), "hash", vec![], position)?;
        let stored: Vec<(String, Object)> = {
            let dict = dict_rc.borrow();
            dict.keys()
                .filter(|key| !is_internal_key(key))
                .map(|key| (key.clone(), reconstruct_key(&dict, key)))
                .collect()
        };
        for (key, candidate) in stored {
            if crate::vm::utils::is_primitive_key(&candidate) {
                continue;
            }
            let candidate_hash =
                self.send_to_object(candidate.clone(), "hash", vec![], position)?;
            if !candidate_hash.equals(&wanted_hash) {
                continue;
            }
            // The key being looked up is the one asked, which is what Ruby's
            // hash lookup does.
            let same =
                self.send_to_object(wanted.clone(), "eql?", vec![candidate.clone()], position)?;
            if same.is_truthy() {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }
}

/// A hash derived from `source` keeps its compare-by-identity setting, which
/// Ruby carries into `slice`, `merge`, `select`, and the rest.
fn carry_identity(
    source: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
    mut built: indexmap::IndexMap<String, Object>,
) -> Object {
    if source.borrow().contains_key(BY_IDENTITY_KEY) {
        built.insert(BY_IDENTITY_KEY.to_string(), Object::Bool(true));
    }
    Object::Dict(Rc::new(RefCell::new(built)))
}

impl VirtualMachine {
    /// Ruby warns when `fetch` was handed both a default value and a block,
    /// since the block is the one it uses.
    fn warn_hash_block_supersedes(&mut self, position: Position) -> Result<(), MetorexError> {
        let file = self
            .current_source_file
            .clone()
            .unwrap_or_else(|| "-".to_string());
        let message = format!(
            "{}:{}: warning: block supersedes default value argument\n",
            file, position.line
        );
        self.warn_through_warning_module(message, position)
    }

    /// Record the hash and the key a KeyError was raised for, which `receiver`
    /// and `key` report.
    fn with_key_error_details(
        &self,
        error: MetorexError,
        receiver: &Object,
        key: &Object,
    ) -> MetorexError {
        let MetorexError::UncaughtException {
            exception,
            location,
            message,
        } = error
        else {
            return error;
        };
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.receiver = Some(Box::new(receiver.clone()));
            details
                .instance_vars
                .insert(crate::vm::KEY_ERROR_KEY.to_string(), key.clone());
        }
        MetorexError::UncaughtException {
            exception,
            location,
            message,
        }
    }
}

/// Record one entry in a hash being rebuilt, keeping the key object beside it
/// when the key does not read back from its rendering.
fn keep_pair(built: &mut indexmap::IndexMap<String, Object>, key: Object, value: Object) {
    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
    if !crate::vm::utils::is_primitive_key(&key) {
        remember_key_object(built, &rendered, &key);
    }
    built.insert(rendered, value);
}

impl VirtualMachine {
    /// A write to ENV reaches the process environment too, so the C library
    /// sees it. Anything the C library reads, such as the time zone, follows
    /// what the program set.
    pub(crate) fn record_environment_change(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        key: &Object,
        value: &Object,
    ) {
        let Some(Object::Dict(environment)) = self.globals().get("ENV") else {
            return;
        };
        if !Rc::ptr_eq(&environment, dict_rc) {
            return;
        }
        let Object::String(name) = key else {
            return;
        };
        let Ok(name) = std::ffi::CString::new(name.as_str()) else {
            return;
        };
        unsafe {
            match value {
                Object::Nil => {
                    libc::unsetenv(name.as_ptr());
                }
                Object::String(text) => {
                    let Ok(setting) = std::ffi::CString::new(text.as_str()) else {
                        return;
                    };
                    libc::setenv(name.as_ptr(), setting.as_ptr(), 1);
                }
                _ => {}
            }
        }
    }
}
