// What an object rooted at BasicObject answers, as against one
// rooted at Object.

use super::*;

impl VirtualMachine {
    /// Whether a name is private on a class or anywhere above it, which is
    /// what keeps it out of the list of methods an object answers to.
    pub(crate) fn method_is_private_anywhere(
        &self,
        holder: &std::rc::Rc<crate::class::Class>,
        name: &str,
    ) -> bool {
        let mut cursor = Some(std::rc::Rc::clone(holder));
        while let Some(current) = cursor {
            if current.find_own_method(name).is_some() || current.is_method_private(name) {
                return current.is_method_private(name);
            }
            for mixin in current.mixin_chain() {
                if mixin.find_own_method(name).is_some() || mixin.is_method_private(name) {
                    return mixin.is_method_private(name);
                }
            }
            cursor = current.superclass();
        }
        false
    }
}

impl VirtualMachine {
    /// A string kept under a name, built once and answered every time after.
    pub(crate) fn memoized_text(&mut self, slot: &str, text: &str) -> Object {
        if let Some(held) = self.globals().get(slot) {
            return held;
        }
        let made = Object::string(text);
        // One string answered over and over is frozen, since a program that
        // changed it would change what everyone else is handed.
        if let Object::String(held) = &made {
            held.freeze();
        }
        self.globals_mut().set(slot, made.clone());
        made
    }
}

impl VirtualMachine {
    /// Whether an object is one of the encodings, which metorex holds as a
    /// class of its own under `Encoding`.
    pub(crate) fn names_an_encoding(&self, receiver: &Object) -> bool {
        let Object::Class(held) = receiver else {
            return false;
        };
        let Some(parent) = held.superclass() else {
            return false;
        };
        parent.name() == "Encoding"
    }
}

/// Whether BasicObject itself defines `method_name`. Its instance methods are
/// the only ones an object outside Object's ancestry answers.
pub(crate) fn basic_object_answers(method_name: &str) -> bool {
    matches!(
        method_name,
        "!" | "=="
            | "!="
            | "__send__"
            | "__id__"
            | "equal?"
            | "instance_eval"
            | "instance_exec"
            | "initialize"
            | "method_missing"
            | "singleton_method_added"
            | "singleton_method_removed"
            | "singleton_method_undefined"
    )
}

/// Whether the receiver's own class chain defines `method_name`. A class
/// rooted at BasicObject may still take one of Kernel's methods on, which is
/// what `define_method(:respond_to?, Kernel.instance_method(:respond_to?))`
/// does, and the native implementation behind it answers then.
pub(crate) fn class_defines(receiver: &Object, method_name: &str) -> bool {
    let Object::Instance(instance) = receiver else {
        return false;
    };
    let class = std::rc::Rc::clone(&instance.borrow().class);
    class
        .find_method(method_name)
        .is_some_and(|method| !method.is_undefined)
}
