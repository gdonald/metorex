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
            Object::Float(fl) => write!(f, "{}", float_text(*fl)),
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
            // A callable displays as its address and where it was written,
            // with a lambda saying so.
            Object::Block(block) => {
                let shape = if block.is_lambda { " (lambda)" } else { "" };
                if let Some(named) = &block.from_symbol {
                    return write!(
                        f,
                        "#<Proc:0x{:016x} (&:{}){}>",
                        Rc::as_ptr(block) as usize,
                        named,
                        shape
                    );
                }
                match (&block.source_file, block.opened_at) {
                    (Some(file), Some(line)) => write!(
                        f,
                        "#<Proc:0x{:016x} {}:{}{}>",
                        Rc::as_ptr(block) as usize,
                        file,
                        line,
                        shape
                    ),
                    _ => write!(f, "#<Proc:0x{:016x}{}>", Rc::as_ptr(block) as usize, shape),
                }
            }
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
                ..
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
    let mut out = String::with_capacity(name.len() + 3);
    out.push_str(":\"");
    for character in name.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{b}' => out.push_str("\\v"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            '\u{1b}' => out.push_str("\\e"),
            // Every other character that does not print is named by the byte
            // it stands for, the way Ruby names one.
            character if character.is_control() => {
                out.push_str(&format!("\\x{:02X}", character as u32))
            }
            character => out.push(character),
        }
    }
    out.push('"');
    out
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
    if let Some(body) = name.strip_prefix("@@").or_else(|| name.strip_prefix('@')) {
        return spells_a_name(body);
    }
    if let Some(body) = name.strip_prefix('$') {
        return spells_a_global(body);
    }
    // A method name may close with one of the three characters a question,
    // a change, or an assignment is written with.
    let body = match name.strip_suffix(['?', '!', '=']) {
        Some(body) => body,
        None => name,
    };
    spells_a_name(body)
}

/// Whether a run of characters spells a bare name. Ruby reads every character
/// outside ASCII as part of one, so `:\u{1F98A}` needs no quotes.
fn spells_a_name(body: &str) -> bool {
    let mut characters = body.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first == '_' || !first.is_ascii() || first.is_ascii_alphabetic()) {
        return false;
    }
    characters.all(|held| held == '_' || !held.is_ascii() || held.is_ascii_alphanumeric())
}

/// Whether a run of characters spells a global name once the dollar sign is
/// set aside: a bare name, the digits of a capture, one of the names Ruby
/// keeps for itself, or a dash and one character.
fn spells_a_global(body: &str) -> bool {
    if body.is_empty() {
        return false;
    }
    if body.chars().all(|held| held.is_ascii_digit()) {
        return true;
    }
    let mut characters = body.chars();
    let first = characters.next().unwrap_or(' ');
    if first == '-' {
        return characters.next().is_some() && characters.next().is_none();
    }
    if body.chars().count() == 1
        && matches!(
            first,
            '~' | '*'
                | '$'
                | '?'
                | '!'
                | '@'
                | '/'
                | '\\'
                | ';'
                | ','
                | '.'
                | '='
                | ':'
                | '<'
                | '>'
                | '"'
                | '&'
                | '`'
                | '\''
                | '+'
        )
    {
        return true;
    }
    spells_a_name(body)
}

/// The text Ruby writes a finite Float as. Every one shows a fractional part,
/// and one whose decimal point sits past the fifteenth significant place or
/// before the fourth place to its left is written in exponent form.
pub fn float_text(value: f64) -> String {
    let scientific = format!("{:e}", value);
    let Some((mantissa, exponent)) = scientific.split_once('e') else {
        return scientific;
    };
    let Ok(place) = exponent.parse::<i32>() else {
        return scientific;
    };
    if (-4..15).contains(&place) {
        let written = format!("{}", value);
        if written.contains('.') {
            return written;
        }
        return format!("{}.0", written);
    }
    let mantissa = if mantissa.contains('.') {
        mantissa.to_string()
    } else {
        format!("{}.0", mantissa)
    };
    format!(
        "{}e{}{:02}",
        mantissa,
        if place < 0 { "-" } else { "+" },
        place.abs()
    )
}
