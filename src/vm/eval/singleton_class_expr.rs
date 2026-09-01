// Singleton-class expression support: `class << target; body; end`.
//
// - Resolves (or lazily materializes) the singleton class for the target.
// - Executes `body` with the singleton class as `self` and as the enclosing
//   lexical scope, so method defs inside land on the singleton class.
// - Returns the last expression value from the body (or nil).

use std::rc::Rc;

use crate::ast::{Expression, Statement};
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

/// Key used to intern the singleton class of a value-kind receiver that
/// has no per-object storage slot of its own (Nil / Bool / Int / Float /
/// Symbol / String). Returns None for receivers that *do* own a slot —
/// those cases are handled directly from the object.
fn primitive_singleton_key(receiver: &Object) -> Option<String> {
    match receiver {
        Object::Nil => Some("__nil__".to_string()),
        Object::Bool(true) => Some("__true__".to_string()),
        Object::Bool(false) => Some("__false__".to_string()),
        Object::Int(_) | Object::Float(_) => None,
        _ => None,
    }
}

/// Display-name the singleton class should carry. Ruby prints singleton
/// classes as `#<Class:<target-inspect>>`; we approximate with a stable
/// tag so ancestors/inspect output is predictable.
fn singleton_class_display_name(receiver: &Object) -> String {
    match receiver {
        Object::Nil => "#<Class:NilClass>".to_string(),
        Object::Bool(true) => "#<Class:TrueClass>".to_string(),
        Object::Bool(false) => "#<Class:FalseClass>".to_string(),
        Object::Class(c) => format!("#<Class:{}>", c.inspect_name()),
        Object::Module(m) => format!("#<Class:{}>", m.inspect_name()),
        // The attached object's own display, so the two agree.
        Object::Instance(_) => format!("#<Class:{}>", receiver),
        _ => "#<Class:Object>".to_string(),
    }
}

impl VirtualMachine {
    /// Look up (and, if necessary, create) the singleton class of `receiver`.
    /// The singleton class records its attached object via the `__attached__`
    /// class-variable so `attached_object` can return it later.
    pub(crate) fn singleton_class_of(&mut self, receiver: &Object) -> Rc<Class> {
        match receiver {
            Object::Instance(inst_rc) => {
                let inst = inst_rc.borrow();
                if let Some(existing) = inst.singleton_class.borrow().clone() {
                    return existing;
                }
                let superclass = Some(Rc::clone(&inst.class));
                let sc = Rc::new(Class::new(
                    singleton_class_display_name(receiver),
                    superclass,
                ));
                sc.set_class_var("__singleton__", Object::Bool(true));
                sc.set_class_var("__attached__", receiver.clone());
                *inst.singleton_class.borrow_mut() = Some(Rc::clone(&sc));
                sc
            }
            Object::Class(class_rc) | Object::Module(class_rc) => {
                if let Some(existing) = class_rc.singleton_class_slot().clone() {
                    return existing;
                }
                let is_module = matches!(receiver, Object::Module(_));
                // Per Ruby: the singleton class of a Class K has K's parent's
                // singleton class as its superclass (not K's parent itself).
                // Build that chain on demand by recursing through `superclass`.
                // For BasicObject (no superclass), the singleton inherits from
                // `Class` itself, which gives the chain Class → Module → Object
                // → Kernel → BasicObject when walking a class's singleton
                // ancestors. A standalone module's singleton inherits from
                // `Module`, matching MRI. Other module-kind classes with no
                // superclass stay ungrounded to avoid unbounded lookup loops.
                let parent_singleton = match class_rc.superclass() {
                    Some(sup) => Some(self.singleton_class_of(&Object::Class(sup))),
                    None if class_rc.name() == "BasicObject" => match self.globals().get("Class") {
                        Some(Object::Class(c)) => Some(c),
                        _ => None,
                    },
                    None if is_module => match self.globals().get("Module") {
                        Some(Object::Class(c)) => Some(c),
                        _ => None,
                    },
                    None => None,
                };
                let sc = Rc::new(Class::new(
                    singleton_class_display_name(receiver),
                    parent_singleton,
                ));
                sc.set_class_var("__singleton__", Object::Bool(true));
                sc.set_class_var("__attached__", receiver.clone());
                class_rc.set_singleton_class(Rc::clone(&sc));
                sc
            }
            other => {
                let key = primitive_singleton_key(other).unwrap_or_else(|| match other {
                    // An exception is a reference type, so each one gets its
                    // own singleton class rather than sharing one per kind.
                    Object::Exception(details) => {
                        format!("__exception_{:p}", Rc::as_ptr(details))
                    }
                    _ => format!("__other_{}", other.type_name()),
                });
                if let Some(existing) = self.primitive_singleton_classes.get(&key) {
                    return Rc::clone(existing);
                }
                let sc = Rc::new(Class::new(singleton_class_display_name(other), None));
                sc.set_class_var("__singleton__", Object::Bool(true));
                sc.set_class_var("__attached__", other.clone());
                self.primitive_singleton_classes.insert(key, Rc::clone(&sc));
                sc
            }
        }
    }

    /// The singleton class a value-kind receiver already has, without
    /// creating one. Used by method lookup, which must not materialize a
    /// singleton class just by asking.
    pub(crate) fn existing_singleton_class(&self, receiver: &Object) -> Option<Rc<Class>> {
        let key = primitive_singleton_key(receiver).or_else(|| match receiver {
            Object::Exception(details) => Some(format!("__exception_{:p}", Rc::as_ptr(details))),
            Object::Instance(_) | Object::Class(_) | Object::Module(_) => None,
            other => Some(format!("__other_{}", other.type_name())),
        })?;
        self.primitive_singleton_classes.get(&key).map(Rc::clone)
    }

    /// Evaluate `class << target; body; end`.
    /// Materializes the singleton class, runs the body with it as the
    /// enclosing lexical scope and `self`, and returns the body's last value.
    pub(crate) fn evaluate_singleton_class_expression(
        &mut self,
        target: &Expression,
        assign_to: Option<&Expression>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let target_obj = self.evaluate_expression(target)?;
        // `class << @receiver = Object.new` stores the object first, so the
        // body's `def`s land on the same one the name now holds.
        if let Some(destination) = assign_to {
            self.assign_value(destination, target_obj.clone())?;
        }
        let singleton_cls = self.singleton_class_of(&target_obj);

        // Enter the singleton class as both the lexical def scope and `self`
        // so nested `def` lands on it.
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define("self".to_string(), Object::Class(Rc::clone(&singleton_cls)));
        self.def_scope_stack.push(Rc::clone(&singleton_cls));

        // Apply class-body semantics (attr_*, def, include, etc.) to the
        // singleton class itself.
        // The body's own last value is the result, so `class << x; self; end`
        // evaluates to the singleton class. Re-evaluating the last statement
        // here instead would run its side effects a second time.
        let apply_result = self.apply_class_body(&singleton_cls, body, position);
        let last = match &apply_result {
            Ok(value) => value.clone(),
            Err(_) => Object::Nil,
        };

        self.def_scope_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }

        apply_result?;
        Ok(last)
    }
}
