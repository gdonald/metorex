// What a clock reads, and the limits a resource carries.

use super::*;

impl VirtualMachine {
    /// What a clock reads, in nanoseconds. A clock is named by the number the
    /// operating system holds it under.
    pub(crate) fn clock_reading(
        &mut self,
        clock: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        let Object::Int(number) = clock else {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("unexpected clock: {}", clock.type_name()),
                position,
            ));
        };
        let mut held = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: `clock_gettime` only writes through the pointer given.
        if unsafe { libc::clock_gettime(*number as libc::clockid_t, &mut held) } != 0 {
            return Err(crate::vm::errors::simple_exception(
                "Errno::EINVAL",
                "Invalid argument - clock_gettime",
                position,
            ));
        }
        Ok(held.tv_sec as f64 * 1_000_000_000.0 + held.tv_nsec as f64)
    }

    /// The smallest step a clock reports, in nanoseconds. The named clocks
    /// stand for the system calls Ruby reads them through, each with a
    /// resolution of its own.
    pub(crate) fn clock_resolution(
        &mut self,
        clock: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        if let Object::Symbol(named) = clock {
            return Ok(match &*named.as_str() {
                "GETTIMEOFDAY_BASED_CLOCK_REALTIME"
                | "GETRUSAGE_BASED_CLOCK_PROCESS_CPUTIME_ID" => 1_000.0,
                "TIME_BASED_CLOCK_REALTIME" => 1_000_000_000.0,
                "CLOCK_BASED_CLOCK_PROCESS_CPUTIME_ID" => 10_000_000.0,
                other => {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("unexpected clock: {other}"),
                        position,
                    ));
                }
            });
        }
        let Object::Int(number) = clock else {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("unexpected clock: {}", clock.type_name()),
                position,
            ));
        };
        let mut held = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: `clock_getres` only writes through the pointer given.
        if unsafe { libc::clock_getres(*number as libc::clockid_t, &mut held) } != 0 {
            return Err(crate::vm::errors::simple_exception(
                "Errno::EINVAL",
                "Invalid argument - clock_getres",
                position,
            ));
        }
        Ok(held.tv_sec as f64 * 1_000_000_000.0 + held.tv_nsec as f64)
    }

    /// The resource number `getrlimit` and `setrlimit` were given. Ruby takes
    /// the number itself, the short name the `RLIMIT_` constant is spelled
    /// with, or an object that answers with either.
    pub(crate) fn rlimit_resource(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<crate::vm::RlimitResource, MetorexError> {
        let named = match argument {
            Some(Object::Int(number)) => return Ok(*number as crate::vm::RlimitResource),
            Some(Object::Symbol(name)) => Some(name.as_str().to_string()),
            Some(Object::String(name)) => Some(name.as_str().to_string()),
            Some(other) if self.answers_to(other, "to_str", position)? => {
                match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                    Object::String(name) => Some(name.as_str().to_string()),
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(named) = named {
            let full = format!("RLIMIT_{named}");
            for &(name, value) in crate::vm::init::PROCESS_CONSTANTS {
                if name == full {
                    return Ok(value as crate::vm::RlimitResource);
                }
            }
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("invalid resource name: {named}"),
                position,
            ));
        }
        let number = self.process_id_argument(argument, position)?;
        Ok(number as crate::vm::RlimitResource)
    }

    /// One of the two bounds a resource limit is made of.
    pub(crate) fn rlimit_value(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<libc::rlim_t, MetorexError> {
        match argument {
            Some(Object::Int(number)) => Ok(*number as libc::rlim_t),
            Some(other) if self.answers_to(other, "to_int", position)? => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(number) => Ok(number as libc::rlim_t),
                    converted => Err(method_argument_type_error(
                        "Process", "Integer", &converted, position,
                    )),
                }
            }
            Some(other) => Err(method_argument_type_error(
                "Process", "Integer", other, position,
            )),
            None => Err(method_argument_error("setrlimit", 2, 1, position)),
        }
    }
}

/// One clock reading written in the unit a caller named. Ruby counts whole
/// units as Integers and fractional ones as Floats.
pub(crate) fn clock_in_unit(
    nanoseconds: f64,
    unit: &str,
    position: Position,
) -> Result<Object, MetorexError> {
    Ok(match unit {
        "nanosecond" => Object::Int(nanoseconds as i64),
        "microsecond" => Object::Int((nanoseconds / 1_000.0) as i64),
        "millisecond" => Object::Int((nanoseconds / 1_000_000.0) as i64),
        "second" => Object::Int((nanoseconds / 1_000_000_000.0) as i64),
        "float_microsecond" => Object::Float(nanoseconds / 1_000.0),
        "float_millisecond" => Object::Float(nanoseconds / 1_000_000.0),
        "float_second" => Object::Float(nanoseconds / 1_000_000_000.0),
        other => {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("unexpected unit: {other}"),
                position,
            ));
        }
    })
}
