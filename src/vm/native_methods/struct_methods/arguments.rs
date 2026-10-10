// The members a struct names, and the subscripts that reach them.

use super::*;

/// Class variable holding the ordered member names of a generated struct class.
pub(crate) const MEMBERS_VAR: &str = "__struct_members__";
/// Class variable holding the `keyword_init:` value the struct was built with.
pub(crate) const KEYWORD_INIT_VAR: &str = "__struct_keyword_init__";
/// Member values live under a prefixed instance-variable name, since a struct
/// member is not an instance variable: `Struct.new(:a).new(1).instance_variables`
/// is empty, and `@a` stays free for the holder to use.
const MEMBER_PREFIX: &str = "__struct_member_";

/// The instance-variable slot a member's value is stored in.
pub(crate) fn member_slot(member: &str) -> String {
    format!("{}{}", MEMBER_PREFIX, member)
}

/// Whether an instance-variable name is a struct member slot rather than one
/// the holder set.
pub(crate) fn is_member_slot(name: &str) -> bool {
    name.starts_with(MEMBER_PREFIX)
}

/// The member names of a generated struct class, or None when `class_rc` is
/// not one. `Struct` itself has no members and answers None.
pub(crate) fn struct_members(class_rc: &Rc<Class>) -> Option<Vec<String>> {
    match class_rc.lookup_class_var(MEMBERS_VAR) {
        Some(Object::Array(names)) => Some(
            names
                .borrow()
                .iter()
                .map(|name| match name {
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => other.to_string(),
                })
                .collect(),
        ),
        _ => None,
    }
}

pub(crate) fn keyword_init(class_rc: &Rc<Class>) -> Object {
    class_rc
        .lookup_class_var(KEYWORD_INIT_VAR)
        .unwrap_or(Object::Nil)
}

pub(crate) fn symbols(names: &[String]) -> Object {
    Object::Array(Rc::new(RefCell::new(
        names
            .iter()
            .map(|name| Object::symbol(name.clone()))
            .collect(),
    )))
}

pub(crate) fn argument_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("ArgumentError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

/// One `values_at` subscript, counting from the end when negative.
pub(crate) fn indexed_value(
    values: &[Object],
    index: i64,
    position: Position,
) -> Result<Object, MetorexError> {
    let length = values.len() as i64;
    let resolved = if index < 0 { index + length } else { index };
    if resolved < 0 {
        return Err(index_error(
            format!("offset {} too small for struct(size:{})", index, length),
            position,
        ));
    }
    if resolved >= length {
        return Err(index_error(
            format!("offset {} too large for struct(size:{})", index, length),
            position,
        ));
    }
    Ok(values[resolved as usize].clone())
}

/// A `values_at` Range subscript. Elements past the end read as nil, while a
/// negative start that falls off the front is a RangeError.
pub(crate) fn range_values(
    values: &[Object],
    range: &Object,
    position: Position,
) -> Result<Vec<Object>, MetorexError> {
    let Object::Range {
        start,
        end,
        exclusive,
        ..
    } = range
    else {
        return Ok(Vec::new());
    };
    let length = values.len() as i64;
    let first = match start.as_ref() {
        Object::Nil => 0,
        Object::Int(value) if *value < 0 => value + length,
        Object::Int(value) => *value,
        _ => 0,
    };
    if first < 0 || first > length {
        return Err(MetorexError::UncaughtException {
            exception: Object::exception("RangeError", format!("{} out of range", range)),
            location: position_to_location(position),
            message: format!("{} out of range", range),
        });
    }
    let mut last = match end.as_ref() {
        Object::Nil => length - 1,
        Object::Int(value) if *value < 0 => value + length,
        Object::Int(value) => {
            if *exclusive {
                value - 1
            } else {
                *value
            }
        }
        _ => length - 1,
    };
    if matches!(end.as_ref(), Object::Int(value) if *value < 0) && *exclusive {
        last -= 1;
    }
    let mut picked = Vec::new();
    for index in first..=last.max(first - 1) {
        picked.push(values.get(index as usize).cloned().unwrap_or(Object::Nil));
    }
    Ok(picked)
}

/// A hash key that is not a primitive is recorded in the sentinel sub-map, so
/// the original object comes back when the hash is walked.
pub(crate) fn remember_key_object(
    pairs: &mut IndexMap<String, Object>,
    rendered: &str,
    key: &Object,
) {
    crate::vm::native_methods::remember_key_object(pairs, rendered, key);
}

pub(crate) fn index_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("IndexError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

pub(crate) fn name_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("NameError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

/// Pull the parser-marked keyword-argument hash off the end of an argument
/// list, leaving the positional arguments behind.
pub(crate) fn take_keyword_arguments(
    arguments: &[Object],
) -> (Vec<Object>, IndexMap<String, Object>) {
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            let keywords = dict
                .iter()
                .filter(|(key, _)| key.as_str() != "__MX_KWARGS__")
                .map(|(key, value)| {
                    (
                        key.strip_prefix(':').unwrap_or(key).to_string(),
                        value.clone(),
                    )
                })
                .collect();
            return (arguments[..arguments.len() - 1].to_vec(), keywords);
        }
    }
    (arguments.to_vec(), IndexMap::new())
}

/// The value stored for `member` on a struct instance.
pub(crate) fn member_value(receiver: &Object, member: &str) -> Object {
    match receiver {
        Object::Instance(instance) => instance
            .borrow()
            .instance_vars
            .get(&member_slot(member))
            .cloned()
            .unwrap_or(Object::Nil),
        _ => Object::Nil,
    }
}

pub(crate) fn member_values(receiver: &Object, members: &[String]) -> Vec<Object> {
    members
        .iter()
        .map(|member| member_value(receiver, member))
        .collect()
}

/// Resolve `[]` / `[]=` / `dig` subscripts, which accept a member name or a
/// positional index counting from either end.
pub(crate) fn resolve_member(
    members: &[String],
    key: &Object,
    position: Position,
) -> Result<String, MetorexError> {
    match key {
        Object::Int(index) => {
            let length = members.len() as i64;
            let resolved = if *index < 0 { index + length } else { *index };
            if resolved < 0 || resolved >= length {
                let direction = if *index < 0 { "small" } else { "large" };
                let message = format!(
                    "offset {} too {} for struct(size:{})",
                    index, direction, length
                );
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("IndexError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            }
            Ok(members[resolved as usize].clone())
        }
        Object::Symbol(name) => resolve_named_member(members, &name.as_str(), position),
        Object::String(name) => resolve_named_member(members, &name.as_str(), position),
        other => Err(MetorexError::type_error(
            format!(
                "no implicit conversion of {} into Integer",
                other.type_name()
            ),
            position_to_location(position),
        )),
    }
}

pub(crate) fn resolve_named_member(
    members: &[String],
    name: &str,
    position: Position,
) -> Result<String, MetorexError> {
    if members.iter().any(|member| member == name) {
        Ok(name.to_string())
    } else {
        Err(name_error(
            format!("no member '{}' in struct", name),
            position,
        ))
    }
}
