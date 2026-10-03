//! The `st_table` hash table C extensions use for their own data: keys and
//! values are machine words, compared and hashed by the table's type, and
//! walked in the order they were added.

use std::collections::HashMap;
use std::ffi::{CStr, c_char};

/// What a walk's function answers to go on, stop, or drop the entry.
const ST_STOP: i32 = 1;
const ST_DELETE: i32 = 2;

/// How a table compares and hashes its keys. `compare` answers 0 for equal.
#[repr(C)]
pub struct StHashType {
    compare: extern "C-unwind" fn(usize, usize) -> i32,
    hash: extern "C-unwind" fn(usize) -> usize,
}

/// The table C holds a pointer to. C reads `num_entries` itself.
#[repr(C)]
pub struct StTable {
    hash_type: *const StHashType,
    num_entries: usize,
    entries: *mut Entries,
}

/// The entries in the order they were added, with a slot emptied when its
/// entry is deleted, and the slots holding each hash.
#[derive(Default)]
struct Entries {
    slots: Vec<Option<(usize, usize)>>,
    buckets: HashMap<usize, Vec<usize>>,
}

extern "C-unwind" fn number_compare(first: usize, second: usize) -> i32 {
    (first != second) as i32
}

extern "C-unwind" fn number_hash(key: usize) -> usize {
    key
}

/// The bytes of the C string at `key`.
fn c_string<'a>(key: usize) -> &'a [u8] {
    // SAFETY: a string table's keys are NUL-terminated C strings.
    unsafe { CStr::from_ptr(key as *const c_char) }.to_bytes()
}

extern "C-unwind" fn string_compare(first: usize, second: usize) -> i32 {
    (c_string(first) != c_string(second)) as i32
}

extern "C-unwind" fn string_hash(key: usize) -> usize {
    hash_bytes(c_string(key).iter().copied())
}

extern "C-unwind" fn string_case_compare(first: usize, second: usize) -> i32 {
    !c_string(first).eq_ignore_ascii_case(c_string(second)) as i32
}

extern "C-unwind" fn string_case_hash(key: usize) -> usize {
    hash_bytes(c_string(key).iter().map(u8::to_ascii_lowercase))
}

/// FNV-1a over `bytes`.
fn hash_bytes(bytes: impl Iterator<Item = u8>) -> usize {
    bytes.fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    }) as usize
}

static NUMBER_TYPE: StHashType = StHashType {
    compare: number_compare,
    hash: number_hash,
};
static STRING_TYPE: StHashType = StHashType {
    compare: string_compare,
    hash: string_hash,
};
static STRING_CASE_TYPE: StHashType = StHashType {
    compare: string_case_compare,
    hash: string_case_hash,
};

fn new_table(hash_type: *const StHashType) -> *mut StTable {
    Box::into_raw(Box::new(StTable {
        hash_type,
        num_entries: 0,
        entries: Box::into_raw(Box::default()),
    }))
}

impl StTable {
    fn hash_type(&self) -> &StHashType {
        // SAFETY: the type is one of the statics above or one C keeps for as
        // long as the table it made with it.
        unsafe { &*self.hash_type }
    }

    fn entries(&mut self) -> &mut Entries {
        // SAFETY: `new_table` made the entries and only `rb_st_free_table`
        // frees them.
        unsafe { &mut *self.entries }
    }

    /// The slot holding `key`, if any.
    fn find(&mut self, key: usize) -> Option<usize> {
        let compare = self.hash_type().compare;
        let hash = (self.hash_type().hash)(key);
        let entries = self.entries();
        let candidates = entries.buckets.get(&hash)?.clone();
        candidates
            .into_iter()
            .find(|slot| entries.slots[*slot].is_some_and(|(held, _)| compare(key, held) == 0))
    }

    fn add(&mut self, key: usize, value: usize) {
        let hash = (self.hash_type().hash)(key);
        let entries = self.entries();
        entries.slots.push(Some((key, value)));
        let slot = entries.slots.len() - 1;
        entries.buckets.entry(hash).or_default().push(slot);
        self.num_entries += 1;
    }

    fn remove(&mut self, slot: usize) -> (usize, usize) {
        let entries = self.entries();
        let removed = entries.slots[slot]
            .take()
            .expect("a slot found holding an entry still holds it");
        for held in entries.buckets.values_mut() {
            held.retain(|candidate| *candidate != slot);
        }
        self.num_entries -= 1;
        removed
    }
}

/// The table C handed over.
fn table<'a>(table: *mut StTable) -> &'a mut StTable {
    // SAFETY: C hands back a pointer one of the `rb_st_init_*` functions
    // answered and has not freed.
    unsafe { &mut *table }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_table(hash_type: *const StHashType) -> *mut StTable {
    new_table(hash_type)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_table_with_size(
    hash_type: *const StHashType,
    _size: usize,
) -> *mut StTable {
    new_table(hash_type)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_numtable() -> *mut StTable {
    new_table(&NUMBER_TYPE)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_numtable_with_size(_size: usize) -> *mut StTable {
    new_table(&NUMBER_TYPE)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_strtable() -> *mut StTable {
    new_table(&STRING_TYPE)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_strtable_with_size(_size: usize) -> *mut StTable {
    new_table(&STRING_TYPE)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_strcasetable() -> *mut StTable {
    new_table(&STRING_CASE_TYPE)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_init_strcasetable_with_size(_size: usize) -> *mut StTable {
    new_table(&STRING_CASE_TYPE)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_free_table(freed: *mut StTable) {
    // SAFETY: the table and its entries came from `Box::into_raw` in
    // `new_table`, and C frees a table once.
    unsafe {
        let freed = Box::from_raw(freed);
        drop(Box::from_raw(freed.entries));
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_clear(cleared: *mut StTable) {
    let cleared = table(cleared);
    *cleared.entries() = Entries::default();
    cleared.num_entries = 0;
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_table_size(held: *mut StTable) -> usize {
    table(held).num_entries
}

/// Sets `key` to `value`, answering 1 when the key was already there.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_insert(held: *mut StTable, key: usize, value: usize) -> i32 {
    let held = table(held);
    match held.find(key) {
        Some(slot) => {
            held.entries().slots[slot] = Some((key, value));
            1
        }
        None => {
            held.add(key, value);
            0
        }
    }
}

/// Adds `key` without looking for it first, which C does for a key it knows
/// is new.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_add_direct(held: *mut StTable, key: usize, value: usize) {
    table(held).add(key, value);
}

/// Answers 1 and stores the value through `value` when `key` is there.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_lookup(held: *mut StTable, key: usize, value: *mut usize) -> i32 {
    let held = table(held);
    let Some(slot) = held.find(key) else {
        return 0;
    };
    if !value.is_null() {
        let (_, found) = held.entries().slots[slot].expect("a found slot holds an entry");
        // SAFETY: C hands over somewhere to put the value, or NULL.
        unsafe { *value = found };
    }
    1
}

/// Removes the key `key` points at, storing the key and value held through
/// the two pointers and answering 1, or storing 0 as the value and answering
/// 0 when it is not there.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_delete(
    held: *mut StTable,
    key: *mut usize,
    value: *mut usize,
) -> i32 {
    let held = table(held);
    // SAFETY: C hands over the address of the key to delete.
    let wanted = unsafe { *key };
    let Some(slot) = held.find(wanted) else {
        if !value.is_null() {
            // SAFETY: C hands over somewhere to put the value, or NULL.
            unsafe { *value = 0 };
        }
        return 0;
    };
    let (removed_key, removed_value) = held.remove(slot);
    // SAFETY: as above, for both pointers.
    unsafe {
        *key = removed_key;
        if !value.is_null() {
            *value = removed_value;
        }
    }
    1
}

/// Calls `function` with each key, value and `data` in the order they were
/// added, stopping or deleting the entry when it answers so.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_st_foreach(
    held: *mut StTable,
    function: extern "C-unwind" fn(usize, usize, usize) -> i32,
    data: usize,
) -> i32 {
    let held = table(held);
    let mut slot = 0;
    while slot < held.entries().slots.len() {
        if let Some((key, value)) = held.entries().slots[slot] {
            match function(key, value, data) {
                ST_STOP => return 0,
                ST_DELETE => {
                    held.remove(slot);
                }
                _ => {}
            }
        }
        slot += 1;
    }
    0
}
