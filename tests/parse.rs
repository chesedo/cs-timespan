// Tests ported from the C# reference implementation:
// https://github.com/dotnet/corefx/blob/master/src/System.Runtime/tests/System/TimeSpanTests.cs
//
// C# test methods covered here: Parse_Valid_TestData, Parse_Invalid_TestData
//
// Notes on translation:
// - C# `ArgumentNullException` for null input has no Rust equivalent (`&str` can't be null)
//   and is therefore omitted.
// - C# `new TimeSpan(h, m, s)`      → `ts3(h, m, s)`       (3-arg: hours, minutes, seconds)
// - C# `new TimeSpan(d, h, m, s)`   → `ts4(d, h, m, s)`    (4-arg: days, hours, minutes, seconds)
// - C# `new TimeSpan(d, h, m, s, ms)` → `ts5(d, h, m, s, ms)` (5-arg)
// - C# `new TimeSpan(ticks)`         → `TimeSpan::from_ticks(ticks)`

use cs_timespan::{Locale, ParseError, TimeSpan};

// ── en-US behaves identically to Invariant (both use '.' as decimal separator) ─

#[test]
fn parse_en_us_same_as_invariant() {
    assert_eq!(
        TimeSpan::parse_with_culture("12:24:02.01", Locale::en),
        Ok(ts5(0, 12, 24, 2, 10)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1.12:24:02.999", Locale::en),
        Ok(ts5(1, 12, 24, 2, 999)),
    );
}

#[test]
fn parse_en_us_rejects_comma_separator() {
    assert_eq!(
        TimeSpan::parse_with_culture("6:12:14:45,348", Locale::en),
        Err(ParseError::InvalidFormat),
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn ts3(h: i64, m: i64, s: i64) -> TimeSpan {
    ts5(0, h, m, s, 0)
}

fn ts4(d: i64, h: i64, m: i64, s: i64) -> TimeSpan {
    ts5(d, h, m, s, 0)
}

fn ts5(d: i64, h: i64, m: i64, s: i64, ms: i64) -> TimeSpan {
    TimeSpan::from_ticks(
        d * TimeSpan::TICKS_PER_DAY
            + h * TimeSpan::TICKS_PER_HOUR
            + m * TimeSpan::TICKS_PER_MINUTE
            + s * TimeSpan::TICKS_PER_SECOND
            + ms * TimeSpan::TICKS_PER_MILLISECOND,
    )
}

// ── Parse_Valid_TestData ──────────────────────────────────────────────────────

#[test]
fn parse_valid_leading_whitespace() {
    assert_eq!(TimeSpan::parse("       12:24:02"), Ok(ts5(0, 12, 24, 2, 0)));
}

#[test]
fn parse_valid_trailing_whitespace() {
    assert_eq!(TimeSpan::parse("12:24:02      "), Ok(ts5(0, 12, 24, 2, 0)));
}

#[test]
fn parse_valid_surrounding_whitespace() {
    assert_eq!(TimeSpan::parse("     12:24:02      "), Ok(ts5(0, 12, 24, 2, 0)));
}

#[test]
fn parse_valid_bare_zero() {
    assert_eq!(TimeSpan::parse("0"), Ok(ts5(0, 0, 0, 0, 0)));
}

#[test]
fn parse_valid_hm() {
    assert_eq!(TimeSpan::parse("12:24"), Ok(ts5(0, 12, 24, 0, 0)));
}

#[test]
fn parse_valid_hms() {
    assert_eq!(TimeSpan::parse("12:24:02"), Ok(ts5(0, 12, 24, 2, 0)));
}

#[test]
fn parse_valid_d_dot_hm() {
    // "12.03:04" — days.hours:minutes (no seconds)
    assert_eq!(TimeSpan::parse("12.03:04"), Ok(ts4(12, 3, 4, 0)));
}

#[test]
fn parse_valid_fractional_two_digits() {
    assert_eq!(
        TimeSpan::parse_with_culture("12:24:02.01", Locale::en),
        Ok(ts5(0, 12, 24, 2, 10)),
    );
}

#[test]
fn parse_valid_fractional_trailing_zeros() {
    // .0 and .0000000 both mean zero sub-second ticks
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.0", Locale::en),
        Ok(ts3(1, 1, 1)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.0000000", Locale::en),
        Ok(ts3(1, 1, 1)),
    );
}

#[test]
fn parse_valid_fractional_precision() {
    // Each additional fractional digit adds sub-millisecond precision
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.1", Locale::en),
        Ok(ts5(0, 1, 1, 1, 100)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.01", Locale::en),
        Ok(ts5(0, 1, 1, 1, 10)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.001", Locale::en),
        Ok(ts5(0, 1, 1, 1, 1)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.0001", Locale::en),
        Ok(TimeSpan::from_ticks(36610001000)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.00001", Locale::en),
        Ok(TimeSpan::from_ticks(36610000100)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.000001", Locale::en),
        Ok(TimeSpan::from_ticks(36610000010)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:1.0000001", Locale::en),
        Ok(TimeSpan::from_ticks(36610000001)),
    );
}

#[test]
fn parse_valid_d_dot_hms() {
    assert_eq!(TimeSpan::parse("1.12:24:02"), Ok(ts4(1, 12, 24, 2)));
}

#[test]
fn parse_valid_d_colon_hms() {
    // Alternative form: days separated by colon instead of dot
    assert_eq!(TimeSpan::parse("1:12:24:02"), Ok(ts4(1, 12, 24, 2)));
}

#[test]
fn parse_valid_empty_seconds_with_fraction() {
    // "01.23:45:.67" — seconds component is empty, fraction is present
    assert_eq!(
        TimeSpan::parse_with_culture("01.23:45:.67", Locale::en),
        Ok(ts5(1, 23, 45, 0, 670)),
    );
}

#[test]
fn parse_valid_full_with_millis() {
    assert_eq!(
        TimeSpan::parse_with_culture("1.12:24:02.999", Locale::en),
        Ok(ts5(1, 12, 24, 2, 999)),
    );
}

#[test]
fn parse_valid_hm_fractional_precision() {
    // h:m:.fraction — seconds component empty
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:.1", Locale::en),
        Ok(TimeSpan::from_ticks(36601000000)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:.01", Locale::en),
        Ok(TimeSpan::from_ticks(36600100000)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:.001", Locale::en),
        Ok(TimeSpan::from_ticks(36600010000)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:.0001", Locale::en),
        Ok(TimeSpan::from_ticks(36600001000)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:.00001", Locale::en),
        Ok(TimeSpan::from_ticks(36600000100)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:.000001", Locale::en),
        Ok(TimeSpan::from_ticks(36600000010)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("1:1:.0000001", Locale::en),
        Ok(TimeSpan::from_ticks(36600000001)),
    );
}

#[test]
fn parse_valid_near_max_value() {
    // Values approaching TimeSpan.MaxValue
    assert_eq!(
        TimeSpan::parse("10675199"),
        Ok(TimeSpan::from_ticks(9223371936000000000)),
    );
    assert_eq!(
        TimeSpan::parse("10675199:00:00"),
        Ok(TimeSpan::from_ticks(9223371936000000000)),
    );
    assert_eq!(
        TimeSpan::parse("10675199:02:00:00"),
        Ok(TimeSpan::from_ticks(9223372008000000000)),
    );
    assert_eq!(
        TimeSpan::parse("10675199:02:48:00"),
        Ok(TimeSpan::from_ticks(9223372036800000000)),
    );
    assert_eq!(
        TimeSpan::parse("10675199:02:48:05"),
        Ok(TimeSpan::from_ticks(9223372036850000000)),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("10675199:02:48:05.4775", Locale::en),
        Ok(TimeSpan::from_ticks(9223372036854775000)),
    );
}

#[test]
fn parse_valid_common_values() {
    assert_eq!(TimeSpan::parse("00:00:59"), Ok(ts3(0, 0, 59)));
    assert_eq!(TimeSpan::parse("00:59:00"), Ok(ts3(0, 59, 0)));
    assert_eq!(TimeSpan::parse("23:00:00"), Ok(ts3(23, 0, 0)));
    // 24:00:00 is valid — parsed as days
    assert_eq!(TimeSpan::parse("24:00:00"), Ok(ts4(24, 0, 0, 0)));
}

#[test]
fn parse_valid_croatian_culture_comma_separator() {
    // hr-HR uses comma as the decimal separator
    assert_eq!(
        TimeSpan::parse_with_culture("6:12:14:45,348", Locale::hr),
        Ok(ts5(6, 12, 14, 45, 348)),
    );
}

// ── Parse_Invalid_TestData — FormatException cases ────────────────────────────

#[test]
fn parse_invalid_empty_string() {
    assert_eq!(TimeSpan::parse(""), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_lone_minus() {
    assert_eq!(TimeSpan::parse("-"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_garbage() {
    assert_eq!(TimeSpan::parse("garbage"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_date_like_string() {
    assert_eq!(TimeSpan::parse("12/12/12"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_trailing_colon() {
    assert_eq!(TimeSpan::parse("00:"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_negative_component() {
    assert_eq!(TimeSpan::parse("00:00:-01"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_embedded_null_chars() {
    assert_eq!(TimeSpan::parse("\x0012:34:56"), Err(ParseError::InvalidFormat));
    assert_eq!(TimeSpan::parse("1\x0002:34:56"), Err(ParseError::InvalidFormat));
    assert_eq!(TimeSpan::parse("12\x00:34:56"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_double_colon() {
    assert_eq!(TimeSpan::parse("00:00::00"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_trailing_colon_after_seconds() {
    assert_eq!(TimeSpan::parse("00:00:00:"), Err(ParseError::InvalidFormat));
}

#[test]
fn parse_invalid_too_many_components() {
    assert_eq!(
        TimeSpan::parse("00:00:00:00:00:00:00:00"),
        Err(ParseError::InvalidFormat),
    );
}

#[test]
fn parse_invalid_wrong_decimal_separator_for_culture() {
    // hr-HR expects comma; period is invalid
    assert_eq!(
        TimeSpan::parse_with_culture("6:12:14:45.3448", Locale::hr),
        Err(ParseError::InvalidFormat),
    );
}

// ── Parse_Invalid_TestData — OverflowException cases ─────────────────────────

#[test]
fn parse_overflow_too_many_fractional_digits() {
    assert_eq!(
        TimeSpan::parse("1:1:1.99999999"),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_overflow_days_exceed_max() {
    assert_eq!(TimeSpan::parse("2147483647"), Err(ParseError::Overflow));
    assert_eq!(TimeSpan::parse("2147483648"), Err(ParseError::Overflow));
    assert_eq!(TimeSpan::parse("10675200"), Err(ParseError::Overflow));
    assert_eq!(TimeSpan::parse("10675200:00:00"), Err(ParseError::Overflow));
}

#[test]
fn parse_overflow_exceeds_max_value() {
    assert_eq!(
        TimeSpan::parse("10675199:03:00:00"),
        Err(ParseError::Overflow),
    );
    assert_eq!(
        TimeSpan::parse("10675199:02:49:00"),
        Err(ParseError::Overflow),
    );
    assert_eq!(
        TimeSpan::parse("10675199:02:48:06"),
        Err(ParseError::Overflow),
    );
    assert_eq!(
        TimeSpan::parse("-10675199:02:48:06"),
        Err(ParseError::Overflow),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("10675199:02:48:05.4776", Locale::en),
        Err(ParseError::Overflow),
    );
    assert_eq!(
        TimeSpan::parse_with_culture("-10675199:02:48:05.4776", Locale::en),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_overflow_seconds_or_minutes_out_of_range() {
    assert_eq!(TimeSpan::parse("00:00:60"), Err(ParseError::Overflow));
    assert_eq!(TimeSpan::parse("00:60:00"), Err(ParseError::Overflow));
}

#[test]
fn parse_overflow_ambiguous_hour_colon() {
    // "24:00" is ambiguous — treated as hours exceeding max per-component range
    assert_eq!(TimeSpan::parse("24:00"), Err(ParseError::Overflow));
}
