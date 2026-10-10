// The keys a hash keeps its own bookkeeping under.

use super::*;

/// Sentinel key for storing the default proc on hashes created with Hash.new { ... }
pub(crate) const DEFAULT_PROC_KEY: &str = "__MX_DEFAULT_PROC__";
/// The value `Hash.new(default)` answers for a key the hash has no entry for.
pub(crate) const DEFAULT_VALUE_KEY: &str = "__MX_DEFAULT__";
/// Sentinel key for storing original non-primitive key objects
pub(crate) const KEY_OBJECTS_KEY: &str = "__MX_KEY_OBJECTS__";
/// Sentinel marking a hash holding a key that is not a primitive but was
/// placed by how it renders rather than by its `#hash`. Only such a hash has
/// to walk its keys to find one that is `eql?` to a key it was asked for.
pub(crate) const RENDERED_OBJECTS_KEY: &str = "__MX_RENDERED_OBJECTS__";
/// Sentinel marking a hash that `compare_by_identity` was called on.
pub(crate) const BY_IDENTITY_KEY: &str = "__MX_BY_IDENTITY__";
/// Sentinel marking a hash gathered from the keyword arguments of a call to a
/// method named by `ruby2_keywords`, which can be passed on as keywords again.
pub(crate) const RUBY2_KEYWORDS_KEY: &str = "__MX_RUBY2_KEYWORDS__";

/// Check if a key is an internal sentinel key
pub(crate) fn is_internal_key(key: &str) -> bool {
    key == DEFAULT_PROC_KEY
        || key == DEFAULT_VALUE_KEY
        || key == BY_IDENTITY_KEY
        || key == RUBY2_KEYWORDS_KEY
        || key == "__MX_KWARGS__"
        || key == KEY_OBJECTS_KEY
        || key == RENDERED_OBJECTS_KEY
}

/// Reconstruct the original key Object from its string key, using the sentinel
/// __MX_KEY_OBJECTS__ sub-map if present for non-primitive keys.
pub(crate) fn reconstruct_key(dict: &indexmap::IndexMap<String, Object>, key_str: &str) -> Object {
    if let Some(Object::Dict(key_objs)) = dict.get(KEY_OBJECTS_KEY)
        && let Some(obj) = key_objs.borrow().get(key_str)
    {
        return obj.clone();
    }
    crate::vm::utils::dict_key_to_object(key_str)
}
