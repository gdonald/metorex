// The runs of bytes an encoding reads, and the ones it cannot.

use super::*;

impl VirtualMachine {
    /// `scrub` answers the string with every run of bytes its encoding cannot
    /// read put aside: a replacement named alongside stands for each one, a
    /// block is asked what to put there, and without either the encoding's
    /// own replacement character stands in.
    pub(crate) fn scrubbed_string(
        &mut self,
        held: &std::rc::Rc<crate::object::StringValue>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        use crate::vm::native_methods::string_methods::binary_bytes;
        let named = held.encoding_name();
        let bytes = binary_bytes(held);
        let block = match self.pending_block.take() {
            Some(Object::Block(block)) => Some(block),
            _ => None,
        };
        // A replacement of its own is read once, and refused where it is not
        // a string or does not read in the encoding it carries.
        let stands_in = match arguments.first() {
            None | Some(Object::Nil) => None,
            Some(Object::String(written)) => {
                if !crate::vm::native_methods::string_methods::holds_valid_text(written) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "replacement must be valid byte sequence",
                        position,
                    ));
                }
                Some(binary_bytes(written))
            }
            Some(other) => {
                return Err(method_argument_type_error(
                    "scrub", "String", other, position,
                ));
            }
        };
        let runs = split_readable_runs(&bytes, &named);
        let mut built: Vec<u8> = Vec::with_capacity(bytes.len());
        for run in runs {
            match run {
                Run::Read(good) => built.extend(good),
                Run::Broken(bad) => match (&stands_in, &block) {
                    (Some(written), _) => built.extend(written.iter().copied()),
                    (None, Some(block)) => {
                        let piece = crate::object::StringValue::from_bytes(
                            crate::vm::native_methods::pack_format::bytes_to_string(&bad)
                                .to_string(),
                        );
                        piece.set_encoding(named.clone());
                        let answered = self.execute_block_callable(
                            block,
                            vec![Object::String(std::rc::Rc::new(piece))],
                            position,
                        )?;
                        if let Object::String(answered) = answered {
                            built.extend(binary_bytes(&answered));
                        }
                    }
                    (None, None) => built.extend(stands_in_for(&named)),
                },
            }
        }
        Ok(text_in_encoding(&built, &named))
    }
}

/// A run of bytes a string holds, told apart by whether its encoding reads it.
pub(crate) enum Run {
    Read(Vec<u8>),
    Broken(Vec<u8>),
}

/// The runs of bytes an encoding reads, alongside the ones it cannot.
pub(crate) fn split_readable_runs(bytes: &[u8], named: &str) -> Vec<Run> {
    let mut runs = Vec::new();
    match named {
        "UTF-8" | "UTF8-MAC" => {
            let mut at = 0usize;
            while at < bytes.len() {
                match std::str::from_utf8(&bytes[at..]) {
                    Ok(_) => {
                        runs.push(Run::Read(bytes[at..].to_vec()));
                        at = bytes.len();
                    }
                    Err(problem) => {
                        let good = problem.valid_up_to();
                        if good > 0 {
                            runs.push(Run::Read(bytes[at..at + good].to_vec()));
                        }
                        let width = problem.error_len().unwrap_or(bytes.len() - at - good);
                        runs.push(Run::Broken(bytes[at + good..at + good + width].to_vec()));
                        at += good + width;
                    }
                }
            }
        }
        "US-ASCII" => {
            for byte in bytes {
                if byte.is_ascii() {
                    runs.push(Run::Read(vec![*byte]));
                } else {
                    runs.push(Run::Broken(vec![*byte]));
                }
            }
        }
        "UTF-16" | "UTF-16LE" | "UTF-16BE" => {
            let big = !named.ends_with("LE");
            let mut at = 0usize;
            while at + 1 < bytes.len() {
                let unit = if big {
                    ((bytes[at] as u16) << 8) | bytes[at + 1] as u16
                } else {
                    ((bytes[at + 1] as u16) << 8) | bytes[at] as u16
                };
                let paired = (0xd800..0xdc00).contains(&unit);
                if !paired {
                    runs.push(Run::Read(bytes[at..at + 2].to_vec()));
                    at += 2;
                    continue;
                }
                let following = if at + 3 < bytes.len() {
                    if big {
                        ((bytes[at + 2] as u16) << 8) | bytes[at + 3] as u16
                    } else {
                        ((bytes[at + 3] as u16) << 8) | bytes[at + 2] as u16
                    }
                } else {
                    0
                };
                if (0xdc00..0xe000).contains(&following) {
                    runs.push(Run::Read(bytes[at..at + 4].to_vec()));
                    at += 4;
                } else {
                    runs.push(Run::Broken(bytes[at..at + 2].to_vec()));
                    at += 2;
                }
            }
            if at < bytes.len() {
                runs.push(Run::Broken(bytes[at..].to_vec()));
            }
        }
        // Every other encoding reads what it is handed, so nothing is put
        // aside.
        _ => runs.push(Run::Read(bytes.to_vec())),
    }
    runs
}

/// The bytes an encoding writes where it could not read what it was handed.
pub(crate) fn stands_in_for(named: &str) -> Vec<u8> {
    match named {
        "UTF-8" | "UTF8-MAC" => vec![0xef, 0xbf, 0xbd],
        "UTF-16" | "UTF-16BE" => vec![0xff, 0xfd],
        "UTF-16LE" => vec![0xfd, 0xff],
        "UTF-32" | "UTF-32BE" => vec![0x00, 0x00, 0xff, 0xfd],
        "UTF-32LE" => vec![0xfd, 0xff, 0x00, 0x00],
        _ => vec![b'?'],
    }
}

/// A run of bytes handed back as a string in the encoding it was read in.
pub(crate) fn text_in_encoding(bytes: &[u8], named: &str) -> Object {
    if let Ok(text) = std::str::from_utf8(bytes)
        && (named == "UTF-8" || named == "US-ASCII")
    {
        let made = crate::object::StringValue::with_encoding(text.to_string(), named);
        return Object::String(std::rc::Rc::new(made));
    }
    let made = crate::object::StringValue::from_bytes(
        crate::vm::native_methods::pack_format::bytes_to_string(bytes).to_string(),
    );
    made.set_encoding(named);
    Object::String(std::rc::Rc::new(made))
}
