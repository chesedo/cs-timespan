use cs_timespan::{FromFloatError, TimeSpan};

// ── FromDays(double) ──────────────────────────────────────────────────────────
// TimeSpanTests.cs#L771-787

#[test]
fn from_days_f64_test_data() {
    assert_eq!(
        TimeSpan::from_days_f64(100.5).unwrap().ticks(),
        86_832_000_000_000
    );
    assert_eq!(
        TimeSpan::from_days_f64(2.5).unwrap().ticks(),
        2_160_000_000_000
    );
    assert_eq!(
        TimeSpan::from_days_f64(1.0).unwrap().ticks(),
        864_000_000_000
    );
    assert_eq!(TimeSpan::from_days_f64(0.0).unwrap().ticks(), 0);
    assert_eq!(
        TimeSpan::from_days_f64(-1.0).unwrap().ticks(),
        -864_000_000_000
    );
    assert_eq!(
        TimeSpan::from_days_f64(-2.5).unwrap().ticks(),
        -2_160_000_000_000
    );
    assert_eq!(
        TimeSpan::from_days_f64(-100.5).unwrap().ticks(),
        -86_832_000_000_000
    );
}

// TimeSpanTests.cs#L789-801
#[test]
fn from_days_f64_invalid() {
    assert_eq!(
        TimeSpan::from_days_f64(f64::INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_days_f64(f64::NEG_INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(TimeSpan::from_days_f64(1e18), Err(FromFloatError::Overflow));
    assert_eq!(
        TimeSpan::from_days_f64(-1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(TimeSpan::from_days_f64(f64::NAN), Err(FromFloatError::Nan));
}

// ── FromHours(double) ─────────────────────────────────────────────────────────
// TimeSpanTests.cs#L803-819

#[test]
fn from_hours_f64_test_data() {
    assert_eq!(
        TimeSpan::from_hours_f64(100.5).unwrap().ticks(),
        3_618_000_000_000
    );
    assert_eq!(
        TimeSpan::from_hours_f64(2.5).unwrap().ticks(),
        90_000_000_000
    );
    assert_eq!(
        TimeSpan::from_hours_f64(1.0).unwrap().ticks(),
        36_000_000_000
    );
    assert_eq!(TimeSpan::from_hours_f64(0.0).unwrap().ticks(), 0);
    assert_eq!(
        TimeSpan::from_hours_f64(-1.0).unwrap().ticks(),
        -36_000_000_000
    );
    assert_eq!(
        TimeSpan::from_hours_f64(-2.5).unwrap().ticks(),
        -90_000_000_000
    );
    assert_eq!(
        TimeSpan::from_hours_f64(-100.5).unwrap().ticks(),
        -3_618_000_000_000
    );
}

// TimeSpanTests.cs#L821-833
#[test]
fn from_hours_f64_invalid() {
    assert_eq!(
        TimeSpan::from_hours_f64(f64::INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_hours_f64(f64::NEG_INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_hours_f64(1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_hours_f64(-1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(TimeSpan::from_hours_f64(f64::NAN), Err(FromFloatError::Nan));
}

// ── FromMinutes(double) ───────────────────────────────────────────────────────
// TimeSpanTests.cs#L835-851

#[test]
fn from_minutes_f64_test_data() {
    assert_eq!(
        TimeSpan::from_minutes_f64(100.5).unwrap().ticks(),
        60_300_000_000
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(2.5).unwrap().ticks(),
        1_500_000_000
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(1.0).unwrap().ticks(),
        600_000_000
    );
    assert_eq!(TimeSpan::from_minutes_f64(0.0).unwrap().ticks(), 0);
    assert_eq!(
        TimeSpan::from_minutes_f64(-1.0).unwrap().ticks(),
        -600_000_000
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(-2.5).unwrap().ticks(),
        -1_500_000_000
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(-100.5).unwrap().ticks(),
        -60_300_000_000
    );
}

// TimeSpanTests.cs#L853-865
#[test]
fn from_minutes_f64_invalid() {
    assert_eq!(
        TimeSpan::from_minutes_f64(f64::INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(f64::NEG_INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(-1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_minutes_f64(f64::NAN),
        Err(FromFloatError::Nan)
    );
}

// ── FromSeconds(double) ───────────────────────────────────────────────────────
// TimeSpanTests.cs#L867-883

#[test]
fn from_seconds_f64_test_data() {
    assert_eq!(
        TimeSpan::from_seconds_f64(100.5).unwrap().ticks(),
        1_005_000_000
    );
    assert_eq!(TimeSpan::from_seconds_f64(2.5).unwrap().ticks(), 25_000_000);
    assert_eq!(TimeSpan::from_seconds_f64(1.0).unwrap().ticks(), 10_000_000);
    assert_eq!(TimeSpan::from_seconds_f64(0.0).unwrap().ticks(), 0);
    assert_eq!(
        TimeSpan::from_seconds_f64(-1.0).unwrap().ticks(),
        -10_000_000
    );
    assert_eq!(
        TimeSpan::from_seconds_f64(-2.5).unwrap().ticks(),
        -25_000_000
    );
    assert_eq!(
        TimeSpan::from_seconds_f64(-100.5).unwrap().ticks(),
        -1_005_000_000
    );
}

// TimeSpanTests.cs#L885-897
#[test]
fn from_seconds_f64_invalid() {
    assert_eq!(
        TimeSpan::from_seconds_f64(f64::INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_seconds_f64(f64::NEG_INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_seconds_f64(1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_seconds_f64(-1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_seconds_f64(f64::NAN),
        Err(FromFloatError::Nan)
    );
}

// ── FromMilliseconds(double) ──────────────────────────────────────────────────
// Truncates rather than rounds, matching the currently-active .NET Core test data
// (FromMilliseconds_TestData_NetCore); the historical FromMilliseconds_TestData_Desktop
// rounds instead but is dead data in the current test file, never wired to a [Theory].
// TimeSpanTests.cs#L899-915

#[test]
fn from_milliseconds_f64_test_data() {
    assert_eq!(
        TimeSpan::from_milliseconds_f64(1500.5).unwrap().ticks(),
        15_005_000
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(2.5).unwrap().ticks(),
        25_000
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(1.0).unwrap().ticks(),
        10_000
    );
    assert_eq!(TimeSpan::from_milliseconds_f64(0.0).unwrap().ticks(), 0);
    assert_eq!(
        TimeSpan::from_milliseconds_f64(-1.0).unwrap().ticks(),
        -10_000
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(-2.5).unwrap().ticks(),
        -25_000
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(-1500.5).unwrap().ticks(),
        -15_005_000
    );
}

// TimeSpanTests.cs#L928-939
#[test]
fn from_milliseconds_f64_invalid() {
    assert_eq!(
        TimeSpan::from_milliseconds_f64(f64::INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(f64::NEG_INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(-1e18),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_milliseconds_f64(f64::NAN),
        Err(FromFloatError::Nan)
    );
}

// ── FromMicroseconds(double) ──────────────────────────────────────────────────
// No dedicated upstream (double) test data exists for this overload; covered here
// directly instead. Method definition: TimeSpan.cs#L679

#[test]
fn from_microseconds_f64_basic() {
    assert_eq!(TimeSpan::from_microseconds_f64(15.0).unwrap().ticks(), 150);
    assert_eq!(
        TimeSpan::from_microseconds_f64(-15.0).unwrap().ticks(),
        -150
    );
    assert_eq!(TimeSpan::from_microseconds_f64(0.0).unwrap().ticks(), 0);
    // Fractional input; also locks in truncation-toward-zero (25.5 ticks -> 25, not 26)
    assert_eq!(TimeSpan::from_microseconds_f64(2.5).unwrap().ticks(), 25);
    assert_eq!(TimeSpan::from_microseconds_f64(2.55).unwrap().ticks(), 25);
    assert_eq!(TimeSpan::from_microseconds_f64(-2.55).unwrap().ticks(), -25);
    assert_eq!(
        TimeSpan::from_microseconds_f64(f64::NAN),
        Err(FromFloatError::Nan)
    );
    assert_eq!(
        TimeSpan::from_microseconds_f64(f64::INFINITY),
        Err(FromFloatError::Overflow)
    );
    assert_eq!(
        TimeSpan::from_microseconds_f64(f64::NEG_INFINITY),
        Err(FromFloatError::Overflow)
    );
}

// ── Boundary: exact MAX_VALUE/MIN_VALUE round-trip ─────────────────────────────

#[test]
fn from_days_f64_max_value_boundary() {
    let max_days = TimeSpan::MAX_VALUE.total_days();
    assert_eq!(
        TimeSpan::from_days_f64(max_days).unwrap(),
        TimeSpan::MAX_VALUE
    );
}

#[test]
fn from_days_f64_min_value_boundary() {
    let min_days = TimeSpan::MIN_VALUE.total_days();
    assert_eq!(
        TimeSpan::from_days_f64(min_days).unwrap(),
        TimeSpan::MIN_VALUE
    );
}
