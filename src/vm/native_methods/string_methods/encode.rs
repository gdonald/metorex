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
            // `encode` reads the string's bytes in the encoding they are
            // written in and writes the characters out in the one asked for.
            "encode" => {
                let (arguments, options) = encode_options(arguments);
                // With no encoding named, the string is written in the
                // default internal encoding, or kept in its own.
                let wanted = match arguments.first() {
                    Some(named) => self.encode_target_name(string_value, named, position)?,
                    None => match self.globals().get("__Encoding_default_internal") {
                        Some(Object::Class(internal)) => internal.name().to_string(),
                        _ => string_value.encoding_name(),
                    },
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
                let replacement = options
                    .replace
                    .clone()
                    .unwrap_or_else(|| default_replacement(&wanted));
                // Between two names for the same encoding nothing is
                // converted, and `invalid: :replace` scrubs what the bytes do
                // not spell.
                let xml = match &options.xml {
                    Some(named) => Some(xml_escape(named, position)?),
                    None => None,
                };
                let newline = newline_conversion(&options, position)?;
                if held == wanted && xml.is_none() && newline.is_none() {
                    let copy = relabeled_copy(string_value, &wanted);
                    if let (true, Object::String(copied)) = (options.invalid_replace, &copy) {
                        let stands_in = options.replace.map_or(Object::Nil, Object::string);
                        return self
                            .scrubbed_string(copied, &[stands_in], position)
                            .map(Some);
                    }
                    return Ok(Some(copy));
                }
                if self.encoding_without_converter(&held, position)?
                    || self.encoding_without_converter(&wanted, position)?
                {
                    // Text that is all ASCII reads the same in every one of
                    // these that is not a dummy, so it needs no converter.
                    let ascii_only = binary_bytes(string_value).is_ascii();
                    if ascii_only && !dummy_encoding(&held) && !dummy_encoding(&wanted) {
                        return Ok(Some(relabeled_copy(string_value, &wanted)));
                    }
                    return Err(converter_not_found(&held, &wanted, position));
                }
                let reads_bytes = string_value.holds_bytes() || arguments.len() > 1;
                let reading = match wide_encoding(&held) {
                    Some(shape) => wide_text(&binary_bytes(string_value), shape),
                    None if held == "ASCII-8BIT" => binary_text(
                        &binary_bytes(string_value),
                        &wanted,
                        options.undef_replace.then_some(replacement.as_str()),
                        position,
                    )?,
                    None if held == "UTF-8" => self.utf8_text(
                        &binary_bytes(string_value),
                        &options,
                        &replacement,
                        position,
                    )?,
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
                    None if held == "EUC-JP" && reads_bytes && options.invalid_replace => {
                        euc_jp_text_replacing(&binary_bytes(string_value), &replacement)
                    }
                    None if held == "EUC-JP" && reads_bytes => {
                        crate::vm::native_methods::euc_jp_table::euc_jp_text(&binary_bytes(
                            string_value,
                        ))
                    }
                    None => match latin_text(&binary_bytes(string_value), &held) {
                        Some(spelled) if reads_bytes => spelled,
                        _ => string_value.to_text(),
                    },
                };
                // `xml:` writes a character the destination cannot spell as a
                // character reference. Failing that, `undef: :replace` stands
                // the replacement in for it, and failing that `fallback:` is
                // asked for one.
                let reading = match newline {
                    Some(NewlineConversion::Universal) => {
                        reading.replace("\r\n", "\n").replace('\r', "\n")
                    }
                    _ => reading,
                };
                let reading = if let Some(escape) = xml {
                    xml_escaped(&reading, escape, &wanted)
                } else if options.undef_replace {
                    reading
                        .chars()
                        .map(|character| match destination_spells(&wanted, character) {
                            true => character.to_string(),
                            false => replacement.clone(),
                        })
                        .collect()
                } else if let Some(fallback) = &options.fallback {
                    self.fallback_text(reading, fallback, &wanted, position)?
                } else {
                    reading
                };
                let reading = match newline {
                    Some(NewlineConversion::Crlf) => reading.replace('\n', "\r\n"),
                    Some(NewlineConversion::Cr) => reading.replace('\n', "\r"),
                    _ => reading,
                };
                if wanted == "US-ASCII"
                    && let Some(character) = reading.chars().find(|character| !character.is_ascii())
                {
                    let message = format!("U+{:04X} from UTF-8 to US-ASCII", character as u32);
                    return Err(crate::vm::errors::simple_exception(
                        "Encoding::UndefinedConversionError",
                        &message,
                        position,
                    ));
                }
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
