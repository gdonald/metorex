// The primitives the etc library is written on: the password and group
// databases, the system configuration values, and what uname reports.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use std::ffi::{CStr, CString};

/// The text a C string holds, or an empty String for a null pointer.
///
/// # Safety
/// `text` must be null or point at a null-terminated string that outlives
/// the call.
unsafe fn text_at(text: *const libc::c_char) -> String {
    if text.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

/// A password entry as a Dict, which the Ruby side turns into an Etc::Passwd.
///
/// # Safety
/// `entry` must be null or point at a `passwd` the C library still owns.
unsafe fn passwd_dict(entry: *const libc::passwd) -> Object {
    if entry.is_null() {
        return Object::Nil;
    }
    let held = unsafe { &*entry };
    let mut fields: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
    fields.insert(
        ":name".to_string(),
        Object::string(unsafe { text_at(held.pw_name) }),
    );
    fields.insert(
        ":passwd".to_string(),
        Object::string(unsafe { text_at(held.pw_passwd) }),
    );
    fields.insert(":uid".to_string(), Object::Int(held.pw_uid as i64));
    fields.insert(":gid".to_string(), Object::Int(held.pw_gid as i64));
    fields.insert(
        ":gecos".to_string(),
        Object::string(unsafe { text_at(held.pw_gecos) }),
    );
    fields.insert(
        ":dir".to_string(),
        Object::string(unsafe { text_at(held.pw_dir) }),
    );
    fields.insert(
        ":shell".to_string(),
        Object::string(unsafe { text_at(held.pw_shell) }),
    );
    Object::dict(fields)
}

/// A group entry as a Dict, which the Ruby side turns into an Etc::Group.
///
/// # Safety
/// `entry` must be null or point at a `group` the C library still owns.
unsafe fn group_dict(entry: *const libc::group) -> Object {
    if entry.is_null() {
        return Object::Nil;
    }
    let held = unsafe { &*entry };
    let mut members: Vec<Object> = Vec::new();
    // The member list is a run of pointers ending in a null one. A pointer
    // that is not aligned for one is not a list, which is what the group
    // database hands back for an entry that carries no members.
    if !held.gr_mem.is_null()
        && held
            .gr_mem
            .align_offset(std::mem::align_of::<*mut libc::c_char>())
            == 0
    {
        let mut walked = held.gr_mem;
        // SAFETY: the list ends with a null pointer, and the alignment was
        // checked above.
        while !unsafe { *walked }.is_null() {
            members.push(Object::string(unsafe { text_at(*walked) }));
            walked = unsafe { walked.add(1) };
        }
    }
    let mut fields: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
    fields.insert(
        ":name".to_string(),
        Object::string(unsafe { text_at(held.gr_name) }),
    );
    fields.insert(
        ":passwd".to_string(),
        Object::string(unsafe { text_at(held.gr_passwd) }),
    );
    fields.insert(":gid".to_string(), Object::Int(held.gr_gid as i64));
    fields.insert(":mem".to_string(), Object::array(members));
    Object::dict(fields)
}

/// The system configuration names the etc library reports, paired with the
/// numbers the C library knows them by.
fn configuration_names() -> Vec<(&'static str, i64)> {
    vec![
        ("SC_ARG_MAX", libc::_SC_ARG_MAX as i64),
        ("SC_CHILD_MAX", libc::_SC_CHILD_MAX as i64),
        ("SC_HOST_NAME_MAX", libc::_SC_HOST_NAME_MAX as i64),
        ("SC_LOGIN_NAME_MAX", libc::_SC_LOGIN_NAME_MAX as i64),
        ("SC_NGROUPS_MAX", libc::_SC_NGROUPS_MAX as i64),
        ("SC_CLK_TCK", libc::_SC_CLK_TCK as i64),
        ("SC_OPEN_MAX", libc::_SC_OPEN_MAX as i64),
        ("SC_PAGESIZE", libc::_SC_PAGESIZE as i64),
        ("SC_RE_DUP_MAX", libc::_SC_RE_DUP_MAX as i64),
        ("SC_STREAM_MAX", libc::_SC_STREAM_MAX as i64),
        ("SC_SYMLOOP_MAX", libc::_SC_SYMLOOP_MAX as i64),
        ("SC_TTY_NAME_MAX", libc::_SC_TTY_NAME_MAX as i64),
        ("SC_TZNAME_MAX", libc::_SC_TZNAME_MAX as i64),
        ("SC_VERSION", libc::_SC_VERSION as i64),
        ("CS_PATH", libc::_CS_PATH as i64),
    ]
}

impl VirtualMachine {
    /// The primitives `Etc` is written on. Every name here opens with two
    /// underscores, since the library's own methods are written in Ruby.
    pub(crate) fn call_etc_methods(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "__pw_by_uid__" => {
                let Some(Object::Int(uid)) = arguments.first() else {
                    return Ok(Some(Object::Nil));
                };
                // SAFETY: `getpwuid` answers a pointer the C library owns,
                // read before any other call to it.
                Ok(Some(unsafe { passwd_dict(libc::getpwuid(*uid as u32)) }))
            }
            "__pw_by_name__" => {
                let Some(Object::String(name)) = arguments.first() else {
                    return Ok(Some(Object::Nil));
                };
                let Ok(held) = CString::new(name.as_str().as_bytes().to_vec()) else {
                    return Ok(Some(Object::Nil));
                };
                // SAFETY: as above, with a name that lives across the call.
                Ok(Some(unsafe { passwd_dict(libc::getpwnam(held.as_ptr())) }))
            }
            "__pw_next__" => {
                // SAFETY: `getpwent` walks the database one entry at a time.
                Ok(Some(unsafe { passwd_dict(libc::getpwent()) }))
            }
            "__pw_rewind__" => {
                // SAFETY: `setpwent` and `endpwent` only move the cursor.
                unsafe { libc::setpwent() };
                Ok(Some(Object::Nil))
            }
            "__pw_end__" => {
                // SAFETY: as above.
                unsafe { libc::endpwent() };
                Ok(Some(Object::Nil))
            }
            "__gr_by_gid__" => {
                let Some(Object::Int(gid)) = arguments.first() else {
                    return Ok(Some(Object::Nil));
                };
                // SAFETY: `getgrgid` answers a pointer the C library owns.
                Ok(Some(unsafe { group_dict(libc::getgrgid(*gid as u32)) }))
            }
            "__gr_by_name__" => {
                let Some(Object::String(name)) = arguments.first() else {
                    return Ok(Some(Object::Nil));
                };
                let Ok(held) = CString::new(name.as_str().as_bytes().to_vec()) else {
                    return Ok(Some(Object::Nil));
                };
                // SAFETY: as above, with a name that lives across the call.
                Ok(Some(unsafe { group_dict(libc::getgrnam(held.as_ptr())) }))
            }
            "__gr_next__" => {
                // SAFETY: `getgrent` walks the database one entry at a time.
                Ok(Some(unsafe { group_dict(libc::getgrent()) }))
            }
            "__gr_rewind__" => {
                // SAFETY: `setgrent` and `endgrent` only move the cursor.
                unsafe { libc::setgrent() };
                Ok(Some(Object::Nil))
            }
            "__gr_end__" => {
                // SAFETY: as above.
                unsafe { libc::endgrent() };
                Ok(Some(Object::Nil))
            }
            "__login__" => {
                // SAFETY: `getlogin` answers a pointer the C library owns.
                let named = unsafe { text_at(libc::getlogin()) };
                if named.is_empty() {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(Object::string(named)))
            }
            "__uname__" => {
                let mut held: libc::utsname = unsafe { std::mem::zeroed() };
                // SAFETY: `uname` fills the struct it is handed.
                if unsafe { libc::uname(&mut held) } != 0 {
                    return Ok(Some(Object::Nil));
                }
                let mut fields: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
                let mut record = |name: &str, field: &[libc::c_char]| {
                    fields.insert(
                        format!(":{name}"),
                        Object::string(unsafe { text_at(field.as_ptr()) }),
                    );
                };
                record("sysname", &held.sysname);
                record("nodename", &held.nodename);
                record("release", &held.release);
                record("version", &held.version);
                record("machine", &held.machine);
                Ok(Some(Object::dict(fields)))
            }
            "__nprocessors__" => {
                // SAFETY: `sysconf` reads one configuration value.
                let counted = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
                Ok(Some(Object::Int(if counted < 1 { 1 } else { counted })))
            }
            "__sysconf__" => {
                let Some(Object::Int(name)) = arguments.first() else {
                    return Err(method_argument_type_error(
                        "sysconf",
                        "Integer",
                        arguments.first().unwrap_or(&Object::Nil),
                        position,
                    ));
                };
                // SAFETY: `sysconf` reads one configuration value and reports
                // -1 for a name it does not know.
                let answered = unsafe { libc::sysconf(*name as i32) };
                if answered < 0 {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(Object::Int(answered)))
            }
            "__confstr__" => {
                let Some(Object::Int(name)) = arguments.first() else {
                    return Err(method_argument_type_error(
                        "confstr",
                        "Integer",
                        arguments.first().unwrap_or(&Object::Nil),
                        position,
                    ));
                };
                let mut held = [0 as libc::c_char; 1024];
                // SAFETY: `confstr` writes at most the count it is given.
                let written = unsafe { libc::confstr(*name as i32, held.as_mut_ptr(), held.len()) };
                if written == 0 {
                    return Err(simple_exception(
                        "Errno::EINVAL",
                        "Invalid argument - confstr",
                        position,
                    ));
                }
                Ok(Some(Object::string(unsafe { text_at(held.as_ptr()) })))
            }
            "__config_names__" => {
                let mut fields: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
                for (name, value) in configuration_names() {
                    fields.insert(name.to_string(), Object::Int(value));
                }
                Ok(Some(Object::dict(fields)))
            }
            _ => Ok(None),
        }
    }
}
