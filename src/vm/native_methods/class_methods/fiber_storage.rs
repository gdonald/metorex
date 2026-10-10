// The per-fiber store a `Fiber[]` reads and writes.

use super::*;

impl VirtualMachine {
    /// The name a fiber keeps a value under. Ruby takes a Symbol, reads a
    /// String as one, and refuses anything else.
    pub(crate) fn fiber_storage_name(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match given {
            Object::Symbol(name) => Ok(format!(":{name}")),
            Object::String(name) => Ok(format!(":{}", name.as_str())),
            other if self.responds_to(other, "to_str") => {
                match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                    Object::String(name) => Ok(format!(":{}", name.as_str())),
                    _ => Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "wrong argument type (expected a Symbol)",
                        position,
                    )),
                }
            }
            _ => Err(crate::vm::errors::simple_exception(
                "TypeError",
                "wrong argument type (expected a Symbol)",
                position,
            )),
        }
    }

    /// Refuse anything a fiber cannot keep its names in: it has to be a Hash
    /// that may still be written to, and every name in it has to be a Symbol.
    pub(crate) fn check_fiber_storage(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Object::Dict(entries) = given else {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!(
                    "no implicit conversion of {} into Hash",
                    crate::vm::errors::conversion_subject(given)
                ),
                position,
            ));
        };
        if self.object_is_frozen(given) {
            return Err(self.frozen_modification_error(given, position));
        }
        let named: Vec<Object> = {
            let held = entries.borrow();
            held.keys()
                .filter(|slot| !slot.starts_with("__MX_"))
                .map(|slot| crate::vm::utils::dict_key_to_object(slot))
                .collect()
        };
        for key in named {
            if !matches!(key, Object::Symbol(_)) {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "wrong argument type (expected a Symbol)",
                    position,
                ));
            }
        }
        Ok(())
    }
}
