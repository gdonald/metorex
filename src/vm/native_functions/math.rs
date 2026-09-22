// The Math module, and the Float each of its functions reads.

use super::*;

impl VirtualMachine {
    /// The Math module's functions. Each takes its arguments as Floats, which
    /// is what `Float()` reads them as, and answers a Float.
    pub(crate) fn apply_math_function(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(Object::Symbol(name) | Object::String(name)) = arguments.first() else {
            return Err(MetorexError::runtime_error(
                "__math_function__ expects the function name first",
                crate::vm::utils::position_to_location(position),
            ));
        };
        let name = name.as_str().to_string();
        let mut values = Vec::new();
        for (index, argument) in arguments[1..].iter().enumerate() {
            // `ldexp` scales by a whole number of powers of two, so its second
            // argument is read the way `Integer()` reads one.
            if name == "ldexp" && index == 1 {
                let exponent = match argument {
                    Object::Float(number) => *number,
                    other => self
                        .integer_class_method("try_convert", other, position)
                        .ok()
                        .and_then(|converted| match converted {
                            Object::Int(number) => Some(number as f64),
                            _ => None,
                        })
                        .ok_or_else(|| {
                            let message = format!(
                                "can't convert {} into Integer",
                                self.builtins().class_of(other).name()
                            );
                            MetorexError::UncaughtException {
                                exception: Object::exception("TypeError", message.clone()),
                                location: crate::vm::utils::position_to_location(position),
                                message,
                            }
                        })?,
                };
                values.push(exponent);
                continue;
            }
            values.push(self.math_argument(argument, position)?);
        }
        let first = values.first().copied().unwrap_or_default();
        // NaN names no point in any domain, and every function answers it
        // rather than refusing it.
        if first.is_nan() && !matches!(name.as_str(), "frexp" | "lgamma" | "ldexp") {
            return Ok(Object::Float(first));
        }
        // A function with no answer at this point names the argument as out of
        // its domain, which is what Ruby reports for `Math.sqrt(-1)`.
        let out_of_domain = |name: &str| {
            // Ruby quotes the function's name here, except in `log1p`, which
            // reports it bare.
            let named = match name {
                "log1p" => name.to_string(),
                _ => format!("\"{}\"", name),
            };
            crate::vm::errors::simple_exception(
                "Math::DomainError",
                &format!("Numerical argument is out of domain - {}", named),
                position,
            )
        };
        let answer = match name.as_str() {
            "sqrt" => {
                if first < 0.0 {
                    return Err(out_of_domain("sqrt"));
                }
                first.sqrt()
            }
            "cbrt" => first.cbrt(),
            "sin" => first.sin(),
            "cos" => first.cos(),
            "tan" => first.tan(),
            "asin" => {
                if !(-1.0..=1.0).contains(&first) {
                    return Err(out_of_domain("asin"));
                }
                first.asin()
            }
            "acos" => {
                if !(-1.0..=1.0).contains(&first) {
                    return Err(out_of_domain("acos"));
                }
                first.acos()
            }
            "atan" => first.atan(),
            "atan2" => first.atan2(values.get(1).copied().unwrap_or_default()),
            "sinh" => first.sinh(),
            "cosh" => first.cosh(),
            "tanh" => first.tanh(),
            "asinh" => first.asinh(),
            "acosh" => {
                if first < 1.0 {
                    return Err(out_of_domain("acosh"));
                }
                first.acosh()
            }
            "atanh" => {
                if first.abs() > 1.0 {
                    return Err(out_of_domain("atanh"));
                }
                first.atanh()
            }
            "exp" => first.exp(),
            "log" => {
                if first < 0.0 {
                    return Err(out_of_domain("log"));
                }
                let logarithm = match arguments.get(1).and_then(exact_log2) {
                    Some(exact) => exact,
                    None => first.log2(),
                };
                match values.get(1) {
                    Some(base) => logarithm / base.log2(),
                    None => logarithm * std::f64::consts::LN_2,
                }
            }
            "log2" => {
                if first < 0.0 {
                    return Err(out_of_domain("log2"));
                }
                match arguments.get(1).and_then(exact_log2) {
                    Some(exact) => exact,
                    None => first.log2(),
                }
            }
            "log10" => {
                if first < 0.0 {
                    return Err(out_of_domain("log10"));
                }
                match arguments.get(1).and_then(exact_log2) {
                    Some(exact) => exact / (10f64).log2(),
                    None => first.log10(),
                }
            }
            "log1p" => {
                if first < -1.0 {
                    return Err(out_of_domain("log1p"));
                }
                first.ln_1p()
            }
            "expm1" => first.exp_m1(),
            "hypot" => first.hypot(values.get(1).copied().unwrap_or_default()),
            "erf" => error_function(first),
            "erfc" => 1.0 - error_function(first),
            "gamma" => {
                if first == 0.0 {
                    return Ok(Object::Float(f64::INFINITY * first.signum()));
                }
                if first.is_infinite() {
                    if first.is_sign_negative() {
                        return Err(out_of_domain("gamma"));
                    }
                    return Ok(Object::Float(f64::INFINITY));
                }
                if first < 0.0 && first.fract() == 0.0 {
                    return Err(out_of_domain("gamma"));
                }
                // A whole number's gamma is a factorial, and multiplying the
                // factors keeps every digit a double can hold, which the
                // series approximation does not.
                if first > 0.0 && first.fract() == 0.0 && first <= LARGEST_EXACT_FACTORIAL {
                    let mut product = 1.0;
                    let mut factor = 2.0;
                    while factor < first {
                        product *= factor;
                        factor += 1.0;
                    }
                    return Ok(Object::Float(product));
                }
                gamma_function(first)
            }
            // `ldexp` scales by a power of two, and `frexp` splits a number
            // into the pair that does so.
            "ldexp" => {
                let exponent = values.get(1).copied().unwrap_or_default();
                if exponent.is_nan() {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "float NaN out of range of integer",
                        position,
                    ));
                }
                return Ok(Object::Float(
                    crate::vm::native_methods::scale_by_power_of_two(first, exponent as i64),
                ));
            }
            "frexp" => {
                let (fraction, exponent) = split_float(first);
                return Ok(Object::array(vec![
                    Object::Float(fraction),
                    Object::Int(exponent),
                ]));
            }
            // `lgamma` answers the log of the gamma function's magnitude with
            // the sign it dropped.
            "lgamma" => {
                if first.is_infinite() && first < 0.0 {
                    return Err(out_of_domain("lgamma"));
                }
                // The gamma function grows without bound, so its logarithm
                // does too.
                if first.is_infinite() {
                    return Ok(Object::array(vec![
                        Object::Float(f64::INFINITY),
                        Object::Int(1),
                    ]));
                }
                // It has a pole at every whole number at or below zero, so
                // its magnitude there has no logarithm short of infinity. The
                // sign alternates from one pole to the next, and negative
                // zero approaches the pole at zero from the other side.
                if first <= 0.0 && first.fract() == 0.0 {
                    let sign = if first == 0.0 {
                        if first.is_sign_negative() { -1 } else { 1 }
                    } else if (first / 2.0).fract() != 0.0 {
                        1
                    } else {
                        -1
                    };
                    return Ok(Object::array(vec![
                        Object::Float(f64::INFINITY),
                        Object::Int(sign),
                    ]));
                }
                let value = gamma_function(first);
                return Ok(Object::array(vec![
                    Object::Float(value.abs().ln()),
                    Object::Int(if value < 0.0 { -1 } else { 1 }),
                ]));
            }
            other => {
                return Err(MetorexError::runtime_error(
                    format!("unknown math function: {}", other),
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };
        Ok(Object::Float(answer))
    }

    /// A Math argument as a Float. Ruby reads it the way `Float()` does, so a
    /// String or an object that is not a number is refused.
    fn math_argument(&mut self, value: &Object, position: Position) -> Result<f64, MetorexError> {
        match value {
            Object::Int(number) => Ok(*number as f64),
            Object::BigInt(number) => Ok(crate::vm::operators::big_to_float(number)),
            Object::Float(number) => Ok(*number),
            other => {
                let message = format!(
                    "can't convert {} into Float",
                    match other {
                        Object::Nil => "nil".to_string(),
                        _ => self.builtins().class_of(other).name().to_string(),
                    }
                );
                let refuse = MetorexError::UncaughtException {
                    exception: Object::exception("TypeError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                };
                // Only a number answers `to_f` here: a String has one too, and
                // Ruby refuses it all the same.
                let numeric = matches!(other, Object::Instance(instance)
                    if crate::vm::method_invocation::descends_from(&instance.borrow().class, "Numeric"));
                if !numeric {
                    return Err(refuse);
                }
                match self.send_to_object(other.clone(), "to_f", vec![], position)? {
                    Object::Float(number) => Ok(number),
                    Object::Int(number) => Ok(number as f64),
                    _ => Err(refuse),
                }
            }
        }
    }
}

/// The error function, by the series that converges quickly for a small
/// argument and the continued-fraction form beyond it.
pub(crate) fn error_function(value: f64) -> f64 {
    if value.is_nan() {
        return value;
    }
    let magnitude = value.abs();
    if magnitude > 6.0 {
        return value.signum();
    }
    // Abramowitz and Stegun 7.1.26, which is accurate to about 1e-7.
    let t = 1.0 / (1.0 + 0.3275911 * magnitude);
    let polynomial = t
        * (0.254829592
            + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    let answer = 1.0 - polynomial * (-magnitude * magnitude).exp();
    answer * value.signum()
}

/// The largest whole number whose factorial a double still holds. Above it
/// the product overflows to infinity, which is what Ruby answers there too.
pub(crate) const LARGEST_EXACT_FACTORIAL: f64 = 171.0;

/// The gamma function, by the Lanczos approximation.
pub(crate) fn gamma_function(value: f64) -> f64 {
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_81,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if value < 0.5 {
        // The reflection formula carries a negative argument to a positive one.
        return std::f64::consts::PI
            / ((std::f64::consts::PI * value).sin() * gamma_function(1.0 - value));
    }
    let value = value - 1.0;
    let mut series = COEFFICIENTS[0];
    for (index, coefficient) in COEFFICIENTS.iter().enumerate().skip(1) {
        series += coefficient / (value + index as f64);
    }
    let t = value + 7.5;
    (2.0 * std::f64::consts::PI).sqrt() * t.powf(value + 0.5) * (-t).exp() * series
}

/// A float split into the fraction and the power of two that rebuild it, which
/// is the pair `frexp` answers.
pub(crate) fn split_float(value: f64) -> (f64, i64) {
    if value == 0.0 || !value.is_finite() {
        return (value, 0);
    }
    let exponent = value.abs().log2().floor() as i64 + 1;
    let fraction = value / (2f64).powi(exponent as i32);
    (fraction, exponent)
}

/// The base-2 logarithm of an exact integer, which keeps the digits a Float
/// cannot hold: the bit length gives the whole part and the leading bits the
/// fraction, so `Math.log2(2 ** 10001)` is 10001.0 rather than an infinity.
pub(crate) fn exact_log2(value: &Object) -> Option<f64> {
    let value = match value {
        Object::BigInt(number) => (*number).clone(),
        _ => return None,
    };
    if *value <= num_bigint::BigInt::from(0) {
        return None;
    }
    let bits = value.bits() as i64;
    // Keep the leading bits as a Float and let the rest count as the exponent.
    let kept = 64.min(bits);
    let leading = crate::vm::operators::big_to_float(&(&*value >> (bits - kept) as u32));
    Some(leading.log2() + (bits - kept) as f64)
}

/// The ArgumentError Ruby raises for a range whose ends cannot bound a draw.
pub(crate) fn bad_range_value(position: Position) -> MetorexError {
    let message = "bad value for range".to_string();
    MetorexError::UncaughtException {
        exception: Object::exception("ArgumentError", message.clone()),
        location: crate::vm::utils::position_to_location(position),
        message,
    }
}
