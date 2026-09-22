// The directives a format string is made of.

use super::*;

/// How many items a directive applies to.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Count {
    /// No count was written, so the directive applies once.
    One,
    /// A number was written.
    Exactly(usize),
    /// `*` was written: every item that remains.
    Rest,
}

/// One directive with the count that follows it.
pub(crate) struct Directive {
    pub(crate) code: char,
    pub(crate) count: Count,
    /// The byte order a `<` or `>` modifier asked for, where the directive
    /// admits one. None means the directive's own order.
    pub(crate) order: Option<Order>,
    /// Whether `!` or `_` asked for the platform's own width.
    pub(crate) native: bool,
}

/// Which end of a number its first byte holds.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Order {
    Little,
    Big,
}

/// Split a format string into its directives. Whitespace between them is
/// ignored, a `#` runs a comment to the end of the line, and a directive Ruby
/// does not know is refused by name.
pub(crate) fn parse_format(
    format: &str,
    verb: &str,
    position: Position,
) -> Result<Vec<Directive>, MetorexError> {
    let characters: Vec<char> = format.chars().collect();
    let mut directives = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        let code = characters[index];
        if code.is_whitespace() {
            index += 1;
            continue;
        }
        if code == '#' {
            while index < characters.len() && characters[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if !is_known_directive(code) {
            return Err(unknown_directive(code, format, verb, position));
        }
        index += 1;
        // `!` and `_` ask for the platform's own width, and `<` and `>` name
        // the byte order. Only the integer directives admit either, and both
        // may be written, in either order.
        let sized_integer = matches!(
            code,
            's' | 'S' | 'i' | 'I' | 'l' | 'L' | 'q' | 'Q' | 'j' | 'J'
        );
        let mut order = None;
        let mut native = false;
        while let Some(modifier) = characters.get(index) {
            match modifier {
                '!' | '_' => {
                    if !sized_integer {
                        return Err(modifier_without_width(*modifier, position));
                    }
                    native = true;
                }
                '<' | '>' => {
                    if !sized_integer {
                        return Err(modifier_without_width(*modifier, position));
                    }
                    order = Some(if *modifier == '<' {
                        Order::Little
                    } else {
                        Order::Big
                    });
                }
                _ => break,
            }
            index += 1;
        }
        let count = if index < characters.len() && characters[index] == '*' {
            index += 1;
            Count::Rest
        } else if index < characters.len() && characters[index].is_ascii_digit() {
            let start = index;
            while index < characters.len() && characters[index].is_ascii_digit() {
                index += 1;
            }
            let digits: String = characters[start..index].iter().collect();
            match digits.parse::<usize>() {
                // A count past what a native signed word holds is refused
                // rather than wrapped, the way Ruby refuses one.
                Ok(number) if number > isize::MAX as usize => {
                    let message = "pack length too big".to_string();
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        &message,
                        position,
                    ));
                }
                Ok(number) => Count::Exactly(number),
                Err(_) => {
                    let message = "pack length too big".to_string();
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        &message,
                        position,
                    ));
                }
            }
        } else {
            Count::One
        };
        directives.push(Directive {
            code,
            count,
            order,
            native,
        });
    }
    Ok(directives)
}

/// How many bytes an address takes, which is what 'P' and 'p' write.
pub(crate) const POINTER_WIDTH: usize = std::mem::size_of::<u64>();

/// Every directive Ruby reads. A count or a width modifier is not one.
pub(crate) fn is_known_directive(code: char) -> bool {
    matches!(
        code,
        'a' | 'A'
            | 'Z'
            | 'b'
            | 'B'
            | 'h'
            | 'H'
            | 'c'
            | 'C'
            | 's'
            | 'S'
            | 'l'
            | 'L'
            | 'q'
            | 'Q'
            | 'j'
            | 'J'
            | 'i'
            | 'I'
            | 'n'
            | 'N'
            | 'v'
            | 'V'
            | 'U'
            | 'w'
            | 'm'
            | 'M'
            | 'u'
            | 'f'
            | 'F'
            | 'd'
            | 'D'
            | 'e'
            | 'E'
            | 'g'
            | 'G'
            | 'x'
            | 'X'
            | '@'
            | 'p'
            | 'P'
    )
}

/// The bytes UTF-8 spells a value with, carried past the range Unicode names
/// the way Ruby's `pack("U")` carries it: a lead byte saying how many follow,
/// then six bits of the value in each one.
pub(crate) fn utf8_style_bytes(value: u64) -> Vec<u8> {
    let widths: [(u64, usize, u8); 6] = [
        (0x80, 1, 0x00),
        (0x800, 2, 0xc0),
        (0x10000, 3, 0xe0),
        (0x200000, 4, 0xf0),
        (0x4000000, 5, 0xf8),
        (0x80000000, 6, 0xfc),
    ];
    for (limit, width, lead) in widths {
        if value >= limit {
            continue;
        }
        if width == 1 {
            return vec![value as u8];
        }
        let mut bytes = vec![0u8; width];
        for place in (1..width).rev() {
            bytes[place] = 0x80 | (value >> (6 * (width - 1 - place))) as u8 & 0x3f;
        }
        bytes[0] = lead | (value >> (6 * (width - 1))) as u8;
        return bytes;
    }
    vec![0xfd, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf]
}

/// Ruby's ArgumentError for a width modifier written after a directive that
/// has no platform width to name.
pub(crate) fn modifier_without_width(modifier: char, position: Position) -> MetorexError {
    let message = format!("'{}' allowed only after types sSiIlLqQjJ", modifier);
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
}

pub(crate) fn unknown_directive(
    code: char,
    format: &str,
    verb: &str,
    position: Position,
) -> MetorexError {
    let message = format!("unknown {verb} directive '{code}' in '{format}'");
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
}

/// How wide one item of an integer directive is, in bytes. `native` asks for
/// the platform's own width, which widens a long to eight bytes here and
/// leaves an int at four.
pub(crate) fn integer_width(code: char, native: bool) -> Option<usize> {
    match code {
        'c' | 'C' => Some(1),
        's' | 'S' | 'v' | 'n' => Some(2),
        'l' | 'L' if native => Some(std::mem::size_of::<std::os::raw::c_long>()),
        'l' | 'L' | 'V' | 'N' | 'i' | 'I' => Some(4),
        'q' | 'Q' | 'j' | 'J' => Some(8),
        _ => None,
    }
}

/// Whether a directive reads its bytes with the most significant one first.
pub(crate) fn is_big_endian(code: char) -> bool {
    matches!(code, 'n' | 'N')
}

/// Whether a directive answers a signed number.
pub(crate) fn is_signed(code: char) -> bool {
    matches!(code, 'c' | 's' | 'l' | 'q' | 'j' | 'i')
}
