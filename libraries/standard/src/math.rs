use platipus_reactive::state::Value;

/// Absolute value, preserving numeric kind.
pub fn abs(value: &Value) -> Option<Value> {
    match value {
        Value::Int(n) => Some(Value::int(n.abs())),
        Value::Float(f) => Some(Value::float(f.abs())),
        _ => None,
    }
}

pub fn min(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Some(Value::int(*x.min(y))),
        (Value::Float(x), Value::Float(y)) => Some(Value::float(x.min(*y))),
        _ => None,
    }
}

pub fn max(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Some(Value::int(*x.max(y))),
        (Value::Float(x), Value::Float(y)) => Some(Value::float(x.max(*y))),
        _ => None,
    }
}

/// Clamps `value` into `[lo, hi]`; non-numeric inputs are not clampable.
pub fn clamp(value: &Value, lo: &Value, hi: &Value) -> Option<Value> {
    let floor = max(value, lo)?;
    min(&floor, hi)
}

pub fn floor(value: &Value) -> Option<Value> {
    value.as_float().map(f64::floor).map(|f| Value::float(f))
}

pub fn ceil(value: &Value) -> Option<Value> {
    value.as_float().map(f64::ceil).map(|f| Value::float(f))
}

pub fn round(value: &Value) -> Option<Value> {
    value.as_float().map(f64::round).map(Value::float)
}

pub fn sqrt(value: &Value) -> Option<Value> {
    value.as_float().map(f64::sqrt).filter(|f| !f.is_nan()).map(Value::float)
}

pub fn pow(base: &Value, exponent: &Value) -> Option<Value> {
    match (base, exponent) {
        (Value::Int(b), Value::Int(e)) if *e >= 0 => Some(Value::int(b.pow(*e as u32))),
        (Value::Float(b), Value::Float(e)) => Some(Value::float(b.powf(*e))),
        _ => None,
    }
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
    fn round_floor_ceil_match_std() {
        assert_eq!(round(&Value::float(2.5)), Some(Value::float(3.0)));
        assert_eq!(floor(&Value::float(2.7)), Some(Value::float(2.0)));
        assert_eq!(ceil(&Value::float(2.1)), Some(Value::float(3.0)));
    }

    #[test]
    fn pow_handles_int_and_float() {
        assert_eq!(pow(&Value::int(2), &Value::int(4)), Some(Value::int(16)));
        assert_eq!(pow(&Value::float(2.0), &Value::float(0.5)), Some(Value::float(2.0_f64.sqrt())));
        assert_eq!(pow(&Value::int(2), &Value::int(-1)), None);
    }
}
