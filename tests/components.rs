use cs_timespan::TimeSpan;

// ── Positive values ────────────────────────────────────────────────────────────
// Exercises the Days/Hours/Minutes/Seconds/Milliseconds/Microseconds/Nanoseconds
// properties: TimeSpan.cs#L310-336

#[test]
fn positive_components() {
    let ts = TimeSpan::parse("1.02:03:04.005006700").unwrap();
    assert_eq!(ts.days(), 1);
    assert_eq!(ts.hours(), 2);
    assert_eq!(ts.minutes(), 3);
    assert_eq!(ts.seconds(), 4);
    assert_eq!(ts.milliseconds(), 5);
    assert_eq!(ts.microseconds(), 6);
    assert_eq!(ts.nanoseconds(), 700);
}

// ── Negative values (C# truncates toward zero; components carry the sign) ─────

#[test]
fn negative_components() {
    let ts = -TimeSpan::parse("1.02:03:04.005006700").unwrap();
    assert_eq!(ts.days(), -1);
    assert_eq!(ts.hours(), -2);
    assert_eq!(ts.minutes(), -3);
    assert_eq!(ts.seconds(), -4);
    assert_eq!(ts.milliseconds(), -5);
    assert_eq!(ts.microseconds(), -6);
    assert_eq!(ts.nanoseconds(), -700);
}

// ── Zero ────────────────────────────────────────────────────────────────────────

#[test]
fn zero_components() {
    let ts = TimeSpan::ZERO;
    assert_eq!(ts.days(), 0);
    assert_eq!(ts.hours(), 0);
    assert_eq!(ts.minutes(), 0);
    assert_eq!(ts.seconds(), 0);
    assert_eq!(ts.milliseconds(), 0);
    assert_eq!(ts.microseconds(), 0);
    assert_eq!(ts.nanoseconds(), 0);
}

// ── Boundary: components never exceed their unit's range ───────────────────────

#[test]
fn hours_wraps_within_day() {
    // 25 hours = 1 day + 1 hour
    let ts = TimeSpan::from_ticks(25 * TimeSpan::TICKS_PER_HOUR);
    assert_eq!(ts.days(), 1);
    assert_eq!(ts.hours(), 1);
}

#[test]
fn max_value_components() {
    let ts = TimeSpan::MAX_VALUE;
    assert_eq!(ts.ticks() / TimeSpan::TICKS_PER_DAY, i64::from(ts.days()));
    assert!((0..24).contains(&ts.hours()));
    assert!((0..60).contains(&ts.minutes()));
    assert!((0..60).contains(&ts.seconds()));
}
