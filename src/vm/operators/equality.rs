// Whether two values are equal, and whether one covers the other.

use super::*;

impl VirtualMachine {
    /// Whether two values are equal, which each kind of value answers its
    /// own way.
    pub(crate) fn evaluate_equality(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // by asking that object instead, which is how a class that
        // stands for a number reports equality with one.
        if is_number(&left) && takes_coercion(&right) {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two patterns are the same when their source and flags are.
        if let (Object::Regex(pattern, flags), Object::Regex(other, other_flags)) = (&left, &right)
        {
            use crate::vm::native_methods::comparable_flags;
            return Ok(Object::Bool(
                pattern == other && comparable_flags(flags) == comparable_flags(other_flags),
            ));
        }
        // Two hashes compare entry by entry, looking each key up in
        // the other hash the way any key is looked up, which Hash
        // answers from its own method table.
        if matches!((&left, &right), (Object::Dict(_), Object::Dict(_)))
            && let Some(answer) =
                self.call_hash_method(&left, "==", std::slice::from_ref(&right), position)?
        {
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two ranges compare by their ends and their exclusivity,
        // which Range answers from its own method table.
        if matches!((&left, &right), (Object::Range { .. }, _)) {
            let class = self.builtins().class_of(&left);
            if let Some(answer) = self.call_native_method(
                &class,
                &left,
                "==",
                std::slice::from_ref(&right),
                position,
            )? {
                return Ok(Object::Bool(answer.is_truthy()));
            }
        }
        // For instances, dispatch to user-defined == method if present,
        // or to <=> (Comparable protocol) if the class has <=> defined.
        if let Object::Instance(inst_rc) = &left {
            // Identity shortcut: same object is always ==
            if let Object::Instance(rhs) = &right
                && Rc::ptr_eq(inst_rc, rhs)
            {
                return Ok(Object::Bool(true));
            }
            if let Some((class, method)) = self.lookup_method(&left, "==")
                && !method.is_undefined
            {
                let result =
                    self.invoke_method(class, method, left.clone(), vec![right.clone()], position)?;
                return Ok(Object::Bool(result.is_truthy()));
            }
            // Rational and Complex answer `==` from their native
            // tables rather than a method map, and a Complex with no
            // imaginary part equals the plain number it holds.
            let instance_class = Rc::clone(&inst_rc.borrow().class);
            if matches!(instance_class.name(), "Rational" | "Complex")
                && let Some(result) = self.call_native_method(
                    &instance_class,
                    &left,
                    "==",
                    std::slice::from_ref(&right),
                    position,
                )?
            {
                return Ok(Object::Bool(result.is_truthy()));
            }
            // Comparable protocol: if <=> is defined, use it for ==
            if let Some((cmp_class, cmp_method)) = self.lookup_method(&left, "<=>") {
                let cmp_obj = self.invoke_method(
                    cmp_class,
                    cmp_method,
                    left.clone(),
                    vec![right.clone()],
                    position,
                )?;
                return match cmp_obj {
                    Object::Int(n) => Ok(Object::Bool(n == 0)),
                    Object::Float(f) => Ok(Object::Bool(f == 0.0)),
                    Object::Nil => Ok(Object::Bool(false)),
                    other => {
                        // ArgumentError: <=> must return Integer, Float, or nil
                        let msg = format!(
                            "comparison of {} with {} failed",
                            left.type_name(),
                            other.type_name()
                        );
                        let exc = Object::exception("ArgumentError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                };
            }
        }
        // A Process::Status compares by the number it stands for, so
        // it answers `==` against that number as well as against
        // another status.
        if matches!(&left, Object::Instance(held) if held.borrow().class.name() == "Process::Status")
            && let Some(answer) = self.call_process_status_method(
                &left,
                "==",
                std::slice::from_ref(&right),
                position,
            )?
        {
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // A Set decides equality itself, since it counts anything
        // that says it is one, however that value is built.
        if matches!(left, Object::Set(_))
            && let Some(answer) = self.call_native_method(
                &self.builtins().class_of(&left),
                &left,
                "==",
                std::slice::from_ref(&right),
                position,
            )?
        {
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two hashes holding objects of the program's own are equal
        // when those objects say so, so each value is asked with `==`
        // rather than compared as data.
        if let (Object::Dict(one), Object::Dict(other)) = (&left, &right) {
            if Rc::ptr_eq(one, other) {
                return Ok(Object::Bool(true));
            }
            let held: Vec<(String, Object)> = one
                .borrow()
                .iter()
                .filter(|(key, _)| !key.starts_with("__MX_"))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            let against: Vec<(String, Object)> = other
                .borrow()
                .iter()
                .filter(|(key, _)| !key.starts_with("__MX_"))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            if held.iter().chain(against.iter()).any(|(_, value)| {
                    matches!(value, Object::Instance(_))
                        || matches!(value, Object::Array(items) if items.borrow().iter().any(|item| matches!(item, Object::Instance(_))))
                }) {
                    if held.len() != against.len() {
                        return Ok(Object::Bool(false));
                    }
                    for (key, value) in &held {
                        let Some((_, counterpart)) =
                            against.iter().find(|(other_key, _)| other_key == key)
                        else {
                            return Ok(Object::Bool(false));
                        };
                        let answer = self.evaluate_binary_operation(
                            &BinaryOp::Equal,
                            value.clone(),
                            counterpart.clone(),
                            position,
                        )?;
                        if !answer.is_truthy() {
                            return Ok(Object::Bool(false));
                        }
                    }
                    return Ok(Object::Bool(true));
                }
        }
        // Two arrays are equal when their elements are, which asks
        // each pair rather than comparing the two as data.
        if matches!((&left, &right), (Object::Array(_), Object::Array(_))) {
            let same = self.arrays_equal(&left, &right, position)?;
            return Ok(Object::Bool(same));
        }
        // An object that spells itself as an array answers for the
        // pair, which is how a wrapper around one compares.
        if matches!(left, Object::Array(_))
            && matches!(right, Object::Instance(_))
            && crate::vm::native_methods::array_subclass_value(&right).is_none()
            && self
                .send_to_object(
                    right.clone(),
                    "respond_to?",
                    vec![Object::symbol("to_ary".to_string())],
                    position,
                )?
                .is_truthy()
        {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two strings are equal when they spell the same thing,
        // whether that is read as bytes or as characters, and their
        // encodings can be compared at all.
        if let (Object::String(text), Object::String(other)) = (&left, &right) {
            use crate::vm::native_methods::string_methods::{binary_bytes, strings_comparable};
            return Ok(Object::Bool(
                (binary_bytes(text) == binary_bytes(other) || *text.as_str() == *other.as_str())
                    && strings_comparable(text, other),
            ));
        }
        // Anything else that spells itself as text answers for the
        // pair, which is how a wrapper around a String compares.
        if matches!(left, Object::String(_))
            && crate::vm::native_methods::string_subclass_value(&right).is_none()
            && self.responds_to(&right, "to_str")
        {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        Ok(Object::Bool(left.equals(&right)))
    }

    /// Whether the left operand covers the right, which is what a `when`
    /// clause and a pattern ask.
    pub(crate) fn evaluate_case_equality(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;
        // A number's `===` is its `==`, so it too answers by asking an
        // operand that is not a number.
        if is_number(&left) && takes_coercion(&right) {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Regexp === str: whether the pattern matches anywhere. A
        // Symbol matches on its name, and an object that answers
        // `to_str` on the characters it hands over.
        if let Object::Regex(pattern, flags) = &left {
            let subject = match &right {
                Object::String(text) | Object::Symbol(text) => Some(text.as_str().to_string()),
                other => match crate::vm::native_methods::subject_text(other) {
                    Some(text) => Some(text),
                    None if self.responds_to(other, "to_str") => {
                        match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                            Object::String(text) => Some(text.as_str().to_string()),
                            _ => None,
                        }
                    }
                    None => None,
                },
            };
            let Some(subject) = subject else {
                return Ok(Object::Bool(false));
            };
            let (pattern, flags) = (pattern.as_str().to_string(), flags.as_str().to_string());
            let found = self.regexp_match_data(&pattern, &flags, &subject, 0, position)?;
            return Ok(Object::Bool(found.is_some()));
        }
        // A callable answers for itself: `===` hands the value to
        // the proc and reports whatever it returns, so a proc can
        // stand in for a pattern in a `case`.
        if let Object::Block(block) = &left {
            return block.call(self, vec![right], position);
        }
        // A Set holds a value or it does not, which is the branch a
        // `case` over one picks.
        if matches!(left, Object::Set(_)) {
            let class = self.builtins().class_of(&left);
            if let Some(answer) = self.call_native_method(
                &class,
                &left,
                "include?",
                std::slice::from_ref(&right),
                position,
            )? {
                return Ok(Object::Bool(answer.is_truthy()));
            }
        }
        // A Range covers a value the way `cover?` reports, which is
        // what `case` uses to pick a branch.
        if matches!(left, Object::Range { .. }) {
            let class = self.builtins().class_of(&left);
            if let Some(answer) = self.call_native_method(
                &class,
                &left,
                "cover?",
                std::slice::from_ref(&right),
                position,
            )? {
                return Ok(Object::Bool(answer.is_truthy()));
            }
        }
        // Class/Module === obj: check type membership (Ruby's case
        // equality). Modules also count as the "type test" form so
        // `SomeMod === obj` works the same as `obj.is_a?(SomeMod)`.
        let class_rc_opt = match &left {
            Object::Class(c) | Object::Module(c) => Some(c),
            _ => None,
        };
        if let Some(class_rc) = class_rc_opt {
            if let Object::Exception(exc_ref) = &right {
                // Check if exception type matches or is a subclass
                let exc_type = exc_ref.borrow().exception_type.clone();
                if exc_type == class_rc.name() {
                    return Ok(Object::Bool(true));
                }
                // Check the exception class hierarchy. A namespaced
                // type such as `Errno::EINVAL` is a constant on its
                // module rather than a top-level name.
                let carried = exc_ref.borrow().class.clone().map(Object::Class);
                let resolved = match carried {
                    Some(class) => Some(class),
                    None => match self.globals().get(&exc_type) {
                        Some(class @ Object::Class(_)) => Some(class),
                        _ => self.resolve_qualified_constant(&exc_type),
                    },
                };
                if let Some(Object::Class(exc_class)) = resolved {
                    return Ok(Object::Bool(
                        self.builtins().is_subclass_of(&exc_class, class_rc),
                    ));
                }
                return Ok(Object::Bool(false));
            }
            // A class or a module is an object of Class or of
            // Module, which is what makes `Module === String` true.
            if matches!(right, Object::Class(_) | Object::Module(_)) {
                let meta_name = match &right {
                    Object::Class(_) => "Class",
                    _ => "Module",
                };
                if let Some(Object::Class(meta)) = self.globals().get(meta_name)
                    && self.builtins().is_subclass_of(&meta, class_rc)
                {
                    return Ok(Object::Bool(true));
                }
            }
            // `Left === right` asks whether the right stands as one
            // of the left, which is `right.is_a?(left)` whatever the
            // right is. A class is an object too, so `Module ===
            // String` is true and `String === String` is false.
            {
                if crate::builtin_classes::value_class_name(&right)
                    .is_some_and(|name| name == class_rc.name())
                {
                    return Ok(Object::Bool(true));
                }
                let right_class = self.builtins().class_of(&right);
                if self.builtins().is_subclass_of(&right_class, class_rc) {
                    return Ok(Object::Bool(true));
                }
                if let Object::Instance(inst_rc) = &right {
                    let sc_opt = inst_rc.borrow().singleton_class.borrow().clone();
                    if let Some(sc) = sc_opt
                        && self.builtins().is_subclass_of(&sc, class_rc)
                    {
                        return Ok(Object::Bool(true));
                    }
                }
                return Ok(Object::Bool(false));
            }
        }
        // Object#=== is the same object, or whatever #== says. It
        // consults neither #equal? nor #object_id, so a class that
        // overrides those does not change the answer. A class that
        // defines its own `===`, or reaches one through
        // `method_missing`, wins over that default.
        if let Object::Instance(inst_rc) = &left {
            if let Some((owner, method)) = self.lookup_method(&left, "===")
                && !method.is_undefined
            {
                return self.invoke_method(owner, method, left, vec![right], position);
            }
            if let Some((owner, handler)) = self.lookup_method(&left, "method_missing")
                && !handler.is_undefined
            {
                let arguments = vec![Object::symbol("===".to_string()), right];
                return self.invoke_method(owner, handler, left, arguments, position);
            }
            if let Object::Instance(rhs) = &right
                && Rc::ptr_eq(inst_rc, rhs)
            {
                return Ok(Object::Bool(true));
            }
            return self.evaluate_binary_operation(&Equal, left, right, position);
        }
        // A String's `===` is its `==`, down to the encodings it
        // refuses to compare.
        if matches!(left, Object::String(_)) {
            return self.evaluate_binary_operation(&Equal, left, right, position);
        }
        Ok(Object::Bool(left.equals(&right)))
    }
}
