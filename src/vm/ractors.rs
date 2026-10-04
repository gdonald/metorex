// What a Ractor other than the main one may not reach: most globals, class
// variables, and constants holding objects that are not shareable.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;

/// Where a thread keeps the Ractor it belongs to. A thread with none belongs
/// to the main Ractor.
pub(crate) const RACTOR_VAR: &str = "__thread_ractor";

/// The globals every Ractor has a copy of, or that belong to a frame or a
/// thread, which a Ractor other than the main one may read and write.
const GLOBALS_EVERY_RACTOR_HOLDS: &[&str] = &[
    "stdin", "stdout", "stderr", "VERBOSE", "DEBUG", "~", "_", "!", "@", "&", "`", "'", "+", "/",
    "$", ">", "-w", "-v", "-d", "-0", "-i", "-l", "-p", "-a", "?",
];

impl VirtualMachine {
    /// Whether the code running now belongs to a Ractor other than the main
    /// one.
    pub(crate) fn in_a_non_main_ractor(&self) -> bool {
        let Some(Object::Instance(thread)) = self.thread_current_stack.last() else {
            return false;
        };
        matches!(thread.borrow().get_var(RACTOR_VAR), Some(held) if !matches!(held, Object::Nil))
    }

    /// Refuse a global a Ractor other than the main one may not reach. Code
    /// of the interpreter's own reads what it needs, as Ruby's C code does.
    pub(crate) fn refuse_global_in_ractor(
        &self,
        name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if position.prelude
            || !self.in_a_non_main_ractor()
            || GLOBALS_EVERY_RACTOR_HOLDS.contains(&name)
            || (name != "0" && name.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return Ok(());
        }
        Err(isolation_error(
            &format!("can not access global variable ${name} from non-main Ractor"),
            position,
        ))
    }

    /// Refuse a class variable to a Ractor other than the main one.
    pub(crate) fn refuse_class_variable_in_ractor(
        &self,
        name: &str,
        owner: &std::rc::Rc<crate::class::Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if position.prelude || !self.in_a_non_main_ractor() {
            return Ok(());
        }
        Err(isolation_error(
            &format!(
                "can not access class variables from non-main Ractors (@@{} from {})",
                name.trim_start_matches('@'),
                owner.ruby_name()
            ),
            position,
        ))
    }

    /// Refuse a constant holding an object that is not shareable to a Ractor
    /// other than the main one. `named` is the constant's full name.
    pub(crate) fn refuse_unshareable_constant(
        &mut self,
        named: &str,
        value: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        if position.prelude || !self.in_a_non_main_ractor() || self.is_shareable(value, position)? {
            return Ok(());
        }
        // Ruby writes the word in lower case for a constant read inside a
        // method.
        let who = if self.def_scope_stack.is_empty() {
            "Ractor"
        } else {
            "ractor"
        };
        Err(isolation_error(
            &format!("can not access non-shareable objects in constant {named} by non-main {who}."),
            position,
        ))
    }

    /// Refuse setting a constant to an object that is not shareable from a
    /// Ractor other than the main one.
    pub(crate) fn refuse_unshareable_constant_assignment(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        if position.prelude || !self.in_a_non_main_ractor() || self.is_shareable(value, position)? {
            return Ok(());
        }
        Err(isolation_error(
            "can not set constants with non-shareable objects by non-main Ractors",
            position,
        ))
    }

    /// Whether `value` may be handed between Ractors as it is, which
    /// `Ractor.shareable?` decides.
    fn is_shareable(&mut self, value: &Object, position: Position) -> Result<bool, MetorexError> {
        if matches!(
            value,
            Object::Nil
                | Object::Bool(_)
                | Object::Int(_)
                | Object::Float(_)
                | Object::Symbol(_)
                | Object::Class(_)
                | Object::Module(_)
        ) {
            return Ok(true);
        }
        let ractor = self.globals().get("Ractor").unwrap_or(Object::Nil);
        let answered = self.send_to_object(ractor, "shareable?", vec![value.clone()], position)?;
        Ok(answered.is_truthy())
    }
}

impl VirtualMachine {
    /// The name of the class or module a bare constant written here is read
    /// from: the first scope open around the code, or one of its ancestors,
    /// that holds it, and Object otherwise.
    pub(crate) fn constant_owner_name(&self, name: &str) -> String {
        let nesting: Vec<std::rc::Rc<crate::class::Class>> = match self.method_nesting_stack.last()
        {
            Some(nesting) if !nesting.is_empty() => nesting.clone(),
            _ => self.def_scope_stack.iter().rev().cloned().collect(),
        };
        if let Some(scope) = nesting
            .iter()
            .find(|scope| scope.get_class_var(name).is_some())
        {
            return scope.ruby_name();
        }
        let mut cursor = nesting.first().cloned();
        while let Some(class) = cursor {
            if class.get_class_var(name).is_some() {
                return class.ruby_name();
            }
            cursor = class.superclass();
        }
        "Object".to_string()
    }
}

impl VirtualMachine {
    /// Note that `moved` was sent to another Ractor with `move: true`, so no
    /// method may be called on it here any more.
    pub(crate) fn mark_moved(&mut self, moved: Object) {
        if let Some(address) = moved_address(&moved) {
            self.moved_objects.1.insert(address);
            self.moved_objects.0.push(moved);
        }
    }

    /// Whether `receiver` was moved to another Ractor.
    pub(crate) fn was_moved(&self, receiver: &Object) -> bool {
        !self.moved_objects.1.is_empty()
            && moved_address(receiver)
                .is_some_and(|address| self.moved_objects.1.contains(&address))
    }

    /// Refuse to write out an object moved to another Ractor, which answers
    /// none of the methods the interpreter would ask it for its text.
    pub(crate) fn refuse_moved_conversion(
        &self,
        held: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if !self.was_moved(held) {
            return Ok(());
        }
        Err(crate::vm::errors::simple_exception(
            "NoMethodError",
            &format!("undefined method '{method_name}' for an instance of Ractor::MovedObject"),
            position,
        ))
    }

    /// Refuse a method called on an object moved to another Ractor.
    pub(crate) fn refuse_moved_receiver(
        &self,
        receiver: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        if !self.was_moved(receiver) {
            return Ok(());
        }
        Err(crate::vm::errors::simple_exception(
            "Ractor::MovedError",
            "can not send any methods to a moved object",
            position,
        ))
    }
}

/// The address that names an object a Ractor can move.
fn moved_address(held: &Object) -> Option<usize> {
    match held {
        Object::Instance(instance) => Some(std::rc::Rc::as_ptr(instance) as usize),
        other => VirtualMachine::collection_address(other),
    }
}

fn isolation_error(message: &str, position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("Ractor::IsolationError", message, position)
}
