// Who owns a file, what it may be read and written by, and when it was
// last touched.

use super::*;

impl VirtualMachine {
    /// Who owns a file, what it may be read and written by, and when it was
    /// last touched.
    pub(crate) fn call_file_ownership_methods(
        &mut self,
        _class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // File.umask — returns the current process umask. We stub a
            // typical default so spec helpers that branch on
            // `(File.umask & 0002) == 0` can run; this isn't
            // process-accurate but is enough for the autoload tmp-dir
            // bootstrap.
            // The bits the operating system clears from the mode of every
            // file this process makes. With an argument it is set to that
            // and the one it held is answered.
            "umask" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error("umask", 1, arguments.len(), position));
                }
                let Some(wanted) = arguments.first() else {
                    let held = unsafe { libc::umask(0) };
                    unsafe { libc::umask(held) };
                    return Ok(Some(Object::Int(held as i64)));
                };
                let asked = match wanted {
                    Object::Int(value) => *value,
                    Object::BigInt(_) => {
                        return Err(crate::vm::errors::simple_exception(
                            "RangeError",
                            "bignum too big to convert into `long'",
                            position,
                        ));
                    }
                    other => {
                        match self.send_to_object(other.clone(), "to_int", Vec::new(), position)? {
                            Object::Int(value) => value,
                            _ => {
                                return Err(method_argument_type_error(
                                    "umask", "Integer", other, position,
                                ));
                            }
                        }
                    }
                };
                let held = unsafe { libc::umask(asked as libc::mode_t) };
                Ok(Some(Object::Int(held as i64)))
            }
            // File.stat — returns a stub File::Stat object. The class is
            // memoized as a global so subsequent stat()s share its method
            // table; `world_writable?` and `sticky?` are stubbed to return
            // false. Spec helpers (mspec's `tmp`) gate on these to
            // validate the temp dir mode; metorex creates the dir itself
            // so reporting "safe" is fine.
            // The numbers the operating system keeps about a file. The
            // File::Stat that presents them is written in Ruby on top of
            // this, so only the reading of them is native.
            // File.link(old, new) makes another name for the same file, and
            // File.utime sets the times a file reports.
            // File.chown(uid, gid, *paths) sets who owns a file. A -1 for
            // either leaves that one as it was.
            "chown" | "lchown" => {
                if arguments.len() < 3 {
                    return Err(method_argument_error(
                        method_name,
                        3,
                        arguments.len(),
                        position,
                    ));
                }
                let owner = match &arguments[0] {
                    Object::Int(held) => *held as libc::uid_t,
                    Object::Nil => libc::uid_t::MAX,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let group = match &arguments[1] {
                    Object::Int(held) => *held as libc::gid_t,
                    Object::Nil => libc::gid_t::MAX,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let mut changed = 0i64;
                for argument in &arguments[2..].to_vec() {
                    let path = self.directory_path_argument(method_name, argument, position)?;
                    let Ok(name) = std::ffi::CString::new(path.as_bytes().to_vec()) else {
                        continue;
                    };
                    // SAFETY: `chown` and `lchown` read the name and the two
                    // ids, and touch nothing else.
                    let answer = unsafe {
                        if method_name == "chown" {
                            libc::chown(name.as_ptr(), owner, group)
                        } else {
                            libc::lchown(name.as_ptr(), owner, group)
                        }
                    };
                    if answer != 0 {
                        let problem = std::io::Error::last_os_error();
                        let named = match problem.raw_os_error() {
                            Some(code) if code == libc::ENOENT => "Errno::ENOENT",
                            Some(code) if code == libc::EACCES => "Errno::EACCES",
                            Some(code) if code == libc::ENOTDIR => "Errno::ENOTDIR",
                            _ => "Errno::EPERM",
                        };
                        return Err(crate::vm::errors::simple_exception(
                            named,
                            &format!("{problem} @ chown - {path}"),
                            position,
                        ));
                    }
                    changed += 1;
                }
                Ok(Some(Object::Int(changed)))
            }
            "link" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let (Object::String(existing), Object::String(added)) =
                    (&arguments[0], &arguments[1])
                else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                std::fs::hard_link(&*existing.as_str(), &*added.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::EEXIST",
                        &format!("File exists @ rb_file_s_link - {added}: {problem}"),
                        position,
                    )
                })?;
                Ok(Some(Object::Int(0)))
            }
            // `utime` follows a symlink to the file it names, and `lutime`
            // sets the times on the link itself.
            "utime" | "lutime" => {
                if arguments.len() < 3 {
                    return Err(method_argument_error(
                        method_name,
                        3,
                        arguments.len(),
                        position,
                    ));
                }
                // A time written as nil is this moment, which is what
                // touching a file with no times given sets.
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|held| held.as_secs_f64())
                    .unwrap_or_default();
                let accessed = self.time_in_seconds(&arguments[0], now, position)?;
                let modified = self.time_in_seconds(&arguments[1], now, position)?;
                let mut touched = 0i64;
                let named: Vec<String> = arguments[2..]
                    .iter()
                    .map(|argument| self.path_text(method_name, argument, position))
                    .collect::<Result<Vec<String>, MetorexError>>()?;
                for path in &named {
                    let times = [
                        libc::timeval {
                            tv_sec: accessed.trunc() as libc::time_t,
                            tv_usec: ((accessed.fract() * 1_000_000.0) as libc::suseconds_t).abs(),
                        },
                        libc::timeval {
                            tv_sec: modified.trunc() as libc::time_t,
                            tv_usec: ((modified.fract() * 1_000_000.0) as libc::suseconds_t).abs(),
                        },
                    ];
                    let Ok(name) = std::ffi::CString::new(path.as_str().as_bytes().to_vec()) else {
                        continue;
                    };
                    // SAFETY: both calls read the name and the two times, and
                    // touch nothing else.
                    let answer = if method_name == "lutime" {
                        unsafe { libc::lutimes(name.as_ptr(), times.as_ptr()) }
                    } else {
                        unsafe { libc::utimes(name.as_ptr(), times.as_ptr()) }
                    };
                    if answer != 0 {
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::ENOENT",
                            &format!("No such file or directory @ utime_failed - {path}"),
                            position,
                        ));
                    }
                    touched += 1;
                }
                Ok(Some(Object::Int(touched)))
            }
            _ => Ok(None),
        }
    }
}
