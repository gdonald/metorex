//! Objects wrapping a C pointer, typed by an `rb_data_type_t` or untyped
//! with a mark and a free function, and the allocator a class makes its
//! instances with.

use super::handles::{Value, to_object, to_value};
use super::{Caller, called_from, raise};
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Instance, Object};
use crate::vm::VirtualMachine;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_void};
use std::rc::Rc;

/// MRI's `rb_data_type_t`, which C fills in and metorex only reads.
#[repr(C)]
pub struct DataType {
    wrap_struct_name: *const c_char,
    mark: *const c_void,
    free: *const c_void,
    size: Option<extern "C-unwind" fn(*const c_void) -> usize>,
    compact: *const c_void,
    reserved: *const c_void,
    parent: *const DataType,
    data: *const c_void,
    flags: Value,
}

/// The struct `RDATA` and `RTYPEDDATA` point C at: `struct RData` and
/// `struct RTypedData` share this layout, with the wrapped pointer last.
#[repr(C)]
struct DataSlot {
    flags: Value,
    klass: Value,
    mark_or_fields: usize,
    free_or_type: usize,
    data: *mut c_void,
}

/// A wrapped pointer, and the type it was wrapped with when it is typed.
struct Wrapped {
    slot: Box<DataSlot>,
    data_type: Option<*const DataType>,
}

/// The class variable holding the address of the allocator C gave a class.
const ALLOCATOR_VAR: &str = "__c_allocator__";

thread_local! {
    /// The wrapped pointer of every Data object, by the address of the
    /// instance. The handle table keeps each of them alive, so no address
    /// is reused.
    static WRAPPED: RefCell<HashMap<usize, Wrapped>> = RefCell::new(HashMap::new());
}

fn instance_key(object: &Object) -> Option<usize> {
    match object {
        Object::Instance(instance) => Some(Rc::as_ptr(instance) as usize),
        _ => None,
    }
}

/// Whether `object` wraps a C pointer.
pub(super) fn is_data(object: &Object) -> bool {
    instance_key(object).is_some_and(|key| WRAPPED.with(|held| held.borrow().contains_key(&key)))
}

fn data_type_of(object: &Object) -> Option<*const DataType> {
    let key = instance_key(object)?;
    WRAPPED.with(|held| {
        held.borrow()
            .get(&key)
            .and_then(|wrapped| wrapped.data_type)
    })
}

fn type_name(data_type: *const DataType) -> String {
    // SAFETY: an `rb_data_type_t` names its struct with a C string.
    unsafe { CStr::from_ptr((*data_type).wrap_struct_name) }
        .to_string_lossy()
        .into_owned()
}

fn wrap(
    klass: Value,
    data: *mut c_void,
    first: usize,
    second: usize,
    data_type: Option<*const DataType>,
) -> Value {
    let class = match klass {
        0 => match super::calls::top_level_module("Object") {
            Object::Class(class) => class,
            _ => unreachable!("Object is a class"),
        },
        _ => super::exports::class_from(klass),
    };
    let object = Object::Instance(Instance::new(class));
    let value = to_value(&object);
    let slot = Box::new(DataSlot {
        flags: 0,
        klass,
        mark_or_fields: first,
        free_or_type: second,
        data,
    });
    let key = instance_key(&object).expect("a Data object is an instance");
    WRAPPED.with(|held| held.borrow_mut().insert(key, Wrapped { slot, data_type }));
    value
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_data_object_wrap(
    klass: Value,
    data: *mut c_void,
    mark: *const c_void,
    free: *const c_void,
) -> Value {
    wrap(klass, data, mark as usize, free as usize, None)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_data_typed_object_wrap(
    klass: Value,
    data: *mut c_void,
    data_type: *const DataType,
) -> Value {
    wrap(klass, data, 0, data_type as usize, Some(data_type))
}

/// A Data object wrapping `size` zeroed bytes, for `Data_Make_Struct`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_data_object_zalloc(
    klass: Value,
    size: usize,
    mark: *const c_void,
    free: *const c_void,
) -> Value {
    rb_data_object_wrap(klass, zeroed(size), mark, free)
}

/// A typed Data object wrapping `size` zeroed bytes, for
/// `TypedData_Make_Struct`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_data_typed_object_zalloc(
    klass: Value,
    size: usize,
    data_type: *const DataType,
) -> Value {
    rb_data_typed_object_wrap(klass, zeroed(size), data_type)
}

unsafe extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;
}

fn zeroed(size: usize) -> *mut c_void {
    // SAFETY: calloc takes a count and a size and answers memory C frees.
    unsafe { calloc(1, size.max(1)) }
}

fn expected_data(object: &Object) -> ! {
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &format!(
            "wrong argument type {} (expected Data)",
            displayed_class(object)
        ),
        called_from(),
    ))
}

/// The struct `RDATA` and `RTYPEDDATA` read and write the wrapped pointer
/// through.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_data_struct(object: Value) -> *mut c_void {
    let held = to_object(object);
    let Some(key) = instance_key(&held) else {
        expected_data(&held)
    };
    let slot = WRAPPED.with(|wrapped| {
        wrapped
            .borrow_mut()
            .get_mut(&key)
            .map(|found| &mut *found.slot as *mut DataSlot as *mut c_void)
    });
    slot.unwrap_or_else(|| expected_data(&held))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_typeddata_p(object: Value) -> i32 {
    data_type_of(&to_object(object)).is_some() as i32
}

/// Whether `child` is `parent` or reaches it through its parents.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_typeddata_inherited_p(
    child: *const DataType,
    parent: *const DataType,
) -> i32 {
    let mut current = child;
    while !current.is_null() {
        if std::ptr::eq(current, parent) {
            return 1;
        }
        // SAFETY: each `rb_data_type_t` C passes names its parent or NULL.
        current = unsafe { (*current).parent };
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_typeddata_is_kind_of(object: Value, data_type: *const DataType) -> i32 {
    data_type_of(&to_object(object))
        .is_some_and(|held| rb_typeddata_inherited_p(held, data_type) == 1) as i32
}

/// The class a TypeError names for `object`, or `nil`, `true` or `false`.
fn displayed_class(object: &Object) -> String {
    match object {
        Object::Nil => "nil".to_string(),
        Object::Bool(true) => "true".to_string(),
        Object::Bool(false) => "false".to_string(),
        other => super::calls::class_name_of(other.clone()),
    }
}

/// The pointer a typed Data object wraps, refusing an object that is not
/// one of `data_type` or a type descending from it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_typeddata(
    object: Value,
    data_type: *const DataType,
) -> *mut c_void {
    let held = to_object(object);
    let actual = match data_type_of(&held) {
        Some(held_type) if rb_typeddata_inherited_p(held_type, data_type) == 1 => {
            // SAFETY: the slot of a Data object stays where it was made.
            return unsafe { (*(rb_metorex_data_struct(object) as *const DataSlot)).data };
        }
        Some(held_type) => type_name(held_type),
        None => displayed_class(&held),
    };
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &format!(
            "wrong argument type {} (expected {})",
            actual,
            type_name(data_type)
        ),
        called_from(),
    ))
}

/// What each `T_*` tag is called in a TypeError, by tag.
const TYPE_NAMES: [&str; 0x17] = [
    "",
    "Object",
    "Class",
    "Module",
    "Float",
    "String",
    "Regexp",
    "Array",
    "Hash",
    "Struct",
    "Integer",
    "File",
    "Data",
    "MatchData",
    "Complex",
    "Rational",
    "",
    "nil",
    "true",
    "false",
    "Symbol",
    "Integer",
    "undef",
];
const T_DATA: i32 = 0x0c;

/// Refuses `object` unless its type tag is `expected`. A typed Data object
/// is refused for T_DATA as well, since its pointer is checked by type.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_type(object: Value, expected: i32) {
    let held = to_object(object);
    let typed_data = expected == T_DATA && data_type_of(&held).is_some();
    if !typed_data && super::subclasses::rb_type(object) == expected {
        return;
    }
    let name = usize::try_from(expected)
        .ok()
        .and_then(|tag| TYPE_NAMES.get(tag))
        .filter(|name| !name.is_empty());
    let Some(name) = name else {
        raise(crate::vm::errors::simple_exception(
            "fatal",
            &format!(
                "unknown type 0x{:x} (0x{:x} given)",
                expected,
                super::subclasses::rb_type(object)
            ),
            called_from(),
        ))
    };
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &format!(
            "wrong argument type {} (expected {})",
            displayed_class(&held),
            name
        ),
        called_from(),
    ))
}

/// Makes the instances of `klass` and its subclasses with `allocator`,
/// which takes the class and answers the new object.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_alloc_func(klass: Value, allocator: *const ()) {
    let class = super::exports::class_from(klass);
    class.set_class_var(ALLOCATOR_VAR, Object::Int(allocator as i64));
}

/// The bytes `ObjectSpace.memsize_of` counts for what a typed Data object
/// wraps, through its type's size function.
pub(crate) fn wrapped_size(object: &Object) -> usize {
    let Some(key) = instance_key(object) else {
        return 0;
    };
    let found = WRAPPED.with(|held| {
        held.borrow().get(&key).and_then(|wrapped| {
            wrapped
                .data_type
                .map(|data_type| (data_type, wrapped.slot.data))
        })
    });
    match found {
        // SAFETY: the type C wrapped the pointer with outlives the object.
        Some((data_type, data)) => match unsafe { (*data_type).size } {
            Some(size) => size(data),
            None => 0,
        },
        None => 0,
    }
}

impl VirtualMachine {
    /// A new instance of `class` from the allocator C gave it or a
    /// superclass, or None when none has one.
    pub(crate) fn allocate_through_c(
        &mut self,
        class: &Rc<Class>,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Some(Object::Int(address)) = class.lookup_class_var(ALLOCATOR_VAR) else {
            return Ok(None);
        };
        // SAFETY: `rb_define_alloc_func` was handed a function taking the
        // class and answering the object it made.
        let allocator: extern "C-unwind" fn(Value) -> Value =
            unsafe { std::mem::transmute(address as usize as *const ()) };
        let klass = to_value(&Object::Class(Rc::clone(class)));
        let caller = Caller {
            position,
            block: None,
            keywords_given: false,
            method: None,
        };
        let made = super::enter(self, caller, || allocator(klass))?;
        Ok(Some(to_object(made)))
    }
}
