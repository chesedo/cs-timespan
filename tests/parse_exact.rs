// Tests ported from the C# reference implementation:
// https://github.com/dotnet/corefx/blob/master/src/System.Runtime/tests/System/TimeSpanTests.cs
//
// C# test methods covered here: ParseExact_Valid_TestData, ParseExact_Invalid_TestData
//
// Notes on translation:
// - C# `ArgumentNullException` for null input/format has no Rust equivalent and is omitted.
// - C# `new TimeSpan(h, m, s)`        → `ts3(h, m, s)`
// - C# `new TimeSpan(d, h, m, s)`     → `ts4(d, h, m, s)`
// - C# `new TimeSpan(d, h, m, s, ms)` → `ts5(d, h, m, s, ms)`
// - C# `-new TimeSpan(...)` (negation) → `neg(ts*(...))`

use cs_timespan::{Locale, ParseError, TimeSpan, TimeSpanStyles};

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

fn neg(t: TimeSpan) -> TimeSpan {
    TimeSpan::from_ticks(-t.ticks())
}

// ── ParseExact_Valid — constant format "c" / "t" / "T" ───────────────────────

#[test]
fn parse_exact_constant_format_hms() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::parse_exact("12:24:02", fmt),
            Ok(ts4(0, 12, 24, 2)),
            "format={fmt:?}",
        );
    }
}

#[test]
fn parse_exact_constant_format_d_dot_hms() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::parse_exact("1.12:24:02", fmt),
            Ok(ts4(1, 12, 24, 2)),
            "format={fmt:?}",
        );
    }
}

#[test]
fn parse_exact_constant_format_negative_with_millis() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::parse_exact("-01.07:45:16.999", fmt),
            Ok(neg(ts5(1, 7, 45, 16, 999))),
            "format={fmt:?}",
        );
    }
}

// ── ParseExact_Valid — general short format "g" ───────────────────────────────

#[test]
fn parse_exact_g_bare_integer_is_days() {
    assert_eq!(TimeSpan::parse_exact("12", "g"), Ok(ts4(12, 0, 0, 0)));
}

#[test]
fn parse_exact_g_negative_days() {
    assert_eq!(
        TimeSpan::parse_exact("-12", "g"),
        Ok(TimeSpan::from_ticks(ts4(-12, 0, 0, 0).ticks())),
    );
}

#[test]
fn parse_exact_g_hm() {
    assert_eq!(TimeSpan::parse_exact("12:34", "g"), Ok(ts3(12, 34, 0)));
}

#[test]
fn parse_exact_g_negative_hm() {
    assert_eq!(
        TimeSpan::parse_exact("-12:34", "g"),
        Ok(neg(ts3(12, 34, 0)))
    );
}

#[test]
fn parse_exact_g_hm_with_fraction() {
    assert_eq!(
        TimeSpan::parse_exact("1:2:.3", "g"),
        Ok(ts5(0, 1, 2, 0, 300)),
    );
    assert_eq!(
        TimeSpan::parse_exact("-1:2:.3", "g"),
        Ok(neg(ts5(0, 1, 2, 0, 300))),
    );
}

#[test]
fn parse_exact_g_hms() {
    assert_eq!(
        TimeSpan::parse_exact("12:24:02", "g"),
        Ok(ts4(0, 12, 24, 2)),
    );
}

#[test]
fn parse_exact_g_hms_with_millis() {
    assert_eq!(
        TimeSpan::parse_exact("12:24:02.123", "g"),
        Ok(ts5(0, 12, 24, 2, 123)),
    );
    assert_eq!(
        TimeSpan::parse_exact("-12:24:02.123", "g"),
        Ok(neg(ts5(0, 12, 24, 2, 123))),
    );
}

#[test]
fn parse_exact_g_d_hm_with_fraction() {
    assert_eq!(
        TimeSpan::parse_exact("1:2:3:.4", "g"),
        Ok(ts5(1, 2, 3, 0, 400)),
    );
    assert_eq!(
        TimeSpan::parse_exact("-1:2:3:.4", "g"),
        Ok(neg(ts5(1, 2, 3, 0, 400))),
    );
}

#[test]
fn parse_exact_g_d_hms() {
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02", "g"),
        Ok(ts4(1, 12, 24, 2)),
    );
}

#[test]
fn parse_exact_g_negative_full() {
    assert_eq!(
        TimeSpan::parse_exact("-01:07:45:16.999", "g"),
        Ok(neg(ts5(1, 7, 45, 16, 999))),
    );
}

// ── ParseExact_Valid — general long format "G" ────────────────────────────────

#[test]
fn parse_exact_g_upper_d_hms_with_millis() {
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02.243", "G"),
        Ok(ts5(1, 12, 24, 2, 243)),
    );
}

#[test]
fn parse_exact_g_upper_negative() {
    assert_eq!(
        TimeSpan::parse_exact("-01:07:45:16.999", "G"),
        Ok(neg(ts5(1, 7, 45, 16, 999))),
    );
}

// ── ParseExact_Valid — custom format specifiers ───────────────────────────────

#[test]
fn parse_exact_custom_dd_dot_h_m_s() {
    assert_eq!(
        TimeSpan::parse_exact("12.23:32:43", r"dd\.h\:m\:s"),
        Ok(ts4(12, 23, 32, 43)),
    );
}

#[test]
fn parse_exact_custom_ddd_dot_h_m_s_fff() {
    assert_eq!(
        TimeSpan::parse_exact("012.23:32:43.893", r"ddd\.h\:m\:s\.fff"),
        Ok(ts5(12, 23, 32, 43, 893)),
    );
}

#[test]
fn parse_exact_custom_d_dot_hh_mm_ss() {
    assert_eq!(
        TimeSpan::parse_exact("12.05:02:03", r"d\.hh\:mm\:ss"),
        Ok(ts4(12, 5, 2, 3)),
    );
}

#[test]
fn parse_exact_custom_literal_word_backslash_escaped() {
    assert_eq!(
        TimeSpan::parse_exact(r"12:34 minutes", r"mm\:ss\ \m\i\n\u\t\e\s"),
        Ok(ts3(0, 12, 34)),
    );
}

#[test]
fn parse_exact_custom_literal_word_double_quoted() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r#"mm\:ss\ "minutes""#),
        Ok(ts3(0, 12, 34)),
    );
}

#[test]
fn parse_exact_custom_literal_word_single_quoted() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r"mm\:ss\ 'minutes'"),
        Ok(ts3(0, 12, 34)),
    );
}

#[test]
fn parse_exact_custom_fff_lowercase() {
    assert_eq!(
        TimeSpan::parse_exact("678", "fff"),
        Ok(ts5(0, 0, 0, 0, 678)),
    );
}

#[test]
fn parse_exact_custom_fff_uppercase_optional_digits() {
    assert_eq!(
        TimeSpan::parse_exact("678", "FFF"),
        Ok(ts5(0, 0, 0, 0, 678)),
    );
}

#[test]
fn parse_exact_custom_percent_specifiers() {
    assert_eq!(TimeSpan::parse_exact("3", "%d"), Ok(ts5(3, 0, 0, 0, 0)));
    assert_eq!(TimeSpan::parse_exact("3", "%h"), Ok(ts3(3, 0, 0)));
    assert_eq!(TimeSpan::parse_exact("3", "%m"), Ok(ts3(0, 3, 0)));
    assert_eq!(TimeSpan::parse_exact("3", "%s"), Ok(ts3(0, 0, 3)));
    assert_eq!(TimeSpan::parse_exact("3", "%f"), Ok(ts5(0, 0, 0, 0, 300)));
    assert_eq!(TimeSpan::parse_exact("3", "%F"), Ok(ts5(0, 0, 0, 0, 300)));
}

// ── ParseExact_Invalid — FormatException cases ────────────────────────────────

#[test]
fn parse_exact_invalid_empty_string() {
    assert_eq!(TimeSpan::parse_exact("", "c"), Err(ParseError::Empty),);
}

#[test]
fn parse_exact_invalid_lone_minus() {
    assert_eq!(TimeSpan::parse_exact("-", "c"), Err(ParseError::Empty),);
}

#[test]
fn parse_exact_invalid_garbage() {
    // "garbage" has 0 colons; "c" format requires exactly 2
    assert_eq!(
        TimeSpan::parse_exact("garbage", "c"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_wrong_separator() {
    // '?' replaces a colon → wrong colon count → InvalidStructure
    assert_eq!(
        TimeSpan::parse_exact("1?59:02", "c"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("1:59?02", "c"),
        Err(ParseError::InvalidStructure),
    );
    // '?' replaces the decimal separator → appears as non-digit inside a component
    assert_eq!(
        TimeSpan::parse_exact("1:59:02?123", "c"),
        Err(ParseError::InvalidCharacter),
    );
}

#[test]
fn parse_exact_c_rejects_d_colon_form() {
    // "c" format uses dot separator for days; colon-separated days is only valid in "g"
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02", "c"),
        Err(ParseError::InvalidCharacter),
    );
}

#[test]
fn parse_exact_g_rejects_dot_separated_days() {
    assert_eq!(
        TimeSpan::parse_exact("1.12:24:02", "g"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_g_upper_rejects_colon_without_fractional() {
    // "G" requires the full d:hh:mm:ss.fffffff pattern
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02", "G"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_empty_format_string() {
    assert_eq!(
        TimeSpan::parse_exact("00:00:00", ""),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_unknown_format_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12.5:2", "V"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_percent_not_alone() {
    assert_eq!(
        TimeSpan::parse_exact("1", r"d%"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("1", r"%%d"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_repeated_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hhh\:mm\:ss"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:hh\:ss"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:mm\:mm"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:ss\:ss"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_wrong_digit_count() {
    // Digit count mismatch causes the subsequent literal separator to not match
    assert_eq!(
        TimeSpan::parse_exact("123:34:56", r"hh\:mm\:ss"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:345:56", r"hh\:mm\:ss"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:056", r"hh\:mm\:ss"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_triple_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:mmm\:ss"),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:mm\:sss"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_f_wrong_digit_count() {
    // "ffff" expects exactly 4 fractional digits; "678" is only 3 → input too short
    assert_eq!(
        TimeSpan::parse_exact("678", "ffff"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_f_uppercase_too_many_chars() {
    assert_eq!(
        TimeSpan::parse_exact("00000012", "FFFFFFFF"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_d_too_many_specifiers() {
    // Max is dddddddd (8 d's)
    assert_eq!(
        TimeSpan::parse_exact("000000123", "ddddddddd"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_duplicate_d_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"dd:dd:hh"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_too_many_digits_for_dd() {
    assert_eq!(
        TimeSpan::parse_exact("123:45", r"dd:hh"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_unknown_specifier_vv() {
    assert_eq!(
        TimeSpan::parse_exact("12:34", r"dd:vv"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_ff_repeated() {
    assert_eq!(
        TimeSpan::parse_exact("12:45", "ff:ff"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_unclosed_literal_double_quote() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r#"mm\:ss\ "minutes"#),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_unclosed_literal_single_quote() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r"mm\:ss\ 'minutes"),
        Err(ParseError::InvalidStructure),
    );
}

#[test]
fn parse_exact_invalid_literal_mismatch() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 mints", r#"mm\:ss\ "minutes""#),
        Err(ParseError::InvalidStructure),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34 mints", r"mm\:ss\ 'minutes'"),
        Err(ParseError::InvalidStructure),
    );
}

// ── parse_exact_with_styles (TimeSpanStyles::AssumeNegative) ─────────────────

#[test]
fn parse_exact_with_styles_none_matches_parse_exact() {
    // TimeSpanStyles::None produces identical output to parse_exact.
    assert_eq!(
        TimeSpan::parse_exact_with_styles(
            "1:02:03",
            r"h\:mm\:ss",
            Locale::en,
            TimeSpanStyles::None
        ),
        Ok(ts4(0, 1, 2, 3)),
    );
}

#[test]
fn parse_exact_with_styles_assume_negative_custom_format() {
    // Without a leading '-' in the input, AssumeNegative flips the sign.
    assert_eq!(
        TimeSpan::parse_exact_with_styles(
            "1:02:03",
            r"h\:mm\:ss",
            Locale::en,
            TimeSpanStyles::AssumeNegative
        ),
        Ok(neg(ts4(0, 1, 2, 3))),
    );
}

#[test]
fn parse_exact_with_styles_assume_negative_standard_format() {
    assert_eq!(
        TimeSpan::parse_exact_with_styles(
            "01:02:03",
            "c",
            Locale::en,
            TimeSpanStyles::AssumeNegative
        ),
        Ok(neg(ts4(0, 1, 2, 3))),
    );
}

#[test]
fn parse_exact_with_styles_assume_negative_zero_stays_zero() {
    // Negating zero produces zero.
    assert_eq!(
        TimeSpan::parse_exact_with_styles("0", "%d", Locale::en, TimeSpanStyles::AssumeNegative),
        Ok(TimeSpan::ZERO),
    );
}

// ── ParseExact_Invalid — OverflowException cases ──────────────────────────────

#[test]
fn parse_exact_overflow_hours_out_of_range() {
    assert_eq!(
        TimeSpan::parse_exact("24:24:02", "c"),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_exact_overflow_minutes_out_of_range() {
    assert_eq!(
        TimeSpan::parse_exact("1:60:02", "c"),
        Err(ParseError::Overflow),
    );
    assert_eq!(
        TimeSpan::parse_exact("1.2:60:02", "c"),
        Err(ParseError::Overflow),
    );
    assert_eq!(
        TimeSpan::parse_exact("12:61:02", "g"),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_exact_overflow_seconds_out_of_range() {
    assert_eq!(
        TimeSpan::parse_exact("1:59:60", "c"),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_exact_overflow_hours_exceed_23_in_c_format() {
    // "c" format hours must be 0–23; 24 hours overflows
    assert_eq!(
        TimeSpan::parse_exact("1.24:59:02", "c"),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_exact_overflow_g_upper_too_many_fractional_digits() {
    assert_eq!(
        TimeSpan::parse_exact("1:07:45:16.99999999", "G"),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_exact_overflow_custom_format() {
    // 35 hours exceeds valid range for "h" specifier
    assert_eq!(
        TimeSpan::parse_exact("12.35:32:43", r"dd\.h\:m\:s"),
        Err(ParseError::Overflow),
    );
}

#[test]
fn parse_exact_invalid_custom_wrong_digit_count_for_padded() {
    // "hh" needs 2 digits; input only has 1 then hits ':', which is a non-digit character
    assert_eq!(
        TimeSpan::parse_exact("12.5:2:3", r"d\.hh\:mm\:ss"),
        Err(ParseError::InvalidCharacter),
    );
    assert_eq!(
        TimeSpan::parse_exact("12.5:2", r"d\.hh\:mm\:ss"),
        Err(ParseError::InvalidCharacter),
    );
}
