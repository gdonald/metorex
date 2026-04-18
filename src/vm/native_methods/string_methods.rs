//! Native method implementations for the String class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the String class.
    pub(crate) fn call_string_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(string_value) = receiver else {
            return Ok(None);
        };
        match method_name {
            "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(string_value.chars().count() as i64)))
            }
            "match?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "match?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let (pattern, flags) = match &arguments[0] {
                    Object::Regex(p, f) => (p.as_ref().clone(), f.as_ref().clone()),
                    Object::String(s) => (s.as_ref().clone(), String::new()),
                    other => {
                        return Err(method_argument_type_error(
                            "match?",
                            "Regexp or String",
                            other,
                            position,
                        ));
                    }
                };
                let re_pattern = if flags.contains('i') {
                    format!("(?i){}", pattern)
                } else {
                    pattern
                };
                match regex::Regex::new(&re_pattern) {
                    Ok(re) => Ok(Some(Object::Bool(re.is_match(string_value.as_ref())))),
                    Err(_) => Ok(Some(Object::Bool(false))),
                }
            }
            "upcase" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.to_uppercase())))
            }
            "downcase" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.to_lowercase())))
            }
            "+" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::String(rhs) => {
                        let mut combined = string_value.as_ref().clone();
                        combined.push_str(rhs);
                        Ok(Some(Object::string(combined)))
                    }
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            "trim" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.trim().to_string())))
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let reversed: String = string_value.chars().rev().collect();
                Ok(Some(Object::string(reversed)))
            }
            "last" => {
                let chars: Vec<char> = string_value.chars().collect();
                if chars.is_empty() {
                    Ok(Some(Object::Nil))
                } else {
                    Ok(Some(Object::string(chars.last().unwrap().to_string())))
                }
            }
            "chars" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let chars: Vec<Object> = string_value
                    .chars()
                    .map(|c| Object::string(c.to_string()))
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(chars)))))
            }
            "bytes" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let bytes: Vec<Object> = string_value
                    .bytes()
                    .map(|b| Object::Int(b as i64))
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(bytes)))))
            }
            "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(string_value.chars().count() as i64)))
            }
            // Stream-like predicates so STDOUT/STDERR (stored as String) can be checked
            "tty?" | "isatty" => Ok(Some(Object::Bool(false))),
            "flush" | "sync" | "sync=" | "fsync" => Ok(Some(Object::Nil)),
            // STDOUT/STDERR stream methods (receiver is the "STDOUT"/"STDERR" string).
            "puts" | "print" | "write" => {
                let to_stderr = string_value.as_str() == "STDERR";
                let newline = method_name == "puts";
                let mut out = String::new();
                for arg in arguments.iter() {
                    let s = match arg {
                        Object::String(s) => s.as_str().to_string(),
                        other => format!("{}", other),
                    };
                    out.push_str(&s);
                    if newline && !s.ends_with('\n') {
                        out.push('\n');
                    }
                }
                if newline && arguments.is_empty() {
                    out.push('\n');
                }
                if to_stderr {
                    eprint!("{}", out);
                } else {
                    print!("{}", out);
                }
                Ok(Some(Object::Nil))
            }
            "ljust" | "rjust" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let width = match &arguments[0] {
                    Object::Int(n) => *n,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let pad = if arguments.len() == 2 {
                    match &arguments[1] {
                        Object::String(s) if !s.is_empty() => (**s).clone(),
                        Object::String(_) => {
                            return Err(MetorexError::runtime_error(
                                format!("zero width padding for {}", method_name),
                                crate::vm::utils::position_to_location(position),
                            ));
                        }
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                &arguments[1],
                                position,
                            ));
                        }
                    }
                } else {
                    " ".to_string()
                };
                let current_len = string_value.chars().count() as i64;
                if width <= current_len {
                    return Ok(Some(Object::string(string_value.to_string())));
                }
                let pad_chars: Vec<char> = pad.chars().collect();
                let needed = (width - current_len) as usize;
                let mut padding = String::new();
                for i in 0..needed {
                    padding.push(pad_chars[i % pad_chars.len()]);
                }
                let result = if method_name == "ljust" {
                    format!("{}{}", string_value, padding)
                } else {
                    format!("{}{}", padding, string_value)
                };
                Ok(Some(Object::string(result)))
            }
            "strip" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.trim().to_string())))
            }
            "split" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let parts: Vec<Object> = if arguments.is_empty() {
                    string_value
                        .split_whitespace()
                        .map(|s| Object::string(s.to_string()))
                        .collect()
                } else {
                    match &arguments[0] {
                        Object::String(sep) => string_value
                            .split(sep.as_ref())
                            .map(|s| Object::string(s.to_string()))
                            .collect(),
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                &arguments[0],
                                position,
                            ));
                        }
                    }
                };
                Ok(Some(Object::Array(Rc::new(RefCell::new(parts)))))
            }
            "slice" | "[]" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let (start, len) = match (&arguments[0], &arguments[1]) {
                    (Object::Int(s), Object::Int(l)) => (*s, *l),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let chars: Vec<char> = string_value.chars().collect();
                let char_count = chars.len() as i64;
                let start_idx = if start < 0 {
                    (char_count + start).max(0) as usize
                } else {
                    start.min(char_count) as usize
                };
                let end_idx = (start_idx as i64 + len).min(char_count).max(0) as usize;
                if start_idx > chars.len() {
                    Ok(Some(Object::Nil))
                } else {
                    let sliced: String =
                        chars[start_idx..end_idx.min(chars.len())].iter().collect();
                    Ok(Some(Object::string(sliced)))
                }
            }
            "include?" | "contains?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::String(substr) => {
                        Ok(Some(Object::Bool(string_value.contains(substr.as_ref()))))
                    }
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            "start_with?" | "end_with?" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let mut result = false;
                for arg in arguments {
                    match arg {
                        Object::String(s) => {
                            if method_name == "start_with?" {
                                if string_value.starts_with(s.as_ref()) {
                                    result = true;
                                    break;
                                }
                            } else if string_value.ends_with(s.as_ref()) {
                                result = true;
                                break;
                            }
                        }
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                arg,
                                position,
                            ));
                        }
                    }
                }
                Ok(Some(Object::Bool(result)))
            }
            "starts_with?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::String(prefix) => Ok(Some(Object::Bool(
                        string_value.starts_with(prefix.as_ref()),
                    ))),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            "ends_with?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::String(suffix) => {
                        Ok(Some(Object::Bool(string_value.ends_with(suffix.as_ref()))))
                    }
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            "each_char" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            "each_char requires a block",
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };
                for ch in string_value.chars() {
                    let char_str = Object::string(ch.to_string());
                    let args = vec![char_str];
                    self.execute_block_body(&block, args)?;
                }
                Ok(Some(receiver.clone()))
            }
            "to_i" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let trimmed = string_value.trim();
                let n: i64 = trimmed
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0);
                Ok(Some(Object::Int(n)))
            }
            "to_f" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let trimmed = string_value.trim();
                let n: f64 = trimmed
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0.0);
                Ok(Some(Object::Float(n)))
            }
            "dup" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.as_ref().clone())))
            }
            "gsub" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let pattern = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let replacement = match &arguments[1] {
                    Object::String(s) => s.as_ref().clone(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[1],
                            position,
                        ));
                    }
                };
                Ok(Some(Object::string(
                    string_value.replace(&pattern, &replacement),
                )))
            }
            "sub" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let pattern = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let replacement = match &arguments[1] {
                    Object::String(s) => s.as_ref().clone(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[1],
                            position,
                        ));
                    }
                };
                Ok(Some(Object::string(string_value.replacen(
                    &pattern,
                    &replacement,
                    1,
                ))))
            }
            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(string_value.is_empty())))
            }
            _ => Ok(None),
        }
    }
}
