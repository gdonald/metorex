// The moment, the name, and the path an argument stands for.

use super::*;

impl VirtualMachine {
    /// The moment a value names, in seconds since the epoch. A Time keeps
    /// the part of a second as a Rational, so the object is asked what it
    /// stands for rather than read field by field.
    pub(crate) fn time_in_seconds(
        &mut self,
        value: &Object,
        now: f64,
        position: Position,
    ) -> Result<f64, MetorexError> {
        match value {
            Object::Nil => Ok(now),
            Object::Int(held) => Ok(*held as f64),
            Object::Float(held) => Ok(*held),
            other if self.responds_to(other, "to_f") => {
                match self.send_to_object(other.clone(), "to_f", vec![], position)? {
                    Object::Float(held) => Ok(held),
                    Object::Int(held) => Ok(held as f64),
                    _ => Ok(0.0),
                }
            }
            other => Ok(self_seconds(other).unwrap_or(0.0)),
        }
    }

    /// The name a value stands for: a String as itself, and anything else
    /// through `to_path`, which is how Ruby reads a path argument.
    pub(crate) fn path_text(
        &mut self,
        method_name: &str,
        given: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = given {
            return Ok(path_text(text));
        }
        if self
            .send_to_object(
                given.clone(),
                "respond_to?",
                vec![Object::symbol("to_path".to_string())],
                position,
            )?
            .is_truthy()
        {
            let named = self.send_to_object(given.clone(), "to_path", Vec::new(), position)?;
            if let Object::String(text) = &named {
                return Ok(path_text(text));
            }
        }
        Err(method_argument_type_error(
            method_name,
            "String",
            given,
            position,
        ))
    }

    /// The String an object names itself as, asking `to_path` first and
    /// `to_str` after, or None when it names neither.
    pub(crate) fn path_naming_answer(
        &mut self,
        candidate: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        for name in ["to_path", "to_str"] {
            if self.responds_to(candidate, name) {
                let named = self.send_to_object(candidate.clone(), name, vec![], position)?;
                if matches!(named, Object::String(_)) {
                    return Ok(Some(named));
                }
            }
        }
        // A stream stands for the file it is open on, which is what the
        // predicates read when they are handed one.
        if self.responds_to(candidate, "to_io") {
            let named = self.send_to_object(candidate.clone(), "to_io", vec![], position)?;
            if let Object::Instance(_) = &named
                && self.responds_to(&named, "fileno")
            {
                // A stream opened over a file names that file. One opened
                // over anything else is named by its descriptor, which the
                // system carries under a directory of its own.
                let path = self.send_to_object(named.clone(), "path", vec![], position)?;
                if matches!(path, Object::String(_)) {
                    return Ok(Some(path));
                }
                let number = self.send_to_object(named.clone(), "fileno", vec![], position)?;
                if let Object::Int(held) = number {
                    return Ok(Some(Object::string(format!("/dev/fd/{held}"))));
                }
            }
            if matches!(named, Object::String(_) | Object::Instance(_)) {
                return Ok(Some(named));
            }
        }
        Ok(None)
    }
}

/// The seconds a Time stands for, read off the object rather than through a
/// method call, so `File.utime` takes one without asking the virtual machine.
fn self_seconds(value: &Object) -> Option<f64> {
    let Object::Instance(instance) = value else {
        return None;
    };
    let instance = instance.borrow();
    if instance.class.name() != "Time" {
        return None;
    }
    // A Time keeps the whole seconds apart from the part of a second, and
    // both are needed for a file to be given the time it names.
    let whole = match instance.instance_vars.get("seconds") {
        Some(Object::Int(held)) => *held as f64,
        Some(Object::Float(held)) => *held,
        _ => return None,
    };
    let fraction = match instance.instance_vars.get("fraction") {
        Some(Object::Int(held)) => *held as f64,
        Some(Object::Float(held)) => *held,
        _ => 0.0,
    };
    Some(whole + fraction)
}

/// Whether a File or Dir class method reads its first argument as a path.
/// Only the names answered natively belong here: the ones written in Ruby
/// ask for `to_path` themselves, and converting twice would call it twice.
pub(crate) fn names_a_path(method_name: &str) -> bool {
    matches!(
        method_name,
        "exist?"
            | "exists?"
            | "file?"
            | "directory?"
            | "executable?"
            | "symlink?"
            | "readlink"
            | "size"
            | "size?"
            | "symlink"
            | "link"
            | "rename"
    )
}

/// The methods that take a second path, which is named the same way the
/// first one is.
pub(crate) fn names_a_second_path(method_name: &str) -> bool {
    matches!(method_name, "symlink" | "link" | "rename")
}

/// The characters a path names. A name standing for the bytes it was read
/// from is read back through those bytes, so the name reaches the operating
/// system the way it was written.
pub(crate) fn path_text(held: &Rc<crate::object::StringValue>) -> String {
    // A binary string's characters are its bytes, whether or not it was
    // built from them.
    let binary = matches!(held.encoding_name().as_str(), "ASCII-8BIT" | "BINARY");
    if !held.holds_bytes() && !binary {
        return held.as_str().to_string();
    }
    let bytes = crate::vm::native_methods::string_methods::binary_bytes(held);
    match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => held.as_str().to_string(),
    }
}

/// The mode a count of flag bits names. The access bits say whether the file
/// is read, written, or both, and the rest say how it is opened.
/// Whether a mode string spells an access mode Ruby reads: `r`, `w`, or `a`
/// first, then any of `+`, `b` or `t` but not both of the last two, and `x`
/// only after `w`, with encodings after a colon.
pub(crate) fn valid_access_mode(mode: &str) -> bool {
    let access = mode.split(':').next().unwrap_or("");
    let mut letters = access.chars();
    let Some(first) = letters.next() else {
        return false;
    };
    if !matches!(first, 'r' | 'w' | 'a') {
        return false;
    }
    let mut binary = false;
    let mut text = false;
    for letter in letters {
        match letter {
            '+' => {}
            'b' => binary = true,
            't' => text = true,
            'x' if first == 'w' => {}
            _ => return false,
        }
    }
    !(binary && text)
}

pub(crate) fn mode_of_flags(bits: i64) -> String {
    let bits = bits as libc::c_int;
    let appends = bits & libc::O_APPEND != 0;
    let truncates = bits & libc::O_TRUNC != 0;
    match bits & libc::O_ACCMODE {
        held if held == libc::O_WRONLY => {
            if appends {
                "a".to_string()
            } else {
                "w".to_string()
            }
        }
        held if held == libc::O_RDWR => {
            if truncates {
                "w+".to_string()
            } else if appends {
                "a+".to_string()
            } else {
                "r+".to_string()
            }
        }
        _ => "r".to_string(),
    }
}

/// Whether an argument is the hash of keywords a call carries rather than one
/// the program wrote out as an argument of its own.
pub(crate) fn names_keywords(given: &Object) -> bool {
    let Object::Dict(entries) = given else {
        return false;
    };
    entries
        .borrow()
        .contains_key(crate::vm::param_binding::KWARGS_MARKER)
}
