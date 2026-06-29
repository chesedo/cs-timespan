use cs_timespan::TimeSpan;

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
