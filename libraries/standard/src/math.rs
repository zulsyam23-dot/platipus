use platipus_reactive::state::Value;

/// Absolute value, preserving numeric kind.
pub fn abs(value: &Value) -> Option<Value> {
    match value {
        Value::Int(n) => n.checked_abs().map(Value::int),
        Value::Float(f) => Some(Value::float(f.abs())),
        _ => None,
    }
}

pub fn min(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Some(Value::int(*x.min(y))),
        (Value::Float(x), Value::Float(y)) => {
            if x.is_nan() || y.is_nan() {
                return Some(Value::float(f64::NAN));
            }
            Some(Value::float(x.min(*y)))
        }
        _ => None,
    }
}

pub fn max(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Some(Value::int(*x.max(y))),
        (Value::Float(x), Value::Float(y)) => {
            if x.is_nan() || y.is_nan() {
                return Some(Value::float(f64::NAN));
            }
            Some(Value::float(x.max(*y)))
        }
        _ => None,
    }
}

/// Clamps `value` into `[lo, hi]`; a non-numeric input or an inverted range
/// (`lo > hi`) is rejected instead of silently clamping to a boundary.
pub fn clamp(value: &Value, lo: &Value, hi: &Value) -> Option<Value> {
    if let (Value::Int(v), Value::Int(l), Value::Int(h)) = (value, lo, hi) {
        if l > h {
            return None;
        }
        return Some(Value::int((*v).max(*l).min(*h)));
    }
    let (v, l, h) = (value.as_float()?, lo.as_float()?, hi.as_float()?);
    if l > h {
        return None;
    }
    if v.is_nan() || l.is_nan() || h.is_nan() {
        return Some(Value::float(f64::NAN));
    }
    Some(Value::float(v.max(l).min(h)))
}

/// Truncating integer division, for both integers only.
///
/// `/` is true division: it hands back a float whenever the result is not a
/// whole number, so `1 / 2` is `0.5`. `//` cannot spell this in Platipus because
/// it already introduces a line comment, which leaves a call as the only
/// unambiguous way to ask for the integer half. Division by zero has no integer
/// answer, so it reports nothing rather than producing `NaN` or an infinity.
///
/// The operands are matched as integers rather than read through `as_int`,
/// which would quietly accept a float. The web runtime already refuses one, and
/// a helper that answers differently depending on which backend ran it would be
/// worse than one that refuses everywhere.
pub fn idiv(a: &Value, b: &Value) -> Option<Value> {
    let (Value::Int(left), Value::Int(right)) = (a, b) else {
        return None;
    };
    if *right == 0 {
        return None;
    }
    Some(Value::int(left.checked_div(*right)?))
}

pub fn floor(value: &Value) -> Option<Value> {
    value.as_float().map(f64::floor).map(Value::float)
}

pub fn ceil(value: &Value) -> Option<Value> {
    value.as_float().map(f64::ceil).map(Value::float)
}

pub fn round(value: &Value) -> Option<Value> {
    value.as_float().map(f64::round).map(Value::float)
}

pub fn sqrt(value: &Value) -> Option<Value> {
    value
        .as_float()
        .map(f64::sqrt)
        .filter(|f| !f.is_nan())
        .map(Value::float)
}

pub fn pow(base: &Value, exponent: &Value) -> Option<Value> {
    match (base, exponent) {
        (Value::Int(b), Value::Int(e)) if *e >= 0 => {
            // checked_pow saturates detection: fall back to None on overflow
            // instead of panicking in debug builds.
            b.checked_pow(*e as u32).map(Value::int)
        }
        (Value::Float(b), Value::Float(e)) => {
            let result = b.powf(*e);
            result.is_finite().then_some(Value::float(result))
        }
        _ => None,
    }
}

/// Variadic minimum: every argument must be numeric and share a kind.
pub fn min_all(arguments: &[Value]) -> Option<Value> {
    let mut iter = arguments.iter();
    let first = iter.next()?.clone();
    match first {
        Value::Int(_) => {
            let mut acc = first.as_int()?;
            for argument in iter {
                acc = acc.min(argument.as_int()?);
            }
            Some(Value::int(acc))
        }
        Value::Float(_) => {
            let mut acc = first.as_float()?;
            for argument in iter {
                let value = argument.as_float()?;
                if acc.is_nan() || value.is_nan() {
                    return Some(Value::float(f64::NAN));
                }
                acc = acc.min(value);
            }
            Some(Value::float(acc))
        }
        _ => None,
    }
}

/// Variadic maximum: every argument must be numeric and share a kind.
pub fn max_all(arguments: &[Value]) -> Option<Value> {
    let mut iter = arguments.iter();
    let first = iter.next()?.clone();
    match first {
        Value::Int(_) => {
            let mut acc = first.as_int()?;
            for argument in iter {
                acc = acc.max(argument.as_int()?);
            }
            Some(Value::int(acc))
        }
        Value::Float(_) => {
            let mut acc = first.as_float()?;
            for argument in iter {
                let value = argument.as_float()?;
                if acc.is_nan() || value.is_nan() {
                    return Some(Value::float(f64::NAN));
                }
                acc = acc.max(value);
            }
            Some(Value::float(acc))
        }
        _ => None,
    }
}

pub fn trunc(value: &Value) -> Option<Value> {
    value.as_float().map(f64::trunc).map(Value::float)
}

pub fn sign(value: &Value) -> Option<Value> {
    match value {
        Value::Int(n) => Some(Value::int(n.signum())),
        Value::Float(f) => Some(Value::float(f.signum())),
        _ => None,
    }
}

pub fn log(value: &Value) -> Option<Value> {
    value.as_float().map(f64::ln).filter(|f| !f.is_nan()).map(Value::float)
}

pub fn log2(value: &Value) -> Option<Value> {
    value.as_float().map(f64::log2).filter(|f| !f.is_nan()).map(Value::float)
}

pub fn log10(value: &Value) -> Option<Value> {
    value.as_float().map(f64::log10).filter(|f| !f.is_nan()).map(Value::float)
}

pub fn exp(value: &Value) -> Option<Value> {
    value.as_float().map(f64::exp).filter(|f| f.is_finite()).map(Value::float)
}

pub fn sin(value: &Value) -> Option<Value> {
    value.as_float().map(f64::sin).map(Value::float)
}

pub fn cos(value: &Value) -> Option<Value> {
    value.as_float().map(f64::cos).map(Value::float)
}

pub fn tan(value: &Value) -> Option<Value> {
    value.as_float().map(f64::tan).map(Value::float)
}

pub fn atan2(y: &Value, x: &Value) -> Option<Value> {
    Some(Value::float(y.as_float()?.atan2(x.as_float()?)))
}

pub fn hypot(a: &Value, b: &Value) -> Option<Value> {
    Some(Value::float(a.as_float()?.hypot(b.as_float()?)))
}

pub fn cbrt(value: &Value) -> Option<Value> {
    value.as_float().map(f64::cbrt).map(Value::float)
}

/// 32-bit wrapping multiply, mirroring `Math.imul`.
pub fn imul(a: &Value, b: &Value) -> Option<Value> {
    let x = a.as_int()?;
    let y = b.as_int()?;
    Some(Value::int((x as i32).wrapping_mul(y as i32) as i64))
}

/// Zero-extend to an unsigned 32-bit integer, like `x >>> 0`.
pub fn u32(value: &Value) -> Option<Value> {
    let x = value.as_int()?;
    Some(Value::int(x as u32 as i64))
}

pub fn pi() -> Option<Value> {
    Some(Value::float(std::f64::consts::PI))
}

pub fn e() -> Option<Value> {
    Some(Value::float(std::f64::consts::E))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abs_works_for_both_numeric_kinds() {
        assert_eq!(abs(&Value::int(-3)), Some(Value::int(3)));
        assert_eq!(abs(&Value::float(-1.5)), Some(Value::float(1.5)));
        assert_eq!(abs(&Value::text("x")), None);
    }

    #[test]
    fn abs_rejects_the_unrepresentable_minimum_integer() {
        assert_eq!(abs(&Value::int(i64::MIN)), None);
    }

    #[test]
    fn clamp_bounds_the_value() {
        assert_eq!(
            clamp(&Value::int(5), &Value::int(0), &Value::int(3)),
            Some(Value::int(3))
        );
        assert_eq!(
            clamp(&Value::int(-5), &Value::int(0), &Value::int(3)),
            Some(Value::int(0))
        );
    }

    #[test]
    fn clamp_propagates_nan_and_rejects_inverted_bounds() {
        assert!(
            clamp(
                &Value::float(f64::NAN),
                &Value::float(0.0),
                &Value::float(1.0)
            )
            .unwrap()
            .as_float()
            .unwrap()
            .is_nan()
        );
        assert_eq!(clamp(&Value::int(1), &Value::int(3), &Value::int(0)), None);
    }

    #[test]
    fn round_floor_ceil_match_std() {
        assert_eq!(round(&Value::float(2.5)), Some(Value::float(3.0)));
        assert_eq!(floor(&Value::float(2.7)), Some(Value::float(2.0)));
        assert_eq!(ceil(&Value::float(2.1)), Some(Value::float(3.0)));
    }

    #[test]
    fn pow_handles_int_and_float() {
        assert_eq!(pow(&Value::int(2), &Value::int(4)), Some(Value::int(16)));
        assert_eq!(
            pow(&Value::float(2.0), &Value::float(0.5)),
            Some(Value::float(2.0_f64.sqrt()))
        );
        assert_eq!(pow(&Value::int(2), &Value::int(-1)), None);
    }

    #[test]
    fn integer_division_truncates_towards_zero() {
        assert_eq!(idiv(&Value::int(7), &Value::int(2)), Some(Value::int(3)));
        assert_eq!(idiv(&Value::int(-7), &Value::int(2)), Some(Value::int(-3)));
        assert_eq!(idiv(&Value::int(8), &Value::int(2)), Some(Value::int(4)));
        assert_eq!(idiv(&Value::int(1), &Value::int(2)), Some(Value::int(0)));
    }

    #[test]
    fn integer_division_refuses_a_zero_divisor() {
        // `/` would answer `Infinity` here. There is no integer to report, so
        // the call produces nothing rather than a number the caller cannot use.
        assert_eq!(idiv(&Value::int(1), &Value::int(0)), None);
    }

    #[test]
    fn integer_division_refuses_a_float_operand() {
        assert_eq!(idiv(&Value::int(7), &Value::float(2.0)), None);
        assert_eq!(idiv(&Value::float(7.0), &Value::int(2)), None);
    }
}
