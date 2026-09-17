//! The String methods that read a character set the way `tr` writes one, plus
//! the pure transforms that sit alongside them.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;

/// One `tr`-style character set: the characters it names, and whether the set
/// was written with a leading `^` and so means every character but those.
struct CharacterSet {
    members: Vec<char>,
    negated: bool,
}

impl CharacterSet {
    /// Read a set written the way `tr`, `count`, `delete`, and `squeeze` take
    /// one: `a-z` is a range, a leading `^` negates, and `\` escapes the
    /// character after it. A range whose end comes before its start is
    /// refused, naming the range as Ruby names it.
    fn parse(source: &str) -> Result<Self, String> {
        let characters: Vec<char> = source.chars().collect();
        let negated = characters.len() > 1 && characters[0] == '^';
        let mut members = Vec::new();
        let mut index = if negated { 1 } else { 0 };
        while index < characters.len() {
            if characters[index] == '\\' && index + 1 < characters.len() {
                members.push(characters[index + 1]);
                index += 2;
                continue;
            }
            // `a-z` names every character between the two ends, while a `-`
            // at either end of the set is a character of its own.
            if index + 2 < characters.len() && characters[index + 1] == '-' {
                let (start, end) = (characters[index], characters[index + 2]);
                if start > end {
                    return Err(format!(
                        "invalid range \"{}-{}\" in string transliteration",
                        start, end
                    ));
                }
                for point in (start as u32)..=(end as u32) {
                    if let Some(member) = char::from_u32(point) {
                        members.push(member);
                    }
                }
                index += 3;
                continue;
            }
            members.push(characters[index]);
            index += 1;
        }
        Ok(Self { members, negated })
    }

    fn holds(&self, character: char) -> bool {
        self.members.contains(&character) != self.negated
    }
}

/// Whether every set given holds the character, which is how `count`,
/// `delete`, and `squeeze` read more than one.
fn all_hold(sets: &[CharacterSet], character: char) -> bool {
    sets.iter().all(|set| set.holds(character))
}

/// The replacement `tr` puts in place of a character, or None when the
/// destination set is empty and the character is dropped instead.
fn translation_for(from: &CharacterSet, to: &CharacterSet, character: char) -> Option<char> {
    if to.members.is_empty() {
        return None;
    }
    if from.negated {
        return Some(*to.members.last().expect("a non-empty destination set"));
    }
    let at = from
        .members
        .iter()
        .position(|member| *member == character)?;
    Some(
        *to.members
            .get(at)
            .unwrap_or_else(|| to.members.last().expect("a non-empty destination set")),
    )
}

impl VirtualMachine {
    /// The characters an argument stands for, taking `to_str` from an object
    /// that answers one, the way Ruby reads a String argument.
    pub(crate) fn string_argument(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match argument {
            Object::String(text) => Ok(text.as_str().to_string()),
            // An object of the program's own is asked whether it spells
            // itself as text, since its answer may be written rather than
            // declared.
            other if self.answers_to(other, "to_str", position)? => {
                match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                    Object::String(text) => Ok(text.as_str().to_string()),
                    converted => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &converted,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                method_name,
                "String",
                other,
                position,
            )),
        }
    }

    /// The whole number an argument stands for, taking `to_int` from an
    /// object that answers one.
    pub(crate) fn integer_argument(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        match argument {
            Object::Int(number) => Ok(*number),
            other if self.answers_to(other, "to_int", position)? => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(number) => Ok(number),
                    converted => Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &converted,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                method_name,
                "Integer",
                other,
                position,
            )),
        }
    }

    /// Read every argument as a `tr`-style set, refusing anything that is not
    /// a String.
    fn character_sets(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Vec<CharacterSet>, MetorexError> {
        let mut sets = Vec::new();
        for argument in arguments {
            // A set spelled with bytes that are not characters in its own
            // encoding cannot be read at all, which is what Ruby reports
            // before it looks at any range inside.
            if let Object::String(spelled) = argument
                && !crate::vm::native_methods::string_methods::holds_valid_text(spelled)
            {
                let message = format!("invalid byte sequence in {}", spelled.encoding_name());
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                });
            }
            let text = self.string_argument(method_name, argument, position)?;
            match CharacterSet::parse(&text) {
                Ok(set) => sets.push(set),
                Err(message) => {
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
            }
        }
        Ok(sets)
    }

    /// The String methods that read character sets, split apart, or measure
    /// the string without changing it.
    pub(crate) fn call_string_set_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(string_value) = receiver else {
            return Ok(None);
        };
        let text = string_value.as_str().to_string();
        match method_name {
            "count" | "delete" | "squeeze" => {
                if method_name != "squeeze" && arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let sets = self.character_sets(method_name, arguments, position)?;
                match method_name {
                    "count" => {
                        let counted = text
                            .chars()
                            .filter(|character| all_hold(&sets, *character))
                            .count();
                        Ok(Some(Object::Int(counted as i64)))
                    }
                    "delete" => {
                        let kept: String = text
                            .chars()
                            .filter(|character| !all_hold(&sets, *character))
                            .collect();
                        Ok(Some(Object::string(kept)))
                    }
                    _ => {
                        let mut kept = String::new();
                        let mut previous: Option<char> = None;
                        for character in text.chars() {
                            let repeated = previous == Some(character);
                            let squeezable = sets.is_empty() || all_hold(&sets, character);
                            if !(repeated && squeezable) {
                                kept.push(character);
                            }
                            previous = Some(character);
                        }
                        Ok(Some(Object::string(kept)))
                    }
                }
            }
            "tr" | "tr_s" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let sets = self.character_sets(method_name, arguments, position)?;
                let (from, to) = (&sets[0], &sets[1]);
                let mut translated = String::new();
                let mut previous: Option<char> = None;
                for character in text.chars() {
                    if !from.holds(character) {
                        translated.push(character);
                        previous = None;
                        continue;
                    }
                    let Some(replacement) = translation_for(from, to, character) else {
                        continue;
                    };
                    // `tr_s` leaves one character where `tr` would have left a
                    // run of the same replacement.
                    if method_name == "tr_s" && previous == Some(replacement) {
                        continue;
                    }
                    translated.push(replacement);
                    previous = Some(replacement);
                }
                Ok(Some(Object::string(translated)))
            }
            "chop" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // Text written in an encoding of more than one byte to a
                // character is read as characters before the last one comes
                // off, and written back out afterwards.
                if let Some(shape) = crate::vm::native_methods::string_methods::wide_encoding(
                    &string_value.encoding_name(),
                ) {
                    use crate::vm::native_methods::string_methods as text_methods;
                    let reading =
                        text_methods::wide_text(&text_methods::binary_bytes(string_value), shape);
                    let cut = match reading.strip_suffix("\r\n") {
                        Some(rest) => rest.to_string(),
                        None => {
                            let mut letters: Vec<char> = reading.chars().collect();
                            letters.pop();
                            letters.into_iter().collect()
                        }
                    };
                    let made =
                        Object::String(std::rc::Rc::new(crate::object::StringValue::from_bytes(
                            text_methods::bytes_as_text(&text_methods::wide_bytes(&cut, shape)),
                        )));
                    if let Object::String(held) = &made {
                        held.set_encoding(string_value.encoding_name());
                    }
                    return Ok(Some(made));
                }
                // A trailing "\r\n" comes off as one, which is what keeps
                // `chop` from splitting a line ending in half.
                let chopped = if let Some(rest) = text.strip_suffix("\r\n") {
                    rest.to_string()
                } else {
                    let mut characters: Vec<char> = text.chars().collect();
                    characters.pop();
                    characters.into_iter().collect()
                };
                Ok(Some(Object::string(chopped)))
            }
            "delete_prefix" | "delete_suffix" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let affix = self.string_argument(method_name, &arguments[0], position)?;
                let trimmed = if method_name == "delete_prefix" {
                    text.strip_prefix(&affix).unwrap_or(&text)
                } else {
                    text.strip_suffix(&affix).unwrap_or(&text)
                };
                Ok(Some(Object::string(trimmed.to_string())))
            }
            "partition" | "rpartition" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // The matched piece carries the separator's own encoding,
                // while the two pieces around it carry the subject's.
                let mut separator_encoding = string_value.encoding_name();
                let found = match &arguments[0] {
                    Object::Regex(..) => {
                        self.regexp_partition(&text, &arguments[0], method_name, position)?
                    }
                    other => {
                        if let Object::String(given) = other {
                            separator_encoding = given.encoding_name();
                        }
                        let separator = self.string_argument(method_name, other, position)?;
                        let at = if method_name == "partition" {
                            text.find(&separator)
                        } else {
                            text.rfind(&separator)
                        };
                        at.map(|at| (at, at + separator.len()))
                    }
                };
                let encoding = string_value.encoding_name();
                let cut = |piece: String, encoding: String| {
                    Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
                        piece, encoding,
                    )))
                };
                let tagged = |piece: String| {
                    Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
                        piece,
                        encoding.clone(),
                    )))
                };
                let parts = match found {
                    Some((start, end)) => vec![
                        tagged(text[..start].to_string()),
                        cut(text[start..end].to_string(), separator_encoding.clone()),
                        tagged(text[end..].to_string()),
                    ],
                    // A separator that is nowhere in the string leaves the
                    // whole string on the side the search ran from.
                    None if method_name == "partition" => vec![
                        tagged(text.clone()),
                        tagged(String::new()),
                        tagged(String::new()),
                    ],
                    None => vec![
                        tagged(String::new()),
                        tagged(String::new()),
                        tagged(text.clone()),
                    ],
                };
                Ok(Some(Object::array(parts)))
            }
            "intern" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                self.interned_symbol(string_value, position).map(Some)
            }
            "sum" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let bits = match arguments.first() {
                    None => 16,
                    Some(other) => self.integer_argument(method_name, other, position)?,
                };
                let total: i64 = crate::vm::native_methods::pack_format::string_to_bytes(&text)
                    .iter()
                    .map(|byte| i64::from(*byte))
                    .sum();
                // A width of zero or more than fits in an Integer keeps the
                // whole sum, and any other width keeps its low bits.
                let answer = if bits <= 0 || bits >= 64 {
                    total
                } else {
                    total & ((1i64 << bits) - 1)
                };
                Ok(Some(Object::Int(answer)))
            }
            "casecmp" | "casecmp?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A comparison against something that is not a String answers
                // nil rather than raising, which is what Ruby does.
                let converted = self.string_try_convert(&arguments[0], position)?;
                let other = match &converted {
                    Object::String(other) => other.clone(),
                    instance => match super::string_subclass_value(instance) {
                        Some(Object::String(other)) => other,
                        _ => return Ok(Some(Object::Nil)),
                    },
                };
                // Two strings with no encoding in common cannot be ordered,
                // which Ruby reports as nil rather than as an error.
                if compatible_encoding(
                    &text,
                    &string_value.encoding_name(),
                    &other.as_str(),
                    &other.encoding_name(),
                )
                .is_none()
                {
                    return Ok(Some(Object::Nil));
                }
                // Only the ASCII letters fold, which is what keeps `casecmp`
                // apart from a full Unicode comparison.
                let left = fold_ascii_case(&text);
                let right = fold_ascii_case(&other.as_str());
                // `casecmp?` folds the whole of Unicode, where `casecmp`
                // orders by the ASCII letters alone.
                if method_name == "casecmp?" {
                    // Only an encoding that spells the whole of Unicode maps
                    // a letter outside ASCII onto another case of itself.
                    let maps_unicode = unicode_encoding(&string_value.encoding_name())
                        && unicode_encoding(&other.encoding_name());
                    if !maps_unicode {
                        return Ok(Some(Object::Bool(left == right)));
                    }
                    // Folding maps a letter onto the letters it compares
                    // equal to, which is where a sharp s becomes two of them.
                    let folded = |held: &str| held.to_lowercase().replace('\u{df}', "ss");
                    return Ok(Some(Object::Bool(folded(&text) == folded(&other.as_str()))));
                }
                let order = match left.cmp(&right) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                Ok(Some(Object::Int(order)))
            }
            // Text written in two encodings that do not go together cannot
            // be padded with one another.
            "center" | "ljust" | "rjust"
                if matches!(arguments.get(1), Some(Object::String(pad))
                    if super::string_methods::encodings_clash(string_value, pad)) =>
            {
                let Some(Object::String(pad)) = arguments.get(1) else {
                    return Ok(None);
                };
                Err(super::string_methods::clashing_encodings_error(
                    string_value,
                    pad,
                    position,
                ))
            }
            "center" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let width = &self.integer_argument(method_name, &arguments[0], position)?;
                let padding = match arguments.get(1) {
                    None => " ".to_string(),
                    Some(other) => {
                        let given = self.string_argument(method_name, other, position)?;
                        if given.is_empty() {
                            let message = "zero width padding".to_string();
                            return Err(MetorexError::UncaughtException {
                                exception: Object::exception("ArgumentError", message.clone()),
                                location: crate::vm::utils::position_to_location(position),
                                message,
                            });
                        }
                        given
                    }
                };
                let pad_encoding = match arguments.get(1) {
                    Some(Object::String(pad)) => pad.encoding_name(),
                    _ => string_value.encoding_name(),
                };
                let Some(encoding) = compatible_encoding(
                    &text,
                    &string_value.encoding_name(),
                    &padding,
                    &pad_encoding,
                ) else {
                    let message = format!(
                        "incompatible character encodings: {} and {}",
                        string_value.encoding_name(),
                        pad_encoding
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception(
                            "Encoding::CompatibilityError",
                            message.clone(),
                        ),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                let tagged = |piece: String| {
                    Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
                        piece,
                        encoding.clone(),
                    )))
                };
                let held = text.chars().count() as i64;
                if *width <= held {
                    return Ok(Some(tagged(text)));
                }
                // The odd character of an uneven gap goes on the right, which
                // is where Ruby puts it.
                let gap = (*width - held) as usize;
                let left = gap / 2;
                let pad_characters: Vec<char> = padding.chars().collect();
                let run = |count: usize, offset: usize| -> String {
                    (0..count)
                        .map(|at| pad_characters[(at + offset) % pad_characters.len()])
                        .collect()
                };
                Ok(Some(tagged(format!(
                    "{}{}{}",
                    run(left, 0),
                    text,
                    run(gap - left, 0)
                ))))
            }
            // `+str` asks for a string that changes and `-str` for one that
            // does not, so each answers the string itself when it is already
            // that way and a copy when it is not.
            "dedup" | "-@" | "+@" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let wants_frozen = method_name != "+@";
                if !wants_frozen {
                    // A string handed back with notice that it will be frozen
                    // answers a copy that carries no such notice, which is
                    // what asking for a mutable one means.
                    if !string_value.is_frozen() && !string_value.is_chilled() {
                        return Ok(Some(receiver.clone()));
                    }
                    let copy = crate::object::StringValue::with_encoding(
                        string_value.to_text(),
                        string_value.encoding_name(),
                    );
                    return Ok(Some(Object::String(std::rc::Rc::new(copy))));
                }
                // A string carrying instance variables of its own is not
                // shared with anything, so it stands for itself.
                if Self::collection_address(receiver)
                    .and_then(|address| self.collection_variables.get(&address))
                    .is_some_and(|held| !held.is_empty())
                {
                    if string_value.is_frozen() {
                        return Ok(Some(receiver.clone()));
                    }
                    let copy = crate::object::StringValue::with_encoding(
                        string_value.to_text(),
                        string_value.encoding_name(),
                    );
                    copy.freeze();
                    return Ok(Some(Object::String(std::rc::Rc::new(copy))));
                }
                // Two equal strings deduplicate to one frozen object, which
                // is what `-@` and `dedup` hand back.
                Ok(Some(self.deduped_string(string_value)))
            }
            "grapheme_clusters" | "each_grapheme_cluster" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let clusters = encoded_grapheme_clusters(string_value);
                if method_name == "grapheme_clusters" && self.pending_block.is_none() {
                    return Ok(Some(Object::array(clusters)));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => block,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        let size = clusters.len() as i64;
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                Vec::new(),
                                Some(size),
                                position,
                            )
                            .map(Some);
                    }
                };
                for cluster in clusters {
                    self.execute_block_body(&block, vec![cluster])?;
                }
                Ok(Some(receiver.clone()))
            }
            "upto" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let exclusive = matches!(arguments.get(1), Some(other) if other.is_truthy());
                if let Object::String(other) = &arguments[0] {
                    self.check_upto_encodings(string_value, other, position)?;
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => block,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                arguments.to_vec(),
                                None,
                                position,
                            )
                            .map(Some);
                    }
                };
                let last = self.string_argument(method_name, &arguments[0], position)?;
                for step in upto_sequence(&text, &last, exclusive) {
                    self.execute_block_body(&block, vec![Object::string(step)])?;
                }
                Ok(Some(receiver.clone()))
            }
            "scrub" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(held) = receiver else {
                    let Some(Object::String(held)) = super::string_subclass_value(receiver) else {
                        return Ok(None);
                    };
                    return self.scrubbed_string(&held, arguments, position).map(Some);
                };
                let held = std::rc::Rc::clone(held);
                self.scrubbed_string(&held, arguments, position).map(Some)
            }
            "undump" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let held = match receiver {
                    Object::String(held) => Some(std::rc::Rc::clone(held)),
                    _ => match super::string_subclass_value(receiver) {
                        Some(Object::String(held)) => Some(held),
                        _ => None,
                    },
                };
                let carried = held
                    .as_ref()
                    .map(|held| held.encoding_name())
                    .unwrap_or_else(|| "UTF-8".to_string());
                // A dump written in an encoding that spells no ASCII cannot be
                // read back, since the quoting itself is written in ASCII.
                if NOT_ASCII_COMPATIBLE.contains(&carried.as_str()) {
                    return Err(crate::vm::errors::simple_exception(
                        "Encoding::CompatibilityError",
                        &format!("ASCII incompatible encoding: {carried}"),
                        position,
                    ));
                }
                let refuse = |message: &str| {
                    crate::vm::errors::simple_exception("RuntimeError", message, position)
                };
                let (bytes, named) = undump(&text).map_err(refuse)?;
                let named = match named {
                    None => carried,
                    Some(name) => {
                        if !crate::vm::init::ENCODING_NAMES
                            .iter()
                            .any(|(_, display, _)| *display == name)
                        {
                            return Err(refuse("dumped string has unknown encoding name"));
                        }
                        name
                    }
                };
                let made = crate::object::StringValue::from_bytes(
                    super::pack_format::bytes_to_string(&bytes).to_string(),
                );
                made.set_encoding(named);
                Ok(Some(Object::String(std::rc::Rc::new(made))))
            }
            _ => Ok(None),
        }
    }

    /// `String.try_convert`: a String answers itself, an object with `to_str`
    /// answers what that gives, and anything else answers nil.
    pub(crate) fn string_try_convert(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if matches!(argument, Object::String(_)) || super::string_subclass_value(argument).is_some()
        {
            return Ok(argument.clone());
        }
        if !self.responds_to(argument, "to_str") {
            return Ok(Object::Nil);
        }
        let source = self.builtins().class_of(argument).name().to_string();
        let converted = self.send_to_object(argument.clone(), "to_str", vec![], position)?;
        if matches!(converted, Object::Nil | Object::String(_))
            || super::string_subclass_value(&converted).is_some()
        {
            return Ok(converted);
        }
        let message = format!(
            "can't convert {} into String ({}#to_str gives {})",
            source,
            source,
            self.builtins().class_of(&converted).name()
        );
        Err(MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }

    /// The bounds a Regexp separator matched, which `partition` and
    /// `rpartition` split the string on.
    fn regexp_partition(
        &mut self,
        text: &str,
        pattern: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<Option<(usize, usize)>, MetorexError> {
        let Object::Regex(source, flags) = pattern else {
            return Ok(None);
        };
        let Some(compiled) = crate::vm::native_methods::regexp_methods::compile(source, flags)
        else {
            let message = format!("invalid pattern for {}", method_name);
            return Err(MetorexError::runtime_error(
                message,
                crate::vm::utils::position_to_location(position),
            ));
        };
        // `rpartition` wants the rightmost match, which can start inside an
        // earlier one, so every start is tried rather than only the
        // non-overlapping runs `find_iter` walks.
        let found = if method_name == "partition" {
            compiled.find_at(text, 0)
        } else {
            (0..=text.len())
                .rev()
                .filter(|at| text.is_char_boundary(*at))
                .find_map(|at| {
                    compiled
                        .find_at(text, at)
                        .filter(|found| found.start() == at)
                })
        };
        // The groups the separator captured are what `$1` and the rest read,
        // so the match is recorded the way any other match is.
        match &found {
            Some(matched) => {
                self.regexp_match_data(source, flags, text, matched.start(), position)?;
            }
            None => {
                self.globals_mut().set(
                    crate::vm::native_methods::regexp_methods::LAST_MATCH,
                    Object::Nil,
                );
            }
        }
        Ok(found.map(|found| (found.start(), found.end())))
    }
}

/// The character that joins what stands on either side of it into one
/// cluster, which is how a flag and a rainbow spell one emoji.
const ZERO_WIDTH_JOINER: char = '\u{200d}';

/// The clusters a string breaks into: one leading character, then every
/// combining mark that attaches to it, and whatever a zero width joiner
/// pulls along after it.
fn grapheme_clusters(text: &str) -> Vec<String> {
    let mut clusters: Vec<String> = Vec::new();
    let mut joining = false;
    for character in text.chars() {
        let attaches = joining || is_combining_mark(character) || character == ZERO_WIDTH_JOINER;
        joining = character == ZERO_WIDTH_JOINER;
        if attaches && let Some(last) = clusters.last_mut() {
            last.push(character);
            continue;
        }
        clusters.push(character.to_string());
    }
    clusters
}

/// The clusters a string breaks into, each tagged with the string's own
/// encoding. An encoding that spells a character in more than one byte is
/// read out of those bytes and written back into them.
pub(crate) fn encoded_grapheme_clusters(string_value: &crate::object::StringValue) -> Vec<Object> {
    use super::string_methods::{
        binary_bytes, bytes_as_text, wide_bytes, wide_encoding, wide_text,
    };
    let named = string_value.encoding_name();
    if let Some(shape) = wide_encoding(&named)
        && !super::string_methods::dummy_encoding(&named)
    {
        return grapheme_clusters(&wide_text(&binary_bytes(string_value), shape))
            .iter()
            .map(|cluster| {
                let made = crate::object::StringValue::from_bytes(bytes_as_text(&wide_bytes(
                    cluster, shape,
                )));
                made.set_encoding(named.clone());
                Object::String(std::rc::Rc::new(made))
            })
            .collect();
    }
    // An encoding that has one character to the byte has one cluster to the
    // byte with it, since nothing there attaches to anything.
    if matches!(named.as_str(), "ASCII-8BIT" | "BINARY")
        || super::string_methods::dummy_encoding(&named)
        || matches!(named.as_str(), "Shift_JIS" | "Windows-31J" | "MacJapanese")
    {
        return super::string_methods::encoded_characters(string_value);
    }
    grapheme_clusters(&string_value.as_str())
        .into_iter()
        .map(|cluster| {
            Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
                cluster,
                named.clone(),
            )))
        })
        .collect()
}

/// Whether a character attaches to the one before it rather than standing on
/// its own, which is what keeps a cluster together.
fn is_combining_mark(character: char) -> bool {
    matches!(character as u32,
        0x0300..=0x036F
        | 0x0483..=0x0489
        | 0x0591..=0x05BD
        | 0x0610..=0x061A
        | 0x064B..=0x065F
        | 0x0670
        | 0x06D6..=0x06DC
        | 0x0900..=0x0903
        | 0x093A..=0x094F
        | 0x0951..=0x0957
        | 0x1AB0..=0x1AFF
        | 0x1DC0..=0x1DFF
        | 0x20D0..=0x20F0
        | 0xFE00..=0xFE0F
        | 0xFE20..=0xFE2F)
}

/// Read back what `dump` wrote, answering None when the text is not something
/// `dump` could have produced.
fn undump(text: &str) -> Result<(Vec<u8>, Option<String>), &'static str> {
    // A dump of a string in an encoding of its own carries the name after the
    // quoted run, as `"...".force_encoding("NAME")`.
    let (body, named) = match text.rfind("\".force_encoding(") {
        None => (text, None),
        Some(at) => {
            let (body, tail) = text.split_at(at + 1);
            let inside = tail
                .strip_prefix(".force_encoding(")
                .and_then(|held| held.strip_suffix(')'))
                .ok_or("invalid dumped string")?;
            let name = inside
                .strip_prefix('"')
                .and_then(|held| held.strip_suffix('"'))
                .ok_or("invalid dumped string")?;
            (body, Some(name.to_string()))
        }
    };
    let inner = body.strip_prefix('"').ok_or("invalid dumped string")?;
    let inner = inner
        .strip_suffix('"')
        .ok_or("unterminated dumped string")?;
    let mut bytes: Vec<u8> = Vec::new();
    let mut characters = inner.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\\' {
            match character {
                '"' => return Err("invalid dumped string"),
                '\0' => return Err("string contains null byte"),
                held if !held.is_ascii() => return Err("non-ASCII character detected"),
                held => bytes.push(held as u8),
            }
            continue;
        }
        let escaped = characters.next().ok_or("invalid dumped string")?;
        match escaped {
            'n' => bytes.push(b'\n'),
            't' => bytes.push(b'\t'),
            'r' => bytes.push(b'\r'),
            '0' => bytes.push(0),
            'a' => bytes.push(7),
            'b' => bytes.push(8),
            'v' => bytes.push(11),
            'f' => bytes.push(12),
            'e' => bytes.push(27),
            's' => bytes.push(b' '),
            '\\' | '"' | '#' => bytes.push(escaped as u8),
            'u' => {
                let mut digits = String::new();
                if characters.peek() == Some(&'{') {
                    characters.next();
                    let mut closed = false;
                    for digit in characters.by_ref() {
                        if digit == '}' {
                            closed = true;
                            break;
                        }
                        digits.push(digit);
                    }
                    if !closed {
                        return Err("invalid Unicode escape");
                    }
                } else {
                    for _ in 0..4 {
                        digits.push(characters.next().ok_or("invalid Unicode escape")?);
                    }
                }
                let point =
                    u32::from_str_radix(&digits, 16).map_err(|_| "invalid Unicode escape")?;
                let spelled = char::from_u32(point).ok_or("invalid Unicode escape")?;
                let mut room = [0u8; 4];
                bytes.extend(spelled.encode_utf8(&mut room).as_bytes().iter().copied());
            }
            'x' => {
                let mut digits = String::new();
                for _ in 0..2 {
                    match characters.peek() {
                        Some(digit) if digit.is_ascii_hexdigit() => {
                            digits.push(*digit);
                            characters.next();
                        }
                        _ => break,
                    }
                }
                if digits.len() != 2 {
                    return Err("invalid hex escape");
                }
                bytes.push(u8::from_str_radix(&digits, 16).map_err(|_| "invalid hex escape")?);
            }
            _ => return Err("invalid dumped string"),
        }
    }
    Ok((bytes, named))
}

/// The encodings that carry no ASCII characters at all, so a string tagged
/// with one cannot be compared against a string tagged with another.
const NOT_ASCII_COMPATIBLE: &[&str] = &[
    "ISO-2022-JP",
    "UTF-16",
    "UTF-16BE",
    "UTF-16LE",
    "UTF-32",
    "UTF-32BE",
    "UTF-32LE",
];

impl VirtualMachine {
    /// Refuse an `upto` between two strings whose encodings have no common
    /// ground, which is what Ruby refuses before it compares them.
    fn check_upto_encodings(
        &mut self,
        left: &crate::object::StringValue,
        right: &crate::object::StringValue,
        position: Position,
    ) -> Result<(), MetorexError> {
        let (from, to) = (left.encoding_name(), right.encoding_name());
        if from == to {
            return Ok(());
        }
        if !NOT_ASCII_COMPATIBLE.contains(&from.as_str())
            && !NOT_ASCII_COMPATIBLE.contains(&to.as_str())
        {
            return Ok(());
        }
        let message = format!("incompatible character encodings: {} and {}", from, to);
        Err(MetorexError::UncaughtException {
            exception: Object::exception("Encoding::CompatibilityError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }
}

/// Every string `upto` yields, from `first` up to `last`.
fn upto_sequence(first: &str, last: &str, exclusive: bool) -> Vec<String> {
    let held = first.chars().count();
    let target = last.chars().count();
    if held > target || (held == target && first > last) {
        return Vec::new();
    }
    // Two single characters step by code point, so `"9".upto("A")` walks the
    // punctuation between them rather than counting.
    if held == 1 && target == 1 {
        let from = first.chars().next().expect("one character");
        let to = last.chars().next().expect("one character");
        let stop = if exclusive { to as u32 } else { to as u32 + 1 };
        return (from as u32..stop)
            .filter_map(char::from_u32)
            .map(String::from)
            .collect();
    }
    let mut steps = Vec::new();
    let mut current = first.to_string();
    loop {
        if exclusive && current == last {
            break;
        }
        steps.push(current.clone());
        if current == last {
            break;
        }
        let next = crate::vm::native_methods::string_methods::successor_of(&current);
        if next.chars().count() > target {
            break;
        }
        current = next;
    }
    steps
}

/// Fold only the ASCII letters, which is the comparison `casecmp` makes.
fn fold_ascii_case(text: &str) -> String {
    text.chars()
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

/// The encoding two strings can both be read in, or None when they hold
/// characters their encodings disagree about.
fn compatible_encoding(
    left: &str,
    left_encoding: &str,
    right: &str,
    right_encoding: &str,
) -> Option<String> {
    if left_encoding == right_encoding {
        return Some(left_encoding.to_string());
    }
    // A string that is all ASCII reads the same under either encoding, so the
    // other one decides.
    if left.is_ascii() {
        return Some(right_encoding.to_string());
    }
    if right.is_ascii() {
        return Some(left_encoding.to_string());
    }
    None
}

/// Whether an encoding spells the whole of Unicode, which is what decides
/// whether a letter outside ASCII maps onto another case of itself.
fn unicode_encoding(named: &str) -> bool {
    named.starts_with("UTF-") || named == "US-ASCII" || named == "CESU-8"
}

impl VirtualMachine {
    /// `scrub` answers the string with every run of bytes its encoding cannot
    /// read put aside: a replacement named alongside stands for each one, a
    /// block is asked what to put there, and without either the encoding's
    /// own replacement character stands in.
    fn scrubbed_string(
        &mut self,
        held: &std::rc::Rc<crate::object::StringValue>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        use super::string_methods::binary_bytes;
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
                if !super::string_methods::holds_valid_text(written) {
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
                            super::pack_format::bytes_to_string(&bad).to_string(),
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
enum Run {
    Read(Vec<u8>),
    Broken(Vec<u8>),
}

/// The runs of bytes an encoding reads, alongside the ones it cannot.
fn split_readable_runs(bytes: &[u8], named: &str) -> Vec<Run> {
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
fn stands_in_for(named: &str) -> Vec<u8> {
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
fn text_in_encoding(bytes: &[u8], named: &str) -> Object {
    if let Ok(text) = std::str::from_utf8(bytes)
        && (named == "UTF-8" || named == "US-ASCII")
    {
        let made = crate::object::StringValue::with_encoding(text.to_string(), named);
        return Object::String(std::rc::Rc::new(made));
    }
    let made = crate::object::StringValue::from_bytes(
        super::pack_format::bytes_to_string(bytes).to_string(),
    );
    made.set_encoding(named);
    Object::String(std::rc::Rc::new(made))
}
