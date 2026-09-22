// Mapping the case of letters, with the options Ruby allows.

use super::*;

/// Which way a case mapping runs.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CaseWanted {
    Up,
    Down,
}

/// The options a case mapping was asked for.
pub(crate) struct CaseOptions {
    ascii_only: bool,
    turkic: bool,
    folding: bool,
}

/// Read the Symbols naming how a case mapping should run. Ruby allows one
/// option, and allows Turkic and Lithuanian together.
pub(crate) fn case_options(
    method_name: &str,
    arguments: &[Object],
    position: Position,
) -> Result<CaseOptions, MetorexError> {
    let mut named = Vec::new();
    for argument in arguments {
        let Object::Symbol(name) = argument else {
            let message = format!("invalid option {}", argument);
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &message,
                position,
            ));
        };
        let name = name.as_str().to_string();
        if !matches!(
            name.as_str(),
            "ascii" | "turkic" | "lithuanian" | "fold" | "downcase"
        ) || (name == "fold" && !method_name.starts_with("downcase"))
        {
            let message = format!("invalid option :{}", name);
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &message,
                position,
            ));
        }
        named.push(name);
    }
    // Turkic and Lithuanian are the only pair that go together.
    if named.len() > 2
        || (named.len() == 2
            && !named
                .iter()
                .all(|name| name == "turkic" || name == "lithuanian"))
    {
        let message = "too many options".to_string();
        return Err(crate::vm::errors::simple_exception(
            "ArgumentError",
            &message,
            position,
        ));
    }
    Ok(CaseOptions {
        ascii_only: named.iter().any(|name| name == "ascii"),
        turkic: named.iter().any(|name| name == "turkic"),
        folding: named.iter().any(|name| name == "fold"),
    })
}

/// Text with its letters mapped the way the options ask for.
pub(crate) fn mapped_case(text: &str, wanted: CaseWanted, options: &CaseOptions) -> String {
    if options.ascii_only {
        return match wanted {
            CaseWanted::Up => text.chars().map(|held| held.to_ascii_uppercase()).collect(),
            CaseWanted::Down => text.chars().map(|held| held.to_ascii_lowercase()).collect(),
        };
    }
    if options.turkic {
        // Turkish keeps the dot of an `i` apart from the letter itself, so
        // the two `i`s map to their own pairs.
        return text
            .chars()
            .flat_map(|held| match (wanted, held) {
                (CaseWanted::Down, '\u{130}') => vec!['i'],
                (CaseWanted::Down, 'I') => vec!['\u{131}'],
                (CaseWanted::Up, 'i') => vec!['\u{130}'],
                (CaseWanted::Up, '\u{131}') => vec!['I'],
                (CaseWanted::Up, _) => held.to_uppercase().collect(),
                (CaseWanted::Down, _) => held.to_lowercase().collect(),
            })
            .collect();
    }
    let mapped = match wanted {
        CaseWanted::Up => text.to_uppercase(),
        CaseWanted::Down => text.to_lowercase(),
    };
    // Folding maps a letter onto the letters it compares equal to, which is
    // where a sharp s becomes two of them.
    if options.folding {
        return mapped.replace('\u{df}', "ss");
    }
    mapped
}

/// The first letter raised and the rest lowered. Raising one letter may give
/// several, and only the first of those stays raised.
pub(crate) fn capitalized_case(text: &str, options: &CaseOptions) -> String {
    let mut letters = text.chars();
    let Some(first) = letters.next() else {
        return String::new();
    };
    let raised = mapped_case(&first.to_string(), CaseWanted::Up, options);
    let mut made = String::new();
    let mut raised_letters = raised.chars();
    if let Some(leading) = raised_letters.next() {
        made.push(leading);
    }
    let rest: String = raised_letters.collect();
    made.push_str(&mapped_case(&rest, CaseWanted::Down, options));
    let remainder: String = letters.collect();
    made.push_str(&mapped_case(&remainder, CaseWanted::Down, options));
    made
}

/// Each letter turned the other way.
pub(crate) fn swapped_case(text: &str, options: &CaseOptions) -> String {
    text.chars()
        .flat_map(|letter| {
            if options.ascii_only && !letter.is_ascii() {
                return vec![letter];
            }
            let wanted = if letter.is_uppercase() {
                CaseWanted::Down
            } else if letter.is_lowercase() {
                CaseWanted::Up
            } else {
                return vec![letter];
            };
            mapped_case(&letter.to_string(), wanted, options)
                .chars()
                .collect()
        })
        .collect()
}
