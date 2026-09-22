// How far a shift moves.

use super::*;

impl VirtualMachine {
    pub(crate) fn shift_count(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: crate::lexer::Position,
    ) -> Result<ShiftCount, crate::error::MetorexError> {
        let held = match argument {
            Object::Int(_) | Object::BigInt(_) => argument.clone(),
            other if self.responds_to(other, "to_int") => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    answered @ (Object::Int(_) | Object::BigInt(_)) => answered,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            argument,
                            position,
                        ));
                    }
                }
            }
            other => {
                return Err(method_argument_type_error(
                    method_name,
                    "Integer",
                    other,
                    position,
                ));
            }
        };
        let negative = matches!(&held, Object::Int(count) if *count < 0)
            || matches!(&held, Object::BigInt(count) if **count < num_bigint::BigInt::from(0));
        let written = match held {
            Object::Int(count) => Some(count),
            Object::BigInt(count) => count.to_string().parse::<i64>().ok(),
            _ => None,
        };
        Ok(ShiftCount { written, negative })
    }
}

/// How far a shift moves, read the way Ruby reads it: an Integer outright, or
/// anything that answers `to_int` with one. A count too wide for a machine
/// word has no place to name, and the sign says which way it went.
pub(crate) struct ShiftCount {
    pub written: Option<i64>,
    pub negative: bool,
}
