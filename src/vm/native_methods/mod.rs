//! Native (built-in) method implementations for the virtual machine.
//!
//! This module contains the implementations of all built-in methods for
//! standard classes like Object, String, and Array.

pub(crate) mod array_methods;
pub(crate) mod ast_methods;
mod class_methods;
pub(crate) use class_methods::MODULE_FUNCTION_VISIBILITY;
pub(crate) use class_methods::is_native_kernel_method;
pub(crate) use module_methods::{REFINEMENT_KEY_PREFIX, REFINEMENT_LABEL_KEY};
mod constant_visibility;
mod define_method;
mod exception_methods;
mod file_methods;
mod float_methods;
mod hash_methods;
mod int_methods;
pub(crate) mod kernel_conversion;
mod method_object_methods;
mod module_methods;
mod object_methods;
mod range_methods;
pub(crate) mod rational_methods;
pub(crate) use rational_methods::rational_parts;
mod set_methods;
mod string_methods;
mod struct_methods;
pub(crate) use struct_methods::struct_members;

/// Instance variable a String subclass keeps its characters in.
pub(crate) const STRING_SUBCLASS_VAR: &str = "__string__";

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
        // Binding receiver
        if let Object::Binding(binding) = receiver
            && method_name == "receiver"
        {
            return Ok(Some(binding.receiver.clone().unwrap_or(Object::Nil)));
        }

        // Binding#eval — minimal implementation: handles `eval("self")`
        // (returns the binding's receiver). Anything else returns Nil; full
        // re-parse-in-binding-scope is substantial and not needed here.
        if let Object::Binding(binding) = receiver
            && method_name == "eval"
            && let Some(Object::String(s)) = arguments.first()
            && s.as_str().trim() == "self"
        {
            return Ok(Some(binding.receiver.clone().unwrap_or(Object::Nil)));
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

        // Block/Lambda methods
        if let Object::Block(block) = receiver {
            match method_name {
                "call" | "[]" => {
                    return Ok(Some(block.call(self, arguments.to_vec(), position)?));
                }
                "binding" => {
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
            if let Some(result) =
                self.call_string_method(&text, method_name, arguments, position)?
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

        // Dispatch to the appropriate class-specific method implementation
        match class.name() {
            "Object" | "Proc" | "Method" => {
                self.call_object_method(receiver, method_name, arguments, position)
            }
            "String" => self.call_string_method(receiver, method_name, arguments, position),
            "Integer" => self.call_int_method(receiver, method_name, arguments, position),
            "Array" => self.call_array_method(receiver, method_name, arguments, position),
            "Hash" => self.call_hash_method(receiver, method_name, arguments, position),
            "Float" => self.call_float_method(receiver, method_name, arguments, position),
            "Range" => self.call_range_method(receiver, method_name, arguments, position),
            "Rational" => self.call_rational_method(receiver, method_name, arguments, position),
            "Set" => self.call_set_method(receiver, method_name, arguments, position),
            "Exception" => self.call_exception_method(receiver, method_name, arguments, position),
            "Thread" => self.call_thread_method(receiver, method_name, arguments, position),
            "Queue" | "SizedQueue" => {
                self.call_queue_method(receiver, method_name, arguments, position)
            }
            "Mutex" => self.call_mutex_method(receiver, method_name, arguments, position),
            "ConditionVariable" => {
                self.call_condition_variable_method(receiver, method_name, arguments, position)
            }
            "File" => self.call_file_handle_method(receiver, method_name, arguments, position),
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
            Some(Object::String(s)) if s.as_str() == "$stderr"
        );
        if !placeholder && let Some(obj) = stderr_obj {
            let line = format!("{}\n", msg);
            for cand in ["write", "<<"] {
                if let Some((cls, method)) = self.lookup_method(&obj, cand) {
                    let arg = Object::String(Rc::new(line.clone()));
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
            Object::String(s) => Ok((**s).clone()),
            Object::Symbol(s) => Ok((**s).clone()),
            _ => {
                if let Some((cls, m)) = self.lookup_method(arg, "to_str")
                    && !m.is_undefined
                {
                    let result = self.invoke_method(cls, m, arg.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok((*s).clone());
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
                let cached = inst.borrow().get_var("__thread_value").cloned();
                if let Some(val) = cached {
                    return Ok(Some(if method_name == "join" {
                        receiver.clone()
                    } else {
                        val
                    }));
                }
                // Drop this thread from the pending list — about to run.
                self.pending_threads.retain(|t| {
                    if let (Object::Instance(a), Object::Instance(b)) = (t, receiver) {
                        !Rc::ptr_eq(a, b)
                    } else {
                        true
                    }
                });
                let block_obj = inst
                    .borrow()
                    .get_var("__thread_block")
                    .cloned()
                    .unwrap_or(Object::Nil);
                // Push this thread onto the "current thread" stack so
                // `Thread.current` returns the right instance for the
                // duration of the block (and `Thread.current[:k] = v`
                // writes thread-locals to this thread, not the caller).
                self.thread_current_stack.push(receiver.clone());
                let value_result: Result<Object, MetorexError> = if let Object::Block(b) = block_obj
                {
                    self.execute_block_body(&b, vec![])
                } else {
                    Ok(Object::Nil)
                };
                self.thread_current_stack.pop();
                let value = value_result?;
                inst.borrow_mut()
                    .set_var("__thread_value".to_string(), value.clone());
                Ok(Some(if method_name == "join" {
                    receiver.clone()
                } else {
                    value
                }))
            }
            // Thread-local storage: `t[:k]` and `t[:k] = v`. Backed by an
            // ivar Hash on the Thread instance.
            "[]" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::method_argument_error(
                        "[]",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key_str = match &arguments[0] {
                    Object::Symbol(s) => (**s).clone(),
                    Object::String(s) => (**s).clone(),
                    _ => return Ok(Some(Object::Nil)),
                };
                let locals = inst.borrow().get_var("__thread_locals").cloned();
                if let Some(Object::Dict(d)) = locals {
                    return Ok(Some(
                        d.borrow().get(&key_str).cloned().unwrap_or(Object::Nil),
                    ));
                }
                Ok(Some(Object::Nil))
            }
            "[]=" => {
                if arguments.len() != 2 {
                    return Err(crate::vm::errors::method_argument_error(
                        "[]=",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let key_str = match &arguments[0] {
                    Object::Symbol(s) => (**s).clone(),
                    Object::String(s) => (**s).clone(),
                    _ => return Ok(Some(arguments[1].clone())),
                };
                let existing = inst.borrow().get_var("__thread_locals").cloned();
                let dict = match existing {
                    Some(Object::Dict(d)) => d,
                    _ => {
                        let d = Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));
                        inst.borrow_mut()
                            .set_var("__thread_locals".to_string(), Object::Dict(Rc::clone(&d)));
                        d
                    }
                };
                dict.borrow_mut().insert(key_str, arguments[1].clone());
                Ok(Some(arguments[1].clone()))
            }
            "alive?" | "stop?" => Ok(Some(Object::Bool(false))),
            "status" => Ok(Some(Object::Bool(false))),
            _ => Ok(None),
        }
    }

    /// Instance-level methods on file handles produced by `File.open`.
    /// Implements just enough of IO/File: `puts`, `print`, `write`, `<<`,
    /// `close`, `closed?`. Reads are not supported (handles are write-mode
    /// only for spec-helper purposes).
    pub(crate) fn call_file_handle_method(
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
        // Only intercept if this instance was produced by `File.open`
        // (carries `__file_path`); otherwise return None and let the
        // generic class methods (which include reopens of `File`) handle
        // the call.
        let path = match inst.borrow().get_var("__file_path").cloned() {
            Some(Object::String(s)) => s.as_ref().clone(),
            _ => return Ok(None),
        };
        let append_text = |contents: String| -> Result<(), MetorexError> {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| {
                    MetorexError::runtime_error(
                        format!("Failed to open '{}' for write: {}", path, e),
                        crate::error::SourceLocation::new(0, 0, 0),
                    )
                })?;
            f.write_all(contents.as_bytes()).map_err(|e| {
                MetorexError::runtime_error(
                    format!("Failed to write to '{}': {}", path, e),
                    crate::error::SourceLocation::new(0, 0, 0),
                )
            })?;
            Ok(())
        };
        match method_name {
            "puts" => {
                if arguments.is_empty() {
                    append_text("\n".to_string())?;
                } else {
                    for arg in arguments {
                        let s = match arg {
                            Object::String(s) => s.as_ref().clone(),
                            other => format!("{}", other),
                        };
                        let mut line = s;
                        if !line.ends_with('\n') {
                            line.push('\n');
                        }
                        append_text(line)?;
                    }
                }
                Ok(Some(Object::Nil))
            }
            "print" | "write" | "<<" => {
                for arg in arguments {
                    let s = match arg {
                        Object::String(s) => s.as_ref().clone(),
                        other => format!("{}", other),
                    };
                    append_text(s)?;
                }
                Ok(Some(receiver.clone()))
            }
            // Each line of the file, with its terminator, yielded to the
            // block or returned as an array.
            "each_line" | "readlines" => {
                let contents = std::fs::read_to_string(&path).map_err(|error| {
                    MetorexError::runtime_error(
                        format!("Failed to read '{}': {}", path, error),
                        crate::error::SourceLocation::new(0, 0, 0),
                    )
                })?;
                let lines: Vec<Object> = contents
                    .split_inclusive('\n')
                    .map(|line| Object::String(Rc::new(line.to_string())))
                    .collect();
                match self.pending_block.take() {
                    Some(Object::Block(block)) => {
                        for line in lines {
                            self.execute_block_body(&block, vec![line])?;
                        }
                        Ok(Some(receiver.clone()))
                    }
                    _ => Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(lines))))),
                }
            }
            "read" => {
                let contents = std::fs::read_to_string(&path).map_err(|error| {
                    MetorexError::runtime_error(
                        format!("Failed to read '{}': {}", path, error),
                        crate::error::SourceLocation::new(0, 0, 0),
                    )
                })?;
                Ok(Some(Object::String(Rc::new(contents))))
            }
            "close" => Ok(Some(Object::Nil)),
            "closed?" => Ok(Some(Object::Bool(false))),
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
                if let Some(item) = arguments.first() {
                    items_arr.borrow_mut().push(item.clone());
                }
                Ok(Some(receiver.clone()))
            }
            "pop" | "deq" | "shift" => {
                if items_arr.borrow().is_empty() {
                    // In real Ruby, `Queue#pop` on an empty queue blocks
                    // until another thread pushes. Our Thread.new is
                    // lazy/synchronous, so block-waiting is meaningless;
                    // instead we drain the pending Thread.new list and
                    // run their blocks once. That's enough to unblock
                    // common "thread A pushes, thread B pops" coordination
                    // patterns in mspec fixtures (autoload's
                    // check_before_during_thread_after, etc.).
                    while let Some(thread_obj) = self.pending_threads.pop() {
                        if let Object::Instance(inst) = &thread_obj {
                            let already_run = inst.borrow().get_var("__thread_value").is_some();
                            if !already_run {
                                let block_obj = inst
                                    .borrow()
                                    .get_var("__thread_block")
                                    .cloned()
                                    .unwrap_or(Object::Nil);
                                self.thread_current_stack.push(thread_obj.clone());
                                let value_result = if let Object::Block(b) = block_obj {
                                    self.execute_block_body(&b, vec![])
                                } else {
                                    Ok(Object::Nil)
                                };
                                self.thread_current_stack.pop();
                                let value = value_result?;
                                inst.borrow_mut()
                                    .set_var("__thread_value".to_string(), value);
                            }
                        }
                        if !items_arr.borrow().is_empty() {
                            break;
                        }
                    }
                }
                let val = if items_arr.borrow().is_empty() {
                    Object::Nil
                } else {
                    items_arr.borrow_mut().remove(0)
                };
                Ok(Some(val))
            }
            "size" | "length" | "count" => Ok(Some(Object::Int(items_arr.borrow().len() as i64))),
            "empty?" => Ok(Some(Object::Bool(items_arr.borrow().is_empty()))),
            "clear" => {
                items_arr.borrow_mut().clear();
                Ok(Some(receiver.clone()))
            }
            "close" | "closed?" => Ok(Some(Object::Bool(false))),
            _ => Ok(None),
        }
    }

    /// Mutex instance methods. Single-threaded stubs: `synchronize` just
    /// invokes the block, `lock`/`unlock` are no-ops, and `locked?` always
    /// reports false. Enough for `Mutex.new.synchronize { ... }` patterns
    /// in spec fixtures (CyclicBarrier, ThreadSafeCounter).
    pub(crate) fn call_mutex_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        _arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "synchronize" => {
                let block = self.pending_block.take();
                if let Some(Object::Block(b)) = block {
                    let result = self.execute_block_body(&b, vec![])?;
                    Ok(Some(result))
                } else {
                    let exc = Object::exception(
                        "ArgumentError",
                        "Mutex#synchronize requires a block".to_string(),
                    );
                    Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: super::utils::position_to_location(position),
                        message: "Mutex#synchronize requires a block".to_string(),
                    })
                }
            }
            "lock" | "unlock" => Ok(Some(receiver.clone())),
            "locked?" => Ok(Some(Object::Bool(false))),
            "try_lock" => Ok(Some(Object::Bool(true))),
            "owned?" => Ok(Some(Object::Bool(false))),
            _ => Ok(None),
        }
    }

    /// ConditionVariable instance methods. Single-threaded stubs: `wait` is
    /// a no-op (in real Ruby it would block until `broadcast` / `signal`,
    /// but we have no other thread to do the waking, so blocking would
    /// deadlock); `signal` / `broadcast` are no-ops too.
    pub(crate) fn call_condition_variable_method(
        &mut self,
        _receiver: &Object,
        method_name: &str,
        _arguments: &[Object],
        _position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "wait" | "signal" | "broadcast" => Ok(Some(Object::Nil)),
            _ => Ok(None),
        }
    }
}

/// Whether `name` is a syntactically valid Ruby constant name: must start
/// with an uppercase letter and contain only word characters after.
/// Multibyte letters are allowed (Ruby permits `CS_CONSTλ`). Used by
/// `Module#autoload`, `Module#const_set`, and `Module#const_defined?` to
/// reject lowercase / numeric / punctuated names with a NameError.
pub(crate) fn is_valid_constant_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}
