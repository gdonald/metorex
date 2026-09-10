//! `Module#private_constant`, `Module#public_constant`, and
//! `Module#deprecate_constant`, plus the `Warning[]` category switches that
//! decide whether a deprecation warning is emitted.

use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    /// Dispatch the constant-visibility methods for a class or module
    /// receiver. Returns `Ok(None)` when `method_name` is not one of them.
    pub(crate) fn call_constant_visibility_methods(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let class_rc = match receiver {
            Object::Class(c) | Object::Module(c) => c,
            _ => return Ok(None),
        };

        match method_name {
            "private_constant" | "public_constant" => {
                let make_private = method_name == "private_constant";
                let names = constant_names(self, method_name, arguments, position)?;
                require_own_constants(class_rc, &names, position)?;
                for name in names {
                    if make_private {
                        class_rc.mark_private_constant(name);
                    } else {
                        class_rc.unmark_private_constant(&name);
                    }
                }
                Ok(Some(Object::Nil))
            }
            "deprecate_constant" => {
                let names = constant_names(self, method_name, arguments, position)?;
                require_own_constants(class_rc, &names, position)?;
                for name in names {
                    class_rc.mark_deprecated_constant(name);
                }
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }

    /// Emit MRI's deprecation warning for `Owner::NAME` when the constant is
    /// marked deprecated and the `:deprecated` warning category is on.
    pub(crate) fn warn_deprecated_constant(
        &mut self,
        class_rc: &Rc<Class>,
        name: &str,
        position: Position,
    ) {
        if !class_rc.is_deprecated_constant(name) || !self.warning_category_enabled("deprecated") {
            return;
        }
        let message = format!(
            "warning: constant {}::{} is deprecated",
            constant_owner(class_rc),
            name
        );
        self.emit_warning_to_stderr(&message, position);
    }

    /// Turn a warning category on, which `-w` does for the deprecation
    /// warnings a plain run keeps quiet.
    pub fn enable_warning_category(&mut self, category: &str) {
        if let Some(Object::Module(warning) | Object::Class(warning)) =
            self.globals().get("Warning")
        {
            warning.set_class_var(category_key(category), Object::Bool(true));
        }
    }

    /// `Warning[:category]` — a category no one has set reads as its own
    /// default, and one Ruby does not know reads as off.
    pub(crate) fn warning_category_enabled(&self, category: &str) -> bool {
        match self.warning_category_slot(category) {
            Some(Object::Bool(enabled)) => enabled,
            _ => category == "experimental",
        }
    }

    /// The stored `Warning[:category]` value, if the `Warning` module and the
    /// category are both present.
    fn warning_category_slot(&self, category: &str) -> Option<Object> {
        match self.globals().get("Warning") {
            Some(Object::Module(warning)) | Some(Object::Class(warning)) => {
                warning.get_class_var(&category_key(category))
            }
            _ => None,
        }
    }

    /// `Warning[]` and `Warning[]=`. Returns `Ok(None)` for any other
    /// receiver or method so normal dispatch continues.
    pub(crate) fn call_warning_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if class_rc.name() != "Warning" {
            return Ok(None);
        }
        // `Warning[]`/`Warning[]=` are native, so they are invisible to the
        // generic `respond_to?`. Other names fall through to it.
        if matches!(method_name, "respond_to?" | "respond_to_missing?") {
            let named = match arguments.first() {
                Some(Object::Symbol(name) | Object::String(name)) => name.as_str().to_string(),
                _ => return Ok(None),
            };
            return Ok(match named.as_str() {
                "[]" | "[]=" | "categories" => Some(Object::Bool(true)),
                _ => None,
            });
        }
        // `Warning.categories` — every category Ruby knows, in the order it
        // lists them.
        if method_name == "categories" && arguments.is_empty() {
            let categories = WARNING_CATEGORIES
                .iter()
                .map(|name| Object::symbol((*name).to_string()))
                .collect();
            return Ok(Some(Object::array(categories)));
        }
        if !matches!(method_name, "[]" | "[]=") {
            return Ok(None);
        }
        let category = self.warning_category_argument(arguments.first(), position)?;
        match method_name {
            "[]" => Ok(Some(Object::Bool(self.warning_category_enabled(&category)))),
            _ => {
                let value = arguments.get(1).cloned().unwrap_or(Object::Nil);
                let enabled = crate::vm::utils::is_truthy(&value);
                class_rc.set_class_var(category_key(&category), Object::Bool(enabled));
                Ok(Some(value))
            }
        }
    }

    /// The category `Warning[]` was indexed with. Ruby takes a Symbol and
    /// nothing else, and only one it knows.
    fn warning_category_argument(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<String, MetorexError> {
        let Some(Object::Symbol(name)) = argument else {
            let given = argument.cloned().unwrap_or(Object::Nil);
            let message = format!(
                "no implicit conversion of {} into Symbol",
                self.builtins().class_of(&given).name()
            );
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: position_to_location(position),
                message,
            });
        };
        if !WARNING_CATEGORIES.contains(&&*name.as_str()) {
            let message = format!("unknown category: {}", name);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        Ok(name.as_str().to_string())
    }
}

/// The categories `Warning.categories` reports, which are the only ones
/// `Warning[]` accepts.
const WARNING_CATEGORIES: &[&str] = &[
    "deprecated",
    "experimental",
    "performance",
    "strict_unused_block",
];

/// Class-variable slot backing one `Warning[]` category.
fn category_key(category: &str) -> String {
    format!("__category_{}", category)
}

/// Name to print on the left of `::` in constant messages. Anonymous modules
/// fall back to their `#<Module:0x…>` inspect form.
fn constant_owner(class_rc: &Rc<Class>) -> String {
    let name = class_rc.ruby_name();
    if name.is_empty() {
        class_rc.inspect_name()
    } else {
        name
    }
}

/// Coerce the argument list into constant names. Every one of these methods
/// accepts any mix of Symbols and Strings.
fn constant_names(
    vm: &mut VirtualMachine,
    method_name: &str,
    arguments: &[Object],
    position: Position,
) -> Result<Vec<String>, MetorexError> {
    arguments
        .iter()
        .map(|arg| vm.coerce_method_name(arg, method_name, position))
        .collect()
}

/// Every name must be a constant of `class_rc` itself. Ruby's constant
/// visibility methods do not reach inherited constants, and name one that is
/// missing with a NameError.
fn require_own_constants(
    class_rc: &Rc<Class>,
    names: &[String],
    position: Position,
) -> Result<(), MetorexError> {
    for name in names {
        if class_rc.get_class_var(name).is_none() && class_rc.get_autoload(name).is_none() {
            let message = format!(
                "constant {}::{} not defined",
                constant_owner(class_rc),
                name
            );
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
    }
    Ok(())
}
