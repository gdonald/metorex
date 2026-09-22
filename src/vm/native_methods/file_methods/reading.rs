// Reading a file, and what the operating system records about it.

use super::*;

impl VirtualMachine {
    /// Reading a file, and what the operating system records about it.
    pub(crate) fn call_file_reading_methods(
        &mut self,
        _class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // File.binread(path) answers the bytes the file holds, one to a
            // character, so a file that is not text reads back unchanged.
            "binread" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                if std::path::Path::new(&*path.as_str()).is_dir() {
                    return Err(crate::vm::errors::simple_exception(
                        "Errno::EISDIR",
                        &format!("Is a directory @ io_fread - {path}"),
                        position,
                    ));
                }
                // A count and an offset name a run of bytes rather than
                // the whole file. Neither may be negative: there is no run
                // of fewer than no bytes, and none before the first.
                let counted = match arguments.get(1) {
                    Some(Object::Int(held)) if *held < 0 => {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "negative length -1 given",
                            position,
                        ));
                    }
                    Some(Object::Int(held)) => Some(*held as usize),
                    _ => None,
                };
                let from = match arguments.get(2) {
                    Some(Object::Int(held)) if *held < 0 => {
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::EINVAL",
                            "Invalid argument - negative offset",
                            position,
                        ));
                    }
                    Some(Object::Int(held)) => *held as usize,
                    _ => 0,
                };
                let held = std::fs::read(&*path.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_sysopen - {path}: {problem}"),
                        position,
                    )
                })?;
                if from >= held.len() && counted.is_some() {
                    return Ok(Some(Object::Nil));
                }
                let rest = &held[from.min(held.len())..];
                let taken = match counted {
                    Some(wanted) => &rest[..wanted.min(rest.len())],
                    None => rest,
                };
                Ok(Some(
                    crate::vm::native_methods::pack_format::bytes_to_string(taken),
                ))
            }
            // File.read(path) answers everything the file holds.
            // The Ruby side reads a file through `File.open`, so this answers
            // only where a run of bytes is wanted with no options at all.
            "__read_file__" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                // A directory opens but reads as the wrong kind of thing,
                // which Ruby reports separately from a missing name.
                if std::path::Path::new(&*path.as_str()).is_dir() {
                    return Err(crate::vm::errors::simple_exception(
                        "Errno::EISDIR",
                        &format!("Is a directory @ io_fread - {path}"),
                        position,
                    ));
                }
                // A count and an offset name a run of bytes rather than the
                // whole file, which is what `IO.read(name, 4)` asks for.
                let counted = match arguments.get(1) {
                    Some(Object::Int(held)) => Some(*held as usize),
                    _ => None,
                };
                let from = match arguments.get(2) {
                    Some(Object::Int(held)) => *held as usize,
                    _ => 0,
                };
                if counted.is_none() && from == 0 {
                    let held = std::fs::read_to_string(&*path.as_str()).map_err(|problem| {
                        crate::vm::errors::simple_exception(
                            "Errno::ENOENT",
                            &format!("No such file or directory @ rb_sysopen - {path}: {problem}"),
                            position,
                        )
                    })?;
                    return Ok(Some(Object::string(held)));
                }
                let bytes = std::fs::read(&*path.as_str()).map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_sysopen - {path}: {problem}"),
                        position,
                    )
                })?;
                if from >= bytes.len() && counted.is_some() {
                    return Ok(Some(Object::Nil));
                }
                let rest = &bytes[from.min(bytes.len())..];
                let taken = match counted {
                    Some(wanted) => &rest[..wanted.min(rest.len())],
                    None => rest,
                };
                Ok(Some(
                    crate::vm::native_methods::pack_format::bytes_to_string(taken),
                ))
            }
            "__stat_fields__" => {
                use std::os::unix::fs::MetadataExt;
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(path) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                let follow = arguments[1].is_truthy();
                let held = if follow {
                    std::fs::metadata(&*path.as_str())
                } else {
                    std::fs::symlink_metadata(&*path.as_str())
                };
                let held = held.map_err(|problem| {
                    crate::vm::errors::simple_exception(
                        "Errno::ENOENT",
                        &format!("No such file or directory @ rb_file_s_stat - {path}: {problem}"),
                        position,
                    )
                })?;
                let kind = held.file_type();
                let mode = held.mode();
                let ftype = if kind.is_dir() {
                    "directory"
                } else if kind.is_symlink() {
                    "link"
                } else if mode & 0o170000 == 0o020000 {
                    "characterSpecial"
                } else if mode & 0o170000 == 0o060000 {
                    "blockSpecial"
                } else if mode & 0o170000 == 0o010000 {
                    "fifo"
                } else if mode & 0o170000 == 0o140000 {
                    "socket"
                } else {
                    "file"
                };
                let mut fields: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
                let mut record = |name: &str, value: Object| {
                    fields.insert(format!(":{name}"), value);
                };
                record("dev", Object::Int(held.dev() as i64));
                record("ino", Object::Int(held.ino() as i64));
                record("mode", Object::Int(mode as i64));
                record("nlink", Object::Int(held.nlink() as i64));
                record("uid", Object::Int(held.uid() as i64));
                record("gid", Object::Int(held.gid() as i64));
                record("rdev", Object::Int(held.rdev() as i64));
                record("size", Object::Int(held.size() as i64));
                record("blksize", Object::Int(held.blksize() as i64));
                record("blocks", Object::Int(held.blocks() as i64));
                // A time is kept to the nanosecond, and a whole number of
                // seconds would drop what a file's `usec` reports.
                let with_fraction = |seconds: i64, nanoseconds: i64| {
                    Object::Float(seconds as f64 + nanoseconds as f64 / 1_000_000_000.0)
                };
                record("atime", with_fraction(held.atime(), held.atime_nsec()));
                record("mtime", with_fraction(held.mtime(), held.mtime_nsec()));
                record("ctime", with_fraction(held.ctime(), held.ctime_nsec()));
                // Only some platforms keep the time a file was made.
                let born = held
                    .created()
                    .ok()
                    .and_then(|held| held.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|held| Object::Float(held.as_secs_f64()));
                record("birthtime", born.unwrap_or(Object::Nil));
                record("ftype", Object::string(ftype.to_string()));
                Ok(Some(Object::Dict(Rc::new(std::cell::RefCell::new(fields)))))
            }
            _ => Ok(None),
        }
    }
}
