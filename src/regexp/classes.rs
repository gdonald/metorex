//! Whether a character belongs to a named run or to a written-out class.

use super::node::{Class, ClassItem, Named};

/// Whether `letter` belongs to the run `named` stands for.
pub fn in_named(named: &Named, letter: char) -> bool {
    match named {
        // The escapes Ruby writes with a backslash cover ASCII alone, while
        // the bracket names below cover every character the name fits.
        Named::AsciiDigit => letter.is_ascii_digit(),
        Named::AsciiWord => letter.is_ascii_alphanumeric() || letter == '_',
        Named::AsciiSpace => matches!(letter, ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}'),
        Named::Digit => is_decimal(letter),
        Named::Word => is_word(letter),
        Named::Space => letter.is_whitespace(),
        Named::Alpha => letter.is_alphabetic(),
        Named::Alnum => letter.is_alphanumeric(),
        Named::Upper => letter.is_uppercase(),
        Named::Lower => letter.is_lowercase(),
        Named::Punct => is_punctuation(letter),
        Named::Print => is_graphic(letter) || letter == ' ',
        Named::Graph => is_graphic(letter),
        Named::Cntrl => letter.is_control(),
        Named::Blank => letter == ' ' || letter == '\t' || is_space_separator(letter),
        Named::XDigit => letter.is_ascii_hexdigit(),
        Named::Ascii => letter.is_ascii(),
        Named::Property(name) => in_property(name, letter),
    }
}

/// Whether the character is one a reader sees, as against a space or a
/// control code. A formatting mark and a character set aside for private use
/// both count, since neither is a space and neither is a control code.
fn is_graphic(letter: char) -> bool {
    !letter.is_whitespace() && !letter.is_control()
}

/// The digits a number is written with, which is narrower than every
/// character that stands for a number.
fn is_decimal(letter: char) -> bool {
    matches!(letter,
        '\u{30}'..='\u{39}'
            | '\u{660}'..='\u{669}'
            | '\u{6f0}'..='\u{6f9}'
            | '\u{7c0}'..='\u{7c9}'
            | '\u{966}'..='\u{96f}'
            | '\u{9e6}'..='\u{9ef}'
            | '\u{a66}'..='\u{a6f}'
            | '\u{ae6}'..='\u{aef}'
            | '\u{b66}'..='\u{b6f}'
            | '\u{be6}'..='\u{bef}'
            | '\u{c66}'..='\u{c6f}'
            | '\u{ce6}'..='\u{cef}'
            | '\u{d66}'..='\u{d6f}'
            | '\u{de6}'..='\u{def}'
            | '\u{e50}'..='\u{e59}'
            | '\u{ed0}'..='\u{ed9}'
            | '\u{f20}'..='\u{f29}'
            | '\u{1040}'..='\u{1049}'
            | '\u{1090}'..='\u{1099}'
            | '\u{17e0}'..='\u{17e9}'
            | '\u{1810}'..='\u{1819}'
            | '\u{1946}'..='\u{194f}'
            | '\u{19d0}'..='\u{19d9}'
            | '\u{1a80}'..='\u{1a89}'
            | '\u{1a90}'..='\u{1a99}'
            | '\u{1b50}'..='\u{1b59}'
            | '\u{1bb0}'..='\u{1bb9}'
            | '\u{1c40}'..='\u{1c49}'
            | '\u{1c50}'..='\u{1c59}'
            | '\u{a620}'..='\u{a629}'
            | '\u{a8d0}'..='\u{a8d9}'
            | '\u{a900}'..='\u{a909}'
            | '\u{a9d0}'..='\u{a9d9}'
            | '\u{a9f0}'..='\u{a9f9}'
            | '\u{aa50}'..='\u{aa59}'
            | '\u{abf0}'..='\u{abf9}'
            | '\u{ff10}'..='\u{ff19}'
            | '\u{104a0}'..='\u{104a9}'
            | '\u{10d30}'..='\u{10d39}'
            | '\u{11066}'..='\u{1106f}'
            | '\u{110f0}'..='\u{110f9}'
            | '\u{11136}'..='\u{1113f}'
            | '\u{111d0}'..='\u{111d9}'
            | '\u{112f0}'..='\u{112f9}'
            | '\u{11450}'..='\u{11459}'
            | '\u{114d0}'..='\u{114d9}'
            | '\u{11650}'..='\u{11659}'
            | '\u{116c0}'..='\u{116c9}'
            | '\u{11730}'..='\u{11739}'
            | '\u{118e0}'..='\u{118e9}'
            | '\u{11950}'..='\u{11959}'
            | '\u{11c50}'..='\u{11c59}'
            | '\u{11d50}'..='\u{11d59}'
            | '\u{11da0}'..='\u{11da9}'
            | '\u{11f50}'..='\u{11f59}'
            | '\u{16a60}'..='\u{16a69}'
            | '\u{16ac0}'..='\u{16ac9}'
            | '\u{16b50}'..='\u{16b59}'
            | '\u{1d7ce}'..='\u{1d7ff}'
            | '\u{1e140}'..='\u{1e149}'
            | '\u{1e2f0}'..='\u{1e2f9}'
            | '\u{1e4f0}'..='\u{1e4f9}'
            | '\u{1e950}'..='\u{1e959}'
            | '\u{1fbf0}'..='\u{1fbf9}')
}

/// Whether the character carries no shape of its own: the formatting marks
/// and the ranges set aside for private use.
fn is_invisible(letter: char) -> bool {
    matches!(letter,
        '\u{ad}'
            | '\u{600}'..='\u{605}'
            | '\u{61c}'
            | '\u{6dd}'
            | '\u{70f}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
            | '\u{110bd}'
            | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0001}'
            | '\u{e0020}'..='\u{e007f}'
            | '\u{e000}'..='\u{f8ff}'
            | '\u{f0000}'..='\u{ffffd}'
            | '\u{100000}'..='\u{10fffd}')
}

/// The marks that sit on the character before them, which a word takes along
/// with the letters it is spelled from.
fn is_mark(letter: char) -> bool {
    matches!(letter,
        '\u{300}'..='\u{36f}'
            | '\u{483}'..='\u{489}'
            | '\u{591}'..='\u{5bd}'
            | '\u{610}'..='\u{61a}'
            | '\u{64b}'..='\u{65f}'
            | '\u{670}'
            | '\u{6d6}'..='\u{6dc}'
            | '\u{0e31}'
            | '\u{0e34}'..='\u{0e3a}'
            | '\u{1ab0}'..='\u{1aff}'
            | '\u{1dc0}'..='\u{1dff}'
            | '\u{20d0}'..='\u{20f0}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{fe20}'..='\u{fe2f}')
}

/// Whether the character is one a word is spelled with: a letter, a digit, a
/// mark carried on one of those, the underscore, and the two join controls
/// that hold a word together without showing.
fn is_word(letter: char) -> bool {
    letter.is_alphabetic()
        || is_decimal(letter)
        || letter == '_'
        || is_mark(letter)
        || matches!(letter, '\u{200c}' | '\u{200d}')
}

fn is_space_separator(letter: char) -> bool {
    matches!(
        letter,
        '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

fn is_punctuation(letter: char) -> bool {
    if letter.is_ascii() {
        return letter.is_ascii_punctuation();
    }
    is_graphic(letter) && !letter.is_alphanumeric() && !is_invisible(letter)
}

/// Whether the character carries the Unicode property `name`. The names are
/// read without regard to case, underscores, or hyphens, which is how Ruby
/// reads them.
fn in_property(name: &str, letter: char) -> bool {
    let plain: String = name
        .chars()
        .filter(|held| *held != '_' && *held != '-' && *held != ' ')
        .flat_map(|held| held.to_lowercase())
        .collect();
    match plain.as_str() {
        "alpha" | "alphabetic" | "l" | "letter" => letter.is_alphabetic(),
        "alnum" => letter.is_alphanumeric(),
        "digit" | "nd" | "decimalnumber" => is_decimal(letter),
        "n" | "number" => letter.is_numeric(),
        "space" | "whitespace" | "white_space" => letter.is_whitespace(),
        "upper" | "uppercase" | "lu" => letter.is_uppercase(),
        "lower" | "lowercase" | "ll" => letter.is_lowercase(),
        "word" => is_word(letter),
        "ascii" => letter.is_ascii(),
        "cntrl" | "control" | "cc" => letter.is_control(),
        "print" => is_graphic(letter) || letter == ' ',
        "graph" => is_graphic(letter),
        "punct" | "punctuation" | "p" => is_punctuation(letter),
        "blank" => letter == ' ' || letter == '\t' || is_space_separator(letter),
        "xdigit" => letter.is_ascii_hexdigit(),
        "any" => true,
        "assigned" => letter != '\u{0}',
        "s" | "symbol" => is_graphic(letter) && !letter.is_alphanumeric() && !letter.is_ascii(),
        "z" | "separator" => letter.is_whitespace(),
        "latin" => in_script(letter, Script::Latin),
        "greek" => in_script(letter, Script::Greek),
        "cyrillic" => in_script(letter, Script::Cyrillic),
        "han" => in_script(letter, Script::Han),
        "hiragana" => matches!(letter, '\u{3041}'..='\u{3096}' | '\u{309d}'..='\u{309f}'),
        "katakana" => {
            // The prolonged sound mark and the iteration marks belong to no
            // script of their own, so the run is written around them.
            matches!(letter,
                '\u{30a1}'..='\u{30fa}'
                    | '\u{30fd}'..='\u{30ff}'
                    | '\u{31f0}'..='\u{31ff}'
                    | '\u{ff66}'..='\u{ff6f}'
                    | '\u{ff71}'..='\u{ff9d}')
        }
        "arabic" => ('\u{0600}'..='\u{06ff}').contains(&letter),
        "hebrew" => ('\u{0590}'..='\u{05ff}').contains(&letter),
        "hangul" => {
            ('\u{1100}'..='\u{11ff}').contains(&letter)
                || ('\u{ac00}'..='\u{d7af}').contains(&letter)
        }
        "emoji" => ('\u{1f300}'..='\u{1faff}').contains(&letter) || letter == '\u{263a}',
        _ => false,
    }
}

enum Script {
    Latin,
    Greek,
    Cyrillic,
    Han,
}

fn in_script(letter: char, script: Script) -> bool {
    match script {
        Script::Latin => {
            letter.is_ascii_alphabetic()
                || ('\u{00c0}'..='\u{024f}').contains(&letter)
                || ('\u{1e00}'..='\u{1eff}').contains(&letter)
        }
        Script::Greek => {
            ('\u{0370}'..='\u{03ff}').contains(&letter)
                || ('\u{1f00}'..='\u{1fff}').contains(&letter)
        }
        Script::Cyrillic => {
            ('\u{0400}'..='\u{04ff}').contains(&letter)
                || ('\u{0500}'..='\u{052f}').contains(&letter)
        }
        Script::Han => {
            ('\u{4e00}'..='\u{9fff}').contains(&letter)
                || ('\u{3400}'..='\u{4dbf}').contains(&letter)
                || ('\u{f900}'..='\u{faff}').contains(&letter)
        }
    }
}

/// Whether the character is carried on the one before it rather than
/// standing on its own: a mark, a skin-tone modifier, or a selector saying
/// how the character before it is drawn.
pub fn is_carried(letter: char) -> bool {
    is_mark(letter) || matches!(letter, '\u{1f3fb}'..='\u{1f3ff}' | '\u{fe00}'..='\u{fe0f}')
}

/// Whether `letter` belongs to the class.
pub fn in_class(class: &Class, letter: char) -> bool {
    if class.ascii_only && !letter.is_ascii() {
        return class.negated;
    }
    // The parts are the groups `&&` separated, and a character belongs to the
    // class only when it belongs to every one of them.
    let inside = class
        .parts
        .iter()
        .all(|items| items.iter().any(|item| in_item(item, letter)));
    inside != class.negated
}

fn in_item(item: &ClassItem, letter: char) -> bool {
    match item {
        ClassItem::Letter(held) => *held == letter,
        ClassItem::Span(low, high) => *low <= letter && letter <= *high,
        ClassItem::Named(named, negated) => in_named(named, letter) != *negated,
        ClassItem::Nested(class) => in_class(class, letter),
    }
}
