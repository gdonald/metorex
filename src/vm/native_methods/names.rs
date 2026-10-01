// Whether a name is one a core class answers natively, and whether
// it reads as a constant.

use super::*;

/// How Ruby names a value that cannot stand in for a number: true, false and
/// nil by their own spelling, and everything else by its class.
pub(crate) fn unconvertible_wording(value: &Object, named: &str) -> String {
    match value {
        Object::Bool(true) => "true".to_string(),
        Object::Bool(false) => "false".to_string(),
        Object::Nil => "nil".to_string(),
        _ => named.to_string(),
    }
}

/// The range Ruby keeps a thread's priority inside.
pub(crate) const PRIORITY_FLOOR: i64 = -3;
pub(crate) const PRIORITY_CEILING: i64 = 3;

/// Whether two objects are the same Thread.
pub(crate) fn same_thread(one: &Object, other: &Object) -> bool {
    matches!((one, other), (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b))
}

/// Whether `name` is a syntactically valid Ruby constant name: must start
/// with an uppercase letter and contain only word characters after.
/// Multibyte letters are allowed (Ruby permits `CS_CONSTλ`). Used by
/// `Module#autoload`, `Module#const_set`, and `Module#const_defined?` to
/// reject lowercase / numeric / punctuated names with a NameError.
/// The text a name is spelled with. A name carried as the bytes of an
/// encoding of its own is read back through that encoding, so the characters
/// it names are the ones the program wrote.
pub(crate) fn name_text(held: &std::rc::Rc<crate::object::StringValue>) -> String {
    if !held.holds_bytes() {
        return held.as_str().to_string();
    }
    let named = held.encoding_name();
    let bytes = string_methods::binary_bytes(held);
    if named == "EUC-JP" {
        return euc_jp_table::euc_jp_text(&bytes);
    }
    if string_methods::spells_shift_jis(&named) {
        return shift_jis_table::shift_jis_text(&bytes);
    }
    if named == "ISO-2022-JP" {
        return euc_jp_table::iso_2022_jp_text(&bytes);
    }
    match string_methods::latin_text(&bytes, &named) {
        Some(text) => text,
        None => held.as_str().to_string(),
    }
}

/// The text a string spells when it is read back in the named encoding
/// rather than in the one it carries, which is what a magic comment in
/// eval'd source asks for.
pub(crate) fn text_in_encoding(
    held: &std::rc::Rc<crate::object::StringValue>,
    named: &str,
) -> String {
    if !held.holds_bytes() || held.encoding_name() == named {
        return name_text(held);
    }
    let bytes = string_methods::binary_bytes(held);
    if named == "EUC-JP" {
        return euc_jp_table::euc_jp_text(&bytes);
    }
    if string_methods::spells_shift_jis(named) {
        return shift_jis_table::shift_jis_text(&bytes);
    }
    if named == "ISO-2022-JP" {
        return euc_jp_table::iso_2022_jp_text(&bytes);
    }
    if let Some(text) = string_methods::latin_text(&bytes, named) {
        return text;
    }
    // An encoding metorex has no table for keeps the bytes that are not
    // UTF-8 as escaped characters, which a literal turns back into bytes.
    crate::file_loader::escaped_source_text(&bytes)
}

pub(crate) fn is_valid_constant_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => {}
        _ => return false,
    }
    // A name written in an encoding of its own carries bytes the program
    // reads as letters, so anything above ASCII counts as one.
    chars.all(|c| c.is_alphanumeric() || c == '_' || (c as u32) >= 0x80)
}

/// Whether a name is one Module answers natively, which is what a method
/// taken from a class reaches before the class's own instance methods.
pub(crate) fn is_native_module_method(name: &str) -> bool {
    class_methods::NATIVE_MODULE_METHODS
        .iter()
        .any(|(held, _, _)| *held == name)
}

/// Whether a name is one of the functions Kernel carries, which are reachable
/// both without a receiver and through `Kernel` itself.
pub(crate) fn is_kernel_private_function(name: &str) -> bool {
    class_methods::KERNEL_PRIVATE_FUNCTIONS.contains(&name)
}

/// Whether Enumerable writes a method over `each`, so a class of the
/// program's own that writes `each` decides what it walks.
pub(crate) fn enumerable_walks_through_each(name: &str) -> bool {
    matches!(
        name,
        "map"
            | "collect"
            | "flat_map"
            | "select"
            | "filter"
            | "reject"
            | "find"
            | "detect"
            | "find_all"
            | "each_with_object"
            | "each_with_index"
            | "inject"
            | "reduce"
            | "sort_by"
            | "group_by"
            | "partition"
            | "min_by"
            | "max_by"
            | "sum"
            | "count"
            | "to_a"
            | "entries"
    )
}

/// Which store a thread-local method reads. Ruby keeps the fiber-local names
/// `Thread#[]` reaches apart from the thread-wide ones `thread_variable_get`
/// reaches, and a name set through one is not seen through the other.
impl VirtualMachine {
    /// The instance variable one of the two thread-local stores lives under.
    /// `thread_variable_*` is shared by every fiber the thread runs, while
    /// `[]` and its family belong to the fiber that wrote them, so the fiber
    /// running now is part of that store's name.
    pub(crate) fn thread_local_store(&self, method_name: &str, receiver: &Object) -> String {
        if method_name.starts_with("thread_variable") {
            return "__thread_variables".to_string();
        }
        // A name kept on another thread belongs to that thread's own root
        // fiber, since the fiber running now is none of its.
        // Outside every thread block the thread running is the main one,
        // which the stack does not name.
        let current = match self.thread_current_stack.last() {
            Some(held) => Some(held.clone()),
            None => self.globals().get("__Thread_main"),
        };
        let running = matches!(
            (&current, receiver),
            (Some(Object::Instance(a)), Object::Instance(b)) if Rc::ptr_eq(a, b)
        );
        let held = self.fiber_current_handle();
        // A thread's own body is its root fiber, so the names it keeps there
        // belong to the thread rather than to a fiber the program made.
        if !running
            || held == crate::vm::fibers::ROOT_FIBER
            || self.thread_body_fibers.contains(&held)
        {
            return "__thread_locals_root".to_string();
        }
        format!("__thread_locals_{held}")
    }
}
