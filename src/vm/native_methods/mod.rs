//! Native (built-in) method implementations for the virtual machine.
//!
//! This module contains the implementations of all built-in methods for
//! standard classes like Object, String, and Array.

pub(crate) mod array_methods;
pub(crate) mod ast_methods;
pub(crate) mod class_methods;
pub(crate) use class_methods::MODULE_FUNCTION_VISIBILITY;
pub(crate) use class_methods::is_native_kernel_method;
pub(crate) use class_methods::native_module_method_stub;
pub(crate) use hash_methods::remember_key_object;
pub(crate) use method_object_methods::{block_parameter_list, method_parameter_list};
pub(crate) use module_methods::{REFINEMENT_KEY_PREFIX, REFINEMENT_LABEL_KEY};
mod binding_methods;
mod socket_addresses;
pub(crate) use socket_addresses::OpenSockets;
pub(crate) use streams::OpenStreams;
mod complex_methods;
mod constant_visibility;
pub(crate) mod define_method;
mod digest_algorithms;
mod etc_methods;
mod exception_methods;
mod file_methods;
mod float_methods;
pub(crate) mod hash_methods;
mod int_methods;
mod io_methods;
pub(crate) mod kernel_conversion;
pub(crate) mod method_object_methods;
mod module_methods;
pub(crate) mod object_methods;
mod range_methods;
pub(crate) mod rational_methods;
mod syslog_write;
mod zlib_deflate;
mod zlib_streams;
pub(crate) use object_methods::binary_op_for_method_name;
pub(crate) use rational_methods::{complex_parts, rational_parts};
pub(crate) mod regexp_methods;
pub(crate) use regexp_methods::{
    LAST_MATCH, capture_reference, comparable_flags, compile, subject_text,
};
pub(crate) mod euc_jp_table;
pub(crate) mod glob;
pub(crate) mod normalization_table;
mod set_methods;
pub(crate) mod shift_jis_table;
pub(crate) mod string_methods;
pub(crate) mod string_mutation;
mod string_sets;
pub(crate) mod struct_methods;
mod time_methods;
pub(crate) use struct_methods::struct_members;

/// Instance variable a String subclass keeps its characters in.
pub(crate) const STRING_SUBCLASS_VAR: &str = "__string__";
/// Instance variable an instance of an Array subclass stores its elements in,
/// since a plain Array is a primitive rather than an instance.
pub(crate) const ARRAY_SUBCLASS_VAR: &str = "__array__";
/// Instance variable an instance of a Set subclass stores its elements in,
/// since a plain Set is a primitive rather than an instance.
pub(crate) const SET_SUBCLASS_VAR: &str = "__set__";
/// The instance variable an instance of a Proc subclass holds its block in.
pub(crate) const PROC_SUBCLASS_VAR: &str = "__proc__";
/// Instance variable an instance of a Hash subclass stores its entries in,
/// since a plain Hash is a primitive rather than an instance.
pub(crate) const HASH_SUBCLASS_VAR: &str = "__hash__";
/// Instance variable an instance of a Range subclass stores its ends in,
/// since a plain Range is a primitive rather than an instance.
pub(crate) const RANGE_SUBCLASS_VAR: &str = "__range__";

/// The backing range an instance of a Range subclass holds, or None when
/// `receiver` is not one.
pub(crate) fn range_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(RANGE_SUBCLASS_VAR)
        .cloned()
}

/// The value read as a Range: a Range itself, or the one behind an instance
/// of a Range subclass, which is what lets a subclass stand wherever a range
/// is expected.
pub(crate) fn as_range(value: &Object) -> Option<Object> {
    if matches!(value, Object::Range { .. }) {
        return Some(value.clone());
    }
    match range_subclass_value(value) {
        Some(held @ Object::Range { .. }) => Some(held),
        _ => None,
    }
}

/// The name a Regexp subclass instance keeps its pattern under.
pub(crate) const REGEXP_SUBCLASS_VAR: &str = "__regexp__";

/// The pattern behind an instance of a Regexp subclass.
pub(crate) fn regexp_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(REGEXP_SUBCLASS_VAR)
        .cloned()
}

/// The characters behind an instance of a String subclass.
pub(crate) fn string_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(STRING_SUBCLASS_VAR)
        .cloned()
}
/// The backing array an instance of an Array subclass holds, or None when
/// `receiver` is not one. The Rc is shared, so a mutation through it is
/// visible to the instance.
pub(crate) fn array_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(ARRAY_SUBCLASS_VAR)
        .cloned()
}
/// The backing set an instance of a Set subclass holds, or None when
/// `receiver` is not one. The Rc is shared, so a change through it is visible
/// to the instance.
/// The block an instance of a Proc subclass stands for, or None when
/// `receiver` is not one.
pub(crate) fn proc_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(PROC_SUBCLASS_VAR)
        .cloned()
}

pub(crate) fn set_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(SET_SUBCLASS_VAR)
        .cloned()
}
/// The backing hash an instance of a Hash subclass holds, or None when
/// `receiver` is not one. The Rc is shared, so a change through it is visible
/// to the instance.
pub(crate) fn hash_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(HASH_SUBCLASS_VAR)
        .cloned()
}
/// The dictionary behind a value: a Hash itself, or the one an instance of a
/// Hash subclass keeps.
pub(crate) fn as_dict(value: &Object) -> Option<Object> {
    if matches!(value, Object::Dict(_)) {
        return Some(value.clone());
    }
    match hash_subclass_value(value) {
        Some(held @ Object::Dict(_)) => Some(held),
        _ => None,
    }
}

pub(crate) mod pack_format;
mod streams;
mod visibility;

use super::VirtualMachine;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Attempt to execute a native (built-in) method implementation.
    ///
    /// Returns `Ok(Some(result))` if a native method was found and executed successfully,
    /// `Ok(None)` if no native method exists (allowing fallback to user-defined methods),
    /// or `Err` if the method call failed.
    pub(crate) fn call_native_method(
        &mut self,
        class: &Class,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Object::Binding(binding) = receiver
            && let Some(answered) =
                self.call_binding_methods(binding, method_name, arguments, position)?
        {
            return Ok(Some(answered));
        }

        // `define_singleton_method` works on any receiver, so it is handled
        // before the per-type tables.
        if method_name == "define_singleton_method" {
            return self
                .object_define_singleton_method(receiver, arguments, position)
                .map(Some);
        }

        // Constant visibility and deprecation apply to any class or module
        // receiver, so they are handled before the per-type tables.
        if let Some(result) =
            self.call_constant_visibility_methods(receiver, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // An instance of a Proc subclass answers a callable's methods,
        // applied to the block it stands for.
        if !matches!(
            method_name,
            "class" | "instance_variables" | "is_a?" | "kind_of?"
        ) && let Some(backing @ Object::Block(_)) = proc_subclass_value(receiver)
            && let Some(result) =
                self.call_native_method(class, &backing, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // Block/Lambda methods
        if let Object::Block(block) = receiver {
            match method_name {
                "call" | "[]" | "===" | "yield" => {
                    return Ok(Some(block.call(self, arguments.to_vec(), position)?));
                }
                // A callable is already the block it stands for.
                "to_proc" => return Ok(Some(receiver.clone())),
                "binding" => {
                    // A curried callable is built by the library rather than
                    // written in the program, so there is no scope behind it.
                    if self.carried_variable(receiver, "__curried").is_some() {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "Can't create Binding from C level Proc",
                            position,
                        ));
                    }
                    use crate::object::Binding;
                    let binding = Binding::new(block.captured_vars().clone());
                    return Ok(Some(Object::Binding(Rc::new(binding))));
                }
                _ => {}
            }
        }

        // Module-specific methods (refine, module_eval, stdlib stubs). Falls
        // through to call_class_methods afterwards so Class/Module share the
        // same table for things like `name`, `extend`, `remove_const`, etc.
        if let Object::Module(module_rc) = receiver {
            if let Some(result) =
                self.call_module_methods(module_rc, receiver, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_class_methods(module_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }

        // Class-specific methods (File/Dir dispatch first, then general class methods)
        if let Object::Class(class_rc) = receiver {
            if let Some(result) =
                self.call_struct_class_methods(class_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_file_dir_methods(class_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_io_class_method(class_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_complex_class_method(class_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_class_methods(class_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }

        // An instance of a Module subclass is a module: give it the Module
        // method table, backed by its singleton class so constants and
        // methods defined on it are stored and read back per object.
        if let Object::Instance(instance) = receiver
            && instance.borrow().class.find_method(method_name).is_none()
            && self.is_module_subclass_instance(receiver)
        {
            let backing = self.singleton_class_of(receiver);
            if let Some(result) =
                self.call_class_methods(&backing, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }

        // Method/Block object introspection
        if let Some(result) =
            self.call_method_object_methods(receiver, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // An instance of a String subclass answers String's methods, backed
        // by the characters it was built with.
        if let Some(text) = string_subclass_value(receiver) {
            // `to_s` and `to_str` answer the characters themselves; String
            // implements neither natively because a String already is one.
            if matches!(method_name, "to_s" | "to_str") {
                return Ok(Some(text));
            }
            // A copy of a subclass instance is one of the same class, which
            // the general rules make rather than the characters behind it.
            if matches!(method_name, "clone" | "dup") {
                return self.call_object_method(receiver, method_name, arguments, position);
            }
            // A method that changes the string reaches the characters the
            // instance is backed by, and answers the instance itself.
            if let Some(result) =
                self.call_string_mutation(&text, method_name, arguments, position)?
            {
                if let (Object::String(answered), Object::String(backing)) = (&result, &text)
                    && Rc::ptr_eq(answered, backing)
                {
                    return Ok(Some(receiver.clone()));
                }
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_string_method(&text, method_name, arguments, position)?
            {
                // A method that answers the string it was called on answers
                // the subclass instance, not the characters behind it.
                if let (Object::String(answered), Object::String(backing)) = (&result, &text)
                    && Rc::ptr_eq(answered, backing)
                {
                    return Ok(Some(receiver.clone()));
                }
                return Ok(Some(result));
            }
        }

        // An instance of an Array subclass answers Array's methods, backed
        // by the elements it holds.
        if let Some(elements) = array_subclass_value(receiver) {
            // Array implements neither `to_a` nor `to_ary` natively, because
            // an Array already is one. From a subclass they answer a plain
            // Array and the instance itself.
            match method_name {
                "to_a" | "entries" => {
                    let Object::Array(storage) = &elements else {
                        return Ok(None);
                    };
                    let copied = storage.borrow().clone();
                    return Ok(Some(Object::array(copied)));
                }
                "to_ary" => return Ok(Some(receiver.clone())),
                _ => {}
            }
            if let Some(result) =
                self.call_array_method(&elements, method_name, arguments, position)?
            {
                // A copy of a subclass instance is one of the same class,
                // which is what `clone` and `dup` answer in Ruby.
                if matches!(method_name, "clone" | "dup")
                    && let Object::Array(_) = &result
                    && let Object::Instance(instance) = receiver
                {
                    let class = Rc::clone(&instance.borrow().class);
                    let mut made = crate::object::Instance::new(class);
                    made.set_var(ARRAY_SUBCLASS_VAR.to_string(), result);
                    return Ok(Some(Object::Instance(Rc::new(std::cell::RefCell::new(
                        made,
                    )))));
                }
                return Ok(Some(result));
            }
        }

        // An instance of a Range subclass answers Range's methods, backed by
        // the ends it was built with.
        if let Some(ends) = range_subclass_value(receiver)
            && let Some(result) = self.call_range_method(&ends, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // An instance of a Hash subclass answers Hash's methods, backed by
        // the entries it holds.
        if let Some(entries) = hash_subclass_value(receiver) {
            // Hash implements `to_hash` nowhere else, because a Hash already
            // is one, and from a subclass it answers the instance itself.
            if method_name == "to_hash" {
                return Ok(Some(receiver.clone()));
            }
            // `to_h` answers a plain Hash, which is what a subclass instance
            // converts into.
            if method_name == "to_h" && arguments.is_empty() && self.pending_block.is_none() {
                let Object::Dict(stored) = &entries else {
                    return Ok(None);
                };
                let copied = stored.borrow().clone();
                return Ok(Some(Object::Dict(Rc::new(std::cell::RefCell::new(copied)))));
            }
            // A subclass that writes its own `each` is what Enumerable is
            // written over, so the walking methods go through that body
            // rather than through the native table.
            if let Object::Instance(instance) = receiver
                && enumerable_walks_through_each(method_name)
                && instance
                    .borrow()
                    .class
                    .find_own_method("each")
                    .is_some_and(|held| !held.body.is_empty())
            {
                return Ok(None);
            }
            // A subclass that writes its own `default` decides what a missing
            // key reads as, and the backing hash knows nothing of it.
            if method_name == "[]"
                && arguments.len() == 1
                && let Object::Dict(dict_rc) = &entries
                && let Some((owner, method)) = self.lookup_method(receiver, "default")
                && !method.is_undefined
                && self
                    .hash_find_key(dict_rc, &arguments[0], position)?
                    .is_none()
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
            if let Some(result) =
                self.call_hash_method(&entries, method_name, arguments, position)?
            {
                // `merge` answers a hash of the receiver's own class, where
                // the rest of the table answers a plain one.
                if method_name == "merge"
                    && let Object::Dict(_) = &result
                    && let Object::Instance(instance) = receiver
                {
                    let class = std::rc::Rc::clone(&instance.borrow().class);
                    let mut made = crate::object::Instance::new(class);
                    made.set_var(HASH_SUBCLASS_VAR.to_string(), result);
                    return Ok(Some(Object::Instance(Rc::new(std::cell::RefCell::new(
                        made,
                    )))));
                }
                return Ok(Some(result));
            }
        }

        // An instance of a Set subclass answers Set's methods, applied to the
        // set it is backed by.
        if let Some(backing @ Object::Set(_)) = set_subclass_value(receiver)
            && let Some(result) =
                self.call_set_method(&backing, method_name, arguments, position)?
        {
            // A method that answers the set itself answers the instance.
            return Ok(Some(match result {
                Object::Set(inner) if matches!(&backing, Object::Set(outer) if Rc::ptr_eq(&inner, outer)) => {
                    receiver.clone()
                }
                other => other,
            }));
        }

        // A pattern written as a literal is frozen where it stands. One
        // built by `Regexp.new` is not, so a program may still change it.
        if method_name == "frozen?"
            && let Object::Regex(pattern, _) = receiver
        {
            let built = self
                .built_patterns
                .contains(&(Rc::as_ptr(pattern) as *const _ as usize));
            return Ok(Some(Object::Bool(!built)));
        }
        // The encoding a pattern is read in is settled from the pattern's own
        // address, which is where the encoding of the string it was built
        // from was recorded.
        // The source a pattern was written from is tagged with the encoding
        // the pattern is read in.
        if method_name == "source"
            && let Object::Regex(pattern, flags) = receiver
            && arguments.is_empty()
        {
            let named = self.pattern_encoding_name(pattern, flags);
            let made = crate::object::StringValue::with_encoding(pattern.to_string(), named);
            return Ok(Some(Object::String(Rc::new(made))));
        }
        if method_name == "encoding"
            && let Object::Regex(pattern, flags) = receiver
        {
            if !arguments.is_empty() {
                return Err(crate::vm::errors::method_argument_error(
                    method_name,
                    0,
                    arguments.len(),
                    position,
                ));
            }
            let named = self.pattern_encoding_name(pattern, flags);
            return Ok(Some(self.encoding_object(&named)));
        }
        // A Regexp is a primitive rather than an instance, so its methods are
        // dispatched before the class-name table below.
        if let Object::Regex(pattern, flags) = receiver
            && let Some(result) =
                self.call_regexp_method(pattern, flags, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // An instance of a Regexp subclass answers Regexp's methods, backed
        // by the pattern it was built with.
        if let Some(Object::Regex(pattern, flags)) = regexp_subclass_value(receiver) {
            if matches!(method_name, "clone" | "dup") {
                return self.call_object_method(receiver, method_name, arguments, position);
            }
            if let Some(result) =
                self.call_regexp_method(&pattern, &flags, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }

        // Instances of a generated struct class get Struct's instance methods.
        if let Object::Instance(instance) = receiver {
            let instance_class = Rc::clone(&instance.borrow().class);
            if let Some(members) = struct_methods::struct_members(&instance_class)
                && let Some(result) = self.call_struct_instance_method(
                    &instance_class,
                    &members,
                    receiver,
                    method_name,
                    arguments,
                    position,
                )?
            {
                return Ok(Some(result));
            }
        }

        // A fiber carries the number naming its coroutine, so an instance of
        // a subclass answers the same methods as one of Fiber itself.
        if let Object::Instance(instance) = receiver
            && instance.borrow().get_var("__fiber__").is_some()
            && let Some(result) =
                self.call_fiber_method(receiver, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // A thread carries the block it runs, so an instance of a subclass
        // answers the same methods as one of Thread itself.
        if let Object::Instance(instance) = receiver
            && instance.borrow().get_var("__thread_block").is_some()
            && class.name() != "Thread"
            && let Some(result) =
                self.call_thread_method(receiver, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // Dispatch to the appropriate class-specific method implementation
        match class.name() {
            "Object" | "Proc" | "Method" => {
                self.call_object_method(receiver, method_name, arguments, position)
            }
            "String" => {
                if let Some(answer) =
                    self.call_string_mutation(receiver, method_name, arguments, position)?
                {
                    return Ok(Some(answer));
                }
                let answer = self.call_string_method(receiver, method_name, arguments, position)?;
                Ok(carry_string_encoding(receiver, method_name, answer))
            }
            // Symbol shares String's character-level methods (`length`,
            // `upcase`, `start_with?`, comparison). Object's come first so
            // `class`, `inspect`, and friends report Symbol.
            "Symbol" => {
                // The names a symbol answers for itself come first, since
                // Object answers `to_s` for anything at all.
                let Object::Symbol(text) = receiver else {
                    return Ok(None);
                };
                // The names a Symbol answers for itself: `id2name` and `name`
                // give its characters, and `intern` and `to_sym` give it back.
                // A symbol named in ASCII is written in ASCII, and one with
                // any other character is written in UTF-8.
                let named_in = if text.as_str().is_ascii() {
                    "US-ASCII".to_string()
                } else {
                    text.encoding_name()
                };
                match method_name {
                    // `name` answers one frozen string per symbol, where
                    // `id2name` hands back a fresh one each time.
                    "name" => {
                        let slot = format!("__symbol_name_{}", text.as_str());
                        let held = self.memoized_text(&slot, &text.as_str());
                        if let Object::String(made) = &held {
                            made.set_encoding(named_in.clone());
                            made.freeze();
                        }
                        return Ok(Some(held));
                    }
                    "id2name" | "to_s" => {
                        let spelled = crate::object::StringValue::with_encoding(
                            text.to_text(),
                            named_in.clone(),
                        );
                        // A name standing for the bytes an encoding spells it
                        // with keeps standing for them.
                        if text.holds_bytes() {
                            spelled.mark_bytes();
                        }
                        // Ruby 3.4 hands this string back with notice that a
                        // later release will freeze it, so the first change
                        // made to it says so.
                        spelled.chill(format!(
                            "warning: string returned by :{}.to_s will be frozen in the future",
                            text.as_str()
                        ));
                        return Ok(Some(Object::String(Rc::new(spelled))));
                    }
                    "encoding" => {
                        let named = Object::String(Rc::new(
                            crate::object::StringValue::with_encoding(text.to_text(), named_in),
                        ));
                        return self.call_string_method(&named, method_name, arguments, position);
                    }
                    "intern" | "to_sym" => return Ok(Some(receiver.clone())),
                    // A Symbol compares its case only against another Symbol,
                    // where a String compares against anything that reads as
                    // one.
                    "casecmp" | "casecmp?" if arguments.len() == 1 => {
                        let Object::Symbol(other) = &arguments[0] else {
                            return Ok(Some(Object::Nil));
                        };
                        let left = Object::String(Rc::clone(text));
                        let right = Object::String(Rc::clone(other));
                        return self.call_string_method(&left, method_name, &[right], position);
                    }
                    _ => {}
                }
                if let Some(result) =
                    self.call_object_method(receiver, method_name, arguments, position)?
                {
                    return Ok(Some(result));
                }
                let as_string = Object::String(Rc::clone(text));
                let answered =
                    self.call_string_method(&as_string, method_name, arguments, position)?;
                // The methods that answer a name answer a Symbol from a
                // Symbol, where String's own answer a String.
                let answers_a_symbol = matches!(
                    method_name,
                    "upcase" | "downcase" | "capitalize" | "swapcase" | "succ" | "next"
                );
                Ok(match answered {
                    Some(Object::String(text)) if answers_a_symbol => Some(Object::Symbol(text)),
                    other => other,
                })
            }
            "Integer" => self.call_int_method(receiver, method_name, arguments, position),
            "Array" => self.call_array_method(receiver, method_name, arguments, position),
            "Hash" => self.call_hash_method(receiver, method_name, arguments, position),
            "Float" => self.call_float_method(receiver, method_name, arguments, position),
            "Range" => self.call_range_method(receiver, method_name, arguments, position),
            "Rational" => self.call_rational_method(receiver, method_name, arguments, position),
            "Complex" => self.call_complex_method(receiver, method_name, arguments, position),
            "Set" => self.call_set_method(receiver, method_name, arguments, position),
            "Exception" => self.call_exception_method(receiver, method_name, arguments, position),
            "Thread" => self.call_thread_method(receiver, method_name, arguments, position),
            "Fiber" => self.call_fiber_method(receiver, method_name, arguments, position),
            "Queue" | "SizedQueue" => {
                self.call_queue_method(receiver, method_name, arguments, position)
            }
            "Mutex" => self.call_mutex_method(receiver, method_name, arguments, position),
            "ConditionVariable" => {
                self.call_condition_variable_method(receiver, method_name, arguments, position)
            }
            "IO" => self.call_io_handle_method(receiver, method_name, arguments, position),
            "Process::Status" => {
                self.call_process_status_method(receiver, method_name, arguments, position)
            }
            _ => Ok(None),
        }
    }

    /// Emit a warning line. If `$stderr` has been reassigned to an object that
    /// responds to `write` / `<<` (e.g. mspec's `IOStub` for the `complain`
    /// matcher), route the message there so tests can capture it. Otherwise
    /// fall back to writing the line directly to the process's stderr.
    pub(crate) fn emit_warning_to_stderr(&mut self, msg: &str, position: Position) {
        // Re-running an already-required file to satisfy an autoload repeats
        // assignments Ruby would have run once, so the warnings they produce
        // describe the re-run rather than the program.
        if self.autoload_reload_depth > 0 {
            return;
        }
        let stderr_obj = self.globals().get("stderr");
        let placeholder = matches!(
            &stderr_obj,
            Some(Object::String(s)) if *s.as_str() == *"$stderr"
        );
        if !placeholder && let Some(obj) = stderr_obj {
            let line = format!("{}\n", msg);
            for cand in ["write", "<<"] {
                if let Some((cls, method)) = self.lookup_method(&obj, cand) {
                    let arg = Object::string(line.clone());
                    let _ = self.invoke_method(cls, method, obj.clone(), vec![arg], position);
                    return;
                }
            }
        }
        eprintln!("{}", msg);
    }

    /// Coerce an object into a method-name `String`. Strings and symbols are
    /// taken at face value; for other receivers we invoke `to_str` (matching
    /// Ruby's implicit type coercion). A receiver that lacks `to_str` raises
    /// `TypeError`; a `to_str` that returns a non-String also raises
    /// `TypeError`. Errors raised inside `to_str` (e.g. `NoMethodError`)
    /// propagate unchanged so callers see the original error class.
    pub(crate) fn coerce_method_name(
        &mut self,
        arg: &Object,
        caller: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        match arg {
            Object::String(s) => Ok(name_text(s)),
            Object::Symbol(s) => Ok(s.as_str().to_string()),
            _ => {
                if let Some((cls, m)) = self.lookup_method(arg, "to_str")
                    && !m.is_undefined
                {
                    let result = self.invoke_method(cls, m, arg.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.as_str().to_string());
                    }
                    let source_class = self.builtins().class_of(arg).name().to_string();
                    let msg = format!("can't convert {} into String", source_class);
                    let _ = result;
                    let exc = Object::exception("TypeError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: crate::vm::utils::position_to_location(position),
                        message: msg,
                    });
                }
                let msg = format!(
                    "{} is not a symbol nor a string",
                    match arg {
                        Object::Instance(inst) => format!("#<{}>", inst.borrow().class.name()),
                        _ => arg.to_string(),
                    }
                );
                let _ = caller;
                let exc = Object::exception("TypeError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: crate::vm::utils::position_to_location(position),
                    message: msg,
                })
            }
        }
    }

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

    /// The methods a fiber answers. The coroutine behind one lives in the
    /// interpreter, and the object carries the number naming it.
    pub(crate) fn call_fiber_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(inst) = receiver else {
            return Ok(None);
        };
        let handle = match inst.borrow().get_var("__fiber__") {
            Some(Object::Int(number)) => *number as usize,
            _ => {
                return Err(crate::vm::errors::simple_exception(
                    "FiberError",
                    "uninitialized fiber",
                    position,
                ));
            }
        };
        match method_name {
            "resume" => {
                let stepped =
                    self.fiber_resume(handle, receiver.clone(), arguments.to_vec(), position)?;
                match stepped {
                    crate::vm::fibers::FiberStep::Suspended(handed) => Ok(Some(match handed {
                        crate::vm::fibers::SuspendOutput::Yielded(value) => value,
                        crate::vm::fibers::SuspendOutput::TransferTo { .. } => Object::Nil,
                    })),
                    crate::vm::fibers::FiberStep::Finished(value) => Ok(Some(value)),
                    crate::vm::fibers::FiberStep::Failed(trouble) => Err(trouble),
                }
            }
            "transfer" => {
                // Transferring to the fiber already holding the interpreter
                // leaves it where it is. The root fiber is what a program
                // transfers back to when it is finished with another.
                if handle == crate::vm::fibers::ROOT_FIBER && self.fiber_frames.is_empty() {
                    return Ok(Some(Object::Nil));
                }
                let stepped =
                    self.fiber_transfer(handle, receiver.clone(), arguments.to_vec(), position)?;
                match stepped {
                    crate::vm::fibers::FiberStep::Suspended(handed) => Ok(Some(match handed {
                        crate::vm::fibers::SuspendOutput::Yielded(value) => value,
                        crate::vm::fibers::SuspendOutput::TransferTo { .. } => Object::Nil,
                    })),
                    crate::vm::fibers::FiberStep::Finished(value) => Ok(Some(value)),
                    crate::vm::fibers::FiberStep::Failed(trouble) => Err(trouble),
                }
            }
            "kill" => {
                self.fiber_kill(handle, position);
                Ok(Some(Object::Nil))
            }
            "raise" => {
                let raised = self.build_raise_exception(arguments, position)?;
                let stepped = self.fiber_raise(handle, receiver.clone(), raised, position)?;
                match stepped {
                    crate::vm::fibers::FiberStep::Suspended(handed) => Ok(Some(match handed {
                        crate::vm::fibers::SuspendOutput::Yielded(value) => value,
                        crate::vm::fibers::SuspendOutput::TransferTo { .. } => Object::Nil,
                    })),
                    crate::vm::fibers::FiberStep::Finished(value) => Ok(Some(value)),
                    crate::vm::fibers::FiberStep::Failed(trouble) => Err(trouble),
                }
            }
            "blocking?" => Ok(Some(Object::Bool(self.fiber_is_blocking(handle)))),
            // The names a fiber keeps are its own, so only the fiber running
            // may read or replace them.
            "storage" => {
                if handle != self.fiber_current_handle() {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "Fiber storage can only be accessed from the Fiber it belongs to",
                        position,
                    ));
                }
                Ok(Some(self.fiber_storage(handle)))
            }
            "storage=" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::Nil => self.fiber_set_storage(handle, None),
                    held => {
                        self.check_fiber_storage(held, position)?;
                        self.fiber_set_storage(handle, Some(held.clone()));
                    }
                }
                Ok(Some(arguments[0].clone()))
            }
            "alive?" => Ok(Some(Object::Bool(self.fiber_is_alive(handle)))),
            "inspect" | "to_s" => {
                let address = Rc::as_ptr(inst) as usize;
                let status = self.fiber_status(handle);
                let named = self.builtins().class_of(receiver).name().to_string();
                let written = match self.fiber_source(handle) {
                    Some((file, line)) => {
                        format!("#<{named}:0x{address:016x} {file}:{line} ({status})>")
                    }
                    None => format!("#<{named}:0x{address:016x} ({status})>"),
                };
                Ok(Some(Object::string(written)))
            }
            _ => Ok(None),
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
                    let mut here = crate::object::Instance::new(self.backtrace_location_class());
                    for named in ["lineno", "path", "absolute_path"] {
                        let held = innermost.borrow().get_var(named).cloned();
                        if let Some(value) = held {
                            here.set_var(named.to_string(), value);
                        }
                    }
                    here.set_var(
                        "label".to_string(),
                        Object::string("Thread#backtrace_locations"),
                    );
                    places.insert(0, Object::Instance(Rc::new(std::cell::RefCell::new(here))));
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
            "raise" => {
                // `raise` with nothing named raises a RuntimeError, and the
                // thread has to be handed something for it to take.
                let handed = if arguments.is_empty() {
                    vec![Object::string("unhandled exception".to_string())]
                } else {
                    arguments.to_vec()
                };
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

    /// Mutex instance methods. Metorex runs a thread's block on the thread
    /// that made it, so a lock never has to wait: it records who holds it and
    /// refuses a second taking, which is what the visible behavior rests on.
    pub(crate) fn call_mutex_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        _arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(inst) = receiver else {
            return Ok(None);
        };
        let inst = Rc::clone(inst);
        let held = matches!(
            inst.borrow().get_var("__mutex_locked"),
            Some(Object::Bool(true))
        );
        match method_name {
            "synchronize" => {
                let block = self.pending_block.take();
                let Some(Object::Block(body)) = block else {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "Mutex#synchronize requires a block",
                        position,
                    ));
                };
                self.call_mutex_method(receiver, "lock", &[], position)?;
                let answer = self.execute_block_body(&body, vec![]);
                self.call_mutex_method(receiver, "unlock", &[], position)?;
                Ok(Some(answer?))
            }
            "lock" => {
                if held && self.mutex_is_owned(&inst) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "deadlock; recursive locking",
                        position,
                    ));
                }
                // A lock something else holds is waited for, so whatever else
                // the program has to run gets a turn until it is let go.
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
                while matches!(
                    inst.borrow().get_var("__mutex_locked"),
                    Some(Object::Bool(true))
                ) && std::time::Instant::now() < deadline
                {
                    self.wait_for_other_threads(position);
                }
                self.mark_mutex_held(&inst);
                // A thread told to stop while it waited takes the lock first,
                // so whatever unwinds next still holds it.
                self.raise_if_thread_killed(position)?;
                Ok(Some(receiver.clone()))
            }
            "try_lock" => {
                if held {
                    return Ok(Some(Object::Bool(false)));
                }
                self.mark_mutex_held(&inst);
                Ok(Some(Object::Bool(true)))
            }
            "unlock" => {
                if !held {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "Attempt to unlock a mutex which is not locked",
                        position,
                    ));
                }
                if !self.mutex_is_owned(&inst) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "Attempt to unlock a mutex which is locked by another thread",
                        position,
                    ));
                }
                inst.borrow_mut()
                    .set_var("__mutex_locked".to_string(), Object::Bool(false));
                Ok(Some(receiver.clone()))
            }
            // `Mutex#sleep` lets the lock go, waits to be woken, and takes
            // the lock again, which is what a condition variable waits on.
            "sleep" => {
                let wanted = match _arguments.first() {
                    None | Some(Object::Nil) => None,
                    Some(given) => Some(self.float_value_of(given, position)?),
                };
                if wanted.is_some_and(|seconds| seconds < 0.0) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "time interval must not be negative",
                        position,
                    ));
                }
                if !held || !self.mutex_is_owned(&inst) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "Attempt to unlock a mutex which is not locked",
                        position,
                    ));
                }
                let started = std::time::Instant::now();
                self.call_mutex_method(receiver, "unlock", &[], position)?;
                // The lock is taken again before anything that interrupted
                // the sleep is raised, so an `ensure` around the sleep sees
                // the thread still holding it.
                let stopped = match wanted {
                    None => self.sleep_until_woken(position),
                    Some(_) => {
                        self.wait_for_other_threads(position);
                        Ok(())
                    }
                };
                self.call_mutex_method(receiver, "lock", &[], position)?;
                if let Some(handed) = self.exception_handed_to_thread() {
                    self.call_native_function("raise", handed, position)?;
                }
                self.raise_if_thread_killed(position)?;
                stopped?;
                Ok(Some(Object::Int(started.elapsed().as_secs() as i64)))
            }
            "locked?" => Ok(Some(Object::Bool(held))),
            "owned?" => Ok(Some(Object::Bool(held && self.mutex_is_owned(&inst)))),
            _ => Ok(None),
        }
    }

    /// The fiber a lock is held under. A thread's own body is its root fiber
    /// rather than one the program made, so a lock taken there is the
    /// thread's rather than a fiber's.
    fn lock_holding_fiber(&self) -> i64 {
        let held = self.fiber_current_handle();
        if self.thread_body_fibers.contains(&held) {
            return -1;
        }
        held as i64
    }

    /// Record the thread and fiber running now as the ones holding the lock.
    fn mark_mutex_held(&mut self, inst: &Rc<std::cell::RefCell<crate::object::Instance>>) {
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
    fn mutex_is_owned(&mut self, inst: &Rc<std::cell::RefCell<crate::object::Instance>>) -> bool {
        let fiber = self.lock_holding_fiber();
        if !matches!(inst.borrow().get_var("__mutex_fiber"), Some(Object::Int(held)) if *held == fiber)
        {
            return false;
        }
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
        let main = Object::Instance(Rc::new(std::cell::RefCell::new(made)));
        self.globals_mut().set("__Thread_main", main.clone());
        main
    }

    /// What `Thread#raise` handed the thread running now, taken so it is
    /// raised once and no more.
    fn exception_handed_to_thread(&mut self) -> Option<Vec<Object>> {
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

/// How Ruby names a value that cannot stand in for a number: true, false and
/// nil by their own spelling, and everything else by its class.
fn unconvertible_wording(value: &Object, named: &str) -> String {
    match value {
        Object::Bool(true) => "true".to_string(),
        Object::Bool(false) => "false".to_string(),
        Object::Nil => "nil".to_string(),
        _ => named.to_string(),
    }
}

/// The range Ruby keeps a thread's priority inside.
const PRIORITY_FLOOR: i64 = -3;
const PRIORITY_CEILING: i64 = 3;

/// Whether two objects are the same Thread.
fn same_thread(one: &Object, other: &Object) -> bool {
    matches!((one, other), (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b))
}

/// Whether `name` is a syntactically valid Ruby constant name: must start
/// with an uppercase letter and contain only word characters after.
/// Multibyte letters are allowed (Ruby permits `CS_CONSTλ`). Used by
/// `Module#autoload`, `Module#const_set`, and `Module#const_defined?` to
/// reject lowercase / numeric / punctuated names with a NameError.
/// The text a name is spelled with. A name carried as the bytes of an
/// encoding of its own is read back through that encoding, so the characters
/// it names are the ones the program wrote.
pub(crate) fn name_text(held: &std::rc::Rc<crate::object::StringValue>) -> String {
    if !held.holds_bytes() {
        return held.as_str().to_string();
    }
    let named = held.encoding_name();
    let bytes = string_methods::binary_bytes(held);
    if named == "EUC-JP" {
        return euc_jp_table::euc_jp_text(&bytes);
    }
    if string_methods::spells_shift_jis(&named) {
        return shift_jis_table::shift_jis_text(&bytes);
    }
    if named == "ISO-2022-JP" {
        return euc_jp_table::iso_2022_jp_text(&bytes);
    }
    match string_methods::latin_text(&bytes, &named) {
        Some(text) => text,
        None => held.as_str().to_string(),
    }
}

pub(crate) fn is_valid_constant_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

pub(crate) use int_methods::{RoundingMode, exact_ratio, split_rounding_mode};

/// Multiply by a power of two in steps small enough that each factor is a
/// Float, so an exponent far outside the Float range still scales correctly.
pub(crate) fn scale_by_power_of_two(value: f64, exponent: i64) -> f64 {
    const STEP: i64 = 500;
    let mut value = value;
    let mut remaining = exponent;
    while remaining != 0 {
        let step = remaining.clamp(-STEP, STEP);
        value *= (2f64).powi(step as i32);
        remaining -= step;
        if value == 0.0 || !value.is_finite() {
            break;
        }
    }
    value
}

impl VirtualMachine {
    /// The Enumerator a method answers when it is called without the block it
    /// would have yielded to. It remembers the receiver and the call, so
    /// walking it runs the method with a block of the Enumerator's own.
    pub(crate) fn make_enumerator(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, crate::error::MetorexError> {
        let Some(enumerator @ Object::Class(_)) = self.globals().get("Enumerator") else {
            return Err(crate::error::MetorexError::runtime_error(
                "Enumerator is not defined",
                crate::vm::utils::position_to_location(position),
            ));
        };
        // The size an Enumerator reports without walking it, which for a
        // collection is how many elements it holds. A search reports none,
        // since how far it runs depends on what it finds.
        let counted = !matches!(method_name, "rindex" | "index" | "find_index");
        let size = match receiver {
            Object::Array(elements) if counted => Object::Int(elements.borrow().len() as i64),
            Object::Dict(entries) if counted => Object::Int(entries.borrow().len() as i64),
            _ => Object::Nil,
        };
        let call = vec![
            receiver.clone(),
            Object::symbol(method_name.to_string()),
            Object::array(arguments.to_vec()),
            size,
        ];
        self.send_to_object(enumerator, "over", call, position)
    }
}

/// A string cut from another is in the same encoding, so an answer still
/// carrying the encoding a fresh literal gets is retagged with the
/// receiver's. A method that named an encoding of its own keeps it.
fn carry_string_encoding(
    receiver: &Object,
    method_name: &str,
    answer: Option<Object>,
) -> Option<Object> {
    // A method that pads with a string of its own works out which encoding
    // the two have in common, `encode` is named an encoding outright, and
    // `inspect` writes its answer in the encoding answers are written in, so
    // each is already tagged.
    if matches!(
        method_name,
        "center" | "ljust" | "rjust" | "+" | "encode" | "inspect"
    ) {
        return answer;
    }
    let Object::String(source) = receiver else {
        return answer;
    };
    let held = source.encoding_name();
    if held == crate::object::string_value::DEFAULT_ENCODING && !source.holds_bytes() {
        return answer;
    }
    let carry = |derived: &crate::object::StringValue| {
        if derived.encoding_name() != crate::object::string_value::DEFAULT_ENCODING
            || derived.holds_bytes()
        {
            return;
        }
        derived.set_encoding(held.clone());
        // Characters that stand for bytes go on standing for bytes in
        // whatever the method made out of them.
        if source.holds_bytes() {
            derived.mark_bytes();
        }
    };
    match &answer {
        Some(Object::String(derived)) => carry(derived),
        // The pieces a string was cut into read the way the whole one did.
        // A split around a separator hands that separator back as it was
        // written, so its own reading stands.
        Some(Object::Array(pieces)) if !matches!(method_name, "partition" | "rpartition") => {
            for piece in pieces.borrow().iter() {
                if let Object::String(derived) = piece {
                    carry(derived);
                }
            }
        }
        _ => {}
    }
    answer
}

impl VirtualMachine {
    /// Whether an object answers to a name, asking it with `respond_to?` when
    /// it is one of the program's own, since its answer may be written rather
    /// than declared.
    pub(crate) fn answers_to(
        &mut self,
        held: &Object,
        name: &str,
        position: crate::lexer::Position,
    ) -> Result<bool, crate::error::MetorexError> {
        if self.responds_to(held, name) {
            return Ok(true);
        }
        if !matches!(held, Object::Instance(_)) {
            return Ok(false);
        }
        // Ruby asks after a private method too when it is looking for the
        // one that converts, so the second argument is passed. A
        // `respond_to?` written to take one argument alone is asked again
        // without it.
        let asked = self.send_to_object(
            held.clone(),
            "respond_to?",
            vec![Object::symbol(name.to_string()), Object::Bool(true)],
            position,
        );
        let answer = match asked {
            Ok(answer) => answer,
            Err(_) => self.send_to_object(
                held.clone(),
                "respond_to?",
                vec![Object::symbol(name.to_string())],
                position,
            )?,
        };
        Ok(answer.is_truthy())
    }
}

/// Whether a name is one Module answers natively, which is what a method
/// taken from a class reaches before the class's own instance methods.
pub(crate) fn is_native_module_method(name: &str) -> bool {
    class_methods::NATIVE_MODULE_METHODS
        .iter()
        .any(|(held, _, _)| *held == name)
}

/// Whether a name is one of the functions Kernel carries, which are reachable
/// both without a receiver and through `Kernel` itself.
pub(crate) fn is_kernel_private_function(name: &str) -> bool {
    class_methods::KERNEL_PRIVATE_FUNCTIONS.contains(&name)
}

/// Whether Enumerable writes a method over `each`, so a class of the
/// program's own that writes `each` decides what it walks.
fn enumerable_walks_through_each(name: &str) -> bool {
    matches!(
        name,
        "map"
            | "collect"
            | "flat_map"
            | "select"
            | "filter"
            | "reject"
            | "find"
            | "detect"
            | "find_all"
            | "each_with_object"
            | "each_with_index"
            | "inject"
            | "reduce"
            | "sort_by"
            | "group_by"
            | "partition"
            | "min_by"
            | "max_by"
            | "sum"
            | "count"
            | "to_a"
            | "entries"
    )
}

/// Which store a thread-local method reads. Ruby keeps the fiber-local names
/// `Thread#[]` reaches apart from the thread-wide ones `thread_variable_get`
/// reaches, and a name set through one is not seen through the other.
impl VirtualMachine {
    /// The instance variable one of the two thread-local stores lives under.
    /// `thread_variable_*` is shared by every fiber the thread runs, while
    /// `[]` and its family belong to the fiber that wrote them, so the fiber
    /// running now is part of that store's name.
    fn thread_local_store(&self, method_name: &str, receiver: &Object) -> String {
        if method_name.starts_with("thread_variable") {
            return "__thread_variables".to_string();
        }
        // A name kept on another thread belongs to that thread's own root
        // fiber, since the fiber running now is none of its.
        // Outside every thread block the thread running is the main one,
        // which the stack does not name.
        let current = match self.thread_current_stack.last() {
            Some(held) => Some(held.clone()),
            None => self.globals().get("__Thread_main"),
        };
        let running = matches!(
            (&current, receiver),
            (Some(Object::Instance(a)), Object::Instance(b)) if Rc::ptr_eq(a, b)
        );
        let held = self.fiber_current_handle();
        // A thread's own body is its root fiber, so the names it keeps there
        // belong to the thread rather than to a fiber the program made.
        if !running
            || held == crate::vm::fibers::ROOT_FIBER
            || self.thread_body_fibers.contains(&held)
        {
            return "__thread_locals_root".to_string();
        }
        format!("__thread_locals_{held}")
    }
}

/// How many are waiting on a queue for something to be put there.
fn queue_waiting_count(inst: &Rc<std::cell::RefCell<crate::object::Instance>>) -> i64 {
    match inst.borrow().get_var("__queue_waiting") {
        Some(Object::Int(held)) => *held,
        _ => 0,
    }
}
