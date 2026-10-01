// Writing a value into whatever a target names.

use super::*;
use crate::lexer::Position;

/// An assignment target with whatever it needs ahead of its value already
/// evaluated.
pub(crate) enum PreparedTarget<'a> {
    /// A variable or a bare constant, which needs nothing evaluated first.
    Named(&'a Expression),
    /// A target written on a receiver.
    Receiver(ReceiverTarget<'a>),
    /// `(a, b)` among the targets of a multiple assignment, and where it
    /// was written.
    Group(Vec<PreparedTarget<'a>>, crate::lexer::Position),
    /// `*rest` among the targets of a multiple assignment.
    Splat(Box<PreparedTarget<'a>>),
}

/// A target written on a receiver, which a compound assignment reads and
/// writes through the one evaluation of that receiver.
pub(crate) enum ReceiverTarget<'a> {
    /// `receiver.name = value`, or `receiver&.name = value` when `safe`.
    Setter {
        receiver_expression: &'a Expression,
        receiver: Object,
        reader: &'a str,
        safe: bool,
        position: Position,
    },
    /// `receiver[subscripts] = value`. `sends` is set for the forms that
    /// always write through a `[]=` call: a splat subscript, and several.
    Subscript {
        receiver: Object,
        subscripts: Vec<Object>,
        sends: bool,
        position: Position,
    },
    /// `Ns::Name = value`.
    ScopedConstant {
        namespace: Object,
        name: &'a str,
        position: Position,
    },
}

impl VirtualMachine {
    /// Evaluate the value of a constant assignment inside a class or module
    /// body, honoring the `CONST ||= value` form for a constant that has no
    /// value yet.
    pub(crate) fn evaluate_constant_assignment(
        &mut self,
        statement: &Statement,
        value: &Expression,
    ) -> Result<Object, MetorexError> {
        let Statement::Assignment { target, .. } = statement else {
            return self.evaluate_expression(value);
        };
        match self.conditional_assignment_to_new_constant(target, value) {
            Some(right) => self.evaluate_expression(right),
            None => self.evaluate_expression(value),
        }
    }

    /// For `CONST ||= value` where `CONST` is not defined, the right-hand
    /// side of the desugared `CONST = CONST || value`. `None` for every other
    /// assignment, including one to a constant that already has a value.
    pub(crate) fn conditional_assignment_to_new_constant<'a>(
        &mut self,
        target: &Expression,
        value: &'a Expression,
    ) -> Option<&'a Expression> {
        let Expression::BinaryOp {
            op: crate::ast::BinaryOp::Or,
            left,
            right,
            ..
        } = value
        else {
            return None;
        };
        // A global nothing has set is read as nil the same way, and without
        // the notice reading it would give.
        let same_global = matches!(
            (target, left.as_ref()),
            (
                Expression::GlobalVariable { name, .. },
                Expression::GlobalVariable { name: read, .. }
            ) if name == read
        );
        if !constant_target_matches(target, left) && !same_global {
            return None;
        }
        match self.eval_defined(target) {
            Ok(Object::Nil) => Some(right),
            _ => None,
        }
    }

    /// Prepare each target of a multiple assignment in the order written.
    pub(crate) fn prepare_targets<'a>(
        &mut self,
        targets: &'a [Expression],
    ) -> Result<Vec<PreparedTarget<'a>>, MetorexError> {
        let mut prepared = Vec::with_capacity(targets.len());
        for target in targets {
            prepared.push(self.prepare_target(target)?);
        }
        Ok(prepared)
    }

    /// Hand each value to the prepared target standing in the same place,
    /// with the splat, where there is one, taking everything the others
    /// leave.
    pub(crate) fn spread_into_prepared(
        &mut self,
        targets: &[PreparedTarget],
        source: &[Object],
    ) -> Result<(), MetorexError> {
        let splat_at = targets
            .iter()
            .position(|target| matches!(target, PreparedTarget::Splat(_)));
        let Some(splat_at) = splat_at else {
            for (index, target) in targets.iter().enumerate() {
                let value = source.get(index).cloned().unwrap_or(Object::Nil);
                self.store_prepared(target, value)?;
            }
            return Ok(());
        };
        for (index, target) in targets[..splat_at].iter().enumerate() {
            let value = source.get(index).cloned().unwrap_or(Object::Nil);
            self.store_prepared(target, value)?;
        }
        // The splat takes what the targets on either side of it do not,
        // which is none at all when there are fewer values than named
        // targets.
        let after = targets.len() - splat_at - 1;
        let taken = source.len().saturating_sub(splat_at + after);
        let collected: Vec<Object> = source.iter().skip(splat_at).take(taken).cloned().collect();
        self.store_prepared(&targets[splat_at], Object::array(collected))?;
        for (offset, target) in targets[splat_at + 1..].iter().enumerate() {
            let value = source
                .get(splat_at + taken + offset)
                .cloned()
                .unwrap_or(Object::Nil);
            self.store_prepared(target, value)?;
        }
        Ok(())
    }

    /// Evaluate what a target needs ahead of its value: the receiver of a
    /// setter, the receiver and subscripts of `[]=`, and the namespace of
    /// `Ns::Name`. Ruby evaluates these left to right before the right-hand
    /// side.
    pub(crate) fn prepare_target<'a>(
        &mut self,
        target: &'a Expression,
    ) -> Result<PreparedTarget<'a>, MetorexError> {
        let on_receiver = match target {
            Expression::Array { elements, position } => {
                return Ok(PreparedTarget::Group(
                    self.prepare_targets(elements)?,
                    *position,
                ));
            }
            Expression::Splat { expression, .. } => {
                return Ok(PreparedTarget::Splat(Box::new(
                    self.prepare_target(expression)?,
                )));
            }
            Expression::Index {
                array,
                index,
                position,
            } => {
                let receiver = self.evaluate_expression(array)?;
                // `held[*subscripts] = value` spreads the values across the
                // subscript, the same way `held.[]=(*subscripts, value)` does.
                let (subscripts, sends) = match index.as_ref() {
                    Expression::Splat { .. } => (
                        self.evaluate_arguments(std::slice::from_ref(index.as_ref()))?,
                        true,
                    ),
                    single => (vec![self.evaluate_expression(single)?], false),
                };
                ReceiverTarget::Subscript {
                    receiver,
                    subscripts,
                    sends,
                    position: *position,
                }
            }
            Expression::MethodCall {
                receiver: receiver_expression,
                method,
                arguments,
                position,
                ..
            } => {
                let receiver = self.evaluate_expression(receiver_expression)?;
                match (method.as_str(), arguments.as_slice()) {
                    // `values[start, length] = other` parses as a call to
                    // `[]`, and assigning to it sends `[]=` with the same
                    // subscripts.
                    ("[]", _) => ReceiverTarget::Subscript {
                        receiver,
                        subscripts: self.evaluate_arguments(arguments)?,
                        sends: true,
                        position: *position,
                    },
                    // `held&.name = value` writes only when the receiver is
                    // there.
                    (crate::parser::SAFE_CALL, [Expression::Symbol { value: named, .. }]) => {
                        ReceiverTarget::Setter {
                            receiver_expression,
                            receiver,
                            reader: named,
                            safe: true,
                            position: *position,
                        }
                    }
                    // The parser accepts no other call as a target than one
                    // without arguments, which names a setter.
                    _ => ReceiverTarget::Setter {
                        receiver_expression,
                        receiver,
                        reader: method,
                        safe: false,
                        position: *position,
                    },
                }
            }
            Expression::ScopeResolution {
                namespace,
                name,
                position,
            } => ReceiverTarget::ScopedConstant {
                namespace: self.evaluate_expression(namespace)?,
                name,
                position: *position,
            },
            named => return Ok(PreparedTarget::Named(named)),
        };
        Ok(PreparedTarget::Receiver(on_receiver))
    }

    /// Write `value` through a target whose receiver was evaluated already.
    pub(crate) fn store_prepared(
        &mut self,
        target: &PreparedTarget,
        value: Object,
    ) -> Result<(), MetorexError> {
        match target {
            PreparedTarget::Named(named) => self.store_named(named, value),
            PreparedTarget::Receiver(on_receiver) => self.store_on_receiver(on_receiver, value),
            PreparedTarget::Group(parts, position) => {
                // `(a, b), c = pair, held` names a group of targets, which
                // takes the value apart the way the whole list does.
                let source = self
                    .block_argument_spread(&value, *position)?
                    .unwrap_or_else(|| vec![value.clone()]);
                self.spread_into_prepared(parts, &source)
            }
            PreparedTarget::Splat(inner) => self.store_prepared(inner, value),
        }
    }

    /// Write `value` through a target on a receiver evaluated already.
    fn store_on_receiver(
        &mut self,
        target: &ReceiverTarget,
        value: Object,
    ) -> Result<(), MetorexError> {
        match target {
            ReceiverTarget::Subscript {
                receiver,
                subscripts,
                sends,
                position,
            } => match subscripts.as_slice() {
                [single] if !sends => {
                    self.store_single_subscript(receiver.clone(), single.clone(), value, position)
                }
                _ => {
                    let mut arguments = subscripts.clone();
                    arguments.push(value);
                    self.send_to_object(receiver.clone(), "[]=", arguments, *position)?;
                    Ok(())
                }
            },
            ReceiverTarget::Setter {
                receiver: Object::Nil,
                safe: true,
                ..
            } => Ok(()),
            ReceiverTarget::Setter {
                receiver_expression,
                receiver,
                reader,
                position,
                ..
            } => self.store_through_setter(
                receiver_expression,
                receiver.clone(),
                format!("{}=", reader),
                value,
                position,
            ),
            ReceiverTarget::ScopedConstant {
                namespace,
                name,
                position,
            } => self.store_scoped_constant(namespace.clone(), name, value, position),
        }
    }

    /// Read what a target on a receiver holds now, which a compound
    /// assignment combines with its right-hand side. For `||=`, a constant
    /// not yet defined reads as nil, which is what lets `Ns::Name ||= value`
    /// define it. Any other operator refuses a constant that is not there.
    fn read_on_receiver(
        &mut self,
        target: &ReceiverTarget,
        or_assigning: bool,
    ) -> Result<Object, MetorexError> {
        match target {
            ReceiverTarget::Setter {
                receiver_expression,
                receiver,
                reader,
                position,
                ..
            } => {
                self.refuse_private_reader(receiver_expression, receiver, reader, *position)?;
                self.send_to_object(receiver.clone(), reader, vec![], *position)
            }
            ReceiverTarget::Subscript {
                receiver,
                subscripts,
                position,
                ..
            } => self.send_to_object(receiver.clone(), "[]", subscripts.clone(), *position),
            ReceiverTarget::ScopedConstant {
                namespace: Object::Class(owner) | Object::Module(owner),
                name,
                ..
            } if or_assigning
                && self.const_entry_on(owner, name, true, false).is_none()
                && owner.get_autoload(name).is_none() =>
            {
                Ok(Object::Nil)
            }
            ReceiverTarget::ScopedConstant {
                namespace,
                name,
                position,
            } => self.read_scoped_constant(namespace.clone(), name, position),
        }
    }

    /// A reader called with an explicit receiver has to be public, the same
    /// as the setter it pairs with.
    fn refuse_private_reader(
        &mut self,
        receiver_expression: &Expression,
        receiver: &Object,
        reader: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Object::Instance(instance) = receiver else {
            return Ok(());
        };
        let class = Rc::clone(&instance.borrow().class);
        if crate::vm::method_lookup::names_self(receiver_expression)
            || !self.refuses_explicit_receiver(&class, reader)
        {
            return Ok(());
        }
        Err(visibility_error(&class, reader, position))
    }

    /// Evaluate `target = value` and answer what the assignment answers. A
    /// compound assignment such as `held.count += 1` reads and writes
    /// through one evaluation of the receiver.
    pub(crate) fn evaluate_assignment(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) -> Result<Object, MetorexError> {
        let prepared = self.prepare_target(target)?;
        if let PreparedTarget::Receiver(on_receiver) = &prepared {
            // `held&.name = value` evaluates nothing more when there is no
            // receiver, and answers nil.
            if matches!(
                on_receiver,
                ReceiverTarget::Setter {
                    receiver: Object::Nil,
                    safe: true,
                    ..
                }
            ) {
                return Ok(Object::Nil);
            }
            if let Some(answer) = self.compound_assignment(on_receiver, target, value)? {
                return Ok(answer);
            }
        }
        let evaluated = match self.conditional_assignment_to_new_constant(target, value) {
            // `CONST ||= value` where CONST is not defined yet: Ruby reads the
            // undefined constant as nil rather than raising, so only the
            // right-hand side is evaluated.
            Some(right) => self.evaluate_expression(right)?,
            None => self.evaluate_expression(value)?,
        };
        self.store_prepared(&prepared, evaluated.clone())?;
        Ok(evaluated)
    }

    /// The parser writes `target op= operand` as `target = target op
    /// operand`, with the target cloned into the value. When `value` has that
    /// shape, read the target once through `on_receiver`, combine, and write
    /// back. `None` for a plain assignment.
    fn compound_assignment(
        &mut self,
        on_receiver: &ReceiverTarget,
        target: &Expression,
        value: &Expression,
    ) -> Result<Option<Object>, MetorexError> {
        let answer = match value {
            Expression::BinaryOp {
                op,
                left,
                right,
                position,
            } if left.as_ref() == target => {
                let or_assigning = matches!(op, crate::ast::BinaryOp::Or);
                let current = self.read_on_receiver(on_receiver, or_assigning)?;
                match op {
                    // `||=` and `&&=` write nothing when the value they read
                    // already decides the answer.
                    crate::ast::BinaryOp::Or if is_truthy(&current) => return Ok(Some(current)),
                    crate::ast::BinaryOp::And if !is_truthy(&current) => {
                        return Ok(Some(current));
                    }
                    crate::ast::BinaryOp::Or | crate::ast::BinaryOp::And => {
                        self.evaluate_expression(right)?
                    }
                    _ => {
                        let operand = self.evaluate_expression(right)?;
                        self.operate_on_values(op, current, operand, position)?
                    }
                }
            }
            // `held <<= 2` calls the shift the value defines.
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                position,
                ..
            } if receiver.as_ref() == target => {
                let current = self.read_on_receiver(on_receiver, false)?;
                let operands = self.evaluate_arguments(arguments)?;
                self.send_to_object(current, method, operands, *position)?
            }
            _ => return Ok(None),
        };
        self.store_on_receiver(on_receiver, answer.clone())?;
        Ok(Some(answer))
    }

    /// Assign a value to the given target expression.
    pub(crate) fn assign_value(
        &mut self,
        target: &Expression,
        value: Object,
    ) -> Result<(), MetorexError> {
        let prepared = self.prepare_target(target)?;
        self.store_prepared(&prepared, value)
    }

    /// Write `value` into a variable or a bare constant, the targets that
    /// need nothing evaluated ahead of their value.
    fn store_named(&mut self, target: &Expression, value: Object) -> Result<(), MetorexError> {
        match target {
            Expression::Identifier { name, position } => {
                // Constant-shaped identifiers (`MyClass = ...`) auto-name
                // anonymous classes/modules on first assignment, matching
                // Ruby's `klass.name` behavior after binding to a constant.
                let is_const = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                // A constant bound inside a class or module body names what
                // it holds under that namespace, so `M::Foo` reports its own
                // path rather than the bare constant name.
                if is_const {
                    match self.constant_home() {
                        Some(enclosing) => name_constant_value(&enclosing, name, &value),
                        None => {
                            if let Object::Class(v) | Object::Module(v) = &value {
                                v.assign_name_recursive(name);
                            }
                        }
                    }
                }
                // Constants assigned in a context that has a lexically
                // enclosing class/module land on that scope's class_var
                // table — matching Ruby's behavior, where `Foo = 1` inside
                // a `module M` (or a lambda defined inside one) resolves
                // later as `M::Foo`. At the top level (no enclosing
                // scope), uppercase assignments go to globals (and
                // Object's class_var) so they survive the require'd
                // file's local environment ending. Both record the
                // assignment site for `Module#const_source_location`.
                let assign_file = self
                    .reported_current_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                if is_const && let Some(enclosing) = self.constant_home() {
                    let is_new = enclosing.get_class_var(name).is_none();
                    if !is_new {
                        self.warn_already_initialized(&enclosing, name, *position);
                    }
                    enclosing.set_class_var(name, value);
                    enclosing.set_const_location(name, assign_file, position.line as i64);
                    if is_new {
                        let owner = Object::Class(Rc::clone(&enclosing));
                        self.trigger_const_added_hook(owner, name, *position)?;
                    }
                    return Ok(());
                }
                if is_const {
                    self.globals_mut().set(name.clone(), value.clone());
                    // A constant the program's scope also holds, as the
                    // standard streams are, reads the new value there too.
                    self.environment_mut().set(name, value.clone());
                    let mut owner = None;
                    if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                        if object_class.get_class_var(name).is_none() {
                            owner = Some(Object::Class(Rc::clone(&object_class)));
                        } else {
                            self.warn_already_initialized(&object_class, name, *position);
                        }
                        object_class.set_class_var(name, value);
                        object_class.set_const_location(name, assign_file, position.line as i64);
                    }
                    if let Some(owner) = owner {
                        self.trigger_const_added_hook(owner, name, *position)?;
                    }
                    return Ok(());
                }
                if !self.environment_mut().set(name, value.clone()) {
                    self.environment_mut().define(name.clone(), value);
                }
                Ok(())
            }
            // `::Name = value` binds at the top level whatever class or
            // module body the assignment sits in.
            Expression::TopLevelConstant { name, position } => {
                // Naming the module names the anonymous ones it holds, which
                // is how `A::B` reports its path once `A` has a name.
                if let Object::Class(bound) | Object::Module(bound) = &value {
                    bound.assign_name_recursive(name);
                }
                let assign_file = self
                    .reported_current_file()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default();
                self.globals_mut().set(name.clone(), value.clone());
                self.environment_mut().set(name, value.clone());
                if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                    object_class.set_class_var(name, value);
                    object_class.set_const_location(name, assign_file, position.line as i64);
                }
                Ok(())
            }
            Expression::InstanceVariable { name, position } => {
                // At the top level `self` is `main`, which carries the
                // program's own instance variables the way any other object
                // does.
                let held = self.environment().get("self").or_else(|| {
                    match self.globals().get("TOPLEVEL_BINDING") {
                        Some(Object::Binding(binding)) => binding.receiver.clone(),
                        _ => None,
                    }
                });
                match held {
                    Some(Object::Instance(instance_rc)) => {
                        let is_frozen = instance_rc.borrow().frozen;
                        if is_frozen {
                            let receiver = Object::Instance(Rc::clone(&instance_rc));
                            return Err(self.frozen_modification_error(&receiver, *position));
                        }
                        let mut instance = instance_rc.borrow_mut();
                        instance.set_var(name.clone(), value);
                        Ok(())
                    }
                    // An exception carries its own instance variables, so a
                    // user-defined subclass can set state on itself.
                    Some(Object::Exception(details)) => {
                        details
                            .borrow_mut()
                            .instance_vars
                            .insert(name.clone(), value);
                        Ok(())
                    }
                    Some(Object::Class(class)) => {
                        // Module/class-level instance variables stored as class vars
                        class.set_class_var(format!("@{}", name), value);
                        Ok(())
                    }
                    Some(Object::Module(module)) => {
                        module.set_class_var(format!("@{}", name), value);
                        Ok(())
                    }
                    // Immediates (Bool/Int/Float/Symbol/Nil/etc.) and other
                    // non-instance selves are always frozen; assigning an ivar
                    // raises FrozenError to match Ruby.
                    Some(other) => {
                        // The Ruby class name, not the internal tag: Ruby
                        // reports `NilClass`, never `Nil`.
                        let class_name =
                            crate::vm::native_methods::define_method::ruby_class_name(&other);
                        let msg = format!("can't modify frozen {}: {}", class_name, other);
                        let exc = Object::exception("FrozenError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(*position),
                            message: msg,
                        })
                    }
                    None => Err(MetorexError::runtime_error(
                        format!(
                            "Instance variable @{} can only be used within a method",
                            name
                        ),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::ClassVariable { name, position } => {
                // A class variable belongs to the class or module the code
                // was written in, and a write reaches the one furthest up the
                // chain that already holds it.
                if let Some(home) = self.class_variable_home() {
                    Self::class_var_owner(&home, name).set_class_var(name.clone(), value);
                    return Ok(());
                }
                Err(MetorexError::runtime_error(
                    "class variable access from toplevel".to_string(),
                    position_to_location(*position),
                ))
            }
            Expression::GlobalVariable {
                name: written,
                position,
            } => {
                // A global given a second name writes through to the first.
                let name = self
                    .global_aliases
                    .get(written)
                    .cloned()
                    .unwrap_or_else(|| written.clone());
                let value = self.special_global_value(written, &name, value, *position)?;
                // `$@` is the backtrace of the exception being handled, which
                // the assignment has set rather than a global of its own.
                if name == "@" {
                    return Ok(());
                }
                self.globals_mut().set_variable(name.clone(), value.clone());
                // A `trace_var` hook on this global runs with the new value.
                let name = name.clone();
                self.fire_global_trace(&name, &value, crate::lexer::Position::new(0, 0, 0))?;
                Ok(())
            }
            _ => Err(invalid_assignment_target_error(target)),
        }
    }

    /// Write `value` through `held[idx] = value` on a receiver already
    /// evaluated.
    fn store_single_subscript(
        &mut self,
        obj: Object,
        idx: Object,
        value: Object,
        position: &Position,
    ) -> Result<(), MetorexError> {
        match obj {
            // Every array index assignment goes through `[]=`, which
            // knows the index, start-and-length, and Range forms.
            Object::Array(_) => {
                self.send_to_object(obj.clone(), "[]=", vec![idx, value], *position)?;
                Ok(())
            }
            Object::Dict(dict_rc) => {
                // The environment names and values its variables in
                // text, so a write to it is read that way first.
                if self.dict_is_environment(&dict_rc) {
                    let environment = Object::Dict(Rc::clone(&dict_rc));
                    let given = vec![idx, value];
                    self.call_hash_method(&environment, "[]=", &given, *position)?;
                    return Ok(());
                }
                // Hash/Dict index assignment. Ruby allows any object
                // as a key, and a key carrying its own `#hash` takes
                // the slot that number and `#eql?` place it in.
                self.hash_store(&dict_rc, &idx, value.clone(), *position)?;
                self.record_environment_change(&dict_rc, &idx, &value);
                Ok(())
            }
            Object::Instance(instance_rc) => {
                // An instance of an Array or Hash subclass writes
                // through the storage it is backed by.
                let holder = Object::Instance(Rc::clone(&instance_rc));
                if let Some(backing) = crate::vm::native_methods::array_subclass_value(&holder)
                    .or_else(|| crate::vm::native_methods::hash_subclass_value(&holder))
                {
                    self.send_to_object(backing, "[]=", vec![idx, value], *position)?;
                    return Ok(());
                }
                // A `[]=` written on this one object answers before the one
                // its class carries.
                let class = Rc::clone(&instance_rc.borrow().class);
                let method = {
                    let instance = instance_rc.borrow();
                    instance
                        .find_singleton_method("[]=")
                        .or_else(|| {
                            instance
                                .singleton_class
                                .borrow()
                                .as_ref()
                                .and_then(|singleton| singleton.find_method("[]="))
                        })
                        .or_else(|| class.find_method("[]="))
                };
                if let Some(method) = method {
                    self.invoke_method(
                        class,
                        method,
                        Object::Instance(instance_rc),
                        vec![idx, value],
                        *position,
                    )?;
                    Ok(())
                } else if let Some(members) = crate::vm::native_methods::struct_members(&class) {
                    self.call_struct_instance_method(
                        &class,
                        &members,
                        &Object::Instance(instance_rc),
                        "[]=",
                        &[idx, value],
                        *position,
                    )?;
                    Ok(())
                } else if class.name() == "Thread" {
                    // A Thread keeps its locals under `t[:k] = v`,
                    // which the Thread method table writes. Going
                    // through it keeps one reading of the key for the
                    // subscript and the call.
                    self.call_thread_method(
                        &Object::Instance(Rc::clone(&instance_rc)),
                        "[]=",
                        &[idx, value],
                        *position,
                    )?;
                    Ok(())
                } else {
                    Err(MetorexError::runtime_error(
                        "No []= method defined on instance",
                        position_to_location(*position),
                    ))
                }
            }
            Object::Class(cls) | Object::Module(cls) => {
                // Dispatch to class/module-level `def self.[]=` (stored
                // under the `__class__` prefix).
                let receiver = Object::Class(Rc::clone(&cls));
                let key = "__class__[]=".to_string();
                if let Some(method) = cls.find_method(&key) {
                    self.invoke_method(
                        Rc::clone(&cls),
                        method,
                        receiver,
                        vec![idx, value],
                        *position,
                    )?;
                    Ok(())
                } else if self
                    .call_warning_methods(&cls, "[]=", &[idx.clone(), value.clone()], *position)?
                    .is_some()
                    || self
                        .call_class_methods(&cls, "[]=", &[idx, value], *position)?
                        .is_some()
                {
                    Ok(())
                } else {
                    Err(MetorexError::runtime_error(
                        "No []= method defined on class/module",
                        position_to_location(*position),
                    ))
                }
            }
            // `str[at] = text` writes into the string itself, which
            // every reference to it then reads.
            Object::String(_) => {
                let given = vec![idx, value];
                self.call_string_mutation(&obj, "[]=", &given, *position)?;
                Ok(())
            }
            // Nil receivers swallow `[]=` as a no-op so spec
            // fixtures using stub Thread.current (which is Nil)
            // can run `Thread.current[:in_autoload_rb] = true`
            // without crashing.
            Object::Nil => Ok(()),
            _ => Err(MetorexError::runtime_error(
                "Cannot index assign on this type",
                position_to_location(*position),
            )),
        }
    }

    /// Write `value` through the setter `setter_method` on a receiver already
    /// evaluated. `receiver` is the expression it was written as, which says
    /// whether the call names `self`.
    fn store_through_setter(
        &mut self,
        receiver: &Expression,
        receiver_obj: Object,
        setter_method: String,
        value: Object,
        position: &Position,
    ) -> Result<(), MetorexError> {
        // An instance of an Array or Hash subclass answers the
        // writers of the value it is backed by.
        if let Some(backing) = crate::vm::native_methods::array_subclass_value(&receiver_obj)
            .or_else(|| crate::vm::native_methods::hash_subclass_value(&receiver_obj))
        {
            let class = self.builtins().class_of(&backing);
            if self
                .call_native_method(
                    &class,
                    &backing,
                    &setter_method,
                    std::slice::from_ref(&value),
                    *position,
                )?
                .is_some()
            {
                return Ok(());
            }
        }
        // Look up the setter method and invoke it
        match receiver_obj {
            Object::Instance(instance_rc) => {
                let (class, method_obj) = {
                    let instance = instance_rc.borrow();
                    let class = instance.class.clone();
                    // A writer written on this one object with
                    // `def obj.name=` answers before the one its
                    // class carries, whether it was recorded on
                    // the instance or on its singleton class.
                    let method_obj = instance
                        .find_singleton_method(&setter_method)
                        .or_else(|| {
                            instance
                                .singleton_class
                                .borrow()
                                .as_ref()
                                .and_then(|singleton| singleton.find_method(&setter_method))
                        })
                        .or_else(|| instance.class.find_method(&setter_method));
                    (class, method_obj)
                }; // Borrow is dropped here

                if let Some(method) = method_obj {
                    // Visibility check: an explicit-receiver setter
                    // call (`obj.foo = …`) can only invoke public
                    // methods, mirroring `evaluate_method_call`.
                    let is_explicit_receiver = !crate::vm::method_lookup::names_self(receiver);
                    if is_explicit_receiver
                        && self.refuses_explicit_receiver(&class, &setter_method)
                    {
                        return Err(visibility_error(&class, &setter_method, *position));
                    }
                    self.invoke_method(
                        class,
                        method,
                        Object::Instance(Rc::clone(&instance_rc)),
                        vec![value],
                        *position,
                    )?;
                    Ok(())
                } else if instance_rc
                    .borrow()
                    .class
                    .find_method("method_missing")
                    .is_some()
                {
                    // A class that answers what it was not asked
                    // for decides what a setter with no method
                    // behind it means.
                    let name = Object::symbol(setter_method.clone());
                    self.send_to_object(
                        Object::Instance(Rc::clone(&instance_rc)),
                        "method_missing",
                        vec![name, value],
                        *position,
                    )?;
                    Ok(())
                } else {
                    // A setter the instance answers natively, such
                    // as an open file handle's `lineno=`, lives in
                    // the native table rather than the method map.
                    let receiver = Object::Instance(Rc::clone(&instance_rc));
                    let class = self.builtins().class_of(&receiver);
                    if self
                        .call_native_method(
                            &class,
                            &receiver,
                            &setter_method,
                            std::slice::from_ref(&value),
                            *position,
                        )?
                        .is_some()
                    {
                        return Ok(());
                    }
                    Err(MetorexError::runtime_error(
                        format!("Undefined setter method '{}'", setter_method),
                        position_to_location(*position),
                    ))
                }
            }
            Object::Class(class_rc) => {
                // `def self.name=` is stored the way every other
                // module-level method is, and one written in a
                // `class << self` body sits on the singleton
                // class, so both are looked in before a plain
                // instance method.
                if let Some(method) =
                    crate::vm::method_lookup::module_level_method(&class_rc, &setter_method)
                        .or_else(|| {
                            class_rc
                                .singleton_class_slot()
                                .clone()
                                .and_then(|singleton| singleton.find_method(&setter_method))
                        })
                        .or_else(|| class_rc.find_method(&setter_method))
                {
                    self.invoke_method(
                        Rc::clone(&class_rc),
                        method,
                        Object::Class(class_rc),
                        vec![value],
                        *position,
                    )?;
                    Ok(())
                } else {
                    // A writer the class carries natively, such as
                    // `Encoding.default_internal=`, answers before
                    // the assignment falls back to storing state.
                    if self
                        .call_class_methods(
                            &class_rc,
                            &setter_method,
                            std::slice::from_ref(&value),
                            *position,
                        )?
                        .is_some()
                    {
                        return Ok(());
                    }
                    // Store as class variable
                    class_rc
                        .set_class_var(format!("@{}", setter_method.trim_end_matches('=')), value);
                    Ok(())
                }
            }
            Object::Module(module_rc) => {
                // A writer written in a `class << self` body sits
                // on the singleton class, which is a third place
                // an assignment has to look.
                if let Some(method) =
                    crate::vm::method_lookup::module_level_method(&module_rc, &setter_method)
                        .or_else(|| {
                            module_rc
                                .singleton_class_slot()
                                .clone()
                                .and_then(|singleton| singleton.find_method(&setter_method))
                        })
                        .or_else(|| module_rc.find_method(&setter_method))
                {
                    self.invoke_method(
                        Rc::clone(&module_rc),
                        method,
                        Object::Module(module_rc),
                        vec![value],
                        *position,
                    )?;
                    Ok(())
                } else {
                    // A setter the module answers natively, such
                    // as `Process.maxgroups=`, lives in the native
                    // table rather than in the method map.
                    let receiver = Object::Module(Rc::clone(&module_rc));
                    if self
                        .call_module_methods(
                            &module_rc,
                            &receiver,
                            &setter_method,
                            std::slice::from_ref(&value),
                            *position,
                        )?
                        .is_some()
                    {
                        return Ok(());
                    }
                    module_rc
                        .set_class_var(format!("@{}", setter_method.trim_end_matches('=')), value);
                    Ok(())
                }
            }
            // For other receiver types (immediates: Bool/Int/Float/
            // Symbol/etc., or String), the user's reopened class
            // (`class TrueClass; …; end`) lives in globals, not in
            // `builtins.class_of`. Try the global class first so
            // user-defined accessors fire; fall back to the built-in
            // class otherwise.
            other => {
                let global_class_name = match &other {
                    Object::Bool(true) => Some("TrueClass"),
                    Object::Bool(false) => Some("FalseClass"),
                    Object::Nil => Some("NilClass"),
                    Object::Int(_) => Some("Integer"),
                    Object::Float(_) => Some("Float"),
                    Object::Symbol(_) => Some("Symbol"),
                    _ => None,
                };
                let target_class: Option<Rc<crate::class::Class>> = global_class_name
                    .and_then(|name| match self.globals().get(name) {
                        Some(Object::Class(c)) => Some(c),
                        _ => None,
                    })
                    .or_else(|| Some(self.builtins().class_of(&other)));

                if let Some(class) = target_class
                    && let Some(method) = class.find_method(&setter_method)
                {
                    self.invoke_method(Rc::clone(&class), method, other, vec![value], *position)?;
                    Ok(())
                } else if let Some((owner, method)) = self.lookup_method(&other, &setter_method) {
                    // An exception carries its own class, which is
                    // where a reopened `attr_accessor` lives.
                    self.invoke_method(owner, method, other, vec![value], *position)?;
                    Ok(())
                } else {
                    // A setter the class implements natively, such
                    // as the stream methods the String standing in
                    // for STDOUT answers, is reached last.
                    let class = self.builtins().class_of(&other);
                    let handled = self.call_native_method(
                        &class,
                        &other,
                        &setter_method,
                        std::slice::from_ref(&value),
                        *position,
                    )?;
                    match handled {
                        Some(_) => Ok(()),
                        None => Err(MetorexError::runtime_error(
                            format!(
                                "Cannot call setter method '{}' on {}",
                                setter_method,
                                other.type_name()
                            ),
                            position_to_location(*position),
                        )),
                    }
                }
            }
        }
    }

    /// Bind `name` on a namespace already evaluated, which is `Ns::Name =
    /// value`. Anything other than a class or module is refused by its type.
    fn store_scoped_constant(
        &mut self,
        ns: Object,
        name: &str,
        value: Object,
        position: &Position,
    ) -> Result<(), MetorexError> {
        let owner_for_hook = ns.clone();
        match ns {
            Object::Class(c) | Object::Module(c) => {
                // Rebinding a constant, or one with an autoload still
                // registered, warns unless `$VERBOSE` is nil.
                if self.autoload_const_access_depth == 0
                    && (c.get_class_var(name).is_some() || c.get_autoload(name).is_some())
                {
                    self.warn_already_initialized(&c, name, *position);
                }
                name_constant_value(&c, name, &value);
                // Explicit `Mod::X = value` outside of an autoload
                // load supersedes any pending autoload. (The
                // autoload trigger path manages its own autoload
                // bookkeeping after the load completes.)
                c.remove_autoload(name);
                // Object's constants are top-level constants —
                // publish so bare references resolve.
                if c.name() == "Object" {
                    self.globals_mut().set(name.to_string(), value.clone());
                }
                c.set_class_var(name, value);
                let assign_file = self
                    .reported_current_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                c.set_const_location(name, assign_file, position.line as i64);
                self.trigger_const_added_hook(owner_for_hook, name, *position)?;
                Ok(())
            }
            held => Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!(
                    "{} is not a class/module",
                    crate::vm::native_methods::array_methods::inspect_element(&held)
                ),
                *position,
            )),
        }
    }
}

/// Whether `target` is a constant-shaped assignment target that `left` reads,
/// which is the shape `CONST ||= value` desugars to.
pub(crate) fn constant_target_matches(target: &Expression, left: &Expression) -> bool {
    match (target, left) {
        (
            Expression::Identifier { name, .. },
            Expression::Identifier {
                name: left_name, ..
            },
        ) => name == left_name && name.starts_with(|c: char| c.is_ascii_uppercase()),
        (
            Expression::ScopeResolution { name, .. },
            Expression::ScopeResolution {
                name: left_name, ..
            },
        )
        | (
            Expression::TopLevelConstant { name, .. },
            Expression::TopLevelConstant {
                name: left_name, ..
            },
        ) => name == left_name,
        _ => false,
    }
}

/// Name an anonymous class or module on binding it to a constant. A named
/// namespace makes the name permanent and cascades into the anonymous modules
/// nested under it; an anonymous one only supplies a temporary name.
pub(crate) fn name_constant_value(namespace: &Rc<Class>, const_name: &str, value: &Object) {
    let (Object::Class(bound) | Object::Module(bound)) = value else {
        return;
    };
    let qualified = format!("{}::{}", namespace.inspect_name(), const_name);
    let namespace_name = namespace.ruby_name();
    if namespace_name.is_empty() || namespace_name.contains("#<") {
        bound.set_assigned_name_if_anonymous(&qualified);
    } else {
        bound.assign_name_recursive(&qualified);
    }
}

/// The NoMethodError for a private or protected method called with an
/// explicit receiver.
fn visibility_error(class: &Rc<Class>, name: &str, position: Position) -> MetorexError {
    let marking = if class.is_method_protected(name) && !class.is_method_private(name) {
        "protected"
    } else {
        "private"
    };
    let msg = format!(
        "{} method '{}' called for an instance of {}",
        marking,
        name,
        class.name()
    );
    let exc = Object::exception("NoMethodError", msg.clone());
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message: msg,
    }
}

/// The globals a program may read but not assign: the exception being
/// handled, the last child's status, the loaded features, the input stream
/// and its file, the load path, the switches the command line set, and the
/// parts of the last match, which only an alias can be written to.
const READ_ONLY_GLOBALS: [&str; 15] = [
    "&",
    "`",
    "'",
    "+",
    "!",
    "?",
    "\"",
    "LOADED_FEATURES",
    "<",
    "FILENAME",
    ":",
    "LOAD_PATH",
    "-a",
    "-l",
    "-p",
];

impl VirtualMachine {
    /// What a special global holds once `value` is assigned to it, or the
    /// error the assignment raises. `written` is the name the program used,
    /// which a message names, and `name` the global it reaches.
    fn special_global_value(
        &mut self,
        written: &str,
        name: &str,
        value: Object,
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        // The core library keeps `$FILENAME` up to date as ARGF moves from
        // file to file, which a program cannot do.
        let internal = self
            .current_source_file
            .as_deref()
            .is_some_and(|file| file.starts_with(crate::vm::INTERNAL_FILE_PREFIX));
        if READ_ONLY_GLOBALS.contains(&name) && !internal {
            let message = format!("${written} is a read-only variable");
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        match name {
            "~" => {
                let matchdata = matches!(&value, Object::Instance(instance)
                    if instance.borrow().class.name() == "MatchData");
                if !matches!(value, Object::Nil) && !matchdata {
                    let named = class_named(&value);
                    return Err(simple_exception(
                        "TypeError",
                        &format!("wrong argument type {named} (expected MatchData)"),
                        position,
                    ));
                }
                Ok(value)
            }
            "stdout" | "stderr" => {
                let class = self.builtins().class_of(&value);
                let writes = crate::vm::method_invocation::descends_from(&class, "IO")
                    || self
                        .lookup_method(&value, "write")
                        .is_some_and(|(_, method)| !method.is_undefined);
                if !writes {
                    let named = class_named(&value);
                    return Err(simple_exception(
                        "TypeError",
                        &format!("${written} must have write method, {named} given"),
                        position,
                    ));
                }
                Ok(value)
            }
            "/" | "\\" | "," | ";" => {
                let text = match &value {
                    Object::Nil | Object::String(_) => value.clone(),
                    // The field separator `split` falls back on may be a
                    // pattern.
                    Object::Regex(_, _) if name == ";" => value.clone(),
                    other => {
                        match crate::vm::native_methods::subclasses::string_subclass_value(other) {
                            Some(backing) if name == "/" => backing,
                            _ => {
                                return Err(simple_exception(
                                    "TypeError",
                                    &format!(
                                        "value of ${written} must be String{}",
                                        if name == ";" { " or Regexp" } else { "" }
                                    ),
                                    position,
                                ));
                            }
                        }
                    }
                };
                // Ruby has retired the separators, and says so once the
                // deprecated category is asked for.
                if !matches!(text, Object::Nil) && self.warning_category_enabled("deprecated") {
                    let message = format!(
                        "{}non-nil '${written}' is deprecated\n",
                        self.warning_prefix(0, position)
                    );
                    self.warn_through_warning_module(message, position)?;
                }
                // The record separator is held as a frozen plain String of
                // its own unless it was handed one.
                match &text {
                    Object::String(held)
                        if name == "/"
                            && !(held.is_frozen() && matches!(value, Object::String(_))) =>
                    {
                        let copy = Object::string(held.as_str().to_string());
                        if let Object::String(fresh) = &copy {
                            fresh.freeze();
                        }
                        Ok(copy)
                    }
                    _ => Ok(value),
                }
            }
            "." => match value {
                Object::Int(_) => Ok(value),
                Object::Float(number) => Ok(Object::Int(number as i64)),
                other => {
                    let converted = self.integer_for_global(&other, position)?;
                    Ok(converted)
                }
            },
            // The program's name is also what a process listing shows.
            "0" => match &value {
                Object::String(text) => {
                    self.set_process_title(&text.as_str());
                    Ok(value)
                }
                other if self.responds_to(other, "to_str") => {
                    let converted =
                        self.send_to_object(other.clone(), "to_str", Vec::new(), position)?;
                    if let Object::String(text) = &converted {
                        self.set_process_title(&text.as_str());
                    }
                    Ok(converted)
                }
                other => {
                    let named = self.conversion_name(other);
                    Err(simple_exception(
                        "TypeError",
                        &format!("no implicit conversion of {named} into String"),
                        position,
                    ))
                }
            },
            "VERBOSE" => Ok(match value {
                Object::Nil | Object::Bool(false) => value,
                _ => Object::Bool(true),
            }),
            "=" => {
                if self.warning_category_enabled("deprecated") {
                    let message = format!(
                        "{}variable $= is no longer effective; ignored\n",
                        self.warning_prefix(0, position)
                    );
                    self.warn_through_warning_module(message, position)?;
                }
                Ok(value)
            }
            "@" => {
                let raised = self.globals().get("!").unwrap_or(Object::Nil);
                if matches!(raised, Object::Nil) {
                    return Err(simple_exception("ArgumentError", "$! not set", position));
                }
                self.send_to_object(raised, "set_backtrace", vec![value.clone()], position)?;
                Ok(value)
            }
            _ => Ok(value),
        }
    }

    /// An Integer for a value that is not one: a Float truncated, or what
    /// `to_int` answers, which has to be an Integer.
    fn integer_for_global(
        &mut self,
        value: &Object,
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        if matches!(value, Object::Nil) {
            return Err(simple_exception(
                "TypeError",
                "no implicit conversion from nil to integer",
                position,
            ));
        }
        let named = self.conversion_name(value);
        let refused = || {
            simple_exception(
                "TypeError",
                &format!("no implicit conversion of {named} into Integer"),
                position,
            )
        };
        if !self.responds_to(value, "to_int") && self.lookup_method(value, "to_int").is_none() {
            return Err(refused());
        }
        match self.send_to_object(value.clone(), "to_int", Vec::new(), position)? {
            converted @ Object::Int(_) => Ok(converted),
            _ => Err(refused()),
        }
    }
}

/// The name of the class `value` is an instance of, as a message about it
/// spells it.
fn class_named(value: &Object) -> String {
    match value {
        Object::Instance(instance) => instance.borrow().class.name().to_string(),
        other => crate::vm::native_methods::define_method::ruby_class_name(other).to_string(),
    }
}
