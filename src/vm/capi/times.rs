//! Times from C: making one from seconds and a fraction, and reading a Time
//! or a number of seconds back as a `struct timeval` or `struct timespec`.

use super::arrays::array_of;
use super::calls::{answers, call, top_level_module};
use super::handles::{QNIL, Value, to_object, to_value};
use super::numbers::long_from;
use super::{called_from, interpreter, raise};
use crate::object::Object;
use crate::vm::native_methods::from_big;
use num_bigint::BigInt;

const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
const MICROSECONDS_PER_SECOND: i64 = 1_000_000;
/// The offsets `rb_time_timespec_new` reads as the zone the program runs in
/// and as UTC, rather than as a count of seconds.
const LOCAL_OFFSET: i32 = i32::MAX;
const UTC_OFFSET: i32 = i32::MAX - 1;

fn time_class() -> Object {
    top_level_module("Time")
}

fn error(class: &str, message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        class,
        &message,
        called_from(),
    ))
}

/// The exact number of seconds `seconds` and `parts` parts of a second make,
/// with `per_second` parts to a second.
fn exact_seconds(seconds: i64, parts: i64, per_second: i64) -> Object {
    let total = BigInt::from(seconds) * per_second + parts;
    call(
        top_level_module("Kernel"),
        "Rational",
        vec![from_big(total), Object::Int(per_second)],
    )
}

/// A Time `total` seconds after the epoch, in UTC, at a fixed offset, or in
/// the zone the program runs in when neither is given.
fn time_at(total: Object, utc: bool, offset: Object) -> Value {
    let arguments = vec![total, Object::Bool(utc), offset];
    to_value(&call(time_class(), "from_exact", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_time_new(seconds: i64, microseconds: i64) -> Value {
    let total = exact_seconds(seconds, microseconds, MICROSECONDS_PER_SECOND);
    time_at(total, false, Object::Nil)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_time_nano_new(seconds: i64, nanoseconds: i64) -> Value {
    let total = exact_seconds(seconds, nanoseconds, NANOSECONDS_PER_SECOND);
    time_at(total, false, Object::Nil)
}

/// A Time `seconds` after the epoch, in the zone `offset` names, or in the
/// zone the program runs in when it is nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_time_num_new(seconds: Value, offset: Value) -> Value {
    let mut arguments = vec![to_object(seconds)];
    if offset != QNIL {
        let mut keywords = indexmap::IndexMap::new();
        keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
        keywords.insert(":in".to_string(), to_object(offset));
        arguments.push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
            keywords,
        ))));
    }
    to_value(&call(time_class(), "at", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_time_timespec_new(time: &libc::timespec, offset: i32) -> Value {
    let total = exact_seconds(time.tv_sec, time.tv_nsec, NANOSECONDS_PER_SECOND);
    match offset {
        LOCAL_OFFSET => time_at(total, false, Object::Nil),
        UTC_OFFSET => time_at(total, true, Object::Nil),
        _ if -86400 < offset && offset < 86400 => {
            time_at(total, false, Object::Int(i64::from(offset)))
        }
        _ => error("ArgumentError", "utc_offset out of range".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_timespec_now(time: &mut libc::timespec) {
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    time.tv_sec = since_epoch.as_secs() as libc::time_t;
    time.tv_nsec = since_epoch.subsec_nanos() as libc::c_long;
}

/// Splits `value` into whole seconds and nanoseconds the way MRI's
/// `time_timespec` does, refusing a negative one when it is an interval.
fn split_seconds(value: Object, interval: bool) -> (i64, i64) {
    let described = if interval { "time interval" } else { "time" };
    let refuse_negative = |seconds: f64| {
        if interval && seconds < 0.0 {
            error(
                "ArgumentError",
                "time interval must not be negative".to_string(),
            );
        }
    };
    match &value {
        Object::Int(_) | Object::BigInt(_) => {
            let seconds = long_from(to_value(&value));
            refuse_negative(seconds as f64);
            (seconds, 0)
        }
        Object::Float(number) => {
            refuse_negative(*number);
            let mut whole = number.trunc();
            let fraction = number - whole;
            let mut nanoseconds;
            if fraction >= 0.0 {
                nanoseconds = (fraction * 1e9 + 0.5) as i64;
                if nanoseconds >= NANOSECONDS_PER_SECOND {
                    nanoseconds -= NANOSECONDS_PER_SECOND;
                    whole += 1.0;
                }
            } else {
                nanoseconds = (-fraction * 1e9 + 0.5) as i64;
                if nanoseconds > 0 {
                    nanoseconds = NANOSECONDS_PER_SECOND - nanoseconds;
                    whole -= 1.0;
                }
            }
            let seconds = whole as i64;
            if whole != seconds as f64 {
                error("RangeError", format!("{:.6} out of Time range", number));
            }
            (seconds, nanoseconds)
        }
        _ => {
            let parts = if answers(&value, "divmod") {
                array_of(call(value.clone(), "divmod", vec![Object::Int(1)]))
            } else {
                None
            };
            let Some(parts) = parts else {
                error(
                    "TypeError",
                    format!(
                        "can't convert {} into {}",
                        interpreter().conversion_name(&value),
                        described
                    ),
                )
            };
            let whole = parts.first().cloned().unwrap_or(Object::Nil);
            let fraction = parts.get(1).cloned().unwrap_or(Object::Nil);
            let seconds = long_from(to_value(&whole));
            refuse_negative(seconds as f64);
            let scaled = call(fraction, "*", vec![Object::Int(NANOSECONDS_PER_SECOND)]);
            (seconds, long_from(to_value(&scaled)))
        }
    }
}

/// The seconds and nanoseconds of a Time, or of a number of seconds.
fn timespec_of(value: Value, interval: bool) -> (i64, i64) {
    let value = to_object(value);
    let is_time = !interval
        && matches!(
            call(value.clone(), "is_a?", vec![time_class()]),
            Object::Bool(true)
        );
    if !is_time {
        return split_seconds(value, interval);
    }
    let exact = call(value, "to_r", Vec::new());
    let parts = array_of(call(exact, "divmod", vec![Object::Int(1)])).unwrap_or_default();
    let whole = parts.first().cloned().unwrap_or(Object::Nil);
    let fraction = parts.get(1).cloned().unwrap_or(Object::Nil);
    let scaled = call(fraction, "*", vec![Object::Int(NANOSECONDS_PER_SECOND)]);
    let nanoseconds = call(scaled, "floor", Vec::new());
    (
        long_from(to_value(&whole)),
        long_from(to_value(&nanoseconds)),
    )
}

fn timeval_of(value: Value, interval: bool) -> libc::timeval {
    let (seconds, nanoseconds) = timespec_of(value, interval);
    libc::timeval {
        tv_sec: seconds as libc::time_t,
        tv_usec: (nanoseconds / 1000) as libc::suseconds_t,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_time_interval(seconds: Value) -> libc::timeval {
    timeval_of(seconds, true)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_time_timeval(time: Value) -> libc::timeval {
    timeval_of(time, false)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_time_timespec(time: Value) -> libc::timespec {
    let (seconds, nanoseconds) = timespec_of(time, false);
    libc::timespec {
        tv_sec: seconds as libc::time_t,
        tv_nsec: nanoseconds as libc::c_long,
    }
}
