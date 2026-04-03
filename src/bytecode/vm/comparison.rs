// Comparison helper functions for bytecode VM binary operations

use crate::object::Object;

pub fn compare_less(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a < b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a < b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool((*a as f64) < *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a < *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn compare_greater(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a > b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a > b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool(*a as f64 > *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a > *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn compare_less_equal(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a <= b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a <= b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool(*a as f64 <= *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a <= *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn compare_greater_equal(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a >= b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a >= b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool(*a as f64 >= *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a >= *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}
