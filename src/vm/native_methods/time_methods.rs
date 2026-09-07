// The calendar arithmetic behind Time, which the C library carries out so a
// local time follows the zone rules the operating system holds.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::method_argument_error;
use std::ffi::CStr;

unsafe extern "C" {
    /// Re-reads the TZ setting, which a program that changed it needs before
    /// the next local-time call.
    fn tzset();
}

/// Days from 1970-01-01 to the given calendar date, which holds for any year
/// rather than only the span the C library covers.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let shifted = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * shifted + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146097 + day_of_era - 719468
}

/// The calendar date a day count since 1970-01-01 stands for.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146096
    } / 146097;
    let day_of_era = shifted - era * 146097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_position + 2) / 5 + 1;
    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// The calendar fields of a UTC second count, worked out directly so a year
/// outside the C library's range still reads.
fn utc_breakdown(seconds: i64) -> [i64; 8] {
    let days = seconds.div_euclid(86400);
    let within = seconds.rem_euclid(86400);
    let (year, month, day) = civil_from_days(days);
    let weekday = (days + 4).rem_euclid(7);
    let day_of_year = days - days_from_civil(year, 1, 1) + 1;
    [
        year,
        month,
        day,
        within / 3600,
        within % 3600 / 60,
        within % 60,
        weekday,
        day_of_year,
    ]
}

/// A zeroed `tm`, which the C calls fill in.
fn empty_tm() -> libc::tm {
    libc::tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: std::ptr::null_mut(),
    }
}

/// The calendar fields a second count stands for, either in UTC or in the
/// zone the operating system reports.
fn broken_down(seconds: i64, utc: bool) -> (libc::tm, String) {
    let stamp = seconds as libc::time_t;
    let mut parts = empty_tm();
    unsafe {
        tzset();
        if utc {
            libc::gmtime_r(&stamp, &mut parts);
        } else {
            libc::localtime_r(&stamp, &mut parts);
        }
    }
    let zone = if parts.tm_zone.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(parts.tm_zone) }
            .to_string_lossy()
            .into_owned()
    };
    (parts, zone)
}

/// The whole seconds a set of calendar fields stands for.
fn assembled(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    utc: bool,
) -> i64 {
    let mut parts = empty_tm();
    parts.tm_year = (year - 1900) as libc::c_int;
    parts.tm_mon = (month - 1) as libc::c_int;
    parts.tm_mday = day as libc::c_int;
    parts.tm_hour = hour as libc::c_int;
    parts.tm_min = minute as libc::c_int;
    parts.tm_sec = second as libc::c_int;
    parts.tm_isdst = -1;
    if utc {
        return days_from_civil(year, month, day) * 86400 + hour * 3600 + minute * 60 + second;
    }
    unsafe {
        tzset();
        libc::mktime(&mut parts) as i64
    }
}

impl VirtualMachine {
    /// The class-level calendar helpers Time is built on. Everything else
    /// about Time is written in Ruby against these.
    pub(crate) fn call_time_class_methods(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // The seconds and nanoseconds since the epoch, right now.
            "__now__" => {
                let since = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                Ok(Some(Object::array(vec![
                    Object::Int(since.as_secs() as i64),
                    Object::Int(since.subsec_nanos() as i64),
                ])))
            }
            // The calendar fields a second count stands for: year, month,
            // day, hour, minute, second, weekday, day of the year, whether
            // daylight saving is in force, the offset from UTC, and the
            // zone's name.
            "__breakdown__" => {
                let (seconds, utc) = two_arguments(method_name, arguments, position)?;
                if utc {
                    let fields = utc_breakdown(seconds);
                    return Ok(Some(Object::array(vec![
                        Object::Int(fields[0]),
                        Object::Int(fields[1]),
                        Object::Int(fields[2]),
                        Object::Int(fields[3]),
                        Object::Int(fields[4]),
                        Object::Int(fields[5]),
                        Object::Int(fields[6]),
                        Object::Int(fields[7]),
                        Object::Bool(false),
                        Object::Int(0),
                        Object::string("UTC"),
                    ])));
                }
                let (parts, zone) = broken_down(seconds, utc);
                Ok(Some(Object::array(vec![
                    Object::Int(parts.tm_year as i64 + 1900),
                    Object::Int(parts.tm_mon as i64 + 1),
                    Object::Int(parts.tm_mday as i64),
                    Object::Int(parts.tm_hour as i64),
                    Object::Int(parts.tm_min as i64),
                    Object::Int(parts.tm_sec as i64),
                    Object::Int(parts.tm_wday as i64),
                    Object::Int(parts.tm_yday as i64 + 1),
                    Object::Bool(parts.tm_isdst > 0),
                    Object::Int(parts.tm_gmtoff),
                    Object::string(zone),
                ])))
            }
            // The whole seconds a set of calendar fields stands for.
            "__assemble__" => {
                if arguments.len() != 7 {
                    return Err(method_argument_error(
                        method_name,
                        7,
                        arguments.len(),
                        position,
                    ));
                }
                let mut fields = [0i64; 6];
                for (slot, argument) in fields.iter_mut().zip(arguments.iter()) {
                    let Object::Int(value) = argument else {
                        return Ok(None);
                    };
                    *slot = *value;
                }
                let utc = matches!(arguments[6], Object::Bool(true));
                Ok(Some(Object::Int(assembled(
                    fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], utc,
                ))))
            }
            // The rendering a strftime template asks for, without any of the
            // fields Ruby adds on top of the C library's.
            "__format__" => {
                if arguments.len() != 3 {
                    return Err(method_argument_error(
                        method_name,
                        3,
                        arguments.len(),
                        position,
                    ));
                }
                let (Object::String(template), Object::Int(seconds)) =
                    (&arguments[0], &arguments[1])
                else {
                    return Ok(None);
                };
                let utc = matches!(arguments[2], Object::Bool(true));
                Ok(Some(Object::string(formatted(template, *seconds, utc))))
            }
            _ => Ok(None),
        }
    }
}

/// The two arguments the breakdown takes: a second count and whether it is
/// read in UTC.
fn two_arguments(
    method_name: &str,
    arguments: &[Object],
    position: Position,
) -> Result<(i64, bool), MetorexError> {
    if arguments.len() != 2 {
        return Err(method_argument_error(
            method_name,
            2,
            arguments.len(),
            position,
        ));
    }
    let seconds = match &arguments[0] {
        Object::Int(value) => *value,
        Object::Float(value) => *value as i64,
        _ => 0,
    };
    Ok((seconds, matches!(arguments[1], Object::Bool(true))))
}

/// What the C library's `strftime` makes of a template.
fn formatted(template: &str, seconds: i64, utc: bool) -> String {
    let (parts, _) = broken_down(seconds, utc);
    let Ok(pattern) = std::ffi::CString::new(template) else {
        return String::new();
    };
    let mut room = template.len().max(32) * 8 + 64;
    loop {
        let mut buffer = vec![0u8; room];
        let written = unsafe {
            libc::strftime(
                buffer.as_mut_ptr() as *mut libc::c_char,
                room,
                pattern.as_ptr(),
                &parts,
            )
        };
        if written > 0 || template.is_empty() {
            buffer.truncate(written);
            return String::from_utf8_lossy(&buffer).into_owned();
        }
        if room > 1 << 20 {
            return String::new();
        }
        room *= 4;
    }
}
