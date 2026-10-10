// The pieces `encode` and `inspect` are built from.

use super::*;

/// What an `encode` was told to do with bytes the source encoding cannot
/// read and characters the destination cannot spell.
pub(crate) struct EncodeOptions {
    pub(crate) invalid_replace: bool,
    pub(crate) undef_replace: bool,
    pub(crate) replace: Option<String>,
    pub(crate) fallback: Option<Object>,
    pub(crate) xml: Option<Object>,
    pub(crate) newline: Option<Object>,
    pub(crate) newline_flags: Vec<&'static str>,
}

/// The arguments an `encode` was written with, split from its options.
pub(crate) fn encode_options(arguments: &[Object]) -> (&[Object], EncodeOptions) {
    let mut options = EncodeOptions {
        invalid_replace: false,
        undef_replace: false,
        replace: None,
        fallback: None,
        xml: None,
        newline: None,
        newline_flags: Vec::new(),
    };
    let Some(Object::Dict(pairs)) = arguments.last() else {
        return (arguments, options);
    };
    let held = pairs.borrow();
    if !held.contains_key("__MX_KWARGS__") {
        return (arguments, options);
    }
    let asks_replace = |key: &str| matches!(held.get(key), Some(Object::Symbol(named)) if &*named.as_str() == "replace");
    options.invalid_replace = asks_replace(":invalid");
    options.undef_replace = asks_replace(":undef");
    if let Some(Object::String(text)) = held.get(":replace") {
        options.replace = Some(text.as_str().to_string());
    }
    options.fallback = held.get(":fallback").cloned();
    options.newline = held
        .get(":newline")
        .filter(|named| !matches!(named, Object::Nil))
        .cloned();
    for flag in ["universal_newline", "crlf_newline", "cr_newline"] {
        if held.get(&format!(":{flag}")).is_some_and(Object::is_truthy) {
            options.newline_flags.push(flag);
        }
    }
    options.xml = held
        .get(":xml")
        .filter(|named| !matches!(named, Object::Nil))
        .cloned();
    drop(held);
    (&arguments[..arguments.len() - 1], options)
}

/// What stands in for a character when nothing else was named: the Unicode
/// replacement character where the destination spells it, a question mark
/// everywhere else.
pub(crate) fn default_replacement(wanted: &str) -> String {
    if wanted.starts_with("UTF-") {
        "\u{fffd}".to_string()
    } else {
        "?".to_string()
    }
}

/// Whether the destination of an `encode` spells a character.
pub(crate) fn destination_spells(wanted: &str, character: char) -> bool {
    let text = character.to_string();
    if wanted == "ISO-2022-JP" {
        return crate::vm::native_methods::euc_jp_table::iso_2022_jp_bytes(&text).is_ok();
    }
    if spells_shift_jis(wanted) {
        return crate::vm::native_methods::shift_jis_table::shift_jis_bytes(&text).is_ok();
    }
    if wanted == "EUC-JP" {
        return crate::vm::native_methods::euc_jp_table::euc_jp_bytes(&text).is_ok();
    }
    if wanted == "US-ASCII" {
        return character.is_ascii();
    }
    !matches!(latin_bytes(&text, wanted), Some(Err(_)))
}

/// The error `encode` raises for a run of UTF-8 bytes that spells nothing,
/// worded the way Ruby words it: a run cut short by the end of the string is
/// incomplete, and one cut short by another byte names that byte.
pub(crate) fn invalid_utf8_error(
    broken: &[u8],
    after: Option<u8>,
    position: Position,
) -> MetorexError {
    let opens_a_character = matches!(broken[0], 0xc2..=0xf4);
    let incomplete = after.is_none() && opens_a_character;
    let after = after.filter(|_| opens_a_character);
    invalid_bytes_error(broken, after, incomplete, "UTF-8", position)
}

/// The error `encode` raises for bytes of `encoding` that spell nothing:
/// `incomplete` when the input ends inside a character, and naming the byte
/// after when that byte cannot go on with the character. It carries the
/// bytes in `error_bytes` and the byte after in `readagain_bytes`.
pub(crate) fn invalid_bytes_error(
    broken: &[u8],
    after: Option<u8>,
    incomplete: bool,
    encoding: &str,
    position: Position,
) -> MetorexError {
    let spelled = |bytes: &[u8]| {
        let mut written = String::from("\"");
        for byte in bytes {
            if byte.is_ascii_graphic() || *byte == b' ' {
                written.push(*byte as char);
            } else {
                written.push_str(&format!("\\x{byte:02X}"));
            }
        }
        written.push('"');
        written
    };
    let message = match after {
        _ if incomplete => format!("incomplete {} on {encoding}", spelled(broken)),
        Some(next) => format!(
            "{} followed by {} on {encoding}",
            spelled(broken),
            spelled(&[next])
        ),
        None => format!("{} on {encoding}", spelled(broken)),
    };
    let error = crate::vm::errors::simple_exception(
        "Encoding::InvalidByteSequenceError",
        &message,
        position,
    );
    use crate::vm::native_methods::pack_format::bytes_to_string;
    set_exception_attribute(&error, "error_bytes", bytes_to_string(broken));
    let again = after.map_or(Object::Nil, |next| bytes_to_string(&[next]));
    set_exception_attribute(&error, "readagain_bytes", again);
    set_exception_attribute(&error, "truncated", Object::Bool(incomplete));
    error
}

/// Set an instance variable of the exception an error carries.
pub(crate) fn set_exception_attribute(error: &MetorexError, name: &str, value: Object) {
    if let MetorexError::UncaughtException {
        exception: Object::Exception(details),
        ..
    } = error
    {
        details
            .borrow_mut()
            .instance_vars
            .insert(name.to_string(), value);
    }
}

/// Where a run of Shift_JIS or EUC-JP bytes stops spelling characters: the
/// bytes that spell nothing, the byte after them that cannot go on with a
/// character, and whether the input ended inside one.
pub(crate) type BytesProblem = (Vec<u8>, Option<u8>, bool);

pub(crate) fn shift_jis_problem(bytes: &[u8]) -> Option<BytesProblem> {
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if byte < 0x80 || (0xa1..=0xdf).contains(&byte) {
            at += 1;
            continue;
        }
        if !((0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)) {
            return Some((vec![byte], None, false));
        }
        match bytes.get(at + 1) {
            None => return Some((vec![byte], None, true)),
            Some(&trail) if (0x40..=0x7e).contains(&trail) || (0x80..=0xfc).contains(&trail) => {
                at += 2
            }
            Some(&trail) => return Some((vec![byte], Some(trail), false)),
        }
    }
    None
}

pub(crate) fn euc_jp_problem(bytes: &[u8]) -> Option<BytesProblem> {
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if byte < 0x80 {
            at += 1;
            continue;
        }
        // The bytes that may follow the first: one after SS2, two after
        // SS3, and one after any other byte that opens a character.
        let (count, allowed): (usize, fn(u8) -> bool) = match byte {
            0x8e => (1, |next| (0xa1..=0xdf).contains(&next)),
            0x8f => (2, |next| (0xa1..=0xfe).contains(&next)),
            0xa1..=0xfe => (1, |next| (0xa1..=0xfe).contains(&next)),
            _ => return Some((vec![byte], None, false)),
        };
        for offset in 1..=count {
            match bytes.get(at + offset) {
                None => return Some((bytes[at..at + offset].to_vec(), None, true)),
                Some(&next) if allowed(next) => {}
                Some(&next) => return Some((bytes[at..at + offset].to_vec(), Some(next), false)),
            }
        }
        at += count + 1;
    }
    None
}

/// The text a run of EUC-JP bytes spells, with a byte that opens no character
/// written as the text that stands in for one.
pub(crate) fn euc_jp_text_replacing(bytes: &[u8], stands_in: &str) -> String {
    let mut written = String::with_capacity(bytes.len());
    let mut at = 0usize;
    while at < bytes.len() {
        match crate::vm::native_methods::euc_jp_table::euc_jp_character(&bytes[at..]) {
            Some((character, width)) => {
                written.push(character);
                at += width;
            }
            None => {
                written.push_str(stands_in);
                at += 1;
            }
        }
    }
    written
}

/// Whether Ruby names this encoding without converting anything through it.
/// A string tagged with one has a character to the byte, since nothing reads
/// its bytes as text.
/// A symbol written the way `inspect` writes one, reading its name through
/// the encoding it carries. A name that does not read as plain text is
/// quoted, with the bytes it holds written out.
pub(crate) fn symbol_inspect_text(value: &crate::object::StringValue) -> String {
    let named = value.encoding_name();
    let bytes = binary_bytes(value);
    // An encoding metorex names without reading text through it spells its
    // name byte by byte.
    if dummy_encoding(&named) {
        let written: String = bytes.iter().map(|byte| format!("\\x{byte:02X}")).collect();
        return format!(":\"{written}\"");
    }
    // An encoding that does not spell ASCII the way ASCII does is read
    // through, and the characters it names are quoted.
    if let Some(shape) = wide_encoding(&named) {
        return format!(":{:?}", wide_text(&bytes, shape));
    }
    if matches!(named.as_str(), "ASCII-8BIT" | "BINARY") && !bytes.is_ascii() {
        let mut written = String::from(":\"");
        for byte in &bytes {
            if byte.is_ascii_graphic() || *byte == b' ' {
                written.push(char::from(*byte));
            } else {
                written.push_str(&format!("\\x{byte:02X}"));
            }
        }
        written.push('"');
        return written;
    }
    crate::object::inspect_symbol(&value.as_str())
}

pub(crate) fn dummy_encoding(named: &str) -> bool {
    crate::vm::init::ENCODING_NAMES
        .iter()
        .any(|(_, canonical, dummy)| *dummy && *canonical == named)
}

/// Where each Shift_JIS character starts. A byte in the lead ranges is read
/// together with the byte after it, and every other byte stands alone.
pub(crate) fn shift_jis_characters(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut characters = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        let leads = matches!(bytes[at], 0x81..=0x9f | 0xe0..=0xfc) && at + 1 < bytes.len();
        let width = if leads { 2 } else { 1 };
        characters.push(bytes[at..at + width].to_vec());
        at += width;
    }
    characters
}

/// Where each EUC-JP character starts. 0x8E opens a two-byte half-width
/// katakana, 0x8F opens a three-byte character, a byte from 0xA1 opens a
/// two-byte one, and every other byte stands alone. A character the bytes run
/// out part-way through is what is left of it.
pub(crate) fn euc_jp_characters(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut characters = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        let width = match bytes[at] {
            0x8e | 0xa1..=0xfe => 2,
            0x8f => 3,
            _ => 1,
        };
        let stop = (at + width).min(bytes.len());
        characters.push(bytes[at..stop].to_vec());
        at = stop;
    }
    characters
}

/// The characters a string spells, each tagged with the string's own
/// encoding. What counts as a character depends on that encoding: a run of
/// bytes and an encoding Ruby converts nothing through both split at every
/// byte, Shift_JIS and EUC-JP read a lead byte together with the ones after
/// it, and the rest read text.
pub(crate) fn encoded_characters(string_value: &crate::object::StringValue) -> Vec<Object> {
    let named = string_value.encoding_name();
    let from_bytes = |bytes: &[u8]| {
        let spelled: String = bytes.iter().map(|byte| *byte as char).collect();
        let made = crate::object::StringValue::from_bytes(spelled);
        made.set_encoding(named.clone());
        Object::String(Rc::new(made))
    };
    let per_byte = || {
        binary_bytes(string_value)
            .iter()
            .map(|byte| from_bytes(&[*byte]))
            .collect()
    };
    match named.as_str() {
        "ASCII-8BIT" | "BINARY" => per_byte(),
        held if dummy_encoding(held) => per_byte(),
        held if let Some(shape) = wide_encoding(held) => {
            wide_characters(&binary_bytes(string_value), shape)
                .iter()
                .map(|character| from_bytes(character))
                .collect()
        }
        "Shift_JIS" | "Windows-31J" | "MacJapanese" => {
            shift_jis_characters(&binary_bytes(string_value))
                .iter()
                .map(|character| from_bytes(character))
                .collect()
        }
        "EUC-JP" if string_value.holds_bytes() => euc_jp_characters(&binary_bytes(string_value))
            .iter()
            .map(|character| from_bytes(character))
            .collect(),
        _ => string_value
            .as_str()
            .chars()
            .map(|character| {
                let made =
                    crate::object::StringValue::with_encoding(character.to_string(), named.clone());
                Object::String(Rc::new(made))
            })
            .collect(),
    }
}

/// A copy of a string holding the same bytes, tagged with the encoding named.
pub(crate) fn relabeled_copy(string_value: &crate::object::StringValue, named: &str) -> Object {
    let made = crate::object::StringValue::with_encoding(string_value.to_text(), named);
    if string_value.holds_bytes() {
        made.mark_bytes();
    }
    Object::String(Rc::new(made))
}

/// The error `encode` raises when nothing converts one encoding to another.
pub(crate) fn converter_not_found(from: &str, to: &str, position: Position) -> MetorexError {
    let message = format!("code converter not found ({from} to {to})");
    crate::vm::errors::simple_exception("Encoding::ConverterNotFoundError", &message, position)
}

impl VirtualMachine {
    /// The encoding `encode` is asked to write in. A name that is no encoding
    /// at all is refused as a conversion nothing carries out.
    pub(crate) fn encode_target_name(
        &mut self,
        string_value: &crate::object::StringValue,
        named: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let named = match named {
            Object::Class(_) | Object::String(_) => named.clone(),
            other if self.responds_to(other, "to_str") => {
                self.send_to_object(other.clone(), "to_str", vec![], position)?
            }
            other => other.clone(),
        };
        let Object::String(text) = &named else {
            return self.encoding_name_argument(&named, position);
        };
        let written = text.to_text();
        self.encoding_name_argument(&named, position)
            .map_err(|_| converter_not_found(&string_value.encoding_name(), &written, position))
    }

    /// Whether Ruby carries no converter to or from an encoding.
    pub(crate) fn encoding_without_converter(
        &mut self,
        named: &str,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let encoding = self.globals().get("Encoding").unwrap_or(Object::Nil);
        let answer = self.send_to_object(
            encoding,
            "__without_converter__",
            vec![Object::string(named.to_string())],
            position,
        )?;
        Ok(answer.is_truthy())
    }
}

impl VirtualMachine {
    /// The text a run of UTF-8 bytes spells. A run that spells nothing is
    /// replaced when `invalid: :replace` asks for that, and refused otherwise.
    /// The encodings a conversion error names: the step of the conversion
    /// it stopped at.
    pub(crate) fn with_conversion_ends(
        &mut self,
        error: MetorexError,
        source: &str,
        destination: &str,
    ) -> MetorexError {
        let source = self.encoding_object(source);
        let destination = self.encoding_object(destination);
        set_exception_attribute(&error, "source_encoding", source);
        set_exception_attribute(&error, "destination_encoding", destination);
        error
    }

    /// The error a character the destination cannot spell raises. A
    /// conversion from an encoding other than UTF-8 passes through UTF-8,
    /// and the message names every step of it.
    pub(crate) fn undefined_conversion_error(
        &mut self,
        character: char,
        source: &str,
        wanted: &str,
        position: Position,
    ) -> MetorexError {
        // The steps MRI's converter takes: through UTF-8 from anything
        // else, and to ISO-2022-JP through EUC-JP. The character stops at
        // the step out of UTF-8.
        let mut path = vec![source];
        if source != "UTF-8" {
            path.push("UTF-8");
        }
        if wanted == "ISO-2022-JP" {
            path.extend(["EUC-JP", "stateless-ISO-2022-JP"]);
        }
        path.push(wanted);
        let stopped_at = path[path
            .iter()
            .position(|step| *step == "UTF-8")
            .map_or(1, |at| at + 1)];
        let code = character as u32;
        let message = if path.len() == 2 {
            format!("U+{code:04X} from UTF-8 to {wanted}")
        } else {
            format!(
                "U+{code:04X} to {stopped_at} in conversion from {}",
                path.join(" to ")
            )
        };
        let error = crate::vm::errors::simple_exception(
            "Encoding::UndefinedConversionError",
            &message,
            position,
        );
        set_exception_attribute(&error, "error_char", Object::string(character.to_string()));
        self.with_conversion_ends(error, "UTF-8", stopped_at)
    }

    pub(crate) fn utf8_text(
        &mut self,
        bytes: &[u8],
        options: &EncodeOptions,
        replacement: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        let mut written = String::with_capacity(bytes.len());
        let mut read = 0usize;
        for chunk in bytes.utf8_chunks() {
            written.push_str(chunk.valid());
            read += chunk.valid().len();
            let broken = chunk.invalid();
            if broken.is_empty() {
                continue;
            }
            read += broken.len();
            if !options.invalid_replace {
                return Err(invalid_utf8_error(
                    broken,
                    bytes.get(read).copied(),
                    position,
                ));
            }
            written.push_str(replacement);
        }
        Ok(written)
    }
}

impl VirtualMachine {
    /// The text with each character the destination cannot spell asked of
    /// `fallback:`. Anything answering `[]` is asked, which covers a Hash with
    /// its default, a Proc and a Method. A character nothing answers for is
    /// left for the destination to refuse.
    pub(crate) fn fallback_text(
        &mut self,
        reading: String,
        fallback: &Object,
        wanted: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        if !self.responds_to(fallback, "[]") {
            return Ok(reading);
        }
        let mut written = String::with_capacity(reading.len());
        for character in reading.chars() {
            if destination_spells(wanted, character) {
                written.push(character);
                continue;
            }
            let answered = self.send_to_object(
                fallback.clone(),
                "[]",
                vec![Object::string(character.to_string())],
                position,
            )?;
            let stands_in = match &answered {
                Object::Nil => {
                    written.push(character);
                    continue;
                }
                Object::String(text) => text.to_text(),
                other if self.responds_to(other, "to_str") => {
                    match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                        Object::String(text) => text.to_text(),
                        _ => return Err(self.string_conversion_error(other, position)),
                    }
                }
                other => return Err(self.string_conversion_error(other, position)),
            };
            if !stands_in
                .chars()
                .all(|letter| destination_spells(wanted, letter))
            {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "too big fallback string",
                    position,
                ));
            }
            written.push_str(&stands_in);
        }
        Ok(written)
    }
}

/// How `xml:` asks for text to be written: as element content, or as an
/// attribute value, which also escapes the double quote and is quoted.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum XmlEscape {
    Text,
    Attribute,
}

/// What `xml:` names, refusing anything but `:text` and `:attr`.
pub(crate) fn xml_escape(named: &Object, position: Position) -> Result<XmlEscape, MetorexError> {
    let message = match named {
        Object::Symbol(held) if &*held.as_str() == "text" => return Ok(XmlEscape::Text),
        Object::Symbol(held) if &*held.as_str() == "attr" => return Ok(XmlEscape::Attribute),
        Object::Symbol(held) => format!("unexpected value for xml option: {}", held.as_str()),
        _ => "unexpected value for xml option".to_string(),
    };
    Err(crate::vm::errors::simple_exception(
        "ArgumentError",
        &message,
        position,
    ))
}

/// The text escaped for XML, with each character the destination cannot
/// spell written as a hexadecimal character reference.
pub(crate) fn xml_escaped(reading: &str, escape: XmlEscape, wanted: &str) -> String {
    let mut written = String::with_capacity(reading.len() + 2);
    if escape == XmlEscape::Attribute {
        written.push('"');
    }
    for character in reading.chars() {
        match character {
            '&' => written.push_str("&amp;"),
            '<' => written.push_str("&lt;"),
            '>' => written.push_str("&gt;"),
            '"' if escape == XmlEscape::Attribute => written.push_str("&quot;"),
            _ if destination_spells(wanted, character) => written.push(character),
            _ => written.push_str(&format!("&#x{:X};", character as u32)),
        }
    }
    if escape == XmlEscape::Attribute {
        written.push('"');
    }
    written
}

/// How `encode` is asked to write line endings: universal reads CR LF and a
/// lone CR as LF, and the other two write each LF as CR LF or as CR.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum NewlineConversion {
    Universal,
    Crlf,
    Cr,
}

/// The line ending conversion the options ask for. `newline:` is read ahead
/// of the flags, and more than one flag names no converter.
pub(crate) fn newline_conversion(
    options: &EncodeOptions,
    position: Position,
) -> Result<Option<NewlineConversion>, MetorexError> {
    if let Some(named) = &options.newline {
        let message = match named {
            Object::Symbol(held) => match &*held.as_str() {
                "universal" | "lf" => return Ok(Some(NewlineConversion::Universal)),
                "crlf" => return Ok(Some(NewlineConversion::Crlf)),
                "cr" => return Ok(Some(NewlineConversion::Cr)),
                other => format!("unexpected value for newline option: {other}"),
            },
            _ => "unexpected value for newline option".to_string(),
        };
        return Err(crate::vm::errors::simple_exception(
            "ArgumentError",
            &message,
            position,
        ));
    }
    match options.newline_flags.as_slice() {
        [] => Ok(None),
        ["universal_newline"] => Ok(Some(NewlineConversion::Universal)),
        ["crlf_newline"] => Ok(Some(NewlineConversion::Crlf)),
        ["cr_newline"] => Ok(Some(NewlineConversion::Cr)),
        several => Err(crate::vm::errors::simple_exception(
            "Encoding::ConverterNotFoundError",
            &format!("code converter not found ({})", several.join(",")),
            position,
        )),
    }
}

/// The text a run of bytes read as binary stands for in another encoding.
/// ASCII reads as itself, and a byte past it spells no character anywhere
/// else, so it is replaced where `undef: :replace` asks for that and refused
/// otherwise, the refusal naming the UTF-8 step the conversion passes through.
pub(crate) fn binary_text(
    bytes: &[u8],
    wanted: &str,
    replacement: Option<&str>,
    position: Position,
) -> Result<String, MetorexError> {
    let mut written = String::with_capacity(bytes.len());
    for byte in bytes {
        if byte.is_ascii() {
            written.push(*byte as char);
            continue;
        }
        if let Some(stands_in) = replacement {
            written.push_str(stands_in);
            continue;
        }
        let spelled = format!("\"\\x{byte:02X}\"");
        let message = if wanted == "UTF-8" {
            format!("{spelled} from ASCII-8BIT to UTF-8")
        } else {
            format!("{spelled} to UTF-8 in conversion from ASCII-8BIT to UTF-8 to {wanted}")
        };
        let error = crate::vm::errors::simple_exception(
            "Encoding::UndefinedConversionError",
            &message,
            position,
        );
        set_exception_attribute(
            &error,
            "error_char",
            crate::vm::native_methods::pack_format::bytes_to_string(&[*byte]),
        );
        return Err(error);
    }
    Ok(written)
}
