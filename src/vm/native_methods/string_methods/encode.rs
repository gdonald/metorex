// Writing a string out in another encoding.

use super::*;

impl VirtualMachine {
    /// Writing a string out in another encoding.
    pub(crate) fn call_string_encode_method(
        &mut self,
        _receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // String#encode — metorex strings are always UTF-8 and carry no
            // encoding metadata, so re-encoding is the identity.
            // Text that is nothing but ASCII reads the same in every
            // ASCII-compatible encoding, so a copy of it can be tagged with
            // the one asked for without converting anything. Text that is not
            // needs a conversion metorex does not carry out, and the copy
            // keeps the encoding it had.
            // Metorex holds every string's characters as text, so `encode`
            // answers a copy tagged with the encoding asked for rather than
            // rewriting what it holds.
            "encode" => {
                // The last argument may name what to do with bytes the source
                // encoding cannot read.
                let (positional, replacement) = encode_options(arguments);
                let arguments = positional;
                let Some(named) = arguments.first() else {
                    return Ok(Some(Object::string(string_value.to_text())));
                };
                let Ok(wanted) = self.encoding_name_argument(named, position) else {
                    return Ok(Some(Object::string(string_value.to_text())));
                };
                // An encoding that spells a character in more than one byte
                // is carried as the bytes themselves, since they spell
                // nothing the text model can hold. A second name says what
                // the bytes are to be read as rather than what the string
                // says it is written in.
                let held = match arguments.get(1) {
                    Some(source) => self
                        .encoding_name_argument(source, position)
                        .unwrap_or_else(|_| string_value.encoding_name()),
                    None => string_value.encoding_name(),
                };
                let reads_bytes = string_value.holds_bytes() || arguments.len() > 1;
                let reading = match wide_encoding(&held) {
                    Some(shape) => wide_text(&binary_bytes(string_value), shape),
                    None if held == "ISO-2022-JP" && reads_bytes => {
                        crate::vm::native_methods::euc_jp_table::iso_2022_jp_text(&binary_bytes(
                            string_value,
                        ))
                    }
                    None if spells_shift_jis(&held) && reads_bytes => {
                        crate::vm::native_methods::shift_jis_table::shift_jis_text(&binary_bytes(
                            string_value,
                        ))
                    }
                    None if held == "EUC-JP" && reads_bytes => match &replacement {
                        Some(stands_in) => {
                            euc_jp_text_replacing(&binary_bytes(string_value), stands_in)
                        }
                        None => crate::vm::native_methods::euc_jp_table::euc_jp_text(
                            &binary_bytes(string_value),
                        ),
                    },
                    None => match latin_text(&binary_bytes(string_value), &held) {
                        Some(spelled) if reads_bytes => spelled,
                        _ => string_value.to_text(),
                    },
                };
                // ISO-2022-JP writes its Japanese runs between escapes, so
                // the bytes are built rather than mapped one for one.
                if wanted == "ISO-2022-JP" {
                    let bytes =
                        crate::vm::native_methods::euc_jp_table::iso_2022_jp_bytes(&reading)
                            .map_err(|character| {
                                let message =
                                    format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                                crate::vm::errors::simple_exception(
                                    "Encoding::UndefinedConversionError",
                                    &message,
                                    position,
                                )
                            })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if spells_shift_jis(&wanted) {
                    let bytes =
                        crate::vm::native_methods::shift_jis_table::shift_jis_bytes(&reading)
                            .map_err(|character| {
                                let message =
                                    format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                                crate::vm::errors::simple_exception(
                                    "Encoding::UndefinedConversionError",
                                    &message,
                                    position,
                                )
                            })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if wanted == "EUC-JP" {
                    let bytes = crate::vm::native_methods::euc_jp_table::euc_jp_bytes(&reading)
                        .map_err(|character| {
                            let message =
                                format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                            crate::vm::errors::simple_exception(
                                "Encoding::UndefinedConversionError",
                                &message,
                                position,
                            )
                        })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if let Some(spelled) = latin_bytes(&reading, &wanted) {
                    let bytes = spelled.map_err(|character| {
                        let message =
                            format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                        crate::vm::errors::simple_exception(
                            "Encoding::UndefinedConversionError",
                            &message,
                            position,
                        )
                    })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if let Some(shape) = wide_encoding(&wanted) {
                    let made = Object::String(Rc::new(crate::object::StringValue::from_bytes(
                        bytes_as_text(&wide_bytes(&reading, shape)),
                    )));
                    if let Object::String(copied) = &made {
                        copied.set_encoding(wanted);
                    }
                    return Ok(Some(made));
                }
                let copy = Object::string(reading);
                if let Object::String(copied) = &copy {
                    copied.set_encoding(wanted);
                }
                Ok(Some(copy))
            }
            _ => Ok(None),
        }
    }
}
