// Arithmetic helper functions for bytecode VM binary operations

use std::rc::Rc;

use crate::object::Object;

pub fn binary_add(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a + b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a + b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 + b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a + *b as f64)),
        (Object::String(a), Object::String(b)) => {
            Ok(Object::String(Rc::new(format!("{}{}", a, b))))
        }
        _ => Err(format!(
            "Cannot add {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn binary_sub(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a - b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a - b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 - b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a - *b as f64)),
        _ => Err(format!(
            "Cannot subtract {} from {}",
            b.type_name(),
            a.type_name()
        )),
    }
}

pub fn binary_mul(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a * b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a * b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 * b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a * *b as f64)),
        _ => Err(format!(
            "Cannot multiply {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn binary_div(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(_), Object::Int(0)) => Err("Division by zero".to_string()),
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a / b)),
        (Object::Float(_), Object::Float(b)) if *b == 0.0 => Err("Division by zero".to_string()),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a / b)),
        (Object::Int(a), Object::Float(b)) if *b == 0.0 => Err("Division by zero".to_string()),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 / b)),
        (Object::Float(_), Object::Int(0)) => Err("Division by zero".to_string()),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a / *b as f64)),
        _ => Err(format!(
            "Cannot divide {} by {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn binary_mod(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(_), Object::Int(0)) => Err("Modulo by zero".to_string()),
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a % b)),
        _ => Err(format!(
            "Cannot modulo {} by {}",
            a.type_name(),
            b.type_name()
        )),
    }
}
