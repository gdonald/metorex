//! Ranges and arithmetic sequences from C: making a Range, reading its ends
//! back, and turning one into a start and a length within a sequence of a
//! given length.

use super::calls::{answers, call, top_level_module};
use super::handles::{QFALSE, QNIL, QTRUE, Value, to_object, to_value};
use super::numbers::long_from;
use super::{called_from, raise};
use crate::object::Object;

/// The ends of a range and whether it leaves out its last value.
struct Ends {
    first: Object,
    last: Object,
    exclusive: bool,
}

/// What `rb_arithmetic_sequence_extract` hands back, laid out as MRI's
/// `rb_arithmetic_sequence_components_t`.
#[repr(C)]
pub struct SequenceParts {
    begin: Value,
    end: Value,
    step: Value,
    exclude_end: i32,
}

fn is_a(object: &Object, class_path: &str) -> bool {
    let class = call(
        top_level_module("Object"),
        "const_get",
        vec![Object::string(class_path)],
    );
    call(object.clone(), "is_a?", vec![class]).is_truthy()
}

/// The ends of a Range, or of anything else that answers `begin`, `end` and
/// `exclude_end?`. An arithmetic sequence is not read as one.
fn range_ends(object: &Object) -> Option<Ends> {
    let read = |name: &str| call(object.clone(), name, Vec::new());
    if is_a(object, "Range") {
        return Some(Ends {
            first: read("begin"),
            last: read("end"),
            exclusive: read("exclude_end?").is_truthy(),
        });
    }
    if is_a(object, "Enumerator::ArithmeticSequence") {
        return None;
    }
    for name in ["begin", "end", "exclude_end?"] {
        if !answers(object, name) {
            return None;
        }
    }
    Some(Ends {
        first: read("begin"),
        last: read("end"),
        exclusive: read("exclude_end?").is_truthy(),
    })
}

fn out_of_range(object: &Object) -> ! {
    let inspected = call(object.clone(), "inspect", Vec::new()).to_string();
    raise(crate::vm::errors::simple_exception(
        "RangeError",
        &format!("{} out of range", inspected),
        called_from(),
    ))
}

/// The start and length a range covers in a sequence of `length` values,
/// as MRI's `rb_range_component_beg_len` finds them, or None when it starts
/// outside. With `err` 0 or 2 a range reaching past the end is cut short
/// and one starting past it is outside; with `err` 1 neither is.
fn start_and_length(ends: &Ends, length: i64, err: i32) -> Option<(i64, i64)> {
    let start = match ends.first {
        Object::Nil => 0,
        ref first => long_from(to_value(first)),
    };
    let (finish, exclusive) = match ends.last {
        Object::Nil => (-1, false),
        ref last => (long_from(to_value(last)), ends.exclusive),
    };
    let mut start = start;
    let mut finish = finish;
    if start < 0 {
        start += length;
        if start < 0 {
            return None;
        }
    }
    if finish < 0 {
        finish += length;
    }
    if !exclusive {
        finish += 1;
    }
    if err == 0 || err == 2 {
        if start > length {
            return None;
        }
        finish = finish.min(length);
    }
    Some((start, (finish - start).max(0)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_range_new(first: Value, last: Value, exclusive: i32) -> Value {
    let arguments = vec![
        to_object(first),
        to_object(last),
        Object::Bool(exclusive != 0),
    ];
    to_value(&call(top_level_module("Range"), "new", arguments))
}

/// Stores the ends of a range through the pointers and answers 1, or answers
/// 0 for an object that is not one.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_range_values(
    range: Value,
    first: &mut Value,
    last: &mut Value,
    exclusive: &mut i32,
) -> i32 {
    let Some(ends) = range_ends(&to_object(range)) else {
        return 0;
    };
    *first = to_value(&ends.first);
    *last = to_value(&ends.last);
    *exclusive = ends.exclusive as i32;
    1
}

/// Stores the start and length `range` covers in a sequence of `length`
/// values, answering true, nil when it lies outside and `err` is 0, or false
/// for an object that is not a range. With `err` set, outside is a
/// RangeError.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_range_beg_len(
    range: Value,
    start: &mut i64,
    covered: &mut i64,
    length: i64,
    err: i32,
) -> Value {
    let range = to_object(range);
    let Some(ends) = range_ends(&range) else {
        return QFALSE;
    };
    match start_and_length(&ends, length, err) {
        Some((found_start, found_length)) => {
            *start = found_start;
            *covered = found_length;
            QTRUE
        }
        None if err != 0 => out_of_range(&range),
        None => QNIL,
    }
}

/// The begin, end, step and exclusion of an arithmetic sequence, or of a
/// range with a step of 1, or None for anything else.
fn sequence_parts(object: &Object) -> Option<(Ends, Object)> {
    if is_a(object, "Enumerator::ArithmeticSequence") {
        let read = |name: &str| call(object.clone(), name, Vec::new());
        let ends = Ends {
            first: read("begin"),
            last: read("end"),
            exclusive: read("exclude_end?").is_truthy(),
        };
        return Some((ends, read("step")));
    }
    range_ends(object).map(|ends| (ends, Object::Int(1)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_arithmetic_sequence_extract(
    sequence: Value,
    parts: &mut SequenceParts,
) -> i32 {
    let Some((ends, step)) = sequence_parts(&to_object(sequence)) else {
        return 0;
    };
    parts.begin = to_value(&ends.first);
    parts.end = to_value(&ends.last);
    parts.step = to_value(&step);
    parts.exclude_end = ends.exclusive as i32;
    1
}

/// `rb_range_beg_len` for an arithmetic sequence, storing its step too. A
/// negative step walks the sequence from its end.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_arithmetic_sequence_beg_len_step(
    sequence: Value,
    start: &mut i64,
    covered: &mut i64,
    stride: &mut i64,
    length: i64,
    err: i32,
) -> Value {
    let sequence = to_object(sequence);
    let Some((mut ends, step)) = sequence_parts(&sequence) else {
        return QFALSE;
    };
    let step = match step {
        Object::Nil => 1,
        held => long_from(to_value(&held)),
    };
    *stride = step;
    if step < 0 {
        if ends.exclusive && !matches!(ends.last, Object::Nil) {
            ends.last = Object::Int(long_from(to_value(&ends.last)) + 1);
            ends.exclusive = false;
        }
        std::mem::swap(&mut ends.first, &mut ends.last);
    }
    if err == 0 && !(-1..=1).contains(&step) {
        return match start_and_length(&ends, length, 1) {
            Some((found_start, found_length))
                if found_start <= length && found_length <= length =>
            {
                *start = found_start;
                *covered = found_length;
                QTRUE
            }
            _ => out_of_range(&sequence),
        };
    }
    match start_and_length(&ends, length, err) {
        Some((found_start, found_length)) => {
            *start = found_start;
            *covered = found_length;
            QTRUE
        }
        None => QNIL,
    }
}
