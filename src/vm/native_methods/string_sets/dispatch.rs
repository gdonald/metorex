// The String methods that read character sets, split apart, or
// measure what a string holds.

use super::*;

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
    pub(crate) fn character_sets(
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
                    instance => match crate::vm::native_methods::string_subclass_value(instance) {
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
                    if crate::vm::native_methods::string_methods::encodings_clash(string_value, pad)) =>
            {
                let Some(Object::String(pad)) = arguments.get(1) else {
                    return Ok(None);
                };
                Err(
                    crate::vm::native_methods::string_methods::clashing_encodings_error(
                        string_value,
                        pad,
                        position,
                    ),
                )
            }
            "center" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 2),
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
                    let Some(Object::String(held)) =
                        crate::vm::native_methods::string_subclass_value(receiver)
                    else {
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
                    _ => match crate::vm::native_methods::string_subclass_value(receiver) {
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
                    crate::vm::native_methods::pack_format::bytes_to_string(&bytes).to_string(),
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
        if matches!(argument, Object::String(_))
            || crate::vm::native_methods::string_subclass_value(argument).is_some()
        {
            return Ok(argument.clone());
        }
        if !self.responds_to(argument, "to_str") {
            return Ok(Object::Nil);
        }
        let source = self.builtins().class_of(argument).name().to_string();
        let converted = self.send_to_object(argument.clone(), "to_str", vec![], position)?;
        if matches!(converted, Object::Nil | Object::String(_))
            || crate::vm::native_methods::string_subclass_value(&converted).is_some()
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
    pub(crate) fn regexp_partition(
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
