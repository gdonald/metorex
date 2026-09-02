// Display trait implementation for Object

use std::fmt;
use std::rc::Rc;

use super::Object;

// Implement Display for Object to provide string representation
impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Int(i) => write!(f, "{}", i),
            Object::BigInt(i) => write!(f, "{}", i),
            // Ruby spells the non-finite floats out rather than using Rust's
            // "inf" / "NaN" forms.
            Object::Float(fl) if fl.is_nan() => write!(f, "NaN"),
            Object::Float(fl) if fl.is_infinite() => {
                write!(f, "{}Infinity", if *fl < 0.0 { "-" } else { "" })
            }
            // Ruby always shows a Float with a fractional part, so 1.0 reads
            // as "1.0" rather than "1".
            Object::Float(fl) if fl.fract() == 0.0 && fl.abs() < 1e16 => {
                write!(f, "{:.1}", fl)
            }
            Object::Float(fl) => write!(f, "{}", fl),
            Object::String(s) => write!(f, "{}", s),
            Object::Symbol(s) => write!(f, ":{}", s),
            Object::Array(arr) => {
                write!(f, "[")?;
                let elements = arr.borrow();
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Object::Dict(dict) => {
                write!(f, "{{")?;
                let map = dict.borrow();
                let mut written = 0;
                for (key, value) in map.iter() {
                    // The sentinel entries a Hash keeps for its default proc
                    // and non-primitive keys are bookkeeping, not contents.
                    if key.starts_with("__MX_") {
                        continue;
                    }
                    if written > 0 {
                        write!(f, ", ")?;
                    }
                    written += 1;
                    // A Symbol key reads as `name: value`, and every other
                    // kind as `key => value`, the way Ruby shows them. A key
                    // that is not a String keeps its own rendering.
                    match key.strip_prefix(':') {
                        Some(name) => write!(f, "{}: {}", name, value)?,
                        None if key.parse::<f64>().is_ok()
                            || key == "true"
                            || key == "false"
                            || key == "nil" =>
                        {
                            write!(f, "{} => {}", key, value)?
                        }
                        None => write!(f, "{:?} => {}", key, value)?,
                    }
                }
                write!(f, "}}")
            }
            // Ruby's default `to_s` for an object: its class and address.
            Object::Instance(inst) => {
                let class_name = inst.borrow().class.inspect_name();
                write!(f, "#<{}:0x{:016x}>", class_name, Rc::as_ptr(inst) as usize)
            }
            // A class or module displays under the name Ruby reports for it,
            // which includes one set by `set_temporary_name` or derived from
            // an anonymous namespace.
            Object::Class(class) => write!(f, "{}", class.inspect_name()),
            Object::Module(module) => write!(f, "{}", module.inspect_name()),
            Object::Method(method) => write!(f, "<method {}>", method.name),
            Object::Block(_) => write!(f, "<block>"),
            Object::Exception(exc) => {
                let exception = exc.borrow();
                write!(f, "{}: {}", exception.exception_type, exception.message)
            }
            Object::Set(set) => {
                write!(f, "#{{")?;
                let elements = set.borrow();
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem.hash_value)?;
                }
                write!(f, "}}")
            }
            Object::Result(result) => match result {
                Ok(obj) => write!(f, "Ok({})", obj),
                Err(obj) => write!(f, "Err({})", obj),
            },
            Object::NativeFunction(name) => write!(f, "<native function {}>", name),
            Object::Range {
                start,
                end,
                exclusive,
            } => {
                if *exclusive {
                    write!(f, "{}...{}", start, end)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
            Object::Binding(binding) => {
                write!(f, "<Binding with {} vars>", binding.variables.len())
            }
            Object::CompiledFunction(func) => write!(f, "{}", func),
            Object::Regex(pattern, flags) => {
                if flags.is_empty() {
                    write!(f, "/{}/", pattern)
                } else {
                    write!(f, "/{}/{}", pattern, flags)
                }
            }
        }
    }
}
