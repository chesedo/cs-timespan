use cs_timespan::TimeSpan;

// ── Exact fractional values (representable precisely in f64) ─────────────────

#[test]
fn total_days_fractional() {
    // 36 hours = 1.5 days
    let ts = TimeSpan::from_ticks(36 * TimeSpan::TICKS_PER_HOUR);
    assert_eq!(ts.total_days(), 1.5);
}

#[test]
fn total_hours_fractional() {
    // 90 minutes = 1.5 hours
    let ts = TimeSpan::from_ticks(90 * TimeSpan::TICKS_PER_MINUTE);
    assert_eq!(ts.total_hours(), 1.5);
}

#[test]
fn total_minutes_fractional() {
    // 90 seconds = 1.5 minutes
    let ts = TimeSpan::from_ticks(90 * TimeSpan::TICKS_PER_SECOND);
    assert_eq!(ts.total_minutes(), 1.5);
}

#[test]
fn total_seconds_fractional() {
    // 1500 milliseconds = 1.5 seconds
    let ts = TimeSpan::from_ticks(1500 * TimeSpan::TICKS_PER_MILLISECOND);
    assert_eq!(ts.total_seconds(), 1.5);
}

#[test]
fn total_milliseconds_basic() {
    let ts = TimeSpan::from_ticks(15 * TimeSpan::TICKS_PER_MILLISECOND);
    assert_eq!(ts.total_milliseconds(), 15.0);
}

#[test]
fn total_microseconds_basic() {
    let ts = TimeSpan::from_ticks(150); // 150 * 100ns = 15 microseconds
    assert_eq!(ts.total_microseconds(), 15.0);
}

#[test]
fn total_nanoseconds_basic() {
    let ts = TimeSpan::from_ticks(5); // 5 * 100ns = 500 nanoseconds
    assert_eq!(ts.total_nanoseconds(), 500.0);
}

// ── Zero ────────────────────────────────────────────────────────────────────────

#[test]
fn zero_totals() {
    let ts = TimeSpan::ZERO;
    assert_eq!(ts.total_days(), 0.0);
    assert_eq!(ts.total_hours(), 0.0);
    assert_eq!(ts.total_minutes(), 0.0);
    assert_eq!(ts.total_seconds(), 0.0);
    assert_eq!(ts.total_milliseconds(), 0.0);
    assert_eq!(ts.total_microseconds(), 0.0);
    assert_eq!(ts.total_nanoseconds(), 0.0);
}

// ── Negative values ─────────────────────────────────────────────────────────────

#[test]
fn negative_totals() {
    let ts = -TimeSpan::from_ticks(36 * TimeSpan::TICKS_PER_HOUR);
    assert_eq!(ts.total_days(), -1.5);
    assert_eq!(ts.total_hours(), -36.0);
}

// ── TotalMilliseconds clamping at the i64 tick boundaries ───────────────────────
// C# clamps TotalMilliseconds to MinMilliseconds/MaxMilliseconds because casting
// MIN_VALUE/MAX_VALUE ticks to f64 and dividing can round past the true boundary.
// Duplicates TotalMilliseconds_Invalid: TimeSpanTests.cs#L147-L154

#[test]
fn total_milliseconds_clamps_at_max_value() {
    assert_eq!(
        TimeSpan::MAX_VALUE.total_milliseconds(),
        922_337_203_685_477.0
    );
}

#[test]
fn total_milliseconds_clamps_at_min_value() {
    assert_eq!(
        TimeSpan::MIN_VALUE.total_milliseconds(),
        -922_337_203_685_477.0
    );
}
