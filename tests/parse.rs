// Tests ported from the C# reference implementation:
// https://github.com/dotnet/runtime/blob/main/src/libraries/System.Runtime/tests/System.Runtime.Tests/System/TimeSpanTests.cs
//
// C# test methods covered here: Parse_Valid_TestData (TimeSpanTests.cs#L1006-1078),
// Parse_Invalid_TestData (TimeSpanTests.cs#L1106-1142)
//
// Notes on translation:
// - C# `ArgumentNullException` for null input has no Rust equivalent (`&str` can't be null)
//   and is therefore omitted.
// - C# `new TimeSpan(h, m, s)`      → `ts3(h, m, s)`       (3-arg: hours, minutes, seconds)
// - C# `new TimeSpan(d, h, m, s)`   → `ts4(d, h, m, s)`    (4-arg: days, hours, minutes, seconds)
// - C# `new TimeSpan(d, h, m, s, ms)` → `ts5(d, h, m, s, ms)` (5-arg)
// - C# `new TimeSpan(ticks)`         → `TimeSpan::from_ticks(ticks)`

use cs_timespan::{Locale, TimeSpan};

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
        TimeSpan::parse_with_culture("6:12:14:45,348", Locale::en)
            .unwrap_err()
            .to_string(),
        r#"unexpected character ','; expected a digit
  "6:12:14:45,348"
             ^"#,
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

// TimeSpanTests.cs#L1009
#[test]
fn parse_valid_leading_whitespace() {
    assert_eq!(TimeSpan::parse("       12:24:02"), Ok(ts5(0, 12, 24, 2, 0)));
}

// TimeSpanTests.cs#L1010
#[test]
fn parse_valid_trailing_whitespace() {
    assert_eq!(TimeSpan::parse("12:24:02      "), Ok(ts5(0, 12, 24, 2, 0)));
}

// TimeSpanTests.cs#L1011
#[test]
fn parse_valid_surrounding_whitespace() {
    assert_eq!(
        TimeSpan::parse("     12:24:02      "),
        Ok(ts5(0, 12, 24, 2, 0))
    );
}

// TimeSpanTests.cs#L1014
#[test]
fn parse_valid_bare_zero() {
    assert_eq!(TimeSpan::parse("0"), Ok(ts5(0, 0, 0, 0, 0)));
}

// TimeSpanTests.cs#L1017
#[test]
fn parse_valid_hm() {
    assert_eq!(TimeSpan::parse("12:24"), Ok(ts5(0, 12, 24, 0, 0)));
}

// TimeSpanTests.cs#L1020
#[test]
fn parse_valid_hms() {
    assert_eq!(TimeSpan::parse("12:24:02"), Ok(ts5(0, 12, 24, 2, 0)));
}

// TimeSpanTests.cs#L1023
#[test]
fn parse_valid_d_dot_hm() {
    // "12.03:04" — days.hours:minutes (no seconds)
    assert_eq!(TimeSpan::parse("12.03:04"), Ok(ts4(12, 3, 4, 0)));
}

// TimeSpanTests.cs#L1026
#[test]
fn parse_valid_fractional_two_digits() {
    assert_eq!(
        TimeSpan::parse_with_culture("12:24:02.01", Locale::en),
        Ok(ts5(0, 12, 24, 2, 10)),
    );
}

// TimeSpanTests.cs#L1029-1030
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

// TimeSpanTests.cs#L1031-1037
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

// TimeSpanTests.cs#L1040
#[test]
fn parse_valid_d_dot_hms() {
    assert_eq!(TimeSpan::parse("1.12:24:02"), Ok(ts4(1, 12, 24, 2)));
}

// TimeSpanTests.cs#L1043
#[test]
fn parse_valid_d_colon_hms() {
    // Alternative form: days separated by colon instead of dot
    assert_eq!(TimeSpan::parse("1:12:24:02"), Ok(ts4(1, 12, 24, 2)));
}

// TimeSpanTests.cs#L1046
#[test]
fn parse_valid_empty_seconds_with_fraction() {
    // "01.23:45:.67" — seconds component is empty, fraction is present
    assert_eq!(
        TimeSpan::parse_with_culture("01.23:45:.67", Locale::en),
        Ok(ts5(1, 23, 45, 0, 670)),
    );
}

// TimeSpanTests.cs#L1049
#[test]
fn parse_valid_full_with_millis() {
    assert_eq!(
        TimeSpan::parse_with_culture("1.12:24:02.999", Locale::en),
        Ok(ts5(1, 12, 24, 2, 999)),
    );
}

// TimeSpanTests.cs#L1052-1058
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

// TimeSpanTests.cs#L1061-1066
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

// TimeSpanTests.cs#L1067-1070
#[test]
fn parse_valid_common_values() {
    assert_eq!(TimeSpan::parse("00:00:59"), Ok(ts3(0, 0, 59)));
    assert_eq!(TimeSpan::parse("00:59:00"), Ok(ts3(0, 59, 0)));
    assert_eq!(TimeSpan::parse("23:00:00"), Ok(ts3(23, 0, 0)));
    // 24:00:00 is valid — parsed as days
    assert_eq!(TimeSpan::parse("24:00:00"), Ok(ts4(24, 0, 0, 0)));
}

// TimeSpanTests.cs#L1076
#[test]
fn parse_valid_croatian_culture_comma_separator() {
    // hr-HR uses comma as the decimal separator
    assert_eq!(
        TimeSpan::parse_with_culture("6:12:14:45,348", Locale::hr),
        Ok(ts5(6, 12, 14, 45, 348)),
    );
}

// ── Parse_Invalid_TestData — FormatException cases ────────────────────────────

// TimeSpanTests.cs#L1110
#[test]
fn parse_invalid_empty_string() {
    assert_eq!(
        TimeSpan::parse("").unwrap_err().to_string(),
        r#"input is empty
  ""
   ^"#,
    );
}

// TimeSpanTests.cs#L1111
#[test]
fn parse_invalid_lone_minus() {
    assert_eq!(
        TimeSpan::parse("-").unwrap_err().to_string(),
        r#"input is empty
  "-"
   ^"#,
    );
}

// TimeSpanTests.cs#L1112
#[test]
fn parse_invalid_garbage() {
    assert_eq!(
        TimeSpan::parse("garbage").unwrap_err().to_string(),
        r#"unexpected character 'g'; expected a digit
  "garbage"
   ^"#,
    );
}

// TimeSpanTests.cs#L1113
#[test]
fn parse_invalid_date_like_string() {
    assert_eq!(
        TimeSpan::parse("12/12/12").unwrap_err().to_string(),
        r#"unexpected character '/'; expected a digit
  "12/12/12"
     ^"#,
    );
}

// TimeSpanTests.cs#L1114
#[test]
fn parse_invalid_trailing_colon() {
    assert_eq!(
        TimeSpan::parse("00:").unwrap_err().to_string(),
        r#"unrecognised input structure; expected [-][d.]h:mm[:ss[.FFFFFFF]] or [-]d:h:mm:ss[.FFFFFFF]
  "00:"
      ^"#,
    );
}

// TimeSpanTests.cs#L1115
#[test]
fn parse_invalid_negative_component() {
    assert_eq!(
        TimeSpan::parse("00:00:-01").unwrap_err().to_string(),
        r#"unexpected character '-'; expected a digit
  "00:00:-01"
         ^"#,
    );
}

// str::parse::<u64>() accepts an optional leading '+' on its own (e.g. "+123" -> Ok(123)),
// which C#'s TimeSpan parsing does not — the digit-only pre-check must reject it before
// parse::<u64>() ever runs, since parse::<u64>() succeeding wouldn't give us a chance to
// reject it after the fact.
#[test]
fn parse_invalid_leading_plus_component() {
    assert_eq!(
        TimeSpan::parse("+12:24:02").unwrap_err().to_string(),
        r#"unexpected character '+'; expected a digit
  "+12:24:02"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse("00:00:+01").unwrap_err().to_string(),
        r#"unexpected character '+'; expected a digit
  "00:00:+01"
         ^"#,
    );
}

// TimeSpanTests.cs#L1116-1118
#[test]
fn parse_invalid_embedded_null_chars() {
    assert_eq!(
        TimeSpan::parse("\x0012:34:56").unwrap_err().to_string(),
        "unexpected character '\\0'; expected a digit\n  \"\x0012:34:56\"\n   ^",
    );
    assert_eq!(
        TimeSpan::parse("1\x0002:34:56").unwrap_err().to_string(),
        "unexpected character '\\0'; expected a digit\n  \"1\x0002:34:56\"\n    ^",
    );
    assert_eq!(
        TimeSpan::parse("12\x00:34:56").unwrap_err().to_string(),
        "unexpected character '\\0'; expected a digit\n  \"12\x00:34:56\"\n     ^",
    );
}

// TimeSpanTests.cs#L1119
#[test]
fn parse_invalid_double_colon() {
    assert_eq!(
        TimeSpan::parse("00:00::00").unwrap_err().to_string(),
        r#"unrecognised input structure; expected [-][d.]h:mm[:ss[.FFFFFFF]] or [-]d:h:mm:ss[.FFFFFFF]
  "00:00::00"
         ^"#,
    );
}

// TimeSpanTests.cs#L1120
#[test]
fn parse_invalid_trailing_colon_after_seconds() {
    assert_eq!(
        TimeSpan::parse("00:00:00:").unwrap_err().to_string(),
        r#"unrecognised input structure; expected [-][d.]h:mm[:ss[.FFFFFFF]] or [-]d:h:mm:ss[.FFFFFFF]
  "00:00:00:"
            ^"#,
    );
}

// TimeSpanTests.cs#L1121
#[test]
fn parse_invalid_too_many_components() {
    assert_eq!(
        TimeSpan::parse("00:00:00:00:00:00:00:00")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected [-][d.]h:mm[:ss[.FFFFFFF]] or [-]d:h:mm:ss[.FFFFFFF]
  "00:00:00:00:00:00:00:00"
               ^"#,
    );
}

// The caret must point at the first excess component, not always at position 0
// regardless of where the excess actually starts.
#[test]
fn parse_invalid_too_many_components_position() {
    assert_eq!(
        TimeSpan::parse("1:2:3:4:5").unwrap_err().to_string(),
        r#"unrecognised input structure; expected [-][d.]h:mm[:ss[.FFFFFFF]] or [-]d:h:mm:ss[.FFFFFFF]
  "1:2:3:4:5"
           ^"#,
    );
}

// TimeSpanTests.cs#L1124
#[test]
fn parse_invalid_wrong_decimal_separator_for_culture() {
    // hr-HR expects comma; period is invalid
    assert_eq!(
        TimeSpan::parse_with_culture("6:12:14:45.3448", Locale::hr)
            .unwrap_err()
            .to_string(),
        r#"decimal separator does not match the locale
  "6:12:14:45.3448"
             ^"#,
    );
}

// ── Parse_Invalid_TestData — OverflowException cases ─────────────────────────

// C# TimeSpanParse.cs NormalizeAndValidateFraction (line 148): fractions longer than 7 digits
// are accepted when they have enough leading zeros that the significant value fits in 7 digits;
// the value is rounded to the nearest tick. Fractions with no leading zeros and > 7 digits
// overflow because their integer value exceeds MaxFraction (9_999_999).
// TimeSpanTests.cs#L1127
#[test]
fn parse_overflow_too_many_fractional_digits() {
    // No leading zeros: value (99999999) > MaxFraction → Overflow
    assert_eq!(
        TimeSpan::parse("1:1:1.99999999").unwrap_err().to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "1:1:1.99999999"
         ^"#,
    );
}

#[test]
fn parse_frac_leading_zeros_beyond_7_rounds_to_nearest_tick() {
    // ".00000005" (8 digits, 7 leading zeros): 0.5 ticks → rounds up to 1
    assert_eq!(TimeSpan::parse("0:0:0.00000005").unwrap().ticks(), 1);
    // ".00000050" (8 digits, 6 leading zeros): 5.0 ticks → rounds to 5
    assert_eq!(TimeSpan::parse("0:0:0.00000050").unwrap().ticks(), 5);
    // ".000000000" (9 digits, all zeros): rounds to 0
    assert_eq!(TimeSpan::parse("0:0:0.000000000").unwrap().ticks(), 0);
}

// TimeSpanTests.cs#L1129-1132
#[test]
fn parse_overflow_days_exceed_max() {
    assert_eq!(
        TimeSpan::parse("2147483647").unwrap_err().to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "2147483647"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse("2147483648").unwrap_err().to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "2147483648"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse("10675200").unwrap_err().to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "10675200"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse("10675200:00:00").unwrap_err().to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "10675200:00:00"
   ^"#,
    );
}

// TimeSpanTests.cs#L1133-1138
#[test]
fn parse_overflow_exceeds_max_value() {
    assert_eq!(
        TimeSpan::parse("10675199:03:00:00")
            .unwrap_err()
            .to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "10675199:03:00:00"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse("10675199:02:49:00")
            .unwrap_err()
            .to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "10675199:02:49:00"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse("10675199:02:48:06")
            .unwrap_err()
            .to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "10675199:02:48:06"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse("-10675199:02:48:06")
            .unwrap_err()
            .to_string(),
        r#"TimeSpan value is below the minimum representable range
  "-10675199:02:48:06"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse_with_culture("10675199:02:48:05.4776", Locale::en)
            .unwrap_err()
            .to_string(),
        r#"TimeSpan value exceeds the maximum representable range
  "10675199:02:48:05.4776"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse_with_culture("-10675199:02:48:05.4776", Locale::en)
            .unwrap_err()
            .to_string(),
        r#"TimeSpan value is below the minimum representable range
  "-10675199:02:48:05.4776"
   ^"#,
    );
}

// TimeSpanTests.cs#L1139-1140
#[test]
fn parse_overflow_seconds_or_minutes_out_of_range() {
    assert_eq!(
        TimeSpan::parse("00:00:60").unwrap_err().to_string(),
        r#"seconds value 60 is out of range; must be 0-59
  "00:00:60"
         ^"#,
    );
    assert_eq!(
        TimeSpan::parse("00:60:00").unwrap_err().to_string(),
        r#"minutes value 60 is out of range; must be 0-59
  "00:60:00"
      ^"#,
    );
}

// offset_of computes positions via pointer arithmetic on subslices of the original
// input, not by searching for the value's text — so a coincidental earlier
// occurrence of the same digits must not shift the reported position.
#[test]
fn parse_overflow_position_not_confused_by_earlier_duplicate_value() {
    // "61" appears twice: as the (valid, since > 23) days component, and as the
    // out-of-range minutes component. The caret must point at the second one.
    assert_eq!(
        TimeSpan::parse("61:10:61").unwrap_err().to_string(),
        r#"minutes value 61 is out of range; must be 0-59
  "61:10:61"
         ^"#,
    );
}

// TimeSpanTests.cs#L1141
#[test]
fn parse_overflow_ambiguous_hour_colon() {
    // "24:00" is ambiguous — treated as hours exceeding max per-component range
    assert_eq!(
        TimeSpan::parse("24:00").unwrap_err().to_string(),
        r#"hours value 24 is out of range; must be 0-23
  "24:00"
   ^"#,
    );
}

#[test]
fn parse_invalid_whitespace_only() {
    // Whitespace-only input collapses to empty after trimming → Empty error.
    // C# TimeSpanParse.cs: SkipBlanks leaves _pos past all whitespace, then
    // the first structural check fails with SetBadTimeSpanFailure.
    assert_eq!(
        TimeSpan::parse("   ").unwrap_err().to_string(),
        r#"input is empty
  "   "
   ^"#,
    );
}
