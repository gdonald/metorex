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

/// The environment methods whose one argument names a variable or a value,
/// which is written as text however it arrives.
const ENVIRONMENT_TEXT_ARGUMENT: &[&str] = &[
    "[]",
    "delete",
    "has_key?",
    "key?",
    "include?",
    "member?",
    "key",
    "assoc",
    "has_value?",
    "value?",
    "rassoc",
];

/// The subset of those whose argument is a value rather than a name.
const ENVIRONMENT_VALUE_ARGUMENT: &[&str] = &["has_value?", "value?", "rassoc"];

impl VirtualMachine {
    /// The text an argument to an environment method names, refusing anything
    /// that names none.
    fn environment_text(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = value {
            return Ok(text.to_text());
        }
        if self.responds_to(value, "to_str")
            && let Object::String(text) =
                self.send_to_object(value.clone(), "to_str", vec![], position)?
        {
            return Ok(text.to_text());
        }
        let message = format!(
            "no implicit conversion of {} into String",
            self.builtins().class_of(value).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    /// The encoding the environment is read as: the one a program named, or
    /// the one the system is set to.
    fn environment_reading_encoding(&mut self) -> Option<String> {
        if let Some(Object::Class(held)) = self.globals().get("__Encoding_default_internal") {
            return Some(held.name().to_string());
        }
        match self.globals().get("__Encoding_default_external") {
            Some(Object::Class(held)) => Some(held.name().to_string()),
            _ => None,
        }
    }

    /// Refuse a name the environment could never hold.
    fn refuse_bad_variable_name(&self, key: &str, position: Position) -> Result<(), MetorexError> {
        if key.is_empty() || key.contains('=') {
            let message = format!("Invalid argument - setenv({})", key);
            return Err(crate::vm::errors::simple_exception(
                "Errno::EINVAL",
                &message,
                position,
            ));
        }
        Ok(())
    }

    /// Put one variable into the environment, where the C library sees it too.
    fn store_variable(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        key: String,
        value: String,
    ) {
        let named = Object::string(key.clone());
        let held = Object::string(value);
        // A name that reads back as some other kind of value is kept beside
        // the entry, so `ENV["1"]` stays a name rather than a number.
        if !crate::vm::utils::is_primitive_key(&named) {
            let slot = "__MX_KEY_OBJECTS__".to_string();
            let mut objects = match dict_rc.borrow().get(&slot) {
                Some(Object::Dict(held)) => held.borrow().clone(),
                _ => indexmap::IndexMap::new(),
            };
            objects.insert(key.clone(), named.clone());
            dict_rc
                .borrow_mut()
                .insert(slot, Object::Dict(Rc::new(RefCell::new(objects))));
        }
        dict_rc.borrow_mut().insert(key, held.clone());
        self.record_environment_change(dict_rc, &named, &held);
    }

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
        // ENV is a hash the process shares with the operating system, and a
        // handful of methods answer differently on it.
        if self.dict_is_environment(dict_rc) {
            match method_name {
                "to_s" => return Ok(Some(Object::string("ENV"))),
                "rehash" => return Ok(Some(Object::Nil)),
                // A copy of ENV would stop tracking the environment, so Ruby
                // refuses and points at the hash it will make instead. The
                // keywords are read first, so a bad one is reported as such.
                "dup" | "clone" => {
                    if let Some(Object::Dict(entries)) = arguments.first() {
                        for (name, value) in entries.borrow().iter() {
                            let name = name.trim_start_matches(':');
                            if name.starts_with("__MX_") {
                                continue;
                            }
                            let refused =
                                name != "freeze" || !matches!(value, Object::Bool(_) | Object::Nil);
                            if refused {
                                let message = if name == "freeze" {
                                    "unexpected value for freeze: Integer".to_string()
                                } else {
                                    format!("unknown keyword: :{}", name)
                                };
                                return Err(crate::vm::errors::simple_exception(
                                    "ArgumentError",
                                    &message,
                                    position,
                                ));
                            }
                        }
                    }
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!(
                            "Cannot {method_name} ENV, use ENV.to_h to get a copy of ENV as a hash"
                        ),
                        position,
                    ));
                }
                // `to_h` and `to_hash` hand back a hash of their own, so
                // changing it leaves the environment alone.
                "to_h" | "to_hash" if self.pending_block.is_none() => {
                    let copied = dict_rc.borrow().clone();
                    return Ok(Some(Object::Dict(Rc::new(RefCell::new(copied)))));
                }
                // Every key the environment is asked about names a variable,
                // so anything that is not a String is refused outright. Only
                // the keys are checked: `fetch` takes a default after them.
                "fetch" | "values_at" => {
                    let keys = if method_name == "fetch" {
                        arguments.get(..1).unwrap_or(&[])
                    } else {
                        arguments
                    };
                    for argument in keys {
                        if !matches!(argument, Object::String(_))
                            && !self.responds_to(argument, "to_str")
                        {
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &format!(
                                    "no implicit conversion of {} into String",
                                    self.builtins().class_of(argument).name()
                                ),
                                position,
                            ));
                        }
                    }
                }
                // What the environment is looked up by names a String, so
                // anything that reads as one is asked for its text. A name
                // that reads as nothing is refused, while a value that reads
                // as nothing simply matches nothing.
                name if ENVIRONMENT_TEXT_ARGUMENT.contains(&name)
                    && arguments.len() == 1
                    && !matches!(arguments[0], Object::String(_)) =>
                {
                    if self.responds_to(&arguments[0], "to_str") {
                        let named =
                            self.send_to_object(arguments[0].clone(), "to_str", vec![], position)?;
                        return self.call_hash_method(receiver, method_name, &[named], position);
                    }
                    if ENVIRONMENT_VALUE_ARGUMENT.contains(&name) {
                        return Ok(Some(Object::Nil));
                    }
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!(
                            "no implicit conversion of {} into String",
                            self.builtins().class_of(&arguments[0]).name()
                        ),
                        position,
                    ));
                }
                // A variable is named and valued in text, and a name that
                // holds an `=` or nothing at all names no variable.
                "[]=" | "store" if arguments.len() == 2 => {
                    let key = self.environment_text(&arguments[0], position)?;
                    // Setting a name to nil takes it away, and a name the
                    // environment could never hold simply has nothing to take.
                    if matches!(arguments[1], Object::Nil) {
                        let named = Object::string(key);
                        self.record_environment_change(dict_rc, &named, &Object::Nil);
                        return Ok(Some(Object::Nil));
                    }
                    if key.is_empty() || key.contains('=') {
                        let message = format!("Invalid argument - setenv({})", key);
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::EINVAL",
                            &message,
                            position,
                        ));
                    }
                    let value = self.environment_text(&arguments[1], position)?;
                    self.store_variable(dict_rc, key, value);
                    // The assignment answers what it was handed, which is the
                    // same object when that was already text.
                    return Ok(Some(arguments[1].clone()));
                }
                // `slice` looks each name up as text and answers under the
                // objects it was handed.
                "slice" => {
                    let named = self.environment_reading_encoding();
                    let mut made: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
                    let mut keyed: Vec<(String, Object)> = Vec::new();
                    for argument in arguments {
                        let key = self.environment_text(argument, position)?;
                        let Some(value) = dict_rc.borrow().get(&key).cloned() else {
                            continue;
                        };
                        let Some(slot) = crate::vm::utils::object_to_dict_key(argument) else {
                            continue;
                        };
                        made.insert(slot.clone(), retagged(value, &named));
                        keyed.push((slot, argument.clone()));
                    }
                    if !keyed.is_empty() {
                        let mut objects: indexmap::IndexMap<String, Object> =
                            indexmap::IndexMap::new();
                        for (slot, held) in keyed {
                            objects.insert(slot, held);
                        }
                        made.insert(
                            "__MX_KEY_OBJECTS__".to_string(),
                            Object::Dict(Rc::new(RefCell::new(objects))),
                        );
                    }
                    return Ok(Some(Object::Dict(Rc::new(RefCell::new(made)))));
                }
                // What the environment answers is written in the encoding a
                // program asked for, and in the one the system uses otherwise.
                "[]" | "shift" if arguments.len() <= 1 => {
                    let named = self.environment_reading_encoding();
                    if method_name == "shift" {
                        let Some((key, value)) = self.hash_pairs(dict_rc).into_iter().next() else {
                            return Ok(Some(Object::Nil));
                        };
                        self.record_environment_change(dict_rc, &key, &Object::Nil);
                        return Ok(Some(Object::array(vec![
                            retagged(key, &named),
                            retagged(value, &named),
                        ])));
                    }
                    let key = self.environment_text(&arguments[0], position)?;
                    let held = dict_rc.borrow().get(&key).cloned();
                    return Ok(Some(match held {
                        // A value read out of the environment is frozen, so
                        // changing what was handed back cannot reach the
                        // environment behind it.
                        Some(value) => frozen_text(retagged(value, &named)),
                        None => Object::Nil,
                    }));
                }
                // Every name and value a whole hash brings is read as text
                // first, so a bad one stops the change before it starts.
                "replace" | "merge!" | "update" if arguments.len() == 1 => {
                    let Some(Object::Dict(given)) = arguments.first() else {
                        let message = format!(
                            "no implicit conversion of {} into Hash",
                            self.builtins()
                                .class_of(arguments.first().unwrap_or(&Object::Nil))
                                .name()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    };
                    let pairs = self.hash_pairs(given);
                    // `replace` reads every name and value before it changes
                    // anything, where `merge!` changes as it goes.
                    if method_name == "replace" {
                        let mut settled = Vec::new();
                        for (key, value) in &pairs {
                            let key = self.environment_text(key, position)?;
                            let value = self.environment_text(value, position)?;
                            self.refuse_bad_variable_name(&key, position)?;
                            settled.push((key, value));
                        }
                        for key in self.hash_pairs(dict_rc).into_iter().map(|(key, _)| key) {
                            self.record_environment_change(dict_rc, &key, &Object::Nil);
                        }
                        dict_rc.borrow_mut().clear();
                        for (key, value) in settled {
                            self.store_variable(dict_rc, key, value);
                        }
                        return Ok(Some(receiver.clone()));
                    }
                    let block = self.pending_block.take();
                    for (key, value) in pairs {
                        let key = self.environment_text(&key, position)?;
                        let mut value = self.environment_text(&value, position)?;
                        self.refuse_bad_variable_name(&key, position)?;
                        // A block settles a name both sides hold.
                        if let Some(Object::Block(block)) = &block
                            && let Some(held) = dict_rc.borrow().get(&key).cloned()
                        {
                            let given = vec![
                                Object::string(key.clone()),
                                held,
                                Object::string(value.clone()),
                            ];
                            let answered = self.execute_block_callable(block, given, position)?;
                            value = self.environment_text(&answered, position)?;
                        }
                        self.store_variable(dict_rc, key, value);
                    }
                    return Ok(Some(receiver.clone()));
                }
                // What the environment answers is written in the encoding
                // the program asked for, when it asked for one.
                "each" | "each_pair"
                    if arguments.is_empty()
                        && matches!(self.pending_block, Some(Object::Block(_))) =>
                {
                    let Some(Object::Block(block)) = self.pending_block.take() else {
                        return Ok(None);
                    };
                    let named = self.environment_reading_encoding();
                    for (key, value) in self.hash_pairs(dict_rc) {
                        let key = retagged(key, &named);
                        let value = retagged(value, &named);
                        let pair = Object::array(vec![key, value]);
                        match self.execute_block_with_control_flow(&block, vec![pair])? {
                            // `break` ends the walk and answers what it carried,
                            // which is what the call reports.
                            super::super::ControlFlow::Break { value, .. } => {
                                return Ok(Some(value));
                            }
                            super::super::ControlFlow::Return { value, position } => {
                                return Err(MetorexError::NonLocalReturn {
                                    value,
                                    location: position_to_location(position),
                                    home_frame: None,
                                });
                            }
                            _ => {}
                        }
                    }
                    return Ok(Some(receiver.clone()));
                }
                _ => {}
            }
        }
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
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                dict_rc
                    .borrow_mut()
                    .insert(BY_IDENTITY_KEY.to_string(), Object::Bool(true));
                // Entries already held were placed by value, so they are put
                // back where their identity belongs.
                self.call_hash_method(receiver, "rehash", &[], position)?;
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
            // Two hashes holding the same pairs hash alike however they were
            // built, so the pairs are folded in a way the order cannot
            // change. A hash held inside another stands for its kind and its
            // count, which is what lets one that reaches itself hash at all.
            "hash" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let pairs: Vec<(Object, Object)> = {
                    let dict = dict_rc.borrow();
                    dict.iter()
                        .filter(|(key, _)| !is_internal_key(key))
                        .map(|(key, value)| (reconstruct_key(&dict, key), value.clone()))
                        .collect()
                };
                let mut total: i64 = pairs.len() as i64;
                for (key, value) in pairs {
                    let key_digest = self.shallow_digest(&key, position)?;
                    let value_digest = self.shallow_digest(&value, position)?;
                    total =
                        total.wrapping_add(key_digest.wrapping_mul(31).wrapping_add(value_digest));
                }
                Ok(Some(Object::Int(total)))
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
                        // Anything that spells itself out as a callable may
                        // be the default, and one that names its arguments
                        // strictly must take the hash and the key.
                        let held = match other {
                            Object::Block(_) | Object::Method(_) => other.clone(),
                            _ if self.responds_to(other, "to_proc") => {
                                self.send_to_object(other.clone(), "to_proc", vec![], position)?
                            }
                            _ => {
                                return Err(crate::vm::errors::simple_exception(
                                    "TypeError",
                                    &format!(
                                        "wrong default_proc type {} (expected Proc)",
                                        self.builtins().class_of(other).ruby_name()
                                    ),
                                    position,
                                ));
                            }
                        };
                        if let Object::Block(block) = &held
                            && block.is_lambda
                        {
                            let counted = super::method_object_methods::block_arity(block);
                            if counted != 2 {
                                return Err(crate::vm::errors::simple_exception(
                                    "TypeError",
                                    "default_proc takes two arguments (2 for 1)",
                                    position,
                                ));
                            }
                        }
                        let mut dict = dict_rc.borrow_mut();
                        // A hash answers with a default value or a default
                        // block, never both.
                        dict.shift_remove(DEFAULT_VALUE_KEY);
                        dict.insert(DEFAULT_PROC_KEY.to_string(), held);
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
                // A subclass that writes its own `default` decides what a
                // missing key reads as, which is what Ruby asks before it
                // falls back to the default value or block.
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                if found.is_none()
                    && let Some((owner, method)) = self.lookup_method(receiver, "default")
                    && !method.is_undefined
                {
                    return self
                        .invoke_method(
                            owner,
                            method,
                            receiver.clone(),
                            vec![arguments[0].clone()],
                            position,
                        )
                        .map(Some);
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
                // A key the hash holds nothing for reads as the default,
                // since `dig` goes through `[]` the way Ruby's does.
                let found = dict_rc.borrow().get(&key_str).cloned();
                let value = match found {
                    Some(held) => held,
                    None => {
                        let default =
                            self.call_hash_method(receiver, "[]", &arguments[..1], position)?;
                        match default {
                            Some(held) => held,
                            None => return Ok(Some(Object::Nil)),
                        }
                    }
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
                        // An instance of a Hash subclass is merged by the
                        // entries it holds, without `to_hash` being asked for.
                        other
                            if crate::vm::native_methods::hash_subclass_value(other).is_some() =>
                        {
                            crate::vm::native_methods::hash_subclass_value(other)
                                .expect("a hash subclass carries its entries")
                        }
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
                // A block written for one value reads the pair as an array,
                // which is the single value a Hash yields. One written for
                // two reads the key and the value apart.
                let takes_pair_apart = block
                    .binding_parameters()
                    .iter()
                    .filter(|named| !named.starts_with('&'))
                    .count()
                    >= 2;
                let mut mapped = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let given = if takes_pair_apart {
                        vec![key, value]
                    } else {
                        vec![Object::array(vec![key, value])]
                    };
                    mapped.push(self.execute_block_callable(&block, given, position)?);
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
                        | super::super::ControlFlow::Retry { .. }
                        | super::super::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        // `break` ends the walk and answers what it carried,
                        // which is what the call reports.
                        super::super::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
                        }
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
            // Two hashes are equal when they hold the same entries, and
            // each key is looked up in the other hash the way any key is,
            // through `#hash` and `#eql?`. `eql?` compares the values the
            // same strict way; `==` asks them `==`.
            "==" | "eql?" if arguments.len() == 1 => {
                let other = match &arguments[0] {
                    held @ Object::Dict(_) => held.clone(),
                    held => match crate::vm::native_methods::hash_subclass_value(held) {
                        Some(backing @ Object::Dict(_)) => backing,
                        _ => return Ok(Some(Object::Bool(false))),
                    },
                };
                let Object::Dict(other_rc) = &other else {
                    return Ok(Some(Object::Bool(false)));
                };
                if Rc::ptr_eq(dict_rc, other_rc) {
                    return Ok(Some(Object::Bool(true)));
                }
                let pair = (Rc::as_ptr(dict_rc) as usize, Rc::as_ptr(other_rc) as usize);
                if self.hash_comparisons.contains(&pair) {
                    return Ok(Some(Object::Bool(true)));
                }
                self.hash_comparisons.push(pair);
                let answer = self.hash_entries_match(dict_rc, other_rc, method_name, position);
                self.hash_comparisons.pop();
                answer.map(|same| Some(Object::Bool(same)))
            }
            // `to_h` without a block answers the hash itself, and `to_hash`
            // always does.
            "to_hash" => Ok(Some(receiver.clone())),
            // Ruby recomputes every key's place, so a key whose `#hash`
            // changed since it was stored is found again and two keys that
            // have become equal collapse into the entry stored first.
            "rehash" => {
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
                let by_identity = dict_rc.borrow().contains_key(BY_IDENTITY_KEY);
                let held: Vec<(Object, Object)> = {
                    let dict = dict_rc.borrow();
                    dict.iter()
                        .filter(|(slot, _)| !is_internal_key(slot))
                        .map(|(slot, value)| (reconstruct_key(&dict, slot), value.clone()))
                        .collect()
                };
                let carried: Vec<(String, Object)> = {
                    let dict = dict_rc.borrow();
                    dict.iter()
                        .filter(|(slot, _)| is_internal_key(slot) && *slot != KEY_OBJECTS_KEY)
                        .map(|(slot, value)| (slot.clone(), value.clone()))
                        .collect()
                };
                let mut rebuilt = indexmap::IndexMap::new();
                for (key, value) in held {
                    let slot = self.dict_slot_in(&rebuilt, &key, by_identity, position)?;
                    if !rebuilt.contains_key(&slot) {
                        remember_key_object(&mut rebuilt, &slot, &key);
                    }
                    rebuilt.insert(slot, value);
                }
                for (slot, value) in carried {
                    rebuilt.insert(slot, value);
                }
                *dict_rc.borrow_mut() = rebuilt;
                Ok(Some(receiver.clone()))
            }
            "store" => {
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
                self.hash_store(dict_rc, &arguments[0], arguments[1].clone(), position)?;
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
            // `transform_keys` rebuilds the hash under new keys: a hash
            // argument names the replacement for the keys it holds, and a
            // block answers one for every key it does not.
            "transform_keys" | "transform_keys!" => {
                let in_place = method_name == "transform_keys!";
                let mapping = match arguments.first() {
                    None => None,
                    Some(Object::Dict(given)) => Some(Rc::clone(given)),
                    Some(other) => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!("no implicit conversion of {} into Hash", other.type_name()),
                            position,
                        ));
                    }
                };
                let has_block = matches!(self.pending_block, Some(Object::Block(_)));
                if in_place && (has_block || mapping.is_some()) && self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                if !has_block && mapping.is_none() {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let replacements: indexmap::IndexMap<String, Object> = match &mapping {
                    Some(given) => self
                        .hash_pairs(given)
                        .into_iter()
                        .map(|(key, value)| {
                            (
                                crate::vm::utils::object_to_dict_key(&key).unwrap_or_default(),
                                value,
                            )
                        })
                        .collect(),
                    None => indexmap::IndexMap::new(),
                };
                let mut built = indexmap::IndexMap::new();
                let mut broke_with = None;
                // The pair the block broke out of, plus the ones it never
                // reached. They keep the keys they already had.
                let mut left_as_written: Vec<(Object, Object)> = Vec::new();
                let pairs = self.hash_pairs(dict_rc);
                let mut walked = pairs.iter();
                for (key, value) in walked.by_ref() {
                    let (key, value) = (key.clone(), value.clone());
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    let new_key = match replacements.get(&rendered) {
                        Some(named) => named.clone(),
                        None => match &block {
                            Some(block) => {
                                match self.execute_block_callable(
                                    block,
                                    vec![key.clone()],
                                    position,
                                ) {
                                    Ok(answered) => answered,
                                    Err(MetorexError::BlockBreak {
                                        value: broke_value, ..
                                    }) => {
                                        broke_with = Some(broke_value);
                                        left_as_written.push((key, value));
                                        break;
                                    }
                                    Err(error) => return Err(error),
                                }
                            }
                            None => key.clone(),
                        },
                    };
                    keep_pair(&mut built, new_key, value);
                }
                // A `break` leaves the entries the block never reached under
                // the keys they already had, and what was transformed keeps
                // the key it was given.
                if broke_with.is_some() {
                    left_as_written.extend(walked.map(|(key, value)| (key.clone(), value.clone())));
                    for (key, value) in left_as_written {
                        let rendered =
                            crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                        if built.contains_key(&rendered) {
                            continue;
                        }
                        keep_pair(&mut built, key, value);
                    }
                }
                if !in_place {
                    // The copy is a plain Hash comparing keys by value,
                    // however the receiver compares its own.
                    return Ok(Some(Object::Dict(Rc::new(RefCell::new(built)))));
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
            // `transform_values` rebuilds the hash with what the block
            // answers for each value.
            "transform_values" => {
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
                // A hash with nothing in it spells the same in every
                // encoding, and one with something in it is written in the
                // encoding the first key was rendered in.
                let mut writing = "US-ASCII".to_string();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let rendered = self.render_pair(&key, &value, position);
                    match rendered {
                        Ok((rendered, held)) => {
                            if parts.is_empty() {
                                writing = held;
                            }
                            parts.push(rendered);
                        }
                        Err(error) => {
                            crate::object::end_rendering();
                            return Err(error);
                        }
                    }
                }
                crate::object::end_rendering();
                let made = crate::object::StringValue::with_encoding(
                    format!("{{{}}}", parts.join(", ")),
                    writing,
                );
                Ok(Some(Object::String(Rc::new(made))))
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
                self.hash_store(dict_rc, &arguments[0], arguments[1].clone(), position)?;
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
                    // An instance of a Hash subclass is read by the entries it
                    // holds, whatever `to_hash` it was given.
                    other if crate::vm::native_methods::hash_subclass_value(other).is_some() => {
                        crate::vm::native_methods::hash_subclass_value(other)
                            .expect("a hash subclass carries its entries")
                    }
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
            "transform_values!" => {
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

/// The object a hash keeps for a key it was handed. Ruby stores a String key
/// as a frozen copy, so a later write through the original text leaves the
/// hash alone; a key already frozen is kept as it stands.
pub(crate) fn stored_key_object(key: &Object) -> Object {
    let Object::String(text) = key else {
        return key.clone();
    };
    if text.is_frozen() {
        return key.clone();
    }
    let copy = Object::string(text.as_str().to_string());
    if let Object::String(copied) = &copy {
        copied.freeze();
    }
    copy
}

/// The prefix on every slot name a key's own `#hash` produced, chosen from
/// the control range so no rendered key can collide with one.
const HASHED_SLOT_PREFIX: char = '\u{1}';

/// A hash key that is not a primitive is recorded in the sentinel sub-map, so
/// the original object comes back when the hash is walked.
pub(crate) fn remember_key_object(
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
    /// The slot an object occupies in a hash. A key that carries its own
    /// `#hash` is placed by that number and then told apart from anything
    /// sharing it by `#eql?`, which is how Ruby decides whether two keys name
    /// the same entry. A hash comparing by identity places every key by its
    /// object id instead, so two equal strings stay two entries.
    pub(crate) fn dict_slot_in(
        &mut self,
        pairs: &indexmap::IndexMap<String, Object>,
        key: &Object,
        by_identity: bool,
        position: Position,
    ) -> Result<String, MetorexError> {
        if by_identity {
            let id = self.object_identity(key, position)?;
            return Ok(format!("{HASHED_SLOT_PREFIX}i{id}"));
        }
        if !self.key_hashes_for_itself(key) {
            return Ok(crate::vm::utils::object_to_dict_key(key).unwrap_or_default());
        }
        let hashed = match self.send_to_object(key.clone(), "hash", vec![], position)? {
            Object::Int(number) => number,
            other => self.object_identity(&other, position)?,
        };
        let mut slot = 0usize;
        loop {
            let candidate = format!("{HASHED_SLOT_PREFIX}h{hashed}#{slot}");
            if !pairs.contains_key(&candidate) {
                return Ok(candidate);
            }
            let stored = reconstruct_key(pairs, &candidate);
            // A key looks itself up without being asked, so a class whose
            // `#eql?` refuses its own object still finds its entry.
            if crate::vm::native_methods::object_methods::same_object(&stored, key) {
                return Ok(candidate);
            }
            let same = self
                .send_to_object(key.clone(), "eql?", vec![stored], position)?
                .is_truthy();
            if same {
                return Ok(candidate);
            }
            slot += 1;
        }
    }

    /// The slot an object occupies in a live hash, reading the hash's own
    /// identity setting.
    pub(crate) fn dict_slot_for(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        key: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let (pairs, by_identity) = {
            let dict = dict_rc.borrow();
            (dict.clone(), dict.contains_key(BY_IDENTITY_KEY))
        };
        self.dict_slot_in(&pairs, key, by_identity, position)
    }

    /// Whether a key answers `#hash` and `#eql?` of its own, which is what
    /// makes the bucket-and-compare placement the right one for it. Anything
    /// else is placed by how it renders, which is cheaper and keeps the
    /// primitives reading back from their slot names.
    fn key_hashes_for_itself(&mut self, key: &Object) -> bool {
        if matches!(key, Object::Instance(_)) {
            return true;
        }
        // Ruby places the immediates by value without asking them, so a
        // `TrueClass#hash` written in the program is never reached.
        if matches!(
            key,
            Object::Bool(_)
                | Object::Int(_)
                | Object::BigInt(_)
                | Object::Float(_)
                | Object::String(_)
                | Object::Symbol(_)
                | Object::Nil
        ) {
            return false;
        }
        matches!(self.lookup_method(key, "hash"), Some((_, method)) if !method.is_undefined)
    }

    /// The object id a hash comparing by identity places a key by.
    fn object_identity(&mut self, key: &Object, position: Position) -> Result<i64, MetorexError> {
        match self.send_to_object(key.clone(), "object_id", vec![], position)? {
            Object::Int(number) => Ok(number),
            _ => Ok(0),
        }
    }

    /// Whether two hashes hold the same entries. Each of the receiver's keys
    /// is looked up in the other hash the way any key is, and the values are
    /// compared the strict way for `eql?` and the ordinary way for `==`.
    fn hash_entries_match(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        other_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        method_name: &str,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let ours = self.hash_pairs(dict_rc);
        let theirs = self.hash_pairs(other_rc);
        if ours.len() != theirs.len() {
            return Ok(false);
        }
        let comparison = if method_name == "eql?" { "eql?" } else { "==" };
        for (key, value) in ours {
            let Some(slot) = self.hash_find_key(other_rc, &key, position)? else {
                return Ok(false);
            };
            let Some(held) = other_rc.borrow().get(&slot).cloned() else {
                return Ok(false);
            };
            let same = self
                .send_to_object(value, comparison, vec![held], position)?
                .is_truthy();
            if !same {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The key and value of every entry, with the sentinels left out.
    pub(crate) fn hash_pairs_for_digest(
        &self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
    ) -> Vec<(Object, Object)> {
        self.hash_pairs(dict_rc)
    }

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

    /// Store one entry, recording the key object beside it. A hash that
    /// already holds a matching key keeps the object it was given the first
    /// time, which is what `keys` reports afterwards.
    pub(crate) fn hash_store(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        key: &Object,
        value: Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let rendered = self.dict_slot_for(dict_rc, key, position)?;
        let mut dict = dict_rc.borrow_mut();
        let already_held = dict.contains_key(&rendered);
        if !already_held {
            // A hash comparing by identity keeps the key it was handed, since
            // a copy would be a different object and never found again.
            let stored = if rendered.starts_with("\u{1}i") {
                key.clone()
            } else {
                stored_key_object(key)
            };
            if !crate::vm::utils::is_primitive_key(key)
                || rendered.starts_with(HASHED_SLOT_PREFIX)
                || matches!(stored, Object::String(_))
            {
                remember_key_object(&mut dict, &rendered, &stored);
            }
        }
        dict.insert(rendered, value);
        Ok(())
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
    ) -> Result<(String, String), MetorexError> {
        let (rendered_value, _) = self.rendered_with_encoding(value, position)?;
        if let Object::Symbol(name) = key {
            let spelled = Object::String(Rc::clone(name));
            let (quoted, writing) = self.rendered_with_encoding(&spelled, position)?;
            // A symbol that is a plain name prints without quotes, as long as
            // the encoding the answer is written in has room for every
            // character of it.
            let plain =
                is_plain_symbol_name(&name.as_str()) && quoted == format!("\"{}\"", name.as_str());
            let shown = if plain {
                name.as_str().to_string()
            } else {
                quoted
            };
            return Ok((format!("{}: {}", shown, rendered_value), writing));
        }
        let (rendered_key, writing) = self.rendered_with_encoding(key, position)?;
        Ok((format!("{} => {}", rendered_key, rendered_value), writing))
    }

    /// What `inspect` answered for an object, and the encoding it wrote the
    /// answer in.
    fn rendered_with_encoding(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<(String, String), MetorexError> {
        let rendered = self.inspected_object(obj, position)?;
        Ok(match &rendered {
            Object::String(text) => (text.to_string(), text.encoding_name()),
            other => (format!("{}", other), "US-ASCII".to_string()),
        })
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
        Ok(self.hash_locate_key(dict_rc, wanted, position)?.1)
    }

    /// The slot a key would take, and the slot it already occupies when the
    /// hash holds it. Both come back from one walk so a key is asked for its
    /// `#hash` once per lookup.
    pub(crate) fn hash_locate_key(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        wanted: &Object,
        position: Position,
    ) -> Result<(String, Option<String>), MetorexError> {
        let slot = self.dict_slot_for(dict_rc, wanted, position)?;
        if dict_rc.borrow().contains_key(&slot) {
            return Ok((slot.clone(), Some(slot)));
        }
        // A hash built outside the program, such as the environment, holds
        // keys placed by how they render rather than by `#hash`, and the
        // environment places one by the text it names. Those are found by
        // walking the keys the hash recorded.
        let rendered = crate::vm::utils::object_to_dict_key(wanted).unwrap_or_default();
        if dict_rc.borrow().contains_key(&rendered) {
            return Ok((slot, Some(rendered)));
        }
        if crate::vm::utils::is_primitive_key(wanted) {
            return Ok((slot, None));
        }
        let stored: Vec<(String, Object)> = {
            let dict = dict_rc.borrow();
            dict.keys()
                .filter(|slot| !is_internal_key(slot) && !slot.starts_with(HASHED_SLOT_PREFIX))
                .map(|slot| (slot.clone(), reconstruct_key(&dict, slot)))
                .collect()
        };
        for (held, candidate) in stored {
            if crate::vm::native_methods::object_methods::same_object(&candidate, wanted) {
                return Ok((slot.clone(), Some(held)));
            }
            if crate::vm::utils::is_primitive_key(&candidate) {
                continue;
            }
            let same = self
                .send_to_object(wanted.clone(), "eql?", vec![candidate], position)?
                .is_truthy();
            if same {
                return Ok((slot.clone(), Some(held)));
            }
        }
        Ok((slot, None))
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
    /// Whether a hash is the one ENV names, which a few methods answer
    /// differently on.
    pub(crate) fn dict_is_environment(
        &self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
    ) -> bool {
        matches!(self.globals().get("ENV"), Some(Object::Dict(held)) if Rc::ptr_eq(&held, dict_rc))
    }

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
        let Ok(name) = std::ffi::CString::new(name.as_str().as_bytes().to_vec()) else {
            return;
        };
        // Ruby removes the entry outright when a name is set to nil, so the
        // name stops being one the environment holds at all.
        if matches!(value, Object::Nil) {
            let mut held = dict_rc.borrow_mut();
            if let Some(key_text) = crate::vm::utils::object_to_dict_key(key) {
                held.shift_remove(&key_text);
            }
        }
        unsafe {
            match value {
                Object::Nil => {
                    libc::unsetenv(name.as_ptr());
                }
                Object::String(text) => {
                    let Ok(setting) = std::ffi::CString::new(text.as_str().as_bytes().to_vec())
                    else {
                        return;
                    };
                    libc::setenv(name.as_ptr(), setting.as_ptr(), 1);
                }
                _ => {}
            }
        }
    }
}

/// Text tagged with an encoding, where one was named. A value of any other
/// kind is answered as it stands.
/// A String frozen where it stands, which is how the environment hands back
/// what it holds.
fn frozen_text(value: Object) -> Object {
    let Object::String(text) = &value else {
        return value;
    };
    let copy = crate::object::StringValue::with_encoding(text.to_text(), text.encoding_name());
    copy.freeze();
    Object::String(std::rc::Rc::new(copy))
}

fn retagged(value: Object, named: &Option<String>) -> Object {
    let (Object::String(text), Some(named)) = (&value, named) else {
        return value;
    };
    Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
        text.to_text(),
        named.clone(),
    )))
}

/// Whether a symbol is named plainly enough to print as a hash key without
/// quotes: a letter or underscore, then letters, digits and underscores, and
/// at most one `?` or `!` to close it.
fn is_plain_symbol_name(name: &str) -> bool {
    let body = name.strip_suffix(['?', '!']).unwrap_or(name);
    let mut letters = body.chars();
    letters
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_')
        && letters.all(|letter| letter.is_alphanumeric() || letter == '_')
}
