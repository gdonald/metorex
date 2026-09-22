// The text a number writes, whole or fractional.

use super::*;

/// The text a whole-number directive writes.
pub(crate) fn written_whole(number: &BigInt, spec: char, directive: &Directive) -> String {
    let base = base_of(spec);
    let negative = matches!(number.sign(), Sign::Minus);
    // A negative number is written as a run of the base's largest digit
    // reaching back forever, unless a sign was asked for.
    let complement = negative && base != 10 && !directive.plus && !directive.space;
    let sign = if negative && !complement {
        "-"
    } else if directive.plus {
        "+"
    } else if directive.space {
        " "
    } else {
        ""
    };
    let mut digits = if complement {
        complement_digits(number, base, spec == 'X')
    } else {
        let size = if negative {
            -number.clone()
        } else {
            number.clone()
        };
        let written = size.to_str_radix(base);
        if spec == 'X' {
            written.to_uppercase()
        } else {
            written
        }
    };
    // A precision of zero writes nothing at all for a value of zero.
    if directive.precision == Some(0) && matches!(number.sign(), Sign::NoSign) {
        digits = String::new();
    }
    let mut prefix = String::new();
    if directive.alternate && !matches!(number.sign(), Sign::NoSign) {
        prefix.push_str(alternate_prefix(spec));
    }
    if let Some(precision) = directive.precision.filter(|held| *held > 0) {
        let filler = if complement {
            largest_digit(base, spec == 'X')
        } else {
            '0'
        };
        let counted = digits.chars().count() as i64;
        if counted < precision {
            let room: String =
                std::iter::repeat_n(filler, (precision - counted) as usize).collect();
            digits = insert_after_marker(&digits, &room, complement);
        }
    } else if directive.alternate && spec == 'o' && !complement && !digits.starts_with('0') {
        digits.insert(0, '0');
    }
    let body = format!("{}{}{}", sign, prefix, digits);
    // Zeros pad from the inside, after any sign and any prefix, and a run
    // reaching back forever pads with the base's largest digit instead.
    if directive.zero
        && !directive.left_align
        && directive.precision.is_none()
        && let Some(width) = directive.width
    {
        {
            let counted = body.chars().count() as i64;
            if counted < width {
                let filler = if complement {
                    largest_digit(base, spec == 'X')
                } else {
                    '0'
                };
                let room: String =
                    std::iter::repeat_n(filler, (width - counted) as usize).collect();
                let padded = if complement {
                    insert_after_marker(&digits, &room, true)
                } else {
                    format!("{}{}", room, digits)
                };
                return format!("{}{}{}", sign, prefix, padded);
            }
        }
    }
    pad(&body, directive, ' ')
}

/// The largest digit a base writes, which is what a run reaching back forever
/// is made of.
pub(crate) fn largest_digit(base: u32, upper: bool) -> char {
    let digit = std::char::from_digit(base - 1, base).expect("a digit of this base");
    if upper {
        digit.to_ascii_uppercase()
    } else {
        digit
    }
}

/// Put text straight after the `..d` that opens a run reaching back forever,
/// or at the front when there is none.
pub(crate) fn insert_after_marker(digits: &str, room: &str, complement: bool) -> String {
    if !complement {
        return format!("{}{}", room, digits);
    }
    let mut letters = digits.chars();
    let marker: String = letters.by_ref().take(3).collect();
    format!("{}{}{}", marker, room, letters.collect::<String>())
}

/// A negative number written the way Ruby writes one in a base of its own:
/// `..` and the base's largest digit stand for the run of them reaching back
/// forever, and the digits after it are what the run leaves.
pub(crate) fn complement_digits(number: &BigInt, base: u32, upper: bool) -> String {
    let largest = largest_digit(base, upper);
    // `~n` is the number whose digits, each taken from the largest, spell the
    // run's tail.
    let flipped = -(number + 1u32);
    let written = flipped.to_str_radix(base);
    let mut digits = String::new();
    for letter in written.chars() {
        let held = letter.to_digit(base).unwrap_or(0);
        let mut turned =
            std::char::from_digit(base - 1 - held, base).expect("a digit of this base");
        if upper {
            turned = turned.to_ascii_uppercase();
        }
        digits.push(turned);
    }
    let tail = digits.trim_start_matches(largest).to_string();
    format!("..{}{}", largest, tail)
}

/// The text a fractional directive writes.
pub(crate) fn written_fraction(number: f64, spec: char, directive: &Directive) -> String {
    if number.is_nan() || number.is_infinite() {
        let body = if number.is_nan() {
            "NaN".to_string()
        } else if number.is_sign_negative() {
            "-Inf".to_string()
        } else if directive.plus {
            "+Inf".to_string()
        } else if directive.space {
            " Inf".to_string()
        } else {
            "Inf".to_string()
        };
        return pad(&body, directive, ' ');
    }
    let negative = number.is_sign_negative();
    let sign = if negative {
        "-"
    } else if directive.plus {
        "+"
    } else if directive.space {
        " "
    } else {
        ""
    };
    let size = number.abs();
    let digits = match spec {
        'f' => decimal_notation(
            size,
            directive.precision.unwrap_or(6).max(0) as usize,
            directive.alternate,
        ),
        'e' | 'E' => {
            let written = exponent_form(
                size,
                directive.precision.unwrap_or(6).max(0) as usize,
                directive.alternate,
            );
            if spec == 'E' {
                written.to_uppercase()
            } else {
                written
            }
        }
        'g' | 'G' => {
            let written = shortest_form(size, directive.precision, directive.alternate);
            if spec == 'G' {
                written.to_uppercase()
            } else {
                written
            }
        }
        _ => {
            let written = hex_form(size, directive.precision, directive.alternate);
            if spec == 'A' {
                written.to_uppercase()
            } else {
                written
            }
        }
    };
    let body = format!("{}{}", sign, digits);
    if directive.zero
        && !directive.left_align
        && let Some(width) = directive.width
    {
        let counted = body.chars().count() as i64;
        if counted < width {
            let room: String = std::iter::repeat_n('0', (width - counted) as usize).collect();
            // A hexadecimal fraction pads after the `0x` that opens it.
            let opens = if digits.len() > 2 && digits[..2].eq_ignore_ascii_case("0x") {
                2
            } else {
                0
            };
            return format!("{}{}{}{}", sign, &digits[..opens], room, &digits[opens..]);
        }
    }
    pad(&body, directive, ' ')
}

/// A number written out in full, with as many places after the point as asked
/// for. A `#` flag keeps the point even where no places follow.
pub(crate) fn decimal_notation(size: f64, places: usize, alternate: bool) -> String {
    let written = format!("{:.*}", places, size);
    if places == 0 && alternate {
        return format!("{}.", written);
    }
    written
}

/// A number written as one digit, a point, and a power of ten.
pub(crate) fn exponent_form(size: f64, places: usize, alternate: bool) -> String {
    let mut power = if size == 0.0 {
        0i32
    } else {
        size.abs().log10().floor() as i32
    };
    let mut lead = if size == 0.0 {
        0.0
    } else {
        size / 10f64.powi(power)
    };
    // Rounding the lead can carry it up to ten, which belongs to the next
    // power instead.
    let rounded = format!("{:.*}", places, lead);
    if rounded.starts_with("10") {
        power += 1;
        lead = size / 10f64.powi(power);
    }
    let mut written = format!("{:.*}", places, lead);
    if places == 0 && alternate {
        written.push('.');
    }
    format!(
        "{}e{}{:02}",
        written,
        if power < 0 { '-' } else { '+' },
        power.abs()
    )
}

/// A number written in whichever of the two forms is shorter, which is what
/// `%g` asks for. Trailing zeros go unless a `#` flag keeps them.
pub(crate) fn shortest_form(size: f64, precision: Option<i64>, alternate: bool) -> String {
    let places = precision.unwrap_or(6).max(1) as usize;
    let power = if size == 0.0 {
        0i32
    } else {
        // The power the number would be written with, after rounding it to
        // the digits asked for.
        let first = size.abs().log10().floor() as i32;
        let rounded = format!("{:.*e}", places - 1, size);
        rounded
            .rsplit('e')
            .next()
            .and_then(|held| held.parse::<i32>().ok())
            .unwrap_or(first)
    };
    if power < -4 || power >= places as i32 {
        let written = exponent_form(size, places - 1, alternate);
        if alternate {
            return written;
        }
        let (digits, power) = written.split_once('e').unwrap_or((written.as_str(), ""));
        return format!("{}e{}", trimmed_zeros(digits), power);
    }
    let room = (places as i32 - 1 - power).max(0) as usize;
    let written = format!("{:.*}", room, size);
    if alternate {
        if room == 0 {
            return format!("{}.", written);
        }
        return written;
    }
    trimmed_zeros(&written)
}

/// A written number with the zeros trailing its fraction taken off, and the
/// point with them where nothing is left after it.
pub(crate) fn trimmed_zeros(written: &str) -> String {
    if !written.contains('.') {
        return written.to_string();
    }
    let trimmed = written.trim_end_matches('0');
    trimmed.trim_end_matches('.').to_string()
}

/// A number written as a hexadecimal fraction and a power of two, which is
/// what `%a` asks for.
pub(crate) fn hex_form(size: f64, precision: Option<i64>, alternate: bool) -> String {
    if size == 0.0 {
        return match precision {
            Some(places) if places > 0 => {
                format!("0x0.{}p+0", "0".repeat(places as usize))
            }
            _ if alternate => "0x0.p+0".to_string(),
            _ => "0x0p+0".to_string(),
        };
    }
    let bits = size.to_bits();
    let raw_power = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = bits & 0x000f_ffff_ffff_ffff;
    let (lead, mantissa, power) = if raw_power == 0 {
        (0u64, mantissa, -1022)
    } else {
        (1u64, mantissa, raw_power - 1023)
    };
    // The mantissa holds thirteen hexadecimal digits.
    let mut digits: String = (0..13)
        .map(|place| {
            let shift = 48 - place * 4;
            let held = ((mantissa >> shift) & 0xf) as u32;
            std::char::from_digit(held, 16).expect("a hexadecimal digit")
        })
        .collect();
    if let Some(places) = precision {
        digits = rounded_hex(&digits, places.max(0) as usize);
    } else {
        digits = digits.trim_end_matches('0').to_string();
    }
    let mut written = format!("0x{}", lead);
    if !digits.is_empty() {
        written.push('.');
        written.push_str(&digits);
    } else if alternate {
        written.push('.');
    }
    format!(
        "{}p{}{}",
        written,
        if power < 0 { '-' } else { '+' },
        power.abs()
    )
}

/// Hexadecimal digits cut to the count asked for, with the last one rounded
/// to the nearest.
pub(crate) fn rounded_hex(digits: &str, places: usize) -> String {
    if places >= digits.len() {
        let mut written = digits.to_string();
        while written.len() < places {
            written.push('0');
        }
        return written;
    }
    let mut kept: Vec<u32> = digits[..places]
        .chars()
        .map(|held| held.to_digit(16).unwrap_or(0))
        .collect();
    let next = digits[places..]
        .chars()
        .next()
        .and_then(|held| held.to_digit(16))
        .unwrap_or(0);
    if next >= 8 {
        let mut at = kept.len();
        while at > 0 {
            at -= 1;
            if kept[at] == 15 {
                kept[at] = 0;
            } else {
                kept[at] += 1;
                break;
            }
        }
    }
    kept.into_iter()
        .map(|held| std::char::from_digit(held, 16).expect("a hexadecimal digit"))
        .collect()
}
