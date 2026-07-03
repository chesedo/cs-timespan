// Tests ported from the C# reference implementation:
// https://github.com/dotnet/runtime/blob/main/src/libraries/System.Runtime/tests/System.Runtime.Tests/System/TimeSpanTests.cs
//
// C# test methods covered here: ParseExact_Valid_TestData (TimeSpanTests.cs#L1162-1206),
// ParseExact_Invalid_TestData (TimeSpanTests.cs#L1252-1304)
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
    -t
}

// ── ParseExact_Valid — constant format "c" / "t" / "T" ───────────────────────

// TimeSpanTests.cs#L1167
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

// TimeSpanTests.cs#L1168
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

// C# TimeSpanParse.cs ParseTime (line 1625): `if (_ch == ':')` makes the second colon
// and seconds component optional — "hh:mm" and "d.hh:mm" are valid "c" inputs.
// (No direct ParseExact_Valid_TestData row for this reduced form; it documents
// StringParser.ParseTime behavior rather than duplicating a specific test case.)
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

// TimeSpanTests.cs#L1169
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

// TimeSpanTests.cs#L1173
#[test]
fn parse_exact_g_bare_integer_is_days() {
    assert_eq!(TimeSpan::parse_exact("12", "g"), Ok(ts4(12, 0, 0, 0)));
}

// TimeSpanTests.cs#L1174
#[test]
fn parse_exact_g_negative_days() {
    assert_eq!(
        TimeSpan::parse_exact("-12", "g"),
        Ok(TimeSpan::from_ticks(ts4(-12, 0, 0, 0).ticks())),
    );
}

// TimeSpanTests.cs#L1175
#[test]
fn parse_exact_g_hm() {
    assert_eq!(TimeSpan::parse_exact("12:34", "g"), Ok(ts3(12, 34, 0)));
}

// TimeSpanTests.cs#L1176
#[test]
fn parse_exact_g_negative_hm() {
    assert_eq!(
        TimeSpan::parse_exact("-12:34", "g"),
        Ok(neg(ts3(12, 34, 0)))
    );
}

// TimeSpanTests.cs#L1177-1178
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

// TimeSpanTests.cs#L1179
#[test]
fn parse_exact_g_hms() {
    assert_eq!(
        TimeSpan::parse_exact("12:24:02", "g"),
        Ok(ts4(0, 12, 24, 2)),
    );
}

// TimeSpanTests.cs#L1180-1181
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

// TimeSpanTests.cs#L1182-1183
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

// TimeSpanTests.cs#L1184
#[test]
fn parse_exact_g_d_hms() {
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02", "g"),
        Ok(ts4(1, 12, 24, 2)),
    );
}

// TimeSpanTests.cs#L1185
#[test]
fn parse_exact_g_negative_full() {
    assert_eq!(
        TimeSpan::parse_exact("-01:07:45:16.999", "g"),
        Ok(neg(ts5(1, 7, 45, 16, 999))),
    );
}

// ── ParseExact_Valid — general long format "G" ────────────────────────────────

// TimeSpanTests.cs#L1188
#[test]
fn parse_exact_g_upper_d_hms_with_millis() {
    assert_eq!(
        TimeSpan::parse_exact("1:12:24:02.243", "G"),
        Ok(ts5(1, 12, 24, 2, 243)),
    );
}

// TimeSpanTests.cs#L1189
#[test]
fn parse_exact_g_upper_negative() {
    assert_eq!(
        TimeSpan::parse_exact("-01:07:45:16.999", "G"),
        Ok(neg(ts5(1, 7, 45, 16, 999))),
    );
}

// ── ParseExact_Valid — custom format specifiers ───────────────────────────────

// TimeSpanTests.cs#L1192
#[test]
fn parse_exact_custom_dd_dot_h_m_s() {
    assert_eq!(
        TimeSpan::parse_exact("12.23:32:43", r"dd\.h\:m\:s"),
        Ok(ts4(12, 23, 32, 43)),
    );
}

// TimeSpanTests.cs#L1193
#[test]
fn parse_exact_custom_ddd_dot_h_m_s_fff() {
    assert_eq!(
        TimeSpan::parse_exact("012.23:32:43.893", r"ddd\.h\:m\:s\.fff"),
        Ok(ts5(12, 23, 32, 43, 893)),
    );
}

// TimeSpanTests.cs#L1194
#[test]
fn parse_exact_custom_d_dot_hh_mm_ss() {
    assert_eq!(
        TimeSpan::parse_exact("12.05:02:03", r"d\.hh\:mm\:ss"),
        Ok(ts4(12, 5, 2, 3)),
    );
}

// TimeSpanTests.cs#L1195
#[test]
fn parse_exact_custom_literal_word_backslash_escaped() {
    assert_eq!(
        TimeSpan::parse_exact(r"12:34 minutes", r"mm\:ss\ \m\i\n\u\t\e\s"),
        Ok(ts3(0, 12, 34)),
    );
}

// TimeSpanTests.cs#L1196
#[test]
fn parse_exact_custom_literal_word_double_quoted() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r#"mm\:ss\ "minutes""#),
        Ok(ts3(0, 12, 34)),
    );
}

// TimeSpanTests.cs#L1197
#[test]
fn parse_exact_custom_literal_word_single_quoted() {
    assert_eq!(
        TimeSpan::parse_exact("12:34 minutes", r"mm\:ss\ 'minutes'"),
        Ok(ts3(0, 12, 34)),
    );
}

// C# DateTimeParse.TryParseQuoteString (DateTimeParse.cs line 4600): '\' inside a
// quoted literal is an escape — '\X' appends X, allowing the quote char to be embedded.
// (No direct ParseExact_Valid_TestData row for this exact input; it exercises the
// same quoted-literal escape mechanism as parse_exact_custom_literal_word_double_quoted
// and parse_exact_custom_literal_word_single_quoted above.)
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

// TimeSpanTests.cs#L1198
#[test]
fn parse_exact_custom_fff_lowercase() {
    assert_eq!(
        TimeSpan::parse_exact("678", "fff"),
        Ok(ts5(0, 0, 0, 0, 678)),
    );
}

// TimeSpanTests.cs#L1199
#[test]
fn parse_exact_custom_fff_uppercase_optional_digits() {
    assert_eq!(
        TimeSpan::parse_exact("678", "FFF"),
        Ok(ts5(0, 0, 0, 0, 678)),
    );
}

// C# TryParseByFormat (TimeSpanParse.cs line 1317): ParseExactDigits return value is
// ignored for 'F' specifiers — zero matched digits is valid (fraction defaults to 0).
// (No direct ParseExact_Valid_TestData row for these exact inputs; this documents
// TryParseByFormat behavior rather than duplicating a specific test case.)
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

// TimeSpanTests.cs#L1200-1205
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

// TimeSpanTests.cs#L1256
#[test]
fn parse_exact_invalid_empty_string() {
    assert_eq!(
        TimeSpan::parse_exact("", "c").unwrap_err().to_string(),
        r#"input is empty
  ""
   ^"#,
    );
}

// TimeSpanTests.cs#L1257
#[test]
fn parse_exact_invalid_lone_minus() {
    assert_eq!(
        TimeSpan::parse_exact("-", "c").unwrap_err().to_string(),
        r#"input is empty
  "-"
   ^"#,
    );
}

// TimeSpanTests.cs#L1258
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

// TimeSpanTests.cs#L1266-1268
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

// TimeSpanTests.cs#L1269
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

// TimeSpanTests.cs#L1271
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

// TimeSpanTests.cs#L1273
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

// TimeSpanTests.cs#L1302
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

// TimeSpanTests.cs#L1303
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

// TimeSpanTests.cs#L1287-1288
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

// TimeSpanTests.cs#L1289-1290, #L1293, #L1296-1297, #L1299
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

// TimeSpanTests.cs#L1291, #L1294
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

// TimeSpanTests.cs#L1289, #L1292, #L1295
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

// TimeSpanTests.cs#L1279
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

// TimeSpanTests.cs#L1280
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

// TimeSpanTests.cs#L1298
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

// TimeSpanTests.cs#L1300
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

// TimeSpanTests.cs#L1301
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

// TimeSpanTests.cs#L1283
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

// TimeSpanTests.cs#L1284
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

// TimeSpanTests.cs#L1285-1286
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

// TimeSpanParse.cs#L1228-1247: `TryParseExactTimeSpan` dispatches the five standard
// format characters ('c'/'t'/'T'/'g'/'G') directly without forwarding `styles` to
// the parser — `AssumeNegative` is therefore invisible to them. Only
// `TryParseByFormat` (custom formats) receives and acts on `styles`.
#[test]
fn parse_exact_with_styles_assume_negative_standard_format() {
    // AssumeNegative is silently ignored for all five standard format strings; the
    // sign comes from the input alone.
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::parse_exact_with_styles(
                "01:02:03",
                fmt,
                Locale::en,
                TimeSpanStyles::AssumeNegative
            ),
            Ok(ts4(0, 1, 2, 3)),
            "format={fmt:?}",
        );
    }
    assert_eq!(
        TimeSpan::parse_exact_with_styles(
            "1:2:03:04",
            "g",
            Locale::en,
            TimeSpanStyles::AssumeNegative
        ),
        Ok(ts4(1, 2, 3, 4)),
    );
    assert_eq!(
        TimeSpan::parse_exact_with_styles(
            "1:02:03:04.0000000",
            "G",
            Locale::en,
            TimeSpanStyles::AssumeNegative
        ),
        Ok(ts4(1, 2, 3, 4)),
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

// ── ParseExact — "c" format edge cases ────────────────────────────────────────
//
// Source: C# TimeSpanParse.cs `StringParser.ParseTime` (lines 1639–1660 in the
// dotnet/runtime reference implementation). ParseTime reads digits in a loop that
// exits when the fraction tick counter `f` reaches 1 (after exactly 7 digits). Any
// character remaining after that exit is detected by `_pos < _str.Length` at the
// call site and causes a FormatException.

#[test]
fn parse_exact_c_trailing_dot_no_fraction() {
    // ParseTime: after the loop for fractional digits, if no digits were read the
    // fraction is zero. A trailing '.' with nothing after it enters the loop but
    // immediately sees end-of-input → zero fraction, ParseTime returns success.
    assert_eq!(
        TimeSpan::parse_exact("00:00:01.", "c").unwrap(),
        ts4(0, 0, 0, 1),
    );
    assert_eq!(
        TimeSpan::parse_exact("1.02:03:04.", "c").unwrap(),
        ts4(1, 2, 3, 4),
    );
}

#[test]
fn parse_exact_c_too_many_fractional_digits() {
    // ParseTime reads exactly 7 fraction digits; the 8th character remains unconsumed.
    // The call site then sees _pos < _str.Length and returns SetBadTimeSpanFailure.
    // "00000001" has 8 digits; the 8th ('1') is at offset 16 in the full input.
    assert_eq!(
        TimeSpan::parse_exact("00:00:01.00000001", "c")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected [-][d.]hh:mm[:ss[.fffffff]]
  "00:00:01.00000001"
                   ^"#,
    );
}

// ── ParseExact_Invalid — OverflowException cases ──────────────────────────────

// TimeSpanTests.cs#L1261
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

// TimeSpanTests.cs#L1262, #L1265, #L1270
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

// TimeSpanTests.cs#L1263
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

// TimeSpanTests.cs#L1264
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

// TimeSpanTests.cs#L1272
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

// C# TimeSpanParse.cs `TryTimeToTicks` (dotnet/runtime) rejects hours >= 24 /
// minutes >= 60 / seconds >= 60 with OverflowException unconditionally — custom
// format parsing routes through the same function as standard formats, so there's
// no normalizing path for out-of-range components.
// TimeSpanTests.cs#L1276
#[test]
fn parse_exact_overflow_custom_format_hours() {
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
fn parse_exact_custom_format_percent_h_overflow() {
    assert_eq!(
        TimeSpan::parse_exact("26", "%h").unwrap_err().to_string(),
        r#"hours value 26 is out of range; must be 0-23
  "26"
   ^"#,
    );
}

#[test]
fn parse_exact_custom_format_percent_m_overflow() {
    assert_eq!(
        TimeSpan::parse_exact("60", "%m").unwrap_err().to_string(),
        r#"minutes value 60 is out of range; must be 0-59
  "60"
   ^"#,
    );
}

#[test]
fn parse_exact_custom_format_percent_s_overflow() {
    assert_eq!(
        TimeSpan::parse_exact("60", "%s").unwrap_err().to_string(),
        r#"seconds value 60 is out of range; must be 0-59
  "60"
   ^"#,
    );
}

// TimeSpanTests.cs#L1277-1278
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

// ── Docs: custom-timespan-format-strings ──────────────────────────────────────
//
// Examples drawn from:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings

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

// "mm" custom specifier — parse "07" as minutes
// TryParseExact("07", "mm", null) → 00:07:00
#[test]
fn doc_custom_mm_parse_minutes() {
    assert_eq!(TimeSpan::parse_exact("07", "mm"), Ok(ts3(0, 7, 0)),);
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

// TimeSpanTests.cs#L1281-1282: format @"hh\mm\ss" = hh (2-digit hours) + literal 'm'
// + m (minutes) + literal 's' + ss. Both inputs fail immediately after reading
// hh="12" because the format expects literal 'm' but the input has ':'.
#[test]
fn parse_exact_invalid_escaped_letter_separator_mismatch() {
    assert_eq!(
        TimeSpan::parse_exact("12:034:56", r"hh\mm\ss")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected hh\mm\ss
  "12:034:56"
     ^"#,
    );
    assert_eq!(
        TimeSpan::parse_exact("12:34:056", r"hh\mm\ss")
            .unwrap_err()
            .to_string(),
        r#"unrecognised input structure; expected hh\mm\ss
  "12:34:056"
     ^"#,
    );
}
