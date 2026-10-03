//! Hashes from C: making, reading, writing and walking them, `rb_hash` on
//! any object, and the hash code functions an extension mixes its own
//! hash with.

use super::calls::{call, top_level_module};
use super::handles::{QNIL, Value, objects_from, to_object, to_value};
use super::{called_from, called_with_block, interpreter, or_raise, raise};
use crate::object::Object;
use num_bigint::{BigInt, Sign};

const FIXNUM_MAX: i64 = i64::MAX >> 1;
const FIXNUM_MIN: i64 = i64::MIN >> 1;

/// The hash code of `object` through its `hash` method, as a Fixnum. A
/// Bignum keeps its low bits and its sign, and anything else is converted
/// with `to_int`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash(object: Value) -> Value {
    let mut code = call(to_object(object), "hash", Vec::new());
    loop {
        let number = match &code {
            Object::Int(number) => BigInt::from(*number),
            Object::BigInt(number) => (**number).clone(),
            other => {
                let converted =
                    or_raise(interpreter().coerce_integer_argument(other, called_from()));
                code = crate::vm::native_methods::from_big(converted);
                continue;
            }
        };
        if let Ok(small) = i64::try_from(&number)
            && (FIXNUM_MIN..=FIXNUM_MAX).contains(&small)
        {
            return to_value(&Object::Int(small));
        }
        let low_bits = number.magnitude().iter_u64_digits().next().unwrap_or(0) as i64;
        let folded = if number.sign() == Sign::Minus {
            low_bits | FIXNUM_MIN
        } else {
            low_bits & FIXNUM_MAX
        };
        return to_value(&Object::Int(folded));
    }
}

/// `Kernel#Hash`: an empty Hash for nil or an empty Array, and otherwise
/// what `to_hash` answers.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_Hash(object: Value) -> Value {
    let arguments = vec![Object::symbol("Hash"), to_object(object)];
    to_value(&call(top_level_module("Kernel"), "__send__", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_new() -> Value {
    to_value(&Object::Dict(Default::default()))
}

/// A new Hash. The capacity only sizes storage, and a negative one is the
/// error MRI's table gives.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_new_capa(capacity: i64) -> Value {
    if capacity < 0 {
        raise(crate::vm::errors::simple_exception(
            "RuntimeError",
            "st_table too big",
            called_from(),
        ));
    }
    rb_hash_new()
}

/// A new Hash comparing its keys by identity.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ident_hash_new() -> Value {
    let hash = to_object(rb_hash_new());
    to_value(&call(hash, "compare_by_identity", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_freeze(hash: Value) -> Value {
    to_value(&call(to_object(hash), "freeze", Vec::new()))
}

/// The value for `key`, or the Hash's default when it holds none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_aref(hash: Value, key: Value) -> Value {
    to_value(&call(to_object(hash), "[]", vec![to_object(key)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_aset(hash: Value, key: Value, value: Value) -> Value {
    call(
        to_object(hash),
        "[]=",
        vec![to_object(key), to_object(value)],
    );
    value
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_clear(hash: Value) -> Value {
    call(to_object(hash), "clear", Vec::new());
    hash
}

/// Removes `key`, answering its value, or nil when the Hash held none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_delete(hash: Value, key: Value) -> Value {
    to_value(&call(to_object(hash), "delete", vec![to_object(key)]))
}

/// Removes each entry the block of the C method running now answers true
/// for, or answers an Enumerator when it was given none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_delete_if(hash: Value) -> Value {
    if let Some(block) = called_with_block() {
        interpreter().pending_block = Some(block);
    }
    to_value(&call(to_object(hash), "delete_if", Vec::new()))
}

/// The value for `key`, raising a KeyError when the Hash holds none,
/// whatever its default.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_fetch(hash: Value, key: Value) -> Value {
    to_value(&call(to_object(hash), "fetch", vec![to_object(key)]))
}

/// The value for `key`, or `missing` when the Hash holds none, whatever its
/// default. `missing` may be Qundef, which is handed back as it is.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_lookup2(hash: Value, key: Value, missing: Value) -> Value {
    let held = to_object(hash);
    if call(held.clone(), "key?", vec![to_object(key)]).is_truthy() {
        return rb_hash_aref(hash, key);
    }
    missing
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_lookup(hash: Value, key: Value) -> Value {
    rb_hash_lookup2(hash, key, QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_size(hash: Value) -> Value {
    to_value(&call(to_object(hash), "size", Vec::new()))
}

/// Sets the value the Hash answers for a key it does not hold.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_set_ifnone(hash: Value, default: Value) -> Value {
    call(to_object(hash), "default=", vec![to_object(default)]);
    hash
}

/// Stores the `count` values at `pairs` as keys and values in turn, a later
/// key replacing an earlier one.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_bulk_insert(count: i64, pairs: *const Value, hash: Value) {
    if count == 0 {
        return;
    }
    let values = objects_from(count, pairs);
    let held = to_object(hash);
    for pair in values.chunks(2) {
        if let [key, value] = pair {
            call(held.clone(), "[]=", vec![key.clone(), value.clone()]);
        }
    }
}

/// What a walk's function answers: go on, stop, or remove the entry.
const ST_STOP: i32 = 1;
const ST_DELETE: i32 = 2;

/// Calls `function` with each key and value and `argument`, stopping or
/// removing the entry as it answers.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_foreach(
    hash: Value,
    function: extern "C-unwind" fn(Value, Value, Value) -> i32,
    argument: Value,
) {
    let held = to_object(hash);
    let Object::Array(pairs) = call(held.clone(), "to_a", Vec::new()) else {
        return;
    };
    let pairs = pairs.borrow().clone();
    for pair in pairs {
        let Object::Array(pair) = pair else {
            continue;
        };
        let (key, value) = {
            let pair = pair.borrow();
            (pair[0].clone(), pair[1].clone())
        };
        match function(to_value(&key), to_value(&value), argument) {
            ST_STOP => return,
            ST_DELETE => {
                call(held.clone(), "delete", vec![key]);
            }
            _ => {}
        }
    }
}

/// The salt `rb_hash_start` adds, fixed for the life of the process as
/// MRI's is.
fn hash_salt() -> u64 {
    static SALT: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *SALT.get_or_init(|| {
        use std::hash::{BuildHasher, Hasher};
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_u32(std::process::id());
        hasher.finish()
    })
}

/// One step of the simplified MurmurHash3 MRI's tables mix with.
fn murmur_step(hash: u64, word: u64) -> u64 {
    const C1: u64 = 0x87c3_7b91_1142_53d5;
    const C2: u64 = 0x4cf5_ad43_2745_937f;
    let word = word.wrapping_mul(C1);
    let hash = (hash ^ word.rotate_left(33)).wrapping_mul(C2);
    hash.rotate_left(24)
}

fn murmur_finish(hash: u64) -> u64 {
    const C1: u64 = 0xbf58_476d_1ce4_e5b9;
    const C2: u64 = 0x94d0_49bb_1331_11eb;
    let mut hash = hash;
    hash ^= hash >> 30;
    hash = hash.wrapping_mul(C1);
    hash ^= hash >> 27;
    hash = hash.wrapping_mul(C2);
    hash ^= hash >> 31;
    hash
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_start(hash: u64) -> u64 {
    hash_salt().wrapping_add(hash)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_uint32(hash: u64, word: u32) -> u64 {
    murmur_step(hash, u64::from(word))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_uint(hash: u64, word: u64) -> u64 {
    murmur_step(hash, word.wrapping_add(hash))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_end(hash: u64) -> u64 {
    murmur_finish(hash)
}
