use cs_timespan::{TimeSpan, TimeSpanOverflow};

// Consts copied from TimeSpanTests.cs#L407-414 (internal consts in TimeSpan; max/min symmetrical)
const MAX_DAYS: i64 = 10_675_199;
const MAX_HOURS: i64 = 256_204_778;
const MAX_MINUTES: i64 = 15_372_286_728;
const MAX_SECONDS: i64 = 922_337_203_685;
const MAX_MILLISECONDS: i64 = 922_337_203_685_477;
const MAX_MICROSECONDS: i64 = 922_337_203_685_477_580;

fn build(vals: [i64; 6]) -> Result<TimeSpan, TimeSpanOverflow> {
    TimeSpan::builder()
        .days(vals[0] as i32)
        .hours(vals[1] as i32)
        .minutes(vals[2])
        .seconds(vals[3])
        .milliseconds(vals[4])
        .microseconds(vals[5])
        .build()
}

// ── FromDays(days, hours, minutes, seconds, milliseconds, microseconds) ───────

// TimeSpanTests.cs#L352-358
#[test]
fn from_days_int_positive() {
    let expected = TimeSpan::from_ticks(
        1 * TimeSpan::TICKS_PER_DAY
            + 2 * TimeSpan::TICKS_PER_HOUR
            + 3 * TimeSpan::TICKS_PER_MINUTE
            + 4 * TimeSpan::TICKS_PER_SECOND
            + 5 * TimeSpan::TICKS_PER_MILLISECOND
            + 6 * TimeSpan::TICKS_PER_MICROSECOND,
    );
    let actual = TimeSpan::builder()
        .days(1)
        .hours(2)
        .minutes(3)
        .seconds(4)
        .milliseconds(5)
        .microseconds(6)
        .build()
        .unwrap();
    assert_eq!(expected, actual);
}

// TimeSpanTests.cs#L359-365
#[test]
fn from_days_int_negative() {
    let expected = TimeSpan::from_ticks(
        -(1 * TimeSpan::TICKS_PER_DAY
            + 2 * TimeSpan::TICKS_PER_HOUR
            + 3 * TimeSpan::TICKS_PER_MINUTE
            + 4 * TimeSpan::TICKS_PER_SECOND
            + 5 * TimeSpan::TICKS_PER_MILLISECOND
            + 6 * TimeSpan::TICKS_PER_MICROSECOND),
    );
    let actual = TimeSpan::builder()
        .days(-1)
        .hours(-2)
        .minutes(-3)
        .seconds(-4)
        .milliseconds(-5)
        .microseconds(-6)
        .build()
        .unwrap();
    assert_eq!(expected, actual);
}

// TimeSpanTests.cs#L366-372
#[test]
fn from_days_int_zero() {
    let actual = TimeSpan::builder()
        .days(0)
        .hours(0)
        .minutes(0)
        .seconds(0)
        .milliseconds(0)
        .microseconds(0)
        .build()
        .unwrap();
    assert_eq!(TimeSpan::ZERO, actual);
}

// TimeSpanTests.cs#L374-379
#[test]
fn from_seconds_int_should_give_result_with_precision() {
    let expected = TimeSpan::from_ticks(
        101 * TimeSpan::TICKS_PER_SECOND + 832 * TimeSpan::TICKS_PER_MILLISECOND,
    );
    let actual = TimeSpan::builder()
        .seconds(101)
        .milliseconds(832)
        .build()
        .unwrap();
    assert_eq!(expected, actual);
}

// TimeSpanTests.cs#L381-386
#[test]
fn from_days_int_should_overflow_when_intermediate_calculation_could_overflow_back_into_valid_range()
 {
    assert_eq!(TimeSpan::from_days(1_067_519_900), Err(TimeSpanOverflow));
}

// TimeSpanTests.cs#L388-396
#[test]
fn from_days_int_should_construct_max_value_approximation() {
    let expected = TimeSpan::MAX_VALUE;
    let actual = TimeSpan::builder()
        .days(expected.days())
        .hours(expected.hours())
        .minutes(i64::from(expected.minutes()))
        .seconds(i64::from(expected.seconds()))
        .milliseconds(i64::from(expected.milliseconds()))
        .microseconds(i64::from(expected.microseconds()))
        .build()
        .unwrap();
    // Should be within TicksPerMicrosecond (10) ticks of expected
    let diff_ticks = expected.ticks() - actual.ticks();
    assert!(
        diff_ticks.abs() < TimeSpan::TICKS_PER_MICROSECOND,
        "Diff ticks was {diff_ticks}"
    );
}

// TimeSpanTests.cs#L397-405
#[test]
fn from_days_int_should_construct_min_value_approximation() {
    let expected = TimeSpan::MIN_VALUE;
    let actual = TimeSpan::builder()
        .days(expected.days())
        .hours(expected.hours())
        .minutes(i64::from(expected.minutes()))
        .seconds(i64::from(expected.seconds()))
        .milliseconds(i64::from(expected.milliseconds()))
        .microseconds(i64::from(expected.microseconds()))
        .build()
        .unwrap();
    let diff_ticks = actual.ticks() - expected.ticks();
    assert!(
        diff_ticks.abs() < TimeSpan::TICKS_PER_MICROSECOND,
        "Diff ticks was {diff_ticks}"
    );
}

// Full parity with TimeSpanTests.cs#L415-453 (FromDays_Int_ShouldOverflowOrUnderflow_Data +
// FromDays_Int_ShouldOverflowOrUnderflow): ports the combinatorial generator itself rather
// than hand-listing all 42 generated rows, since that's what the C# data source does too.
#[test]
fn from_days_int_should_overflow_or_underflow() {
    let individual_max_values: [i64; 6] = [
        MAX_DAYS,
        MAX_HOURS,
        MAX_MINUTES,
        MAX_SECONDS,
        MAX_MILLISECONDS,
        MAX_MICROSECONDS,
    ];

    // Each possibility for individual property to overflow or underflow
    for i in 0..6 {
        let i_val = individual_max_values[i] + 1;

        let mut result_pos = [0i64; 6];
        result_pos[i] = i_val;
        assert_eq!(
            build(result_pos),
            Err(TimeSpanOverflow),
            "expected overflow for {result_pos:?}"
        );

        let mut result_neg = [0i64; 6];
        result_neg[i] = -i_val;
        assert_eq!(
            build(result_neg),
            Err(TimeSpanOverflow),
            "expected overflow for {result_neg:?}"
        );
    }

    // Each possibility for 2 properties to overflow or underflow while neither
    // individually overflows or underflows
    for i in 0..6 {
        for j in (i + 1)..6 {
            let i_val = individual_max_values[i];
            let j_val = individual_max_values[j];

            let mut result_pos = [0i64; 6];
            result_pos[i] = i_val;
            result_pos[j] = j_val;
            assert_eq!(
                build(result_pos),
                Err(TimeSpanOverflow),
                "expected overflow for {result_pos:?}"
            );

            let mut result_neg = [0i64; 6];
            result_neg[i] = -i_val;
            result_neg[j] = -j_val;
            assert_eq!(
                build(result_neg),
                Err(TimeSpanOverflow),
                "expected overflow for {result_neg:?}"
            );
        }
    }
}

// Full parity with TimeSpanTests.cs#L455-484
// (FromDays_Int_ShouldNotOverflow_WhenOverflowingParamIsCounteredByOppositeSignParam_Data +
// ..._ShouldNotOverflow_WhenOverflowingParamIsCounteredByOppositeSignParam): ports the
// combinatorial generator itself rather than hand-listing all 30 generated rows.
#[test]
fn from_days_int_should_not_overflow_when_overflowing_param_is_countered_by_opposite_sign_param() {
    let individual_max_values: [i64; 6] = [
        MAX_DAYS,
        MAX_HOURS,
        MAX_MINUTES,
        MAX_SECONDS,
        MAX_MILLISECONDS,
        MAX_MICROSECONDS,
    ];

    for i in 0..6 {
        for j in 0..6 {
            if i == j {
                continue;
            }
            let i_val = individual_max_values[i] + 1;
            let j_val = individual_max_values[j] + 1;

            let mut result = [0i64; 6];
            result[i] = i_val;
            result[j] = -j_val;
            let actual =
                build(result).unwrap_or_else(|_| panic!("expected no overflow for {result:?}"));
            // 2 individually overflowing or underflowing params with opposite sign should
            // end up close to TimeSpan::from_days(0)
            assert!(actual > TimeSpan::from_days(-1).unwrap());
            assert!(actual < TimeSpan::from_days(1).unwrap());
        }
    }
}

// TimeSpanTests.cs#L485-505 (FromDays_Int_ShouldOverflow, all 16 InlineData rows)
#[test]
fn from_days_int_should_overflow() {
    assert_eq!(
        build([
            MAX_DAYS,
            MAX_HOURS,
            MAX_MINUTES,
            MAX_SECONDS,
            MAX_MILLISECONDS,
            MAX_MICROSECONDS
        ]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(
        build([
            -MAX_DAYS,
            -MAX_HOURS,
            -MAX_MINUTES,
            -MAX_SECONDS,
            -MAX_MILLISECONDS,
            -MAX_MICROSECONDS
        ]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(
        build([
            i64::from(i32::MAX),
            i64::from(i32::MAX),
            i64::MAX,
            i64::MAX,
            i64::MAX,
            i64::MAX
        ]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(
        build([
            i64::from(i32::MIN),
            i64::from(i32::MIN),
            i64::MIN,
            i64::MIN,
            i64::MIN,
            i64::MIN
        ]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(
        build([i64::from(i32::MAX), 0, 0, 0, 0, 0]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(
        build([i64::from(i32::MIN), 0, 0, 0, 0, 0]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(
        build([0, i64::from(i32::MAX), 0, 0, 0, 0]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(
        build([0, i64::from(i32::MIN), 0, 0, 0, 0]),
        Err(TimeSpanOverflow)
    );
    assert_eq!(build([0, 0, i64::MAX, 0, 0, 0]), Err(TimeSpanOverflow));
    assert_eq!(build([0, 0, i64::MIN, 0, 0, 0]), Err(TimeSpanOverflow));
    assert_eq!(build([0, 0, 0, i64::MAX, 0, 0]), Err(TimeSpanOverflow));
    assert_eq!(build([0, 0, 0, i64::MIN, 0, 0]), Err(TimeSpanOverflow));
    assert_eq!(build([0, 0, 0, 0, i64::MAX, 0]), Err(TimeSpanOverflow));
    assert_eq!(build([0, 0, 0, 0, i64::MIN, 0]), Err(TimeSpanOverflow));
    assert_eq!(build([0, 0, 0, 0, 0, i64::MAX]), Err(TimeSpanOverflow));
    assert_eq!(build([0, 0, 0, 0, 0, i64::MIN]), Err(TimeSpanOverflow));
}

// ── FromDays(int) ──────────────────────────────────────────────────────────────

// TimeSpanTests.cs#L507-516
#[test]
fn from_days_int_single_should_create() {
    for days in [0, 1, -1, MAX_DAYS as i32, -(MAX_DAYS as i32)] {
        assert_eq!(
            TimeSpan::from_days(days).unwrap(),
            TimeSpan::from_ticks(i64::from(days) * TimeSpan::TICKS_PER_DAY)
        );
    }
}

// TimeSpanTests.cs#L517-525
#[test]
fn from_days_int_single_should_overflow() {
    for days in [
        (MAX_DAYS + 1) as i32,
        -((MAX_DAYS + 1) as i32),
        i32::MAX,
        i32::MIN,
    ] {
        assert_eq!(TimeSpan::from_days(days), Err(TimeSpanOverflow));
    }
}

// ── FromHours(int) / FromHours(hours, minutes, seconds, milliseconds, microseconds) ──

// TimeSpanTests.cs#L527-536
#[test]
fn from_hours_int_single_should_create() {
    for hours in [0, 1, -1, MAX_HOURS as i32, -(MAX_HOURS as i32)] {
        assert_eq!(
            TimeSpan::from_hours(hours).unwrap(),
            TimeSpan::from_ticks(i64::from(hours) * TimeSpan::TICKS_PER_HOUR)
        );
    }
}

// TimeSpanTests.cs#L537-545
#[test]
fn from_hours_int_single_should_overflow() {
    for hours in [
        (MAX_HOURS + 1) as i32,
        -((MAX_HOURS + 1) as i32),
        i32::MAX,
        i32::MIN,
    ] {
        assert_eq!(TimeSpan::from_hours(hours), Err(TimeSpanOverflow));
    }
}

fn build_hours(
    hours: i32,
    minutes: i64,
    seconds: i64,
    milliseconds: i64,
    microseconds: i64,
) -> Result<TimeSpan, TimeSpanOverflow> {
    TimeSpan::builder()
        .hours(hours)
        .minutes(minutes)
        .seconds(seconds)
        .milliseconds(milliseconds)
        .microseconds(microseconds)
        .build()
}

// TimeSpanTests.cs#L547-570
#[test]
fn from_hours_int_should_create() {
    let cases: [(i32, i64, i64, i64, i64); 13] = [
        (0, 0, 0, 0, 0),
        (1, 1, 1, 1, 1),
        (-1, -1, -1, -1, -1),
        (MAX_HOURS as i32, 0, 0, 0, 0),
        (-(MAX_HOURS as i32), 0, 0, 0, 0),
        (0, MAX_MINUTES, 0, 0, 0),
        (0, -MAX_MINUTES, 0, 0, 0),
        (0, 0, MAX_SECONDS, 0, 0),
        (0, 0, -MAX_SECONDS, 0, 0),
        (0, 0, 0, MAX_MILLISECONDS, 0),
        (0, 0, 0, -MAX_MILLISECONDS, 0),
        (0, 0, 0, 0, MAX_MICROSECONDS),
        (0, 0, 0, 0, -MAX_MICROSECONDS),
    ];
    for (hours, minutes, seconds, milliseconds, microseconds) in cases {
        let expected = TimeSpan::from_ticks(
            i64::from(hours) * TimeSpan::TICKS_PER_HOUR
                + minutes * TimeSpan::TICKS_PER_MINUTE
                + seconds * TimeSpan::TICKS_PER_SECOND
                + milliseconds * TimeSpan::TICKS_PER_MILLISECOND
                + microseconds * TimeSpan::TICKS_PER_MICROSECOND,
        );
        assert_eq!(
            build_hours(hours, minutes, seconds, milliseconds, microseconds).unwrap(),
            expected
        );
    }
}

// TimeSpanTests.cs#L571-585
#[test]
fn from_hours_int_should_overflow() {
    let cases: [(i32, i64, i64, i64, i64); 10] = [
        ((MAX_HOURS + 1) as i32, 0, 0, 0, 0),
        (-((MAX_HOURS + 1) as i32), 0, 0, 0, 0),
        (0, MAX_MINUTES + 1, 0, 0, 0),
        (0, -(MAX_MINUTES + 1), 0, 0, 0),
        (0, 0, MAX_SECONDS + 1, 0, 0),
        (0, 0, -(MAX_SECONDS + 1), 0, 0),
        (0, 0, 0, MAX_MILLISECONDS + 1, 0),
        (0, 0, 0, -(MAX_MILLISECONDS + 1), 0),
        (0, 0, 0, 0, MAX_MICROSECONDS + 1),
        (0, 0, 0, 0, -(MAX_MICROSECONDS + 1)),
    ];
    for (hours, minutes, seconds, milliseconds, microseconds) in cases {
        assert_eq!(
            build_hours(hours, minutes, seconds, milliseconds, microseconds),
            Err(TimeSpanOverflow)
        );
    }
}

// ── FromMinutes(long) / FromMinutes(minutes, seconds, milliseconds, microseconds) ──

// TimeSpanTests.cs#L587-596
#[test]
fn from_minutes_int_single_should_create() {
    for minutes in [0, 1, -1, MAX_MINUTES, -MAX_MINUTES] {
        assert_eq!(
            TimeSpan::from_minutes(minutes).unwrap(),
            TimeSpan::builder().minutes(minutes).build().unwrap()
        );
    }
}

// TimeSpanTests.cs#L597-605
#[test]
fn from_minutes_int_single_should_overflow() {
    for minutes in [MAX_MINUTES + 1, -(MAX_MINUTES + 1), i64::MAX, i64::MIN] {
        assert_eq!(TimeSpan::from_minutes(minutes), Err(TimeSpanOverflow));
    }
}

fn build_minutes(
    minutes: i64,
    seconds: i64,
    milliseconds: i64,
    microseconds: i64,
) -> Result<TimeSpan, TimeSpanOverflow> {
    TimeSpan::builder()
        .minutes(minutes)
        .seconds(seconds)
        .milliseconds(milliseconds)
        .microseconds(microseconds)
        .build()
}

// TimeSpanTests.cs#L607-627
#[test]
fn from_minutes_int_should_create() {
    let cases: [(i64, i64, i64, i64); 11] = [
        (0, 0, 0, 0),
        (1, 1, 1, 1),
        (-1, -1, -1, -1),
        (MAX_MINUTES, 0, 0, 0),
        (-MAX_MINUTES, 0, 0, 0),
        (0, MAX_SECONDS, 0, 0),
        (0, -MAX_SECONDS, 0, 0),
        (0, 0, MAX_MILLISECONDS, 0),
        (0, 0, -MAX_MILLISECONDS, 0),
        (0, 0, 0, MAX_MICROSECONDS),
        (0, 0, 0, -MAX_MICROSECONDS),
    ];
    for (minutes, seconds, milliseconds, microseconds) in cases {
        let expected = TimeSpan::from_ticks(
            minutes * TimeSpan::TICKS_PER_MINUTE
                + seconds * TimeSpan::TICKS_PER_SECOND
                + milliseconds * TimeSpan::TICKS_PER_MILLISECOND
                + microseconds * TimeSpan::TICKS_PER_MICROSECOND,
        );
        assert_eq!(
            build_minutes(minutes, seconds, milliseconds, microseconds).unwrap(),
            expected
        );
    }
}

// TimeSpanTests.cs#L628-640
#[test]
fn from_minutes_int_should_overflow() {
    let cases: [(i64, i64, i64, i64); 8] = [
        (MAX_MINUTES + 1, 0, 0, 0),
        (-(MAX_MINUTES + 1), 0, 0, 0),
        (0, MAX_SECONDS + 1, 0, 0),
        (0, -(MAX_SECONDS + 1), 0, 0),
        (0, 0, MAX_MILLISECONDS + 1, 0),
        (0, 0, -(MAX_MILLISECONDS + 1), 0),
        (0, 0, 0, MAX_MICROSECONDS + 1),
        (0, 0, 0, -(MAX_MICROSECONDS + 1)),
    ];
    for (minutes, seconds, milliseconds, microseconds) in cases {
        assert_eq!(
            build_minutes(minutes, seconds, milliseconds, microseconds),
            Err(TimeSpanOverflow)
        );
    }
}

// ── FromSeconds(long) / FromSeconds(seconds, milliseconds, microseconds) ─────

// TimeSpanTests.cs#L642-651
#[test]
fn from_seconds_int_single_should_create() {
    for seconds in [0, 1, -1, MAX_SECONDS, -MAX_SECONDS] {
        assert_eq!(
            TimeSpan::from_seconds(seconds).unwrap(),
            TimeSpan::builder().seconds(seconds).build().unwrap()
        );
    }
}

// TimeSpanTests.cs#L652-660
#[test]
fn from_seconds_int_single_should_overflow() {
    for seconds in [MAX_SECONDS + 1, -(MAX_SECONDS + 1), i64::MAX, i64::MIN] {
        assert_eq!(TimeSpan::from_seconds(seconds), Err(TimeSpanOverflow));
    }
}

fn build_seconds(
    seconds: i64,
    milliseconds: i64,
    microseconds: i64,
) -> Result<TimeSpan, TimeSpanOverflow> {
    TimeSpan::builder()
        .seconds(seconds)
        .milliseconds(milliseconds)
        .microseconds(microseconds)
        .build()
}

// TimeSpanTests.cs#L662-679
#[test]
fn from_seconds_int_should_create() {
    let cases: [(i64, i64, i64); 9] = [
        (0, 0, 0),
        (1, 1, 1),
        (-1, -1, -1),
        (MAX_SECONDS, 0, 0),
        (-MAX_SECONDS, 0, 0),
        (0, MAX_MILLISECONDS, 0),
        (0, -MAX_MILLISECONDS, 0),
        (0, 0, MAX_MICROSECONDS),
        (0, 0, -MAX_MICROSECONDS),
    ];
    for (seconds, milliseconds, microseconds) in cases {
        let expected = TimeSpan::from_ticks(
            seconds * TimeSpan::TICKS_PER_SECOND
                + milliseconds * TimeSpan::TICKS_PER_MILLISECOND
                + microseconds * TimeSpan::TICKS_PER_MICROSECOND,
        );
        assert_eq!(
            build_seconds(seconds, milliseconds, microseconds).unwrap(),
            expected
        );
    }
}

// TimeSpanTests.cs#L680-699
#[test]
fn from_seconds_int_should_overflow() {
    let cases: [(i64, i64, i64); 15] = [
        (MAX_SECONDS + 1, 0, 0),
        (-(MAX_SECONDS + 1), 0, 0),
        (0, MAX_MILLISECONDS + 1, 0),
        (0, -(MAX_MILLISECONDS + 1), 0),
        (0, 0, MAX_MICROSECONDS + 1),
        (0, 0, -(MAX_MICROSECONDS + 1)),
        (i64::MAX, 0, 0),
        (i64::MIN, 0, 0),
        (0, i64::MAX, 0),
        (0, i64::MIN, 0),
        (0, 0, i64::MAX),
        (0, 0, i64::MIN),
        (MAX_SECONDS, MAX_MILLISECONDS, 0),
        (0, MAX_MILLISECONDS, MAX_MICROSECONDS),
        (MAX_SECONDS, 0, MAX_MICROSECONDS),
    ];
    for (seconds, milliseconds, microseconds) in cases {
        assert_eq!(
            build_seconds(seconds, milliseconds, microseconds),
            Err(TimeSpanOverflow)
        );
    }
}

// ── FromMilliseconds(long) / FromMilliseconds(milliseconds, microseconds) ─────

// TimeSpanTests.cs#L701-719 (the C#-only compile-check expressions at L721-724 have no
// Rust equivalent -- Rust has no overload-resolution ambiguity to guard against, since
// from_milliseconds(i64) and from_milliseconds(f64) are distinctly-named functions)
#[test]
fn from_milliseconds_int_should_create() {
    let cases: [(i64, i64); 9] = [
        (0, 0),
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
        (MAX_MILLISECONDS, 0),
        (-MAX_MILLISECONDS, 0),
        (0, MAX_MICROSECONDS),
        (0, -MAX_MICROSECONDS),
    ];
    for (milliseconds, microseconds) in cases {
        let expected = TimeSpan::from_ticks(
            milliseconds * TimeSpan::TICKS_PER_MILLISECOND
                + microseconds * TimeSpan::TICKS_PER_MICROSECOND,
        );
        assert_eq!(
            TimeSpan::builder()
                .milliseconds(milliseconds)
                .microseconds(microseconds)
                .build()
                .unwrap(),
            expected
        );

        let expected_single = TimeSpan::from_ticks(milliseconds * TimeSpan::TICKS_PER_MILLISECOND);
        assert_eq!(
            TimeSpan::from_milliseconds(milliseconds).unwrap(),
            expected_single
        );
    }
}

// TimeSpanTests.cs#L727-748
#[test]
fn from_milliseconds_int_should_overflow() {
    let cases: [(i64, i64); 12] = [
        (MAX_MILLISECONDS + 1, 0),
        (-(MAX_MILLISECONDS + 1), 0),
        (i64::MAX, 0),
        (i64::MIN, 0),
        (0, MAX_MICROSECONDS + 1),
        (0, -(MAX_MICROSECONDS + 1)),
        (0, i64::MAX),
        (0, i64::MIN),
        (MAX_MILLISECONDS, 1000),
        (-MAX_MILLISECONDS, -1000),
        (1, MAX_MICROSECONDS),
        (-1, -MAX_MICROSECONDS),
    ];
    for (milliseconds, microseconds) in cases {
        assert_eq!(
            TimeSpan::builder()
                .milliseconds(milliseconds)
                .microseconds(microseconds)
                .build(),
            Err(TimeSpanOverflow)
        );
        if microseconds == 0 {
            assert_eq!(
                TimeSpan::from_milliseconds(milliseconds),
                Err(TimeSpanOverflow)
            );
        }
    }
}

// ── FromMicroseconds(long) ─────────────────────────────────────────────────────

// TimeSpanTests.cs#L750-759
#[test]
fn from_microseconds_int_single_should_create() {
    for microseconds in [0, 1, -1, MAX_MICROSECONDS, -MAX_MICROSECONDS] {
        assert_eq!(
            TimeSpan::from_microseconds(microseconds).unwrap(),
            TimeSpan::builder()
                .microseconds(microseconds)
                .build()
                .unwrap()
        );
    }
}

// TimeSpanTests.cs#L760-768
#[test]
fn from_microseconds_int_single_should_overflow() {
    for microseconds in [
        MAX_MICROSECONDS + 1,
        -(MAX_MICROSECONDS + 1),
        i64::MAX,
        i64::MIN,
    ] {
        assert_eq!(
            TimeSpan::from_microseconds(microseconds),
            Err(TimeSpanOverflow)
        );
    }
}
