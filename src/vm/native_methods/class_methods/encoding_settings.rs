// The Encoding class methods.

use super::*;

thread_local! {
    /// The names of the dummy encodings defined while the program runs.
    static ADDED_DUMMIES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    /// The aliases defined while the program runs, each with the name of
    /// the encoding it stands for.
    static ADDED_ALIASES: std::cell::RefCell<Vec<(String, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Whether the encoding named `named` is a dummy: one Ruby names and tags
/// strings with but cannot convert through.
pub(crate) fn is_dummy_encoding(named: &str) -> bool {
    crate::vm::init::ENCODING_NAMES
        .iter()
        .any(|(_, display, dummy)| *dummy && *display == named)
        || ADDED_DUMMIES.with(|held| held.borrow().iter().any(|held| held == named))
}

/// Record `named` as a dummy encoding defined while the program runs.
pub(crate) fn add_dummy_encoding(named: &str) {
    ADDED_DUMMIES.with(|held| held.borrow_mut().push(named.to_string()));
}

/// Record `alias` as a name for the encoding named `display`.
pub(crate) fn add_encoding_alias(alias: &str, display: &str) {
    ADDED_ALIASES.with(|held| {
        held.borrow_mut()
            .push((alias.to_string(), display.to_string()))
    });
}

impl VirtualMachine {
    /// The encodings metorex names, and the settings that say which one is
    /// read and written by default.
    pub(crate) fn call_encoding_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // `Encoding.default_external` names the encoding metorex reads and
        // writes with. Every string is UTF-8, so the setting is remembered and
        // reported without changing how one is stored.
        if class_rc.name() == "Encoding"
            && matches!(method_name, "default_external" | "default_external=")
        {
            if method_name == "default_external=" {
                let Some(value) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                // The external encoding always names one, so nothing stands
                // for "no encoding" there.
                if matches!(value, Object::Nil) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "default external cannot be nil",
                        position,
                    ));
                }
                let settled = self.encoding_setting(class_rc, value, position)?;
                self.globals_mut()
                    .set("__Encoding_default_external", settled);
                return Ok(Answered(value.clone()));
            }
            return Ok(Answered(
                self.globals()
                    .get("__Encoding_default_external")
                    .or_else(|| self.globals().get("Encoding::UTF_8"))
                    .unwrap_or(Object::Nil),
            ));
        }
        // `Encoding.default_internal` names the encoding a string is
        // converted to on the way in. Metorex converts nothing, so it stays
        // nil unless a program sets it.
        if class_rc.name() == "Encoding"
            && matches!(method_name, "default_internal" | "default_internal=")
        {
            if method_name == "default_internal=" {
                let Some(value) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                let settled = self.encoding_setting(class_rc, value, position)?;
                self.globals_mut()
                    .set("__Encoding_default_internal", settled);
                return Ok(Answered(value.clone()));
            }
            return Ok(Answered(
                self.globals()
                    .get("__Encoding_default_internal")
                    .unwrap_or(Object::Nil),
            ));
        }
        // `Encoding.find` answers the encoding a name stands for. The names
        // that stand for a setting rather than an encoding answer what that
        // setting holds.
        // An encoding is a class standing under Encoding, so the readings an
        // encoding answers are handled here rather than from Encoding's own
        // instance methods.
        if class_rc
            .superclass()
            .is_some_and(|parent| parent.name() == "Encoding")
        {
            match method_name {
                "dummy?" => {
                    return Ok(Answered(Object::Bool(is_dummy_encoding(class_rc.name()))));
                }
                // A dummy encoding converts nothing, so nothing it holds
                // stands for ASCII either. The wide UTF forms are not ASCII
                // compatible for the plainer reason that their code units are
                // more than a byte.
                "ascii_compatible?" => {
                    let dummy = is_dummy_encoding(class_rc.name());
                    return Ok(Answered(Object::Bool(
                        !dummy
                            && !class_rc.name().starts_with("UTF-16")
                            && !class_rc.name().starts_with("UTF-32"),
                    )));
                }
                "inspect" => {
                    // Ruby shows ASCII-8BIT under the name BINARY, with the
                    // name it reports alongside.
                    let dummy = is_dummy_encoding(class_rc.name());
                    let shown = if class_rc.name() == "ASCII-8BIT" {
                        "BINARY (ASCII-8BIT)".to_string()
                    } else {
                        class_rc.name().to_string()
                    };
                    let tail = if dummy { " (dummy)" } else { "" };
                    return Ok(Answered(Object::string(format!(
                        "#<Encoding:{}{}>",
                        shown, tail
                    ))));
                }
                _ => {}
            }
        }

        // `Encoding.compatible?` answers the encoding two objects could be
        // read in together, or nil when there is none.
        if class_rc.name() == "Encoding" && method_name == "compatible?" {
            if arguments.len() != 2 {
                return Err(method_argument_error(
                    method_name,
                    2,
                    arguments.len(),
                    position,
                ));
            }
            let answer = self.compatible_encoding(&arguments[0], &arguments[1]);
            return Ok(Answered(match answer {
                Some(named) => self.encoding_object(&named),
                None => Object::Nil,
            }));
        }
        // Every encoding metorex names, each one listed once however many
        // constants reach it.
        if class_rc.name() == "Encoding" && method_name == "list" {
            return Ok(Answered(
                self.globals()
                    .get("__Encoding_list")
                    .unwrap_or_else(|| Object::array(Vec::new())),
            ));
        }
        // The names that stand for an encoding already listed under another
        // name, each paired with the name it stands for.
        if class_rc.name() == "Encoding" && method_name == "aliases" {
            let mut pairs: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
            let keep = crate::vm::native_methods::hash_methods::keep_pair;
            for (alias, display) in crate::vm::init::ENCODING_ALIASES {
                keep(
                    &mut pairs,
                    Object::string(alias.to_string()),
                    Object::string(display.to_string()),
                );
            }
            ADDED_ALIASES.with(|held| {
                for (alias, display) in held.borrow().iter() {
                    keep(
                        &mut pairs,
                        Object::string(alias.clone()),
                        Object::string(display.clone()),
                    );
                }
            });
            // Ruby lists the settings among the aliases, so "locale",
            // "external" and "filesystem" name the encodings they stand for.
            for named in ["locale", "external", "filesystem"] {
                let found =
                    self.call_class_methods(class_rc, "find", &[Object::string(named)], position)?;
                if let Some(encoding) = found {
                    let name = self.send_to_object(encoding, "name", Vec::new(), position)?;
                    keep(&mut pairs, Object::string(named.to_string()), name);
                }
            }
            return Ok(Answered(Object::Dict(Rc::new(std::cell::RefCell::new(
                pairs,
            )))));
        }
        if class_rc.name() == "Encoding" && matches!(method_name, "find" | "[]") {
            let wanted = match arguments.first() {
                Some(Object::String(held)) => held.as_str().to_string(),
                Some(Object::Class(held)) if held.name().contains('-') => {
                    return Ok(Answered(arguments[0].clone()));
                }
                // An encoding is named by a String, and a Symbol is not one.
                // Anything that spells itself out names one all the same.
                Some(other @ Object::Symbol(_)) => {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        other,
                        position,
                    ));
                }
                Some(other) if self.responds_to(other, "to_str") => {
                    let spelled =
                        self.send_to_object(other.clone(), "to_str", Vec::new(), position)?;
                    match spelled {
                        Object::String(held) => held.as_str().to_string(),
                        held => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                &held,
                                position,
                            ));
                        }
                    }
                }
                _ => return Err(method_argument_error(method_name, 1, 0, position)),
            };
            let settled =
                match wanted.to_ascii_lowercase().as_str() {
                    // The locale names an encoding of its own, which is
                    // what the default external starts as but need not stay.
                    "locale" => {
                        let named = Object::Class(Rc::clone(class_rc));
                        let charmap =
                            self.send_to_object(named, "locale_charmap", Vec::new(), position)?;
                        if let charmap @ Object::String(_) = charmap {
                            return self
                                .call_class_methods(
                                    class_rc,
                                    "find",
                                    std::slice::from_ref(&charmap),
                                    position,
                                )
                                .map(nested_answer);
                        }
                        return Ok(Answered(
                            self.globals()
                                .get("__Encoding_default_external")
                                .or_else(|| self.globals().get("Encoding::UTF_8"))
                                .unwrap_or(Object::Nil),
                        ));
                    }
                    "external" | "filesystem" => {
                        return Ok(Answered(
                            self.globals()
                                .get("__Encoding_default_external")
                                .or_else(|| self.globals().get("Encoding::UTF_8"))
                                .unwrap_or(Object::Nil),
                        ));
                    }
                    "internal" => {
                        return Ok(Answered(
                            self.globals()
                                .get("__Encoding_default_internal")
                                .unwrap_or(Object::Nil),
                        ));
                    }
                    "utf8" => "UTF_8",
                    "ascii" | "ansi_x3.4-1968" => "US_ASCII",
                    "binary" => "BINARY",
                    // Every other name is matched against the table the constants
                    // were built from, so a constant and the name it reports find
                    // the same encoding.
                    _ => {
                        // An alias names the encoding it stands for.
                        let aliased = crate::vm::init::ENCODING_ALIASES
                            .iter()
                            .find(|(alias, _)| alias.eq_ignore_ascii_case(&wanted))
                            .map(|(_, display)| *display);
                        let looked_up = aliased.unwrap_or(wanted.as_str());
                        let Some((settled, _, _)) = crate::vm::init::ENCODING_NAMES.iter().find(
                            |(constant, display, _)| {
                                constant.eq_ignore_ascii_case(looked_up)
                                    || display.eq_ignore_ascii_case(looked_up)
                            },
                        ) else {
                            if let Some(found) = self.added_encoding(&wanted) {
                                return Ok(Answered(found));
                            }
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &format!("unknown encoding name - {wanted}"),
                                position,
                            ));
                        };
                        settled
                    }
                };
            return Ok(Answered(
                self.globals()
                    .get(&format!("Encoding::{settled}"))
                    .unwrap_or(Object::Nil),
            ));
        }
        Ok(Unclaimed)
    }
}

impl VirtualMachine {
    /// The encoding defined or aliased while the program runs that `wanted`
    /// names, ignoring case.
    fn added_encoding(&self, wanted: &str) -> Option<Object> {
        let named = ADDED_ALIASES
            .with(|held| {
                held.borrow()
                    .iter()
                    .find(|(alias, _)| alias.eq_ignore_ascii_case(wanted))
                    .map(|(_, display)| display.clone())
            })
            .unwrap_or_else(|| wanted.to_string());
        let mut found = None;
        if let Some(Object::Array(listed)) = self.globals().get("__Encoding_list") {
            found = listed.borrow().iter().find(|held| {
                matches!(held, Object::Class(class) if class.name().eq_ignore_ascii_case(&named))
            }).cloned();
        }
        found
    }
}
