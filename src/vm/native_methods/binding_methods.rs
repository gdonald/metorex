// What a Binding answers: the locals it names, reading and writing them, and
// running code where it was captured.

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Binding, Object};
use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    pub(crate) fn call_binding_methods(
        &mut self,
        binding: &Rc<Binding>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "receiver" => Ok(Some(binding.receiver.clone().unwrap_or(Object::Nil))),
            // The names the binding holds, in the order they were bound.
            "local_variables" => {
                // `self` is bound in the scope but is not a local, so it is
                // not one of the names a binding reports.
                let names: Vec<String> = binding
                    .keys()
                    .into_iter()
                    .filter(|held| held != "self")
                    .collect();
                Ok(Some(Object::array(
                    names.into_iter().map(Object::symbol).collect(),
                )))
            }
            "local_variable_defined?" => {
                let name = self.binding_local_name(arguments, method_name, position)?;
                Ok(Some(Object::Bool(binding.has(&name))))
            }
            "local_variable_get" => {
                let name = self.binding_local_name(arguments, method_name, position)?;
                match binding.get(&name) {
                    Some(cell) => Ok(Some(cell.borrow().clone())),
                    None => Err(crate::vm::errors::simple_exception(
                        "NameError",
                        &format!("local variable '{name}' is not defined for {binding:p}"),
                        position,
                    )),
                }
            }
            "local_variable_set" => {
                let name = self.binding_local_name(arguments, method_name, position)?;
                // A global or one of the names the interpreter keeps for
                // itself is not a local, so a binding refuses to bind it.
                if !name.starts_with(|held: char| held == '_' || held.is_lowercase()) {
                    return Err(crate::vm::errors::simple_exception(
                        "NameError",
                        &format!("wrong local variable name '{name}' for {binding:p}"),
                        position,
                    ));
                }
                let value = arguments.get(1).cloned().unwrap_or(Object::Nil);
                match binding.get(&name) {
                    Some(cell) => *cell.borrow_mut() = value.clone(),
                    None => binding.set(&name, Rc::new(RefCell::new(value.clone()))),
                }
                Ok(Some(value))
            }
            // `b.eval(src)` is `eval(src, b)`, so it runs where the binding
            // was captured and leaves what it binds behind.
            "eval" => {
                let mut passed = vec![
                    arguments.first().cloned().unwrap_or(Object::Nil),
                    Object::Binding(Rc::clone(binding)),
                ];
                passed.extend(arguments.iter().skip(1).cloned());
                self.call_native_function("eval", passed, position)
                    .map(Some)
            }
            // Where the binding was captured, which is the file and line the
            // call to `binding` sits on.
            "source_location" => Ok(Some(match binding.source.borrow().clone() {
                Some((file, line)) => {
                    Object::array(vec![Object::string(file), Object::Int(line as i64)])
                }
                None => Object::Nil,
            })),
            _ => Ok(None),
        }
    }

    /// The local a Binding method was asked about, named by Symbol or String.
    fn binding_local_name(
        &mut self,
        arguments: &[Object],
        method_name: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        match arguments.first() {
            Some(Object::Symbol(name)) | Some(Object::String(name)) => {
                Ok(name.as_str().to_string())
            }
            // A name may be written as anything that spells itself out.
            Some(other) if self.responds_to(other, "to_str") => {
                let spelled = self.send_to_object(other.clone(), "to_str", Vec::new(), position)?;
                match spelled {
                    Object::String(name) => Ok(name.as_str().to_string()),
                    held => Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!("{held} is not a symbol nor a string"),
                        position,
                    )),
                }
            }
            Some(other) => Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("{other} is not a symbol nor a string"),
                position,
            )),
            None => Err(crate::vm::errors::method_argument_error(
                method_name,
                1,
                0,
                position,
            )),
        }
    }
}
