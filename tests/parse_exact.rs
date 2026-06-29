// Tests ported from the C# reference implementation:
// https://github.com/dotnet/corefx/blob/master/src/System.Runtime/tests/System/TimeSpanTests.cs
//
// C# test methods covered here: ParseExact_Valid_TestData, ParseExact_Invalid_TestData
//
// Additional tests derived from the C# documentation examples:
// - https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings
// - https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings
//
// Notes on translation:
// - C# `ArgumentNullException` for null input/format has no Rust equivalent and is omitted.
// - C# `new TimeSpan(h, m, s)`        → `ts3(h, m, s)`
// - C# `new TimeSpan(d, h, m, s)`     → `ts4(d, h, m, s)`
// - C# `new TimeSpan(d, h, m, s, ms)` → `ts5(d, h, m, s, ms)`
// - C# `-new TimeSpan(...)` (negation) → `neg(ts*(...))`

use cs_timespan::{Locale, TimeSpan, TimeSpanStyles};

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

// C# TimeSpanParse.cs ParseTime (line 1384): `if (_ch == ':')` makes the second colon
// and seconds component optional — "hh:mm" and "d.hh:mm" are valid "c" inputs.
#[test]
fn parse_exact_c_hm_no_seconds() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::parse_exact("01:02", fmt),
            Ok(ts3(1, 2, 0)),
            "format={fmt:?}",
        );
        assert_eq!(
            TimeSpan::parse_exact("1.02:03", fmt),
            Ok(ts4(1, 2, 3, 0)),
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

// C# DateTimeParse.TryParseQuoteString (DateTimeParse.cs line 4600): '\' inside a
// quoted literal is an escape — '\X' appends X, allowing the quote char to be embedded.
#[test]
fn parse_exact_custom_backslash_escape_inside_quote() {
    // Format: "5\:00" (double-quoted literal, '\:' inside the quote = literal ':')
    // matches input "5:00" and leaves no specifiers → 0 ticks.
    assert_eq!(
        TimeSpan::parse_exact("5:00", r#""5\:00""#),
        Ok(ts4(0, 0, 0, 0)),
    );
    // Backslash-escape of the quote char itself: '5\'s' → literal "5's"
    assert_eq!(TimeSpan::parse_exact("5's", "'5\\'s'"), Ok(ts4(0, 0, 0, 0)),);
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

// C# TryParseByFormat (TimeSpanParse.cs line 1317): ParseExactDigits return value is
// ignored for 'F' specifiers — zero matched digits is valid (fraction defaults to 0).
#[test]
fn parse_exact_custom_uppercase_f_zero_digits_accepted() {
    // Format has fractional part but input ends after the separator — fraction = 0
    assert_eq!(
        TimeSpan::parse_exact("5.", r"d\.FFFFFFF"),
        Ok(ts4(5, 0, 0, 0)),
    );
    assert_eq!(
        TimeSpan::parse_exact("1:02:03.", r"h\:mm\:ss\.FFFFFFF"),
        Ok(ts3(1, 2, 3)),
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

// C# TryParseByFormat (TimeSpanParse.cs): after consuming '%', the next character is
// re-fed into the full switch including ParseRepeatPattern, so '%' is a transparent
// pass-through that enables run-length specifiers and other token types.
#[test]
fn parse_exact_custom_percent_passthrough() {
    // %dd → same as dd (exact 2-digit days)
    assert_eq!(TimeSpan::parse_exact("05", "%dd"), Ok(ts4(5, 0, 0, 0)),);
    // %'literal' → match the quoted literal
    assert_eq!(
        TimeSpan::parse_exact("minutes", r#"%"minutes""#),
        Ok(ts4(0, 0, 0, 0)),
    );
    // %\X → match the escaped literal character
    assert_eq!(TimeSpan::parse_exact(":", r"%\:"), Ok(ts4(0, 0, 0, 0)),);
}

// ── ParseExact_Invalid — FormatException cases ────────────────────────────────

#[test]
fn parse_exact_invalid_empty_string() {
    assert_eq!(
        TimeSpan::parse_exact("", "c").unwrap_err().to_string(),
        r#"input is empty
  ""
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_lone_minus() {
    assert_eq!(
        TimeSpan::parse_exact("-", "c").unwrap_err().to_string(),
        r#"input is empty
  "-"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_garbage() {
    // "garbage" has 0 colons; "c" format requires exactly 2
    assert_eq!(
        TimeSpan::parse_exact("garbage", "c")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected [-][d.]hh:mm[:ss[.fffffff]]
  "garbage"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_wrong_separator() {
    // '?' replaces the first colon → day_hour = "1?59", no dot, hours = "1?59"
    // → non-digit in hours field → NonDigit
    assert_eq!(
        TimeSpan::parse_exact("1?59:02", "c")
            .unwrap_err()
            .to_string(),
        r#"unexpected character '?'; expected a digit
  "1?59:02"
    ^"#,
    );
    // '?' replaces the second colon → minutes = "59?02", non-digit → NonDigit
    // (seconds are optional since C# ParseTime line 1384; the '?' lands in minutes)
    assert_eq!(
        TimeSpan::parse_exact("1:59?02", "c")
            .unwrap_err()
            .to_string(),
        r#"unexpected character '?'; expected a digit
  "1:59?02"
       ^"#,
    );
    // '?' replaces the decimal separator → appears as non-digit inside a component
    assert_eq!(
        TimeSpan::parse_exact("1:59:02?123", "c")
            .unwrap_err()
            .to_string(),
        r#"unexpected character '?'; expected a digit
  "1:59:02?123"
          ^"#,
    );
}

#[test]
fn parse_exact_c_rejects_d_colon_form() {
    // "c" format uses dot separator for days; colon-separated days is only valid in "g"
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02", "c")
            .unwrap_err()
            .to_string(),
        r#"unexpected character ':'; expected a digit
  "1:12:24:02"
          ^"#,
    );
}

#[test]
fn parse_exact_g_rejects_dot_separated_days() {
    assert_eq!(
        TimeSpan::parse_exact("1.12:24:02", "g")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected [-][d:]h:mm[:ss[.FFFFFFF]]
  "1.12:24:02"
   ^"#,
    );
}

#[test]
fn parse_exact_g_upper_rejects_colon_without_fractional() {
    // "G" requires the full d:hh:mm:ss.fffffff pattern
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02", "G")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected [-]d:hh:mm:ss.fffffff
  "1:12:24:02"
           ^"#,
    );
}

#[test]
fn parse_exact_invalid_empty_format_string() {
    assert_eq!(
        TimeSpan::parse_exact("00:00:00", "")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: empty format string
  ""
   ^"#,
    );
}

// C# TryParseExactTimeSpan (TimeSpanParse.cs line 1228): only dispatches to
// TryParseByFormat when format.Length >= 2; a single non-standard letter is an
// invalid format specifier.
#[test]
fn parse_exact_invalid_single_char_custom_format() {
    assert_eq!(
        TimeSpan::parse_exact("5", "d").unwrap_err().to_string(),
        r#"invalid custom format: 'd' must be prefixed with '%' when used alone (e.g. '%d'); valid specifiers: d, h, m, s, f, F
  "d"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("5", "h").unwrap_err().to_string(),
        r#"invalid custom format: 'h' must be prefixed with '%' when used alone (e.g. '%h'); valid specifiers: d, h, m, s, f, F
  "h"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_unknown_format_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12.5:2", "V")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: 'V' is not a known format specifier; valid specifiers: d, h, m, s, f, F
  "V"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_percent_not_alone() {
    assert_eq!(
        TimeSpan::parse_exact("1", r"d%").unwrap_err().to_string(),
        r#"invalid custom format: '%' at end of format must be followed by a specifier
  "d%"
    ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("1", r"%%d").unwrap_err().to_string(),
        r#"invalid custom format: '%%' is not valid; '%' must be followed by a single specifier character
  "%%d"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_repeated_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:hh\:ss")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate 'hh' specifier in format
  "hh\:hh\:ss"
       ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:mm\:mm")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate 'mm' specifier in format
  "hh\:mm\:mm"
           ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:ss\:ss")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate 'ss' specifier in format
  "hh\:ss\:ss"
           ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"dd\:dd\:hh")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate 'dd' specifier in format
  "dd\:dd\:hh"
       ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:45", r"ff\:ff")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate 'ff' specifier in format
  "ff\:ff"
       ^"#,
    );
}

#[test]
fn parse_exact_invalid_wrong_digit_count() {
    // Digit count mismatch causes the subsequent literal separator to not match
    assert_eq!(
        TimeSpan::parse_exact("123:34:56", r"hh\:mm\:ss")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected hh\:mm\:ss
  "123:34:56"
     ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:345:56", r"hh\:mm\:ss")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected hh\:mm\:ss
  "12:345:56"
        ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:056", r"hh\:mm\:ss")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected hh\:mm\:ss
  "12:34:056"
           ^"#,
    );
}

#[test]
fn parse_exact_invalid_triple_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hhh\:mm\:ss")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: 'h' repeated 3 times; maximum is 2
  "hhh\:mm\:ss"
   ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:mmm\:ss")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: 'm' repeated 3 times; maximum is 2
  "hh\:mmm\:ss"
       ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:56", r"hh\:mm\:sss")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: 's' repeated 3 times; maximum is 2
  "hh\:mm\:sss"
           ^"#,
    );
}

#[test]
fn parse_exact_invalid_f_wrong_digit_count() {
    // "ffff" expects exactly 4 fractional digits; "678" is only 3 → input too short
    assert_eq!(
        TimeSpan::parse_exact("678", "ffff")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected ffff
  "678"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_f_uppercase_too_many_chars() {
    assert_eq!(
        TimeSpan::parse_exact("00000012", "FFFFFFFF")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: 'F' repeated 8 times; maximum is 7
  "FFFFFFFF"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_d_too_many_specifiers() {
    // Max is dddddddd (8 d's)
    assert_eq!(
        TimeSpan::parse_exact("000000123", "ddddddddd")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: 'd' repeated 9 times; maximum is 8
  "ddddddddd"
   ^"#,
    );
}

#[test]
fn parse_exact_invalid_duplicate_percent_h_specifier() {
    assert_eq!(
        TimeSpan::parse_exact("12:34", r"%h\:%h")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate '%h' specifier in format
  "%h\:%h"
       ^"#,
    );
    // Mixed: hh then %h — second (%h) is named in the error
    assert_eq!(
        TimeSpan::parse_exact("12:34", r"hh\:%h")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate '%h' specifier in format
  "hh\:%h"
       ^"#,
    );
    // Mixed: %h then hh — second (hh) is named in the error
    assert_eq!(
        TimeSpan::parse_exact("12:34", r"%h\:hh")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: duplicate 'hh' specifier in format
  "%h\:hh"
       ^"#,
    );
}

#[test]
fn parse_exact_invalid_too_many_digits_for_dd() {
    assert_eq!(
        TimeSpan::parse_exact("123:45", r"dd\:hh")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected dd\:hh
  "123:45"
     ^"#,
    );
}

#[test]
fn parse_exact_invalid_unknown_specifier_vv() {
    assert_eq!(
        TimeSpan::parse_exact("12:34", r"dd\:vv")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: unrecognised character 'v' in format string; valid specifiers: d, h, m, s, f, F — use '\v' to include it as a literal
  "dd\:vv"
       ^"#,
    );
}

#[test]
fn parse_exact_invalid_unclosed_literal_double_quote() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r#"mm\:ss\ "minutes"#)
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: unclosed '"' in format string
  "mm\:ss\ "minutes"
           ^"#,
    );
}

#[test]
fn parse_exact_invalid_unclosed_literal_single_quote() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r"mm\:ss\ 'minutes")
            .unwrap_err()
            .to_string(),
        r#"invalid custom format: unclosed '\'' in format string
  "mm\:ss\ 'minutes"
           ^"#,
    );
}

#[test]
fn parse_exact_invalid_literal_mismatch() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 mints", r#"mm\:ss\ "minutes""#)
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected mm\:ss\ "minutes"
  "12:34 mints"
         ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34 mints", r"mm\:ss\ 'minutes'")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected mm\:ss\ 'minutes'
  "12:34 mints"
         ^"#,
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
        TimeSpan::parse_exact("24:24:02", "c")
            .unwrap_err()
            .to_string(),
        r#"hours value 24 is out of range; must be 0-23
  "24:24:02"
   ^"#,
    );
}

#[test]
fn parse_exact_overflow_minutes_out_of_range() {
    assert_eq!(
        TimeSpan::parse_exact("1:60:02", "c")
            .unwrap_err()
            .to_string(),
        r#"minutes value 60 is out of range; must be 0-59
  "1:60:02"
     ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("1.2:60:02", "c")
            .unwrap_err()
            .to_string(),
        r#"minutes value 60 is out of range; must be 0-59
  "1.2:60:02"
       ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:61:02", "g")
            .unwrap_err()
            .to_string(),
        r#"minutes value 61 is out of range; must be 0-59
  "12:61:02"
      ^"#,
    );
}

#[test]
fn parse_exact_overflow_seconds_out_of_range() {
    assert_eq!(
        TimeSpan::parse_exact("1:59:60", "c")
            .unwrap_err()
            .to_string(),
        r#"seconds value 60 is out of range; must be 0-59
  "1:59:60"
        ^"#,
    );
}

#[test]
fn parse_exact_overflow_hours_exceed_23_in_c_format() {
    // "c" format hours must be 0-23; 24 hours overflows
    assert_eq!(
        TimeSpan::parse_exact("1.24:59:02", "c")
            .unwrap_err()
            .to_string(),
        r#"hours value 24 is out of range; must be 0-23
  "1.24:59:02"
     ^"#,
    );
}

#[test]
fn parse_exact_overflow_g_upper_too_many_fractional_digits() {
    assert_eq!(
        TimeSpan::parse_exact("1:07:45:16.99999999", "G")
            .unwrap_err()
            .to_string(),
        r#"TimeSpan value is outside the representable range
  "1:07:45:16.99999999"
              ^"#,
    );
}

#[test]
fn parse_exact_overflow_custom_format() {
    // 35 hours exceeds valid range for "h" specifier
    assert_eq!(
        TimeSpan::parse_exact("12.35:32:43", r"dd\.h\:m\:s")
            .unwrap_err()
            .to_string(),
        r#"hours value 35 is out of range; must be 0-23
  "12.35:32:43"
      ^"#,
    );
}

#[test]
fn parse_exact_invalid_custom_wrong_digit_count_for_padded() {
    // "hh" needs 2 digits; input only has 1 then hits ':', which is a non-digit character
    assert_eq!(
        TimeSpan::parse_exact("12.5:2:3", r"d\.hh\:mm\:ss")
            .unwrap_err()
            .to_string(),
        r#"unexpected character ':'; expected a digit
  "12.5:2:3"
       ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12.5:2", r"d\.hh\:mm\:ss")
            .unwrap_err()
            .to_string(),
        r#"unexpected character ':'; expected a digit
  "12.5:2"
       ^"#,
    );
}

// ── Docs: standard-timespan-format-strings ────────────────────────────────────
//
// Examples drawn from:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings

// C# doc intro example: TimeSpan.ParseExact("1.03:14:56.1667", "c", null)
// → 1d 3h 14m 56s + fractional .1667 seconds
// ".1667" has 4 digits → 1667 ten-thousandths of a second = 1_667_000 ticks
#[test]
fn doc_standard_c_parse_fractional_ticks() {
    // "1.03:14:56.1667" parsed with "c" → 1.03:14:56.1667000
    let expected = TimeSpan::from_ticks(
        1 * TimeSpan::TICKS_PER_DAY
            + 3 * TimeSpan::TICKS_PER_HOUR
            + 14 * TimeSpan::TICKS_PER_MINUTE
            + 56 * TimeSpan::TICKS_PER_SECOND
            + 1_667_000, // "1667" as 4-digit fraction, padded to 7: 1667000 ticks
    );
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::parse_exact("1.03:14:56.1667", fmt),
            Ok(expected),
            "format={fmt:?}",
        );
    }
}

// C# doc "c" section arithmetic examples — format "c" output
// new TimeSpan(7, 45, 16) → 07:45:16
// new TimeSpan(18, 12, 38) → 18:12:38
// subtraction → -10:27:22
// addition → 1.01:57:54
#[test]
fn doc_standard_c_format_arithmetic() {
    let interval1 = ts3(7, 45, 16);
    let interval2 = ts3(18, 12, 38);
    let diff = TimeSpan::from_ticks(interval1.ticks() - interval2.ticks());
    let sum = TimeSpan::from_ticks(interval1.ticks() + interval2.ticks());
    assert_eq!(interval1.to_string_fmt("c").unwrap(), "07:45:16");
    assert_eq!(interval2.to_string_fmt("c").unwrap(), "18:12:38");
    assert_eq!(diff.to_string_fmt("c").unwrap(), "-10:27:22");
    assert_eq!(sum.to_string_fmt("c").unwrap(), "1.01:57:54");
}

// new TimeSpan(0, 0, 1, 14, 365) + TimeSpan.FromTicks(2143756) with "c"
// = 00:01:14.3650000 + 00:00:00.2143756 = 00:01:14.5793756
#[test]
fn doc_standard_c_format_from_ticks() {
    let interval1 = ts5(0, 0, 1, 14, 365);
    let interval2 = TimeSpan::from_ticks(2143756);
    let sum = TimeSpan::from_ticks(interval1.ticks() + interval2.ticks());
    assert_eq!(interval1.to_string_fmt("c").unwrap(), "00:01:14.3650000");
    assert_eq!(interval2.to_string_fmt("c").unwrap(), "00:00:00.2143756");
    assert_eq!(sum.to_string_fmt("c").unwrap(), "00:01:14.5793756");
}

// C# doc "g" section arithmetic examples (invariant culture = en)
// 7:45:16 - 18:12:38 = -10:27:22   (g)
// 7:45:16 + 18:12:38 = 1:1:57:54   (g)
// 0:01:14.036 + 0:00:00.2143756 = 0:01:14.2503756  (g)
#[test]
fn doc_standard_g_format_arithmetic() {
    let interval1 = ts3(7, 45, 16);
    let interval2 = ts3(18, 12, 38);
    let diff = TimeSpan::from_ticks(interval1.ticks() - interval2.ticks());
    let sum = TimeSpan::from_ticks(interval1.ticks() + interval2.ticks());
    assert_eq!(
        interval1
            .to_string_fmt_with_culture("g", Locale::en)
            .unwrap(),
        "7:45:16"
    );
    assert_eq!(
        diff.to_string_fmt_with_culture("g", Locale::en).unwrap(),
        "-10:27:22"
    );
    assert_eq!(
        sum.to_string_fmt_with_culture("g", Locale::en).unwrap(),
        "1:1:57:54"
    );
}

#[test]
fn doc_standard_g_format_fractional() {
    // new TimeSpan(0, 0, 1, 14, 36) = 0d 0h 1m 14s 36ms
    // TimeSpan.FromTicks(2143756)
    let interval1 = ts5(0, 0, 1, 14, 36);
    let interval2 = TimeSpan::from_ticks(2143756);
    let sum = TimeSpan::from_ticks(interval1.ticks() + interval2.ticks());
    assert_eq!(
        interval1
            .to_string_fmt_with_culture("g", Locale::en)
            .unwrap(),
        "0:01:14.036"
    );
    assert_eq!(
        interval2
            .to_string_fmt_with_culture("g", Locale::en)
            .unwrap(),
        "0:00:00.2143756"
    );
    assert_eq!(
        sum.to_string_fmt_with_culture("g", Locale::en).unwrap(),
        "0:01:14.2503756"
    );
}

// C# doc "G" section arithmetic examples (invariant culture = en)
// 0:07:45:16.0000000 - 0:18:12:38.0000000 = -0:10:27:22.0000000  (G)
// 0:07:45:16.0000000 + 0:18:12:38.0000000 = 1:01:57:54.0000000   (G)
#[test]
fn doc_standard_g_upper_format_arithmetic() {
    let interval1 = ts3(7, 45, 16);
    let interval2 = ts3(18, 12, 38);
    let diff = TimeSpan::from_ticks(interval1.ticks() - interval2.ticks());
    let sum = TimeSpan::from_ticks(interval1.ticks() + interval2.ticks());
    assert_eq!(
        interval1
            .to_string_fmt_with_culture("G", Locale::en)
            .unwrap(),
        "0:07:45:16.0000000"
    );
    assert_eq!(
        interval2
            .to_string_fmt_with_culture("G", Locale::en)
            .unwrap(),
        "0:18:12:38.0000000"
    );
    assert_eq!(
        diff.to_string_fmt_with_culture("G", Locale::en).unwrap(),
        "-0:10:27:22.0000000"
    );
    assert_eq!(
        sum.to_string_fmt_with_culture("G", Locale::en).unwrap(),
        "1:01:57:54.0000000"
    );
}

#[test]
fn doc_standard_g_upper_format_fractional() {
    // new TimeSpan(0, 0, 1, 14, 36) and FromTicks(2143756)
    let interval1 = ts5(0, 0, 1, 14, 36);
    let interval2 = TimeSpan::from_ticks(2143756);
    let sum = TimeSpan::from_ticks(interval1.ticks() + interval2.ticks());
    assert_eq!(
        interval1
            .to_string_fmt_with_culture("G", Locale::en)
            .unwrap(),
        "0:00:01:14.0360000"
    );
    assert_eq!(
        interval2
            .to_string_fmt_with_culture("G", Locale::en)
            .unwrap(),
        "0:00:00:00.2143756"
    );
    assert_eq!(
        sum.to_string_fmt_with_culture("G", Locale::en).unwrap(),
        "0:00:01:14.2503756"
    );
}

// ── Docs: custom-timespan-format-strings ──────────────────────────────────────
//
// Examples drawn from:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings

// Intro formatting example:
// new TimeSpan(1, 12, 23, 62) normalises to 1d 12h 24m 2s
// .ToString("%d")            → "1"
// .ToString(@"dd\.hh\:mm\:ss") → "01.12:24:02"
#[test]
fn doc_custom_intro_format_percent_d() {
    // new TimeSpan(1, 12, 23, 62) = 1d + 12h + 23m + 62s
    // 62s overflows: 1m 2s → 24m 2s total → 1d 12h 24m 2s
    let ts = ts5(1, 12, 24, 2, 0);
    assert_eq!(ts.to_string_fmt("%d").unwrap(), "1");
}

#[test]
fn doc_custom_intro_format_dd_dot_hh_colon_mm_colon_ss() {
    let ts = ts5(1, 12, 24, 2, 0);
    assert_eq!(ts.to_string_fmt(r"dd\.hh\:mm\:ss").unwrap(), "01.12:24:02");
}

// Intro parsing example:
// TryParseExact("6", "%d", null) → 6.00:00:00
// TryParseExact("16:32.05", @"mm\:ss\.ff", null) → 00:16:32.0500000
// TryParseExact("12.035", "ss\\.fff", null) → 00:00:12.0350000
#[test]
fn doc_custom_intro_parse_percent_d_six_days() {
    // "6" with "%d" → 6 days (already covered by parse_exact_custom_percent_specifiers
    // but the doc uses TryParseExact with this explicit input, confirming the pattern)
    assert_eq!(TimeSpan::parse_exact("6", "%d"), Ok(ts4(6, 0, 0, 0)),);
}

#[test]
fn doc_custom_intro_parse_mm_colon_ss_dot_ff() {
    // "16:32.05" with format r"mm\:ss\.ff"
    // mm=16 minutes, ss=32 seconds, ff=05 hundredths = 0.05s = 50ms
    assert_eq!(
        TimeSpan::parse_exact("16:32.05", r"mm\:ss\.ff"),
        Ok(ts5(0, 0, 16, 32, 50)),
    );
}

#[test]
fn doc_custom_intro_parse_ss_dot_fff() {
    // "12.035" with format r"ss\.fff"
    // ss=12 seconds, fff=035 milliseconds
    assert_eq!(
        TimeSpan::parse_exact("12.035", r"ss\.fff"),
        Ok(ts5(0, 0, 0, 12, 35)),
    );
}

// "hh" custom specifier — parse "08" as hours
// TryParseExact("08", "hh", null) → 08:00:00
#[test]
fn doc_custom_hh_parse_leading_zero_hours() {
    assert_eq!(TimeSpan::parse_exact("08", "hh"), Ok(ts3(8, 0, 0)),);
}

// "hh" formatting: new TimeSpan(14, 3, 17) with d\.hh\:mm\:ss → "0.14:03:17"
//                  new TimeSpan(3, 4, 3, 17) with d\.hh\:mm\:ss → "3.04:03:17"
#[test]
fn doc_custom_hh_format_d_dot_hh_colon_mm_colon_ss() {
    // new TimeSpan(14, 3, 17) = 14h 3m 17s → no days
    let ts1 = ts3(14, 3, 17);
    assert_eq!(ts1.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "0.14:03:17");
    // new TimeSpan(3, 4, 3, 17) = 3d 4h 3m 17s
    let ts2 = ts4(3, 4, 3, 17);
    assert_eq!(ts2.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "3.04:03:17");
}

// "mm" custom specifier — parse "07" as minutes
// TryParseExact("07", "mm", null) → 00:07:00
#[test]
fn doc_custom_mm_parse_minutes() {
    assert_eq!(TimeSpan::parse_exact("07", "mm"), Ok(ts3(0, 7, 0)),);
}

// "mm" formatting: (arriveTime - departTime) with hh\:mm → "05:16"
// departTime = new TimeSpan(11, 12, 00), arriveTime = new TimeSpan(16, 28, 00)
#[test]
fn doc_custom_mm_format_travel_time() {
    let depart = ts3(11, 12, 0);
    let arrive = ts3(16, 28, 0);
    let elapsed = TimeSpan::from_ticks(arrive.ticks() - depart.ticks());
    assert_eq!(elapsed.to_string_fmt(r"hh\:mm").unwrap(), "05:16");
}

// "ss" custom specifier — parse "49", reject "9", parse "06"
// TryParseExact("49", "ss") → 00:00:49
// TryParseExact("9", "ss")  → fails (ss requires exactly 2 digits)
// TryParseExact("06", "ss") → 00:00:06
#[test]
fn doc_custom_ss_parse_two_digit_seconds() {
    assert_eq!(TimeSpan::parse_exact("49", "ss"), Ok(ts3(0, 0, 49)),);
    assert_eq!(TimeSpan::parse_exact("06", "ss"), Ok(ts3(0, 0, 6)),);
    // Single digit rejected by "ss" (requires exactly 2 digits)
    assert!(TimeSpan::parse_exact("9", "ss").is_err());
}

// "ss" formatting: FromSeconds(12.60) → "12.600", FromSeconds(6.485) → "06.485"
#[test]
fn doc_custom_ss_format_dot_fff() {
    // 12.60s = 12s + 600ms
    let ts1 = TimeSpan::from_ticks(
        12 * TimeSpan::TICKS_PER_SECOND + 600 * TimeSpan::TICKS_PER_MILLISECOND,
    );
    assert_eq!(ts1.to_string_fmt(r"ss\.fff").unwrap(), "12.600");
    // 6.485s = 6s + 485ms
    let ts2 = TimeSpan::from_ticks(
        6 * TimeSpan::TICKS_PER_SECOND + 485 * TimeSpan::TICKS_PER_MILLISECOND,
    );
    assert_eq!(ts2.to_string_fmt(r"ss\.fff").unwrap(), "06.485");
}

// "F" custom specifier — parse examples from the doc
// h\:m\:ss\.F:
//   "0:0:03."   → 00:00:03 (fraction empty/zero → 0 ticks)
//   "0:0:03.1"  → 00:00:03.1000000
//   "0:0:03.12" → fails (FF accepts at most 1 fractional digit)
#[test]
fn doc_custom_uppercase_f_parse_h_m_ss_dot_f() {
    let fmt = r"h\:m\:ss\.F";
    // empty fraction after the separator → 0 fractional ticks
    assert_eq!(TimeSpan::parse_exact("0:0:03.", fmt), Ok(ts3(0, 0, 3)),);
    // 1 fractional digit → 1 tenth of a second = 1_000_000 ticks
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.1", fmt),
        Ok(ts5(0, 0, 0, 3, 100)),
    );
    // 2 fractional digits → too many for "F" (max 1)
    assert!(TimeSpan::parse_exact("0:0:03.12", fmt).is_err());
}

// "FF" custom specifier parse examples
// h\:m\:ss\.FF:
//   "0:0:03."    → 00:00:03
//   "0:0:03.1"   → 00:00:03.1000000
//   "0:0:03.127" → fails (FF accepts at most 2 fractional digits)
#[test]
fn doc_custom_uppercase_ff_parse_h_m_ss_dot_ff() {
    let fmt = r"h\:m\:ss\.FF";
    assert_eq!(TimeSpan::parse_exact("0:0:03.", fmt), Ok(ts3(0, 0, 3)),);
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.1", fmt),
        Ok(ts5(0, 0, 0, 3, 100)),
    );
    assert!(TimeSpan::parse_exact("0:0:03.127", fmt).is_err());
}

// "FFF" custom specifier parse examples
// h\:m\:ss\.FFF:
//   "0:0:03."     → 00:00:03
//   "0:0:03.12"   → 00:00:03.1200000
//   "0:0:03.1279" → fails (FFF accepts at most 3 fractional digits)
#[test]
fn doc_custom_uppercase_fff_parse_h_m_ss_dot_fff() {
    let fmt = r"h\:m\:ss\.FFF";
    assert_eq!(TimeSpan::parse_exact("0:0:03.", fmt), Ok(ts3(0, 0, 3)),);
    // "0:0:03.12" → 3s + 120ms
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.12", fmt),
        Ok(ts5(0, 0, 0, 3, 120)),
    );
    assert!(TimeSpan::parse_exact("0:0:03.1279", fmt).is_err());
}

// "FFFF" custom specifier parse examples
// h\:m\:ss\.FFFF:
//   "0:0:03."      → 00:00:03
//   "0:0:03.12"    → 00:00:03.1200000
//   "0:0:03.12795" → fails (FFFF accepts at most 4 fractional digits)
#[test]
fn doc_custom_uppercase_ffff_parse_h_m_ss_dot_ffff() {
    let fmt = r"h\:m\:ss\.FFFF";
    assert_eq!(TimeSpan::parse_exact("0:0:03.", fmt), Ok(ts3(0, 0, 3)),);
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.12", fmt),
        Ok(ts5(0, 0, 0, 3, 120)),
    );
    assert!(TimeSpan::parse_exact("0:0:03.12795", fmt).is_err());
}

// "FFFFF" custom specifier parse examples
// h\:m\:ss\.FFFFF:
//   "0:0:03."        → 00:00:03
//   "0:0:03.12"      → 00:00:03.1200000
//   "0:0:03.127956"  → fails (FFFFF accepts at most 5 fractional digits)
#[test]
fn doc_custom_uppercase_fffff_parse_h_m_ss_dot_fffff() {
    let fmt = r"h\:m\:ss\.FFFFF";
    assert_eq!(TimeSpan::parse_exact("0:0:03.", fmt), Ok(ts3(0, 0, 3)),);
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.12", fmt),
        Ok(ts5(0, 0, 0, 3, 120)),
    );
    assert!(TimeSpan::parse_exact("0:0:03.127956", fmt).is_err());
}

// "FFFFFF" custom specifier parse examples
// h\:m\:ss\.FFFFFF:
//   "0:0:03."         → 00:00:03
//   "0:0:03.12"       → 00:00:03.1200000
//   "0:0:03.1279569"  → fails (FFFFFF accepts at most 6 fractional digits)
#[test]
fn doc_custom_uppercase_ffffff_parse_h_m_ss_dot_ffffff() {
    let fmt = r"h\:m\:ss\.FFFFFF";
    assert_eq!(TimeSpan::parse_exact("0:0:03.", fmt), Ok(ts3(0, 0, 3)),);
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.12", fmt),
        Ok(ts5(0, 0, 0, 3, 120)),
    );
    assert!(TimeSpan::parse_exact("0:0:03.1279569", fmt).is_err());
}

// "FFFFFFF" custom specifier parse examples
// h\:m\:ss\.FFFFFFF:
//   "0:0:03."          → 00:00:03
//   "0:0:03.12"        → 00:00:03.1200000
//   "0:0:03.1279569"   → 00:00:03.1279569
#[test]
fn doc_custom_uppercase_fffffff_parse_h_m_ss_dot_fffffff() {
    let fmt = r"h\:m\:ss\.FFFFFFF";
    assert_eq!(TimeSpan::parse_exact("0:0:03.", fmt), Ok(ts3(0, 0, 3)),);
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.12", fmt),
        Ok(ts5(0, 0, 0, 3, 120)),
    );
    // "0:0:03.1279569" → 3s + 1279569 ticks of fraction
    // 1279569 ticks = 127ms + 9569 sub-ms ticks → but ts5 only takes ms
    // Use from_ticks directly: 3s = 30_000_000 ticks, fraction = 1_279_569 ticks
    assert_eq!(
        TimeSpan::parse_exact("0:0:03.1279569", fmt),
        Ok(TimeSpan::from_ticks(
            3 * TimeSpan::TICKS_PER_SECOND + 1_279_569
        )),
    );
}

// "F" formatting examples from the doc
// TimeSpan.Parse("0:0:3.669"):  %F → "6"
// TimeSpan.Parse("0:0:3.091"):  ss\.F → "03."  (zero tenths → nothing after dot but dot is literal)
#[test]
fn doc_custom_uppercase_f_format() {
    // 3.669s → 3s + 669ms → tenths digit = 6
    let ts1 = ts5(0, 0, 0, 3, 669);
    assert_eq!(ts1.to_string_fmt("%F").unwrap(), "6");
    // 3.091s → 3s + 91ms → tenths digit = 0 → F outputs nothing
    // but combined with "ss\.F", the dot is a literal so output is "03."
    let ts2 = ts5(0, 0, 0, 3, 91);
    assert_eq!(ts2.to_string_fmt(r"ss\.F").unwrap(), "03.");
}

// "FF" formatting examples from the doc
// TimeSpan.Parse("0:0:3.697"):  FF → "69"
// TimeSpan.Parse("0:0:3.809"):  ss\.FF → "03.8"  (trailing zero trimmed)
#[test]
fn doc_custom_uppercase_ff_format() {
    let ts1 = ts5(0, 0, 0, 3, 697);
    assert_eq!(ts1.to_string_fmt("FF").unwrap(), "69");
    // 3.809s → hundredths = 80 → trim trailing zero → "8"
    let ts2 = ts5(0, 0, 0, 3, 809);
    assert_eq!(ts2.to_string_fmt(r"ss\.FF").unwrap(), "03.8");
}

// "FFF" formatting examples from the doc
// TimeSpan.Parse("0:0:3.6974"):  FFF → "697"
// TimeSpan.Parse("0:0:3.8009"):  ss\.FFF → "03.8"  (trailing zeros trimmed)
#[test]
fn doc_custom_uppercase_fff_format() {
    // 3.6974s → 3s + 697.4ms → ms component = 697 (truncated by ticks)
    // In ticks: 6974 ten-thousandths = 6974 * 1000 = 6_974_000 sub-second ticks
    // 6_974_000 ticks → milliseconds = 697, sub-ms = 4000 ticks
    let ts1 = TimeSpan::from_ticks(3 * TimeSpan::TICKS_PER_SECOND + 6_974_000);
    assert_eq!(ts1.to_string_fmt("FFF").unwrap(), "697");
    // 3.8009s → 8009 ten-thousandths sub-second = 8_009_000 ticks; ms=800, sub=9000
    // FFF shows milliseconds only = "800", trim trailing zeros → "8"
    let ts2 = TimeSpan::from_ticks(3 * TimeSpan::TICKS_PER_SECOND + 8_009_000);
    assert_eq!(ts2.to_string_fmt(r"ss\.FFF").unwrap(), "03.8");
}

// "FFFFFFF" formatting examples from the doc
// TimeSpan.Parse("0:0:3.6974974"):  FFFFFFF → "6974974"
// TimeSpan.Parse("0:0:3.9500000"):  ss\.FFFFFFF → "03.95"  (trailing zeros trimmed)
#[test]
fn doc_custom_uppercase_fffffff_format() {
    let ts1 = TimeSpan::from_ticks(3 * TimeSpan::TICKS_PER_SECOND + 6_974_974);
    assert_eq!(ts1.to_string_fmt("FFFFFFF").unwrap(), "6974974");
    // 3.9500000s = 3s + 9_500_000 ticks; trailing zeros trimmed → "95"
    let ts2 = TimeSpan::from_ticks(3 * TimeSpan::TICKS_PER_SECOND + 9_500_000);
    assert_eq!(ts2.to_string_fmt(r"ss\.FFFFFFF").unwrap(), "03.95");
}

// Other-characters section: escape and single-quote literals in formatting
// new TimeSpan(0, 32, 45).ToString(@"mm\:ss\ \m\i\n\u\t\e\s") → "32:45 minutes"
// new TimeSpan(0, 32, 45).ToString("mm':'ss' minutes'") → "32:45 minutes"
#[test]
fn doc_custom_other_chars_literal_format() {
    let ts = ts3(0, 32, 45);
    assert_eq!(
        ts.to_string_fmt(r"mm\:ss\ \m\i\n\u\t\e\s").unwrap(),
        "32:45 minutes"
    );
    assert_eq!(
        ts.to_string_fmt("mm':'ss' minutes'").unwrap(),
        "32:45 minutes"
    );
}

// "d" custom specifier — formatting examples
// new TimeSpan(4, 3, 17) with d\.hh\:mm\:ss → "0.04:03:17"
// new TimeSpan(3, 4, 3, 17) with d\.hh\:mm\:ss → "3.04:03:17"
#[test]
fn doc_custom_d_format_d_dot_hh_colon_mm_colon_ss() {
    // new TimeSpan(4, 3, 17) = 0d 4h 3m 17s
    let ts1 = ts3(4, 3, 17);
    assert_eq!(ts1.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "0.04:03:17");
    // new TimeSpan(3, 4, 3, 17) = 3d 4h 3m 17s
    let ts2 = ts4(3, 4, 3, 17);
    assert_eq!(ts2.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "3.04:03:17");
}

// "dd"-"dddddddd" — formatting examples
// new TimeSpan(0, 23, 17, 47) with dd\.hh\:mm\:ss → "00.23:17:47"
// new TimeSpan(365, 21, 19, 45) with dd\.hh\:mm\:ss → "365.21:19:45"
// new TimeSpan(365, 21, 19, 45) with dddd\.hh\:mm\:ss → "0365.21:19:45"
#[test]
fn doc_custom_dd_format_padded_days() {
    let ts1 = ts4(0, 23, 17, 47);
    let ts2 = ts4(365, 21, 19, 45);
    assert_eq!(ts1.to_string_fmt(r"dd\.hh\:mm\:ss").unwrap(), "00.23:17:47");
    assert_eq!(
        ts2.to_string_fmt(r"dd\.hh\:mm\:ss").unwrap(),
        "365.21:19:45"
    );
    assert_eq!(
        ts2.to_string_fmt(r"dddd\.hh\:mm\:ss").unwrap(),
        "0365.21:19:45"
    );
}

// "h" custom specifier — formatting examples
// new TimeSpan(14, 3, 17) with d\.h\:mm\:ss → "0.14:03:17"
// new TimeSpan(3, 4, 3, 17) with d\.h\:mm\:ss → "3.4:03:17"
#[test]
fn doc_custom_h_format_d_dot_h_colon_mm_colon_ss() {
    let ts1 = ts3(14, 3, 17);
    assert_eq!(ts1.to_string_fmt(r"d\.h\:mm\:ss").unwrap(), "0.14:03:17");
    let ts2 = ts4(3, 4, 3, 17);
    assert_eq!(ts2.to_string_fmt(r"d\.h\:mm\:ss").unwrap(), "3.4:03:17");
}

// "m" custom specifier — formatting example
// new TimeSpan(0, 6, 32) with m\:ss → "6:32"
// C# doc also shows new TimeSpan(3, 4, 3, 17) with m\:ss but gives "18:44"
// (this is odd; likely a bug in VB example; C# shows ts2 = new TimeSpan(0, 18, 44))
// We test the straightforward case.
#[test]
fn doc_custom_m_format_m_colon_ss() {
    let ts1 = ts3(0, 6, 32);
    assert_eq!(ts1.to_string_fmt(r"m\:ss").unwrap(), "6:32");
}

// "s" custom specifier — format example
// endTime - startTime = 6s 3ms
// with s\:fff → "6:003"
#[test]
fn doc_custom_s_format_s_colon_fff() {
    // startTime = new TimeSpan(0, 12, 30, 15, 0), endTime = new TimeSpan(0, 12, 30, 21, 3)
    // diff = 6s + 3ms
    let diff = ts5(0, 0, 0, 6, 3);
    assert_eq!(diff.to_string_fmt(r"s\:fff").unwrap(), "6:003");
}

// "f" custom specifier — formatting examples from the single-specifier loop
// TimeSpan.from_ticks(1003498765432) with "c" → 1.15:51:38.8765432
// Then individual specifiers:
//   %f → "8", ff → "87", fff → "876", ..., fffffff → "8765432"
//   s\.f → "29.8" (wait: seconds=38, but sub-second fraction starts at 8)
// Actually ticks=1003498765432:
//   days   = 1003498765432 / 864000000000 = 1 (remainder = 139498765432)
//   hours  = 139498765432 / 36000000000   = 3 (remainder = 31498765432)
//   minutes= 31498765432 / 600000000      = 52 (remainder = 298765432)
//   seconds= 298765432 / 10000000         = 29 (remainder = 8765432)
//   The doc shows s\.f = "29.8", consistent with seconds=29, tenths=8
#[test]
fn doc_custom_f_format_ticks_1003498765432() {
    let ts = TimeSpan::from_ticks(1_003_498_765_432);
    // Verify "c" first to confirm the value
    // 1_003_498_765_432 ticks: 1d 3h 52m 29s .8765432
    assert_eq!(ts.to_string_fmt("c").unwrap(), "1.03:52:29.8765432");
    // Individual f* specifiers
    assert_eq!(ts.to_string_fmt("%f").unwrap(), "8");
    assert_eq!(ts.to_string_fmt("ff").unwrap(), "87");
    assert_eq!(ts.to_string_fmt("fff").unwrap(), "876");
    assert_eq!(ts.to_string_fmt("ffff").unwrap(), "8765");
    assert_eq!(ts.to_string_fmt("fffff").unwrap(), "87654");
    assert_eq!(ts.to_string_fmt("ffffff").unwrap(), "876543");
    assert_eq!(ts.to_string_fmt("fffffff").unwrap(), "8765432");
    // s\.f* specifiers
    assert_eq!(ts.to_string_fmt(r"s\.f").unwrap(), "29.8");
    assert_eq!(ts.to_string_fmt(r"s\.ff").unwrap(), "29.87");
    assert_eq!(ts.to_string_fmt(r"s\.fff").unwrap(), "29.876");
    assert_eq!(ts.to_string_fmt(r"s\.ffff").unwrap(), "29.8765");
    assert_eq!(ts.to_string_fmt(r"s\.fffff").unwrap(), "29.87654");
    assert_eq!(ts.to_string_fmt(r"s\.ffffff").unwrap(), "29.876543");
    assert_eq!(ts.to_string_fmt(r"s\.fffffff").unwrap(), "29.8765432");
}
