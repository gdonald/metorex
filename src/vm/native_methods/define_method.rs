//! `Module#define_method` — installs an instance method from a block, a Proc,
//! a bound `Method`, or an `UnboundMethod`.

use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{BlockStatement, Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    /// Implements `Module#define_method` for a class or module receiver.
    ///
    /// `arguments[0]` is the new method's name; `arguments[1]`, when present,
    /// supplies the body and takes precedence over a literal block.
    pub(crate) fn module_define_method(
        &mut self,
        class_rc: &Rc<Class>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let pending_block = self.pending_block.take();

        if arguments.is_empty() {
            return Err(raise(
                "ArgumentError",
                "wrong number of arguments (given 0, expected 1..2)",
                position,
            ));
        }
        let method_name = self.coerce_method_name(&arguments[0], "define_method", position)?;

        if class_rc.is_frozen() {
            return Err(raise(
                "FrozenError",
                &format!("can't modify frozen class: {}", class_rc.inspect_name()),
                position,
            ));
        }

        // An explicit second argument wins over a literal block, so
        // `define_method(:m, some_proc) { ... }` uses `some_proc`.
        let body_source = match arguments.get(1) {
            Some(object) => object.clone(),
            None => match pending_block {
                Some(object) => object,
                None => {
                    return Err(raise(
                        "ArgumentError",
                        "tried to create Proc object without a block",
                        position,
                    ));
                }
            },
        };

        let method = match &body_source {
            Object::Block(block) => self.method_from_block(&method_name, block),
            Object::Method(source) => {
                self.rebindable_method(class_rc, &method_name, source, position)?
            }
            other => {
                return Err(raise(
                    "TypeError",
                    &format!(
                        "wrong argument type {} (expected Proc/Method/UnboundMethod)",
                        ruby_class_name(other)
                    ),
                    position,
                ));
            }
        };

        class_rc.define_method(&method_name, Rc::new(method));
        self.apply_define_method_visibility(class_rc, &method_name);

        let hook = if class_rc.is_singleton_class() {
            "singleton_method_added"
        } else {
            "method_added"
        };
        self.invoke_class_hook(class_rc, hook, &method_name, position)?;
        if class_rc.current_visibility() == crate::vm::native_methods::MODULE_FUNCTION_VISIBILITY {
            self.copy_to_module_function(class_rc, &method_name, position)?;
        }

        Ok(Object::Symbol(Rc::new(method_name)))
    }

    /// Implements `Object#define_singleton_method`: the method is installed on
    /// the receiver's singleton class, so only that object responds to it.
    pub(crate) fn object_define_singleton_method(
        &mut self,
        receiver: &Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let singleton = self.singleton_class_of(receiver);
        self.module_define_method(&singleton, arguments, position)
    }

    /// Convert a block/Proc into a method body, carrying over its parameter
    /// shape and the variables it closed over.
    fn method_from_block(&mut self, method_name: &str, block: &Rc<BlockStatement>) -> Method {
        let mut regular_params: Vec<String> = Vec::new();
        let mut variadic_param: Option<(usize, String)> = None;
        let mut block_parameter: Option<String> = None;
        for (index, param) in block.parameters.iter().enumerate() {
            if param == crate::object::TRAILING_COMMA_PARAM {
                // A method built from `{ |a,| }` keeps the plain `|a|` arity;
                // only procs destructure on a trailing comma.
                continue;
            }
            if let Some(name) = param.strip_prefix('*') {
                variadic_param = Some((index, name.to_string()));
                regular_params.push(name.to_string());
            } else if let Some(name) = param.strip_prefix('&') {
                block_parameter = Some(name.to_string());
            } else {
                regular_params.push(param.clone());
            }
        }

        let mut method = Method::new(method_name.to_string(), regular_params, block.body.clone());
        method.variadic_param = variadic_param;
        method.block_parameter = block_parameter;
        // Optional block params (`|a, b = 1|`) become the method's default
        // parameters, keyed by positional index.
        for (original_index, expression) in block.parameter_defaults.iter() {
            let positional_index = block.parameters[..*original_index]
                .iter()
                .filter(|p| !p.starts_with('&'))
                .count();
            method
                .default_parameters
                .push((positional_index, expression.clone()));
        }
        method.lambda_body = true;
        method.captured_def_scope = block.captured_def_scope.clone();
        method.captured_vars = Some(if block.captured_vars.is_empty() {
            self.environment().current_scope_var_refs()
        } else {
            block.captured_vars.clone()
        });
        method
    }

    /// Copy a `Method`/`UnboundMethod` under a new name, first checking that
    /// its original owner permits rebinding onto `class_rc`.
    fn rebindable_method(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        source: &Rc<Method>,
        position: Position,
    ) -> Result<Method, MetorexError> {
        // A Proc built by `Method#to_proc` carries its own receiver, so there
        // is nothing to rebind and nothing to validate.
        if source.bound_self.is_none()
            && let Some(owner) = &source.owner_class
        {
            if owner.is_singleton_class() {
                if !singleton_rebind_allowed(owner, class_rc) {
                    return Err(raise(
                        "TypeError",
                        "can't bind singleton method to a different class",
                        position,
                    ));
                }
            } else if owner.superclass().is_some() && !class_rc.has_ancestor(owner) {
                // Only class-owned methods are restricted; a module's methods
                // may be installed anywhere.
                return Err(raise(
                    "TypeError",
                    &format!(
                        "bind argument must be a subclass of {}",
                        owner.inspect_name()
                    ),
                    position,
                ));
            }
        }

        let mut method = (**source).clone();
        method.name = method_name.to_string();
        method.receiver = None;
        method.owner = Some(class_rc.name().to_string());
        method.owner_class = Some(Rc::clone(class_rc));
        Ok(method)
    }

    /// Ruby marks `initialize` and friends private no matter the current
    /// visibility. Otherwise the new method inherits the caller's visibility,
    /// but only when the call is made from inside the target module itself.
    fn apply_define_method_visibility(&mut self, class_rc: &Rc<Class>, method_name: &str) {
        let always_private = matches!(
            method_name,
            "initialize" | "initialize_copy" | "initialize_clone" | "initialize_dup"
        );
        let defined_in_target = matches!(
            self.environment().get("self"),
            Some(Object::Class(ref current)) | Some(Object::Module(ref current))
                if Rc::ptr_eq(current, class_rc)
        );

        let current = class_rc.current_visibility();
        let inherits_private = defined_in_target
            && matches!(
                current.as_str(),
                "private" | crate::vm::native_methods::MODULE_FUNCTION_VISIBILITY
            );
        if always_private || inherits_private {
            class_rc.set_method_private(method_name.to_string());
        } else {
            class_rc.set_method_public(method_name);
        }
    }
}

/// A singleton method may only be reinstalled on the singleton class it came
/// from, or on the singleton class of a descendant of its attached class —
/// mirroring how Ruby's metaclass chain parallels the class chain.
fn singleton_rebind_allowed(owner: &Rc<Class>, target: &Rc<Class>) -> bool {
    if Rc::ptr_eq(owner, target) {
        return true;
    }
    if !target.is_singleton_class() {
        return false;
    }
    match (attached_class(owner), attached_class(target)) {
        (Some(owner_attached), Some(target_attached)) => {
            target_attached.has_ancestor(&owner_attached)
        }
        _ => false,
    }
}

/// The class or module a singleton class was created for, if it has one.
fn attached_class(singleton: &Rc<Class>) -> Option<Rc<Class>> {
    match singleton.get_class_var("__attached__") {
        Some(Object::Class(attached)) | Some(Object::Module(attached)) => Some(attached),
        _ => None,
    }
}

/// Build an exception error of the given class with `message`.
fn raise(class_name: &str, message: &str, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception(class_name, message.to_string()),
        location: position_to_location(position),
        message: message.to_string(),
    }
}

/// Ruby-visible class name, used in the "wrong argument type" message.
fn ruby_class_name(object: &Object) -> &'static str {
    match object {
        Object::Nil => "NilClass",
        Object::Bool(true) => "TrueClass",
        Object::Bool(false) => "FalseClass",
        Object::Int(_) => "Integer",
        Object::Float(_) => "Float",
        Object::String(_) => "String",
        Object::Symbol(_) => "Symbol",
        Object::Array(_) => "Array",
        Object::Dict(_) => "Hash",
        Object::Set(_) => "Set",
        Object::Range { .. } => "Range",
        Object::Class(_) => "Class",
        Object::Module(_) => "Module",
        Object::Exception(_) => "Exception",
        Object::Regex(_, _) => "Regexp",
        _ => "Object",
    }
}
