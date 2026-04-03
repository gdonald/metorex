// Native function dispatch for the bytecode VM

use std::rc::Rc;

use crate::error::MetorexError;
use crate::object::{Method, Object};

use super::BytecodeVm;

impl BytecodeVm {
    pub(super) fn call_native(
        &mut self,
        name: &str,
        args: &[Object],
    ) -> Result<Object, MetorexError> {
        match name {
            "puts" => {
                for arg in args {
                    println!("{}", arg);
                }
                Ok(Object::Nil)
            }
            "print" => {
                for arg in args {
                    print!("{}", arg);
                }
                Ok(Object::Nil)
            }
            "p" => {
                for arg in args {
                    println!("{:?}", arg);
                }
                Ok(Object::Nil)
            }
            "define_method" => {
                // define_method(:name, compiled_function)
                // or define_method(:name) { block }
                // In bytecode context: args[0] = name (String/Symbol),
                // args[1] = CompiledFunction (the block body)
                if args.is_empty() {
                    return Err(self.runtime_err("define_method requires at least one argument"));
                }
                let method_name = match &args[0] {
                    Object::String(s) => s.to_string(),
                    Object::Symbol(s) => s.to_string(),
                    _ => {
                        return Err(self.runtime_err(
                            "define_method: first argument must be a String or Symbol",
                        ));
                    }
                };

                // The function/block should be the second argument
                if args.len() < 2 {
                    return Err(
                        self.runtime_err("define_method requires a function as second argument")
                    );
                }
                let func = match &args[1] {
                    Object::CompiledFunction(f) => Rc::clone(f),
                    _ => {
                        return Err(
                            self.runtime_err("define_method: second argument must be a function")
                        );
                    }
                };

                // Find the class on the stack (the most recent Class in globals
                // or on the stack). For now, store as a global method.
                let method = Method::new(method_name.clone(), vec![], vec![]);
                let method_rc = Rc::new(method);

                // If there's a class context, attach to it
                // Look for a Class on the stack
                let mut attached = false;
                for val in self.stack.iter().rev() {
                    if let Object::Class(class) = val {
                        class.define_method(&method_name, method_rc.clone());
                        let key = format!("{}#{}", class.name(), method_name);
                        self.globals
                            .insert(key, Object::CompiledFunction(func.clone()));
                        attached = true;
                        break;
                    }
                }

                if !attached {
                    // Store as a global function
                    self.globals
                        .insert(method_name, Object::CompiledFunction(func));
                }

                Ok(Object::Nil)
            }
            _ => Err(self.runtime_err(&format!("Unknown native function '{}'", name))),
        }
    }
}
