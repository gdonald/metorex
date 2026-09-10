// Display trait implementation for Object

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::Object;

thread_local! {
    /// The collections currently being rendered. A collection that reaches
    /// itself prints `[...]` or `{...}` rather than recursing forever, which
    /// is what Ruby shows.
    static RENDERING: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// Run `render` with `address` marked as being rendered, answering None when
/// it already was.
pub(crate) fn render_guarded<T>(address: usize, render: impl FnOnce() -> T) -> Option<T> {
    let entered = RENDERING.with(|active| {
        let mut active = active.borrow_mut();
        if active.contains(&address) {
            return false;
        }
        active.push(address);
        true
    });
    if !entered {
        return None;
    }
    let rendered = render();
    RENDERING.with(|active| {
        active.borrow_mut().pop();
    });
    Some(rendered)
}

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
                let elements = arr.borrow().clone();
                let rendered = render_guarded(Rc::as_ptr(arr) as usize, || {
                    let mut body = String::new();
                    for (index, element) in elements.iter().enumerate() {
                        if index > 0 {
                            body.push_str(", ");
                        }
                        body.push_str(&format!("{}", element));
                    }
                    body
                });
                match rendered {
                    Some(body) => write!(f, "[{}]", body),
                    None => write!(f, "[...]"),
                }
            }
            Object::Dict(dict) => {
                let address = Rc::as_ptr(dict) as usize;
                let entered = RENDERING.with(|active| {
                    let mut active = active.borrow_mut();
                    if active.contains(&address) {
                        return false;
                    }
                    active.push(address);
                    true
                });
                if !entered {
                    return write!(f, "{{...}}");
                }
                let outcome = (|| {
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
                })();
                RENDERING.with(|active| {
                    active.borrow_mut().pop();
                });
                outcome
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
            // A Set renders the way `inspect` does, and prints `{...}` when
            // it reaches itself rather than recursing forever.
            Object::Set(set) => {
                let elements: Vec<Object> =
                    set.borrow().iter().map(|held| held.value.clone()).collect();
                let rendered = render_guarded(Rc::as_ptr(set) as usize, || {
                    elements
                        .iter()
                        .map(|element| match element {
                            Object::String(text) => format!("{:?}", text.as_str()),
                            Object::Symbol(name) => inspect_symbol(&name.as_str()),
                            Object::Nil => "nil".to_string(),
                            other => other.to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                });
                match rendered {
                    Some(body) => write!(f, "Set[{}]", body),
                    None => write!(f, "Set[...]"),
                }
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
                write!(f, "<Binding with {} vars>", binding.variable_count())
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

/// Whether `address` is already being rendered further up the stack.
pub(crate) fn rendering_in_progress(address: usize) -> bool {
    RENDERING.with(|active| active.borrow().contains(&address))
}

/// Mark `address` as being rendered. Every call is paired with one to
/// `end_rendering`.
pub(crate) fn begin_rendering(address: usize) {
    RENDERING.with(|active| active.borrow_mut().push(address));
}

/// Drop the innermost mark `begin_rendering` made.
pub(crate) fn end_rendering() {
    RENDERING.with(|active| {
        active.borrow_mut().pop();
    });
}

/// A symbol written the way `inspect` writes one. A name that would not read
/// back as a plain symbol is quoted, so `:"a b"` keeps its space.
pub(crate) fn inspect_symbol(name: &str) -> String {
    if reads_back_plainly(name) {
        return format!(":{name}");
    }
    format!(":{name:?}")
}

/// Whether a symbol's name could be written after a colon and read back as
/// the same symbol.
fn reads_back_plainly(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if matches!(
        name,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            | "=="
            | "==="
            | "!="
            | "<"
            | ">"
            | "<="
            | ">="
            | "<=>"
            | "<<"
            | ">>"
            | "[]"
            | "[]="
            | "!"
            | "~"
            | "&"
            | "|"
            | "^"
            | "=~"
            | "!~"
            | "+@"
            | "-@"
            | "`"
    ) {
        return true;
    }
    // An instance, class, or global variable name is plain once its sigil is
    // set aside.
    let body = name
        .strip_prefix("@@")
        .or_else(|| name.strip_prefix('@'))
        .or_else(|| name.strip_prefix('$'))
        .unwrap_or(name);
    let mut characters = body.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_') {
        return false;
    }
    let rest: Vec<char> = characters.collect();
    let Some((last, middle)) = rest.split_last() else {
        return true;
    };
    if !middle
        .iter()
        .all(|held| held.is_alphanumeric() || *held == '_')
    {
        return false;
    }
    // Only the last character may be one of the three a method name may end
    // with.
    last.is_alphanumeric() || *last == '_' || *last == '?' || *last == '!' || *last == '='
}
