// Reading a Complex out of the text a program writes it as.

/// One parsed component of a complex literal, before it becomes an Object.
pub(crate) enum ComplexComponent {
    Integer(num_bigint::BigInt),
    Float(f64),
    Fraction(num_bigint::BigInt, num_bigint::BigInt),
}

/// The two components of a complex literal, and whether they were written in
/// polar form (`modulus@argument`).
pub(crate) struct ParsedComplex {
    pub real: ComplexComponent,
    pub imaginary: ComplexComponent,
    pub polar: bool,
}

/// Strip the `_` digit separators Ruby allows, refusing a run of two or a
/// separator that is not between digits.
pub(crate) fn strip_digit_separators(text: &str) -> Option<String> {
    if !text.contains('_') {
        return Some(text.to_string());
    }
    let characters: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (index, character) in characters.iter().enumerate() {
        if *character != '_' {
            out.push(*character);
            continue;
        }
        let before = index.checked_sub(1).and_then(|i| characters.get(i));
        let after = characters.get(index + 1);
        if !before.is_some_and(char::is_ascii_digit) || !after.is_some_and(char::is_ascii_digit) {
            return None;
        }
    }
    Some(out)
}

/// Read one numeric component: an integer, a float (with optional exponent),
/// or a `numerator/denominator` fraction. The whole text must be consumed.
pub(crate) fn parse_component(text: &str) -> Option<ComplexComponent> {
    let text = strip_digit_separators(text)?;
    let text = text.as_str();
    if text.is_empty() {
        return None;
    }
    if let Some((numerator, denominator)) = text.split_once('/') {
        let numerator = parse_integer_text(numerator)?;
        let denominator = parse_integer_text(denominator)?;
        return Some(ComplexComponent::Fraction(numerator, denominator));
    }
    if text.contains('.') || text.contains('e') || text.contains('E') {
        // Rust accepts `inf` and `nan`, which Ruby's converter does not.
        if !text
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E'))
        {
            return None;
        }
        return text.parse::<f64>().ok().map(ComplexComponent::Float);
    }
    parse_integer_text(text).map(ComplexComponent::Integer)
}

/// Read a signed run of digits, and nothing else.
pub(crate) fn parse_integer_text(text: &str) -> Option<num_bigint::BigInt> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    num_bigint::BigInt::parse_bytes(text.as_bytes(), 10)
}

/// The index of the sign that splits the real part from the imaginary one: the
/// last `+` or `-` that is neither leading nor part of an exponent.
pub(crate) fn imaginary_sign_index(text: &str) -> Option<usize> {
    let characters: Vec<char> = text.chars().collect();
    for index in (1..characters.len()).rev() {
        if !matches!(characters[index], '+' | '-') {
            continue;
        }
        if matches!(characters[index - 1], 'e' | 'E') {
            continue;
        }
        return Some(index);
    }
    None
}

/// Read a complex literal the way `Complex("...")` does. The whole string must
/// be consumed, so trailing text makes this answer None.
/// The longest run at the start of `text` that spells a complex number, read
/// the lenient way `String#to_c` reads one: a doubled underscore ends the
/// number, a single one between digits is only spacing, and a NUL ends the
/// text. Nothing spelling a number at all answers zero.
pub(crate) fn leading_complex_text(text: &str) -> Option<ParsedComplex> {
    // A NUL ends the text, and so does a doubled underscore.
    let held = text.split('\0').next().unwrap_or("");
    let held = match held.find("__") {
        Some(at) => &held[..at],
        None => held,
    };
    // A single underscore between digits is spacing rather than part of the
    // number, which is how `12_3` reads as 123.
    let spelled: String = held
        .char_indices()
        .filter(|(at, held)| {
            if *held != '_' {
                return true;
            }
            let before = text[..*at].chars().next_back();
            let after = text[at + 1..].chars().next();
            !(before.is_some_and(|one| one.is_ascii_digit())
                && after.is_some_and(|one| one.is_ascii_digit()))
        })
        .map(|(_, held)| held)
        .collect();
    // The longest run that reads as a number is the number, so whatever
    // follows it is left alone.
    let letters: Vec<char> = spelled.chars().collect();
    for end in (1..=letters.len()).rev() {
        let candidate: String = letters[..end].iter().collect();
        if let Some(parsed) = parse_complex_text(&candidate) {
            return Some(parsed);
        }
    }
    None
}

pub(crate) fn parse_complex_text(text: &str) -> Option<ParsedComplex> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some((modulus, argument)) = trimmed.split_once('@') {
        return Some(ParsedComplex {
            real: parse_component(modulus)?,
            imaginary: parse_component(argument)?,
            polar: true,
        });
    }
    let zero = ComplexComponent::Integer(num_bigint::BigInt::from(0));
    let Some(body) = trimmed.strip_suffix(['i', 'I', 'j', 'J']) else {
        return Some(ParsedComplex {
            real: parse_component(trimmed)?,
            imaginary: zero,
            polar: false,
        });
    };
    // `i`, `+i`, and `-i` carry no digits of their own and mean 1i.
    let unit = |sign: i32| ComplexComponent::Integer(num_bigint::BigInt::from(sign));
    match imaginary_sign_index(body) {
        Some(index) => {
            let (real_text, imaginary_text) = body.split_at(index);
            let imaginary = if imaginary_text.len() == 1 {
                unit(if imaginary_text.starts_with('-') {
                    -1
                } else {
                    1
                })
            } else {
                parse_component(imaginary_text)?
            };
            Some(ParsedComplex {
                real: parse_component(real_text)?,
                imaginary,
                polar: false,
            })
        }
        None => {
            let imaginary = match body {
                "" => unit(1),
                "+" => unit(1),
                "-" => unit(-1),
                digits => parse_component(digits)?,
            };
            Some(ParsedComplex {
                real: zero,
                imaginary,
                polar: false,
            })
        }
    }
}
