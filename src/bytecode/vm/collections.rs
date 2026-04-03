// Collection helper functions for bytecode VM index operations

use crate::error::{MetorexError, SourceLocation};
use crate::object::Object;

pub fn index_get(collection: &Object, index: &Object) -> Result<Object, MetorexError> {
    match (collection, index) {
        (Object::Array(arr), Object::Int(i)) => {
            let arr = arr.borrow();
            let idx = if *i < 0 {
                (arr.len() as i64 + i) as usize
            } else {
                *i as usize
            };
            Ok(arr.get(idx).cloned().unwrap_or(Object::Nil))
        }
        (Object::Dict(dict), Object::String(key)) => {
            let dict = dict.borrow();
            Ok(dict.get(key.as_str()).cloned().unwrap_or(Object::Nil))
        }
        _ => Err(MetorexError::runtime_error(
            format!(
                "Cannot index {} with {}",
                collection.type_name(),
                index.type_name()
            ),
            SourceLocation::new(0, 0, 0),
        )),
    }
}

pub fn index_set(collection: &Object, index: &Object, value: &Object) -> Result<(), MetorexError> {
    match (collection, index) {
        (Object::Array(arr), Object::Int(i)) => {
            let mut arr = arr.borrow_mut();
            let idx = if *i < 0 {
                (arr.len() as i64 + i) as usize
            } else {
                *i as usize
            };
            if idx < arr.len() {
                arr[idx] = value.clone();
                Ok(())
            } else {
                Err(MetorexError::runtime_error(
                    "Array index out of bounds",
                    SourceLocation::new(0, 0, 0),
                ))
            }
        }
        (Object::Dict(dict), Object::String(key)) => {
            let mut dict = dict.borrow_mut();
            dict.insert(key.to_string(), value.clone());
            Ok(())
        }
        _ => Err(MetorexError::runtime_error(
            format!(
                "Cannot index {} with {}",
                collection.type_name(),
                index.type_name()
            ),
            SourceLocation::new(0, 0, 0),
        )),
    }
}
