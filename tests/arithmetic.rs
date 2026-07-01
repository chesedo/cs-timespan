use cs_timespan::{FloatError, TimeSpan, TimeSpanOverflow};

fn ts(ticks: i64) -> TimeSpan {
    TimeSpan::from_ticks(ticks)
}

const SEC: i64 = TimeSpan::TICKS_PER_SECOND;
const MIN: i64 = TimeSpan::TICKS_PER_MINUTE;
const HR: i64 = TimeSpan::TICKS_PER_HOUR;
const DAY: i64 = TimeSpan::TICKS_PER_DAY;

// ── Add / Sub ─────────────────────────────────────────────────────────────────

#[test]
fn add_positive() {
    assert_eq!(ts(3 * HR) + ts(2 * HR), ts(5 * HR));
}

#[test]
fn add_negative() {
    assert_eq!(ts(3 * HR) + ts(-5 * HR), ts(-2 * HR));
}

#[test]
fn add_zero() {
    assert_eq!(ts(42 * SEC) + TimeSpan::ZERO, ts(42 * SEC));
}

#[test]
fn sub_positive() {
    assert_eq!(ts(5 * DAY) - ts(2 * DAY), ts(3 * DAY));
}

#[test]
fn sub_produces_negative() {
    assert_eq!(ts(1 * HR) - ts(2 * HR), ts(-1 * HR));
}

#[test]
fn sub_zero() {
    assert_eq!(ts(7 * MIN) - TimeSpan::ZERO, ts(7 * MIN));
}

// ── Overflow ──────────────────────────────────────────────────────────────────
// TimeSpan.cs#L893 (operator+), #L877 (operator-), #L868 (unary operator-):
// C# throws OverflowException on tick overflow; the operator impls here panic
// consistently (not relying on debug-only overflow checks), and checked_*
// gives callers a non-panicking alternative.

#[test]
#[should_panic(expected = "TimeSpan add overflowed")]
fn add_overflow_panics() {
    let _ = TimeSpan::MAX_VALUE + ts(1);
}

#[test]
#[should_panic(expected = "TimeSpan sub overflowed")]
fn sub_overflow_panics() {
    let _ = TimeSpan::MIN_VALUE - ts(1);
}

#[test]
#[should_panic(expected = "TimeSpan neg overflowed")]
fn neg_min_value_panics() {
    let _ = -TimeSpan::MIN_VALUE;
}

#[test]
fn checked_add_ok() {
    assert_eq!(ts(3 * HR).checked_add(ts(2 * HR)), Ok(ts(5 * HR)));
}

#[test]
fn checked_add_overflow() {
    assert_eq!(
        TimeSpan::MAX_VALUE.checked_add(ts(1)),
        Err(TimeSpanOverflow)
    );
}

#[test]
fn checked_sub_ok() {
    assert_eq!(ts(5 * DAY).checked_sub(ts(2 * DAY)), Ok(ts(3 * DAY)));
}

#[test]
fn checked_sub_overflow() {
    assert_eq!(
        TimeSpan::MIN_VALUE.checked_sub(ts(1)),
        Err(TimeSpanOverflow)
    );
}

#[test]
fn checked_neg_ok() {
    assert_eq!(ts(3 * SEC).checked_neg(), Ok(ts(-3 * SEC)));
}

#[test]
fn checked_neg_min_value_overflow() {
    assert_eq!(TimeSpan::MIN_VALUE.checked_neg(), Err(TimeSpanOverflow));
}

// ── Neg ───────────────────────────────────────────────────────────────────────

#[test]
fn neg_positive() {
    assert_eq!(-ts(3 * SEC), ts(-3 * SEC));
}

#[test]
fn neg_negative() {
    assert_eq!(-ts(-3 * SEC), ts(3 * SEC));
}

#[test]
fn neg_zero() {
    assert_eq!(-TimeSpan::ZERO, TimeSpan::ZERO);
}

// ── Duration ──────────────────────────────────────────────────────────────────
// TimeSpan.cs#L416-423

#[test]
fn duration_positive() {
    assert_eq!(ts(5 * SEC).duration(), Ok(ts(5 * SEC)));
}

#[test]
fn duration_negative() {
    assert_eq!(ts(-5 * SEC).duration(), Ok(ts(5 * SEC)));
}

#[test]
fn duration_zero() {
    assert_eq!(TimeSpan::ZERO.duration(), Ok(TimeSpan::ZERO));
}

#[test]
fn duration_min_value_overflows() {
    assert_eq!(TimeSpan::MIN_VALUE.duration(), Err(TimeSpanOverflow));
}

// ── AddAssign / SubAssign ─────────────────────────────────────────────────────

#[test]
fn add_assign() {
    let mut a = ts(1 * HR);
    a += ts(30 * MIN);
    assert_eq!(a, ts(HR + 30 * MIN));
}

#[test]
fn sub_assign() {
    let mut a = ts(2 * DAY);
    a -= ts(6 * HR);
    assert_eq!(a, ts(2 * DAY - 6 * HR));
}

// ── Mul ───────────────────────────────────────────────────────────────────────

#[test]
fn mul_timespan_by_scalar() {
    assert_eq!(ts(3 * SEC) * 4, ts(12 * SEC));
}

#[test]
fn mul_scalar_by_timespan() {
    assert_eq!(4 * ts(3 * SEC), ts(12 * SEC));
}

#[test]
fn mul_by_zero() {
    assert_eq!(ts(5 * HR) * 0, TimeSpan::ZERO);
}

#[test]
fn mul_by_negative() {
    assert_eq!(ts(2 * MIN) * -3, ts(-6 * MIN));
}

#[test]
fn mul_assign() {
    let mut a = ts(5 * SEC);
    a *= 3;
    assert_eq!(a, ts(15 * SEC));
}

// ── Div ───────────────────────────────────────────────────────────────────────

#[test]
fn div_by_scalar() {
    assert_eq!(ts(12 * SEC) / 4, ts(3 * SEC));
}

#[test]
fn div_by_scalar_truncates() {
    // 7 ticks / 2 = 3 ticks (integer truncation toward zero)
    assert_eq!(ts(7) / 2, ts(3));
}

#[test]
fn div_assign() {
    let mut a = ts(12 * MIN);
    a /= 4;
    assert_eq!(a, ts(3 * MIN));
}

#[test]
fn div_timespan_by_timespan_ratio() {
    let ratio = ts(3 * HR) / ts(1 * HR);
    assert!((ratio - 3.0_f64).abs() < 1e-10);
}

#[test]
fn div_timespan_by_timespan_fractional() {
    let ratio = ts(1 * HR) / ts(2 * HR);
    assert!((ratio - 0.5_f64).abs() < 1e-10);
}

// ── Mul<f64> / Div<f64> / multiply / divide ────────────────────────────────────
// TimeSpanTests.cs#L1718-1728 (MultiplicationTestData)

fn multiplication_test_data() -> Vec<(TimeSpan, f64, TimeSpan)> {
    vec![
        (ts(2 * HR + 30 * MIN), 2.0, ts(5 * HR)),
        (
            ts(14 * DAY + 2 * HR + 30 * MIN),
            192.0,
            TimeSpan::from_days(2708).unwrap(),
        ),
        (
            TimeSpan::from_days_f64(366.0).unwrap(),
            std::f64::consts::PI,
            ts(993_446_995_288_779),
        ),
        (
            TimeSpan::from_days_f64(366.0).unwrap(),
            -std::f64::consts::E,
            ts(-859_585_952_922_633),
        ),
        (
            TimeSpan::from_days_f64(29.530_587_981).unwrap(),
            13.0,
            TimeSpan::from_days_f64(29.530_587_981 * 13.0).unwrap(),
        ),
        (
            TimeSpan::from_days_f64(-29.530_587_981).unwrap(),
            -12.0,
            TimeSpan::from_days_f64(-29.530_587_981 * -12.0).unwrap(),
        ),
        (
            TimeSpan::from_days_f64(-29.530_587_981).unwrap(),
            0.0,
            TimeSpan::ZERO,
        ),
        (
            TimeSpan::MAX_VALUE,
            0.5,
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            ts((i64::MAX as f64 * 0.5) as i64),
        ),
    ]
}

// TimeSpanTests.cs#L1754-1759
#[test]
fn mul_f64_test_data() {
    for (timespan, factor, expected) in multiplication_test_data() {
        assert_eq!(timespan * factor, Ok(expected));
        assert_eq!(factor * timespan, Ok(expected));
    }
}

// TimeSpanTests.cs#L1761-1766
#[test]
fn mul_f64_overflow() {
    assert_eq!(TimeSpan::MAX_VALUE * 1.000000001, Err(FloatError::Overflow));
}

// TimeSpanTests.cs#L1768-1773
#[test]
fn mul_f64_nan() {
    assert_eq!(
        TimeSpan::from_days(1).unwrap() * f64::NAN,
        Err(FloatError::Nan)
    );
}

// C#'s Math.Round defaults to MidpointRounding.ToEven; verify exact half-tick
// results round to the nearest even tick, not away from zero.
#[test]
fn mul_f64_midpoint_rounds_to_even() {
    assert_eq!(ts(1) * 0.5, Ok(ts(0)));
    assert_eq!(ts(3) * 0.5, Ok(ts(2)));
    assert_eq!(ts(-1) * 0.5, Ok(ts(0)));
    assert_eq!(ts(-3) * 0.5, Ok(ts(-2)));
}

// TimeSpanTests.cs#L1775-1781
#[test]
fn div_f64_test_data() {
    for (timespan, factor, expected) in multiplication_test_data() {
        let divisor = 1.0 / factor;
        assert_eq!(timespan / divisor, Ok(expected));
    }
}

// TimeSpanTests.cs#L1783-1792
#[test]
fn div_f64_by_zero_overflows() {
    assert_eq!(
        TimeSpan::from_days(1).unwrap().divide(0.0),
        Err(FloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_days(-1).unwrap().divide(0.0),
        Err(FloatError::Overflow)
    );
    assert_eq!(TimeSpan::ZERO.divide(0.0), Err(FloatError::Overflow));
}

// TimeSpanTests.cs#L1794-1798
#[test]
fn div_f64_nan() {
    assert_eq!(
        TimeSpan::from_days(1).unwrap() / f64::NAN,
        Err(FloatError::Nan)
    );
}

// TimeSpanTests.cs#L1800-1804
#[test]
fn multiply_method_test_data() {
    for (timespan, factor, expected) in multiplication_test_data() {
        assert_eq!(timespan.multiply(factor), Ok(expected));
    }
}

// TimeSpanTests.cs#L1806-1810
#[test]
fn multiply_method_overflow() {
    assert_eq!(
        TimeSpan::MAX_VALUE.multiply(1.000000001),
        Err(FloatError::Overflow)
    );
}

// TimeSpanTests.cs#L1812-1816
#[test]
fn multiply_method_nan() {
    assert_eq!(
        TimeSpan::from_days(1).unwrap().multiply(f64::NAN),
        Err(FloatError::Nan)
    );
}

// TimeSpanTests.cs#L1818-1824
#[test]
fn divide_method_test_data() {
    for (timespan, factor, expected) in multiplication_test_data() {
        let divisor = 1.0 / factor;
        assert_eq!(timespan.divide(divisor), Ok(expected));
    }
}

// TimeSpanTests.cs#L1826-1835
#[test]
fn divide_method_by_zero() {
    assert_eq!(
        TimeSpan::from_days(1).unwrap().divide(0.0),
        Err(FloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_days(-1).unwrap().divide(0.0),
        Err(FloatError::Overflow)
    );
    assert_eq!(TimeSpan::ZERO.divide(0.0), Err(FloatError::Overflow));
}

// TimeSpanTests.cs#L1837-1841
#[test]
fn divide_method_nan() {
    assert_eq!(
        TimeSpan::from_days(1).unwrap().divide(f64::NAN),
        Err(FloatError::Nan)
    );
}

// ── Chained expressions ───────────────────────────────────────────────────────

#[test]
fn chained_add_sub() {
    // 7h 45m - 18h 12m = -10h 27m
    let a = ts(7 * HR + 45 * MIN);
    let b = ts(18 * HR + 12 * MIN);
    assert_eq!((a - b).to_string(), "-10:27:00");
}

#[test]
fn chained_add_sum() {
    let a = ts(7 * HR + 45 * MIN + 16 * SEC);
    let b = ts(18 * HR + 12 * MIN + 38 * SEC);
    assert_eq!((a + b).to_string(), "1.01:57:54");
}
