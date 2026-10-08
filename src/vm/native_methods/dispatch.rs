// Which native method a name stands for, and the receiver it runs
// against.

use super::*;

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
                    // The block's `self` is the one where it was written,
                    // which at file scope is `main`.
                    let receiver = block
                        .captured_vars()
                        .get("self")
                        .map(|cell| cell.borrow().clone())
                        .or_else(|| match self.globals().get("TOPLEVEL_BINDING") {
                            Some(Object::Binding(top)) => top.receiver.clone(),
                            _ => None,
                        })
                        .unwrap_or(Object::Nil);
                    let binding = crate::object::Binding::with_receiver(
                        block.captured_vars().clone().into_iter().collect(),
                        receiver,
                    );
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
                    let made = crate::object::Instance::new(class);
                    made.borrow_mut()
                        .set_var(ARRAY_SUBCLASS_VAR.to_string(), result);
                    return Ok(Some(Object::Instance(made)));
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
                    let made = crate::object::Instance::new(class);
                    made.borrow_mut()
                        .set_var(HASH_SUBCLASS_VAR.to_string(), result);
                    return Ok(Some(Object::Instance(made)));
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
            return Ok(Some(Object::Bool(!self.pattern_was_built(pattern))));
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
        if method_name == "fixed_encoding?"
            && let Object::Regex(pattern, flags) = receiver
        {
            return Ok(Some(Object::Bool(
                self.pattern_fixes_encoding(pattern, flags),
            )));
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
            if method_name == "fixed_encoding?" {
                return Ok(Some(Object::Bool(
                    self.pattern_fixes_encoding(&pattern, &flags),
                )));
            }
            // The encoding and the source it is tagged in are the pattern's,
            // which the Regexp the instance holds answers for.
            if matches!(method_name, "encoding" | "source") {
                let pattern = Object::Regex(pattern, flags);
                let class = self.builtins().class_of(&pattern);
                return self.call_native_method(&class, &pattern, method_name, arguments, position);
            }
            if let Some(result) =
                self.call_regexp_method(&pattern, &flags, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }

        // A Regexp allocated and never given a pattern matches in no
        // encoding, which Ruby reports as BINARY.
        if method_name == "encoding"
            && arguments.is_empty()
            && let Object::Instance(instance) = receiver
            && regexp_subclass_value(receiver).is_none()
            && super::class_methods::class_named_in_chain(&instance.borrow().class, "Regexp")
        {
            return Ok(Some(self.encoding_object("ASCII-8BIT")));
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
                // any other character is written in UTF-8. A symbol in an
                // encoding that is not ASCII-compatible keeps its encoding
                // whatever it spells.
                let ascii_compatible =
                    crate::vm::native_methods::string_methods::wide_encoding(&text.encoding_name())
                        .is_none();
                let named_in = if ascii_compatible && text.as_str().is_ascii() {
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
            "Thread::Queue" | "Thread::SizedQueue" => {
                self.call_queue_method(receiver, method_name, arguments, position)
            }
            "Thread::Mutex" => self.call_mutex_method(receiver, method_name, arguments, position),
            "Thread::ConditionVariable" => {
                self.call_condition_variable_method(receiver, method_name, arguments, position)
            }
            "Process::Status" => {
                self.call_process_status_method(receiver, method_name, arguments, position)
            }
            _ => Ok(None),
        }
    }
}
