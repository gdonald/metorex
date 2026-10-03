//! Defining classes and modules from C, at the top level or under another
//! module. A name already defined is reopened, and the constant is looked up
//! the way Ruby code would, so an autoload registered for it runs first.

use super::exports::{class_from, text};
use super::handles::{Value, to_object, to_value};
use super::symbols::symbol_name;
use super::{interpreter, or_raise};
use crate::class::Class;
use crate::error::MetorexError;
use crate::object::Object;
use crate::vm::VirtualMachine;
use std::ffi::c_char;
use std::rc::Rc;

/// Object, which is where a top-level class or module is a constant.
fn object_class() -> Object {
    interpreter()
        .globals()
        .get("Object")
        .expect("Object is defined before any extension loads")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_class(name: *const c_char, superclass: Value) -> Value {
    define_class(object_class(), &text(name), superclass)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_class_under(
    outer: Value,
    name: *const c_char,
    superclass: Value,
) -> Value {
    define_class(to_object(outer), &text(name), superclass)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_class_id_under(
    outer: Value,
    name: Value,
    superclass: Value,
) -> Value {
    define_class(to_object(outer), &symbol_name(name), superclass)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_module(name: *const c_char) -> Value {
    define_module(object_class(), &text(name))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_module_under(outer: Value, name: *const c_char) -> Value {
    define_module(to_object(outer), &text(name))
}

fn define_class(outer: Object, name: &str, superclass: Value) -> Value {
    let class = or_raise(interpreter().define_class_from_c(outer, name, superclass));
    to_value(&Object::Class(class))
}

fn define_module(outer: Object, name: &str) -> Value {
    to_value(&or_raise(interpreter().define_module_from_c(outer, name)))
}

impl VirtualMachine {
    /// The constant `outer` holds under `name` itself, after any autoload
    /// registered for it has run, or None when it holds none. A name Ruby
    /// code could not spell is read from `outer` directly.
    fn constant_held_by(
        &mut self,
        outer: &Object,
        name: &str,
    ) -> Result<Option<Object>, MetorexError> {
        if !crate::vm::native_methods::is_valid_constant_name(name) {
            let (Object::Class(held_by) | Object::Module(held_by)) = outer else {
                return Ok(None);
            };
            return Ok(held_by.get_class_var(name));
        }
        let position = super::called_from();
        let arguments = vec![Object::symbol(name), Object::Bool(false)];
        let defined =
            self.send_to_object(outer.clone(), "const_defined?", arguments.clone(), position)?;
        if !defined.is_truthy() {
            return Ok(None);
        }
        self.send_to_object(outer.clone(), "const_get", arguments, position)
            .map(Some)
    }

    /// The name of the class of what a constant holds, which a TypeError
    /// gives in parentheses.
    fn held_class_name(&mut self, held: &Object) -> Result<String, MetorexError> {
        let class = self.send_to_object(held.clone(), "class", Vec::new(), super::called_from())?;
        Ok(class.to_string())
    }

    /// The class `outer` holds as `name`, made a subclass of `superclass`
    /// when it holds nothing, as MRI's `rb_define_class_id_under` does. A
    /// `superclass` of 0 is C's NULL.
    pub(super) fn define_class_from_c(
        &mut self,
        outer: Object,
        name: &str,
        superclass: Value,
    ) -> Result<Rc<Class>, MetorexError> {
        let position = super::called_from();
        let at_top = outer == object_class();
        let full_name = if at_top {
            name.to_string()
        } else {
            format!("{}::{}", outer, name)
        };
        match self.constant_held_by(&outer, name)? {
            Some(Object::Class(existing)) => {
                let same_superclass = superclass != 0
                    && existing
                        .superclass()
                        .is_some_and(|held| Object::Class(held) == to_object(superclass));
                if same_superclass {
                    return Ok(existing);
                }
                let message = if at_top {
                    format!("superclass mismatch for class {}", name)
                } else {
                    let held = existing.superclass().map_or(Object::Nil, Object::Class);
                    format!(
                        "superclass mismatch for class {} ({} is given but was {})",
                        full_name,
                        to_object(superclass),
                        held
                    )
                };
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ))
            }
            Some(other) => {
                let class_name = self.held_class_name(&other)?;
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("{} is not a class ({})", full_name, class_name),
                    position,
                ))
            }
            None => {
                if superclass == 0 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("no super class for '{}'", full_name),
                        position,
                    ));
                }
                let Object::Class(class) = to_object(super::subclasses::rb_class_new(superclass))
                else {
                    unreachable!("rb_class_new answers a class")
                };
                let Object::Class(superclass) = to_object(superclass) else {
                    unreachable!("rb_class_new accepted only a class")
                };
                self.bind_class_constant(&outer, name, &class, position)?;
                self.trigger_inherited_hook(&superclass, Rc::clone(&class), position)?;
                Ok(class)
            }
        }
    }

    /// Binds `name` in `outer` to `class`, including a name Ruby code could
    /// not spell as a constant.
    fn bind_class_constant(
        &mut self,
        outer: &Object,
        name: &str,
        class: &Rc<Class>,
        position: crate::lexer::Position,
    ) -> Result<(), MetorexError> {
        let value = Object::Class(Rc::clone(class));
        if crate::vm::native_methods::is_valid_constant_name(name) {
            let arguments = vec![Object::symbol(name), value];
            self.send_to_object(outer.clone(), "const_set", arguments, position)?;
            return Ok(());
        }
        let (Object::Class(outer_class) | Object::Module(outer_class)) = outer else {
            unreachable!("a class is defined under a class or module")
        };
        self.assign_constant(outer_class, name, value, position)
    }

    fn define_module_from_c(&mut self, outer: Object, name: &str) -> Result<Object, MetorexError> {
        let position = super::called_from();
        match self.constant_held_by(&outer, name)? {
            Some(existing @ Object::Module(_)) => Ok(existing),
            Some(other) => {
                let class_name = self.held_class_name(&other)?;
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("{} is not a module ({})", name, class_name),
                    position,
                ))
            }
            None => {
                let module = Object::Module(Class::new_module(""));
                self.send_to_object(
                    outer,
                    "const_set",
                    vec![Object::symbol(name), module.clone()],
                    position,
                )?;
                Ok(module)
            }
        }
    }
}

/// Binds a constant as `rb_define_const` does: a name Ruby code could not
/// spell as a constant is warned about and bound all the same.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_const(module: Value, name: *const c_char, value: Value) {
    let name = text(name);
    let module_class = class_from(module);
    let value = to_object(value);
    let machine = interpreter();
    let position = super::called_from();
    let valid = crate::vm::native_methods::is_valid_constant_name(&name);
    if !valid {
        let warning = format!(
            "{}rb_define_const: invalid name '{}' for constant\n",
            machine.warning_prefix(0, position),
            name
        );
        or_raise(machine.warn_through_warning_module(warning, position));
    }
    // A frozen module refuses the name before `const_set` looks at it.
    if valid || module_class.is_frozen() {
        let arguments = vec![Object::symbol(name), value];
        super::calls::call(to_object(module), "const_set", arguments);
        return;
    }
    or_raise(machine.assign_constant(&module_class, &name, value, position));
}
