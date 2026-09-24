//! `String#[]=`, which puts text where the part an index names stood.
//!
//! Ruby settles where the part is before it converts the replacement, so a
//! missing match is refused without asking the replacement for `to_str`.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Object, StringValue};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::native_methods::as_range;
use crate::vm::native_methods::string_methods::{clashing_encodings_error, encodings_clash};
use std::rc::Rc;

const METHOD_NAME: &str = "[]=";

fn index_error(message: &str, position: Position) -> MetorexError {
    simple_exception("IndexError", message, position)
}

impl VirtualMachine {
    pub(crate) fn assign_string_part(
        &mut self,
        receiver: &Object,
        target: &Rc<StringValue>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let held = target.to_text();
        let letters = held.chars().count() as i64;
        let (start, width, value) = match arguments {
            [pattern @ Object::Regex(..), value] => {
                let (start, width) =
                    self.matched_group_span(receiver, pattern, &Object::Int(0), position)?;
                (start, width, value)
            }
            [pattern @ Object::Regex(..), group, value] => {
                let (start, width) = self.matched_group_span(receiver, pattern, group, position)?;
                (start, width, value)
            }
            [index, count, value] => {
                let index = self.coerce_slice_number(index, METHOD_NAME, position)?;
                let count = self.coerce_slice_number(count, METHOD_NAME, position)?;
                (index, count, value)
            }
            [Object::String(wanted), value] => {
                let wanted = wanted.to_text();
                let Some(byte_offset) = held.find(&wanted) else {
                    return Err(index_error("string not matched", position));
                };
                let start = held[..byte_offset].chars().count() as i64;
                (start, wanted.chars().count() as i64, value)
            }
            [index, value] => match as_range(index) {
                Some(range) => {
                    let (start, width) = self.range_span(&range, letters, position)?;
                    (start, width, value)
                }
                None => (
                    self.coerce_slice_number(index, METHOD_NAME, position)?,
                    1,
                    value,
                ),
            },
            _ => {
                return Err(argument_count_error(
                    Arity::Range(2, 3),
                    arguments.len(),
                    position,
                ));
            }
        };
        if width < 0 {
            return Err(index_error(&format!("negative length {width}"), position));
        }
        let replacement = self.replacement_string(value, position)?;
        if encodings_clash(target, &replacement) {
            return Err(clashing_encodings_error(target, &replacement, position));
        }
        if start > letters || start + letters < 0 {
            return Err(index_error(
                &format!("index {start} out of string"),
                position,
            ));
        }
        let start = if start < 0 { start + letters } else { start };
        let width = width.min(letters - start);
        // Text that is nothing but ASCII takes on the encoding of text that
        // is not, which is the encoding Ruby finds the two compatible in.
        if held.is_ascii() && !replacement.as_str().is_ascii() {
            target.set_encoding(replacement.encoding_name());
            if replacement.holds_bytes() {
                target.mark_bytes();
            }
        }
        let characters: Vec<char> = held.chars().collect();
        let (start, end) = (start as usize, (start + width) as usize);
        let mut made: String = characters[..start].iter().collect();
        made.push_str(&replacement.to_text());
        made.extend(characters[end..].iter());
        target.replace_text(made);
        Ok(Object::String(replacement))
    }

    /// Where the group a pattern's match holds starts and how many
    /// characters it spans, with the group named by number or by name.
    fn matched_group_span(
        &mut self,
        receiver: &Object,
        pattern: &Object,
        group: &Object,
        position: Position,
    ) -> Result<(i64, i64), MetorexError> {
        let matched =
            self.send_to_object(pattern.clone(), "match", vec![receiver.clone()], position)?;
        if matches!(matched, Object::Nil) {
            return Err(index_error("regexp not matched", position));
        }
        let group = match group {
            Object::String(_) | Object::Symbol(_) => group.clone(),
            other => {
                let mut nth = self.coerce_slice_number(other, METHOD_NAME, position)?;
                let Object::Int(groups) =
                    self.send_to_object(matched.clone(), "size", vec![], position)?
                else {
                    unreachable!("MatchData#size answers an Integer")
                };
                if nth >= groups || (nth < 0 && -nth >= groups) {
                    return Err(index_error(&format!("index {nth} out of regexp"), position));
                }
                if nth < 0 {
                    nth += groups;
                }
                Object::Int(nth)
            }
        };
        let begin = self.send_to_object(matched.clone(), "begin", vec![group.clone()], position)?;
        let Object::Int(begin) = begin else {
            return Err(index_error(
                &format!("regexp group {group} not matched"),
                position,
            ));
        };
        let Object::Int(end) = self.send_to_object(matched, "end", vec![group], position)? else {
            unreachable!("a group that begins also ends")
        };
        Ok((begin, end - begin))
    }

    /// Where a Range starts in a string of `letters` characters and how many
    /// it covers. An end past either side is cut back to the string, where a
    /// start outside it is refused.
    fn range_span(
        &mut self,
        range: &Object,
        letters: i64,
        position: Position,
    ) -> Result<(i64, i64), MetorexError> {
        let Object::Range {
            start,
            end,
            exclusive,
            ..
        } = range
        else {
            unreachable!("as_range answers only a Range")
        };
        let mut first = match start.as_ref() {
            Object::Nil => 0,
            other => self.coerce_slice_number(other, METHOD_NAME, position)?,
        };
        // An endless range runs through the last character.
        let (mut last, exclusive) = match end.as_ref() {
            Object::Nil => (-1, false),
            other => (
                self.coerce_slice_number(other, METHOD_NAME, position)?,
                *exclusive,
            ),
        };
        let out_of_range = |vm: &mut Self| -> Result<MetorexError, MetorexError> {
            let Object::String(spelled) =
                vm.send_to_object(range.clone(), "inspect", vec![], position)?
            else {
                unreachable!("Range#inspect answers a String")
            };
            Ok(simple_exception(
                "RangeError",
                &format!("{} out of range", spelled.to_text()),
                position,
            ))
        };
        if first < 0 {
            first += letters;
            if first < 0 {
                return Err(out_of_range(self)?);
            }
        }
        if first > letters {
            return Err(out_of_range(self)?);
        }
        if last < 0 {
            last += letters;
        }
        if !exclusive {
            last += 1;
        }
        let last = last.min(letters);
        Ok((first, (last - first).max(0)))
    }

    /// The String a replacement stands for, asking it for `to_str` when it is
    /// not already one.
    fn replacement_string(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Rc<StringValue>, MetorexError> {
        if let Object::String(text) = value {
            return Ok(Rc::clone(text));
        }
        if self.responds_to(value, "to_str")
            && let Object::String(text) =
                self.send_to_object(value.clone(), "to_str", vec![], position)?
        {
            return Ok(text);
        }
        let message = format!(
            "no implicit conversion of {} into String",
            self.builtins().class_of(value).name()
        );
        Err(simple_exception("TypeError", &message, position))
    }
}
