// The value an instance of a core-class subclass holds behind it.

use super::*;

/// Instance variable a String subclass keeps its characters in.
pub(crate) const STRING_SUBCLASS_VAR: &str = "__string__";
/// Instance variable an instance of an Array subclass stores its elements in,
/// since a plain Array is a primitive rather than an instance.
pub(crate) const ARRAY_SUBCLASS_VAR: &str = "__array__";
/// Instance variable an instance of a Set subclass stores its elements in,
/// since a plain Set is a primitive rather than an instance.
pub(crate) const SET_SUBCLASS_VAR: &str = "__set__";
/// The instance variable an instance of a Proc subclass holds its block in.
pub(crate) const PROC_SUBCLASS_VAR: &str = "__proc__";
/// Instance variable an instance of a Hash subclass stores its entries in,
/// since a plain Hash is a primitive rather than an instance.
pub(crate) const HASH_SUBCLASS_VAR: &str = "__hash__";
/// Instance variable an instance of a Range subclass stores its ends in,
/// since a plain Range is a primitive rather than an instance.
pub(crate) const RANGE_SUBCLASS_VAR: &str = "__range__";

/// The backing range an instance of a Range subclass holds, or None when
/// `receiver` is not one.
pub(crate) fn range_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(RANGE_SUBCLASS_VAR)
        .cloned()
}

/// The value read as a Range: a Range itself, or the one behind an instance
/// of a Range subclass, which is what lets a subclass stand wherever a range
/// is expected.
pub(crate) fn as_range(value: &Object) -> Option<Object> {
    if matches!(value, Object::Range { .. }) {
        return Some(value.clone());
    }
    match range_subclass_value(value) {
        Some(held @ Object::Range { .. }) => Some(held),
        _ => None,
    }
}

/// The name a Regexp subclass instance keeps its pattern under.
pub(crate) const REGEXP_SUBCLASS_VAR: &str = "__regexp__";

/// The pattern behind an instance of a Regexp subclass.
pub(crate) fn regexp_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(REGEXP_SUBCLASS_VAR)
        .cloned()
}

/// The characters behind an instance of a String subclass.
pub(crate) fn string_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(STRING_SUBCLASS_VAR)
        .cloned()
}
/// The backing array an instance of an Array subclass holds, or None when
/// `receiver` is not one. The Rc is shared, so a mutation through it is
/// visible to the instance.
pub(crate) fn array_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(ARRAY_SUBCLASS_VAR)
        .cloned()
}
/// The backing set an instance of a Set subclass holds, or None when
/// `receiver` is not one. The Rc is shared, so a change through it is visible
/// to the instance.
/// The block an instance of a Proc subclass stands for, or None when
/// `receiver` is not one.
pub(crate) fn proc_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(PROC_SUBCLASS_VAR)
        .cloned()
}

pub(crate) fn set_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(SET_SUBCLASS_VAR)
        .cloned()
}
/// The backing hash an instance of a Hash subclass holds, or None when
/// `receiver` is not one. The Rc is shared, so a change through it is visible
/// to the instance.
pub(crate) fn hash_subclass_value(receiver: &Object) -> Option<Object> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    instance
        .borrow()
        .instance_vars
        .get(HASH_SUBCLASS_VAR)
        .cloned()
}
/// The dictionary behind a value: a Hash itself, or the one an instance of a
/// Hash subclass keeps.
pub(crate) fn as_dict(value: &Object) -> Option<Object> {
    if matches!(value, Object::Dict(_)) {
        return Some(value.clone());
    }
    match hash_subclass_value(value) {
        Some(held @ Object::Dict(_)) => Some(held),
        _ => None,
    }
}

/// The String, Array, Hash, Set or Regexp an instance of a subclass of one of
/// them holds, or None when `value` is no such instance.
pub(crate) fn subclass_backing(value: &Object) -> Option<Object> {
    string_subclass_value(value)
        .or_else(|| array_subclass_value(value))
        .or_else(|| hash_subclass_value(value))
        .or_else(|| set_subclass_value(value))
        .or_else(|| regexp_subclass_value(value))
}
