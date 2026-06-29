// Tests ported from the C# reference implementation:
// https://github.com/dotnet/corefx/blob/master/src/System.Runtime/tests/System/TimeSpanTests.cs
//
// C# test method covered here: ToString_TestData
//
// Notes on translation:
// - C# `new TimeSpan(h, m, s)`        → `hms(h, m, s)`
// - C# `new TimeSpan(d, h, m, s)`     → `dhms(d, h, m, s)`
// - C# `new TimeSpan(d, h, m, s, ms)` → `dhmsm(d, h, m, s, ms)`
// - C# `new TimeSpan(ticks)`          → `TimeSpan::from_ticks(ticks)`
// - C# `-new TimeSpan(...)`           → `neg(...)` helper
// - Formatting is via `TimeSpan::to_string_fmt(fmt)` (invariant culture)
//   or `TimeSpan::to_string_fmt_with_culture(fmt, culture)`.
// - C# `null` format and `"c"` / `"t"` / `"T"` all produce identical output;
//   the null case maps to `Display` (`ts.to_string()`).

use cs_timespan::{Locale, TimeSpan};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn hms(h: i64, m: i64, s: i64) -> TimeSpan {
    dhmsm(0, h, m, s, 0)
}

fn dhms(d: i64, h: i64, m: i64, s: i64) -> TimeSpan {
    dhmsm(d, h, m, s, 0)
}

fn dhmsm(d: i64, h: i64, m: i64, s: i64, ms: i64) -> TimeSpan {
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

// The large value used as the primary formatting test input throughout the C# suite.
fn input() -> TimeSpan {
    TimeSpan::from_ticks(123_456_789_101_112)
}

// ── Custom single-component specifiers (%d, %h, %m, %s, %f/%F families) ──────

#[test]
fn format_custom_percent_d() {
    assert_eq!(input().to_string_fmt("%d").unwrap(), "142");
}

#[test]
fn format_custom_dd() {
    assert_eq!(input().to_string_fmt("dd").unwrap(), "142");
}

#[test]
fn format_custom_dddddd_padded() {
    assert_eq!(input().to_string_fmt("dddddd").unwrap(), "000142");
}

#[test]
fn format_custom_invalid_d_repeat_too_long() {
    // C# TimeSpanFormat.cs FormatCustomized: repeat count > 8 for 'd' throws FormatException.
    assert_eq!(
        input().to_string_fmt("ddddddddd").unwrap_err().to_string(), // 9 d's
        r#"specifier repeated too many times (max 8)
  "ddddddddd"
           ^"#
    );
}

#[test]
fn format_custom_invalid_hms_repeat_too_long() {
    // C# TimeSpanFormat.cs FormatCustomized: repeat count > 2 for 'h', 'm', 's' throws FormatException.
    assert_eq!(
        input().to_string_fmt("hhh").unwrap_err().to_string(), // 3 h's
        r#"specifier repeated too many times (max 2)
  "hhh"
     ^"#
    );
    assert_eq!(
        input().to_string_fmt("mmm").unwrap_err().to_string(), // 3 m's
        r#"specifier repeated too many times (max 2)
  "mmm"
     ^"#
    );
    assert_eq!(
        input().to_string_fmt("sss").unwrap_err().to_string(), // 3 s's
        r#"specifier repeated too many times (max 2)
  "sss"
     ^"#
    );
}

#[test]
fn format_custom_invalid_frac_repeat_too_long() {
    // C# TimeSpanFormat.cs FormatCustomized: repeat count > 7 for 'f'/'F' throws FormatException.
    assert_eq!(
        input().to_string_fmt("ffffffff").unwrap_err().to_string(), // 8 f's
        r#"specifier repeated too many times (max 7)
  "ffffffff"
          ^"#
    );
    assert_eq!(
        input().to_string_fmt("FFFFFFFF").unwrap_err().to_string(), // 8 F's
        r#"specifier repeated too many times (max 7)
  "FFFFFFFF"
          ^"#
    );
}

#[test]
fn format_custom_percent_unknown_specifier() {
    // C# TimeSpanFormat.cs FormatCustomized line ~419: %x recurses into FormatCustomized
    // with just 'x'; 'x' hits the default case (line ~451) which throws FormatException.
    assert_eq!(
        input().to_string_fmt("%x").unwrap_err().to_string(),
        r#"unrecognised specifier 'x'; valid specifiers: d h m s f F
  "%x"
    ^"#
    );
}

#[test]
fn format_custom_percent_h() {
    assert_eq!(input().to_string_fmt("%h").unwrap(), "21");
}

#[test]
fn format_custom_hh() {
    assert_eq!(input().to_string_fmt("hh").unwrap(), "21");
}

#[test]
fn format_custom_percent_m() {
    assert_eq!(input().to_string_fmt("%m").unwrap(), "21");
}

#[test]
fn format_custom_mm() {
    assert_eq!(input().to_string_fmt("mm").unwrap(), "21");
}

#[test]
fn format_custom_percent_s() {
    assert_eq!(input().to_string_fmt("%s").unwrap(), "18");
}

#[test]
fn format_custom_ss() {
    assert_eq!(input().to_string_fmt("ss").unwrap(), "18");
}

#[test]
fn format_custom_fractional_lowercase_f() {
    // Lowercase f* always emits exactly N digits (no trailing-zero trimming)
    assert_eq!(input().to_string_fmt("%f").unwrap(), "9");
    assert_eq!(input().to_string_fmt("ff").unwrap(), "91");
    assert_eq!(input().to_string_fmt("fff").unwrap(), "910");
    assert_eq!(input().to_string_fmt("ffff").unwrap(), "9101");
    assert_eq!(input().to_string_fmt("fffff").unwrap(), "91011");
    assert_eq!(input().to_string_fmt("ffffff").unwrap(), "910111");
    assert_eq!(input().to_string_fmt("fffffff").unwrap(), "9101112");
}

#[test]
fn format_custom_fractional_uppercase_f() {
    // Uppercase F* trims trailing zeros
    assert_eq!(input().to_string_fmt("%F").unwrap(), "9");
    assert_eq!(input().to_string_fmt("FF").unwrap(), "91");
    assert_eq!(input().to_string_fmt("FFF").unwrap(), "91"); // "910" → trim trailing 0 → "91"
    assert_eq!(input().to_string_fmt("FFFF").unwrap(), "9101");
    assert_eq!(input().to_string_fmt("FFFFF").unwrap(), "91011");
    assert_eq!(input().to_string_fmt("FFFFFF").unwrap(), "910111");
    assert_eq!(input().to_string_fmt("FFFFFFF").unwrap(), "9101112");
}

#[test]
fn format_custom_composite_dd_dot_ss() {
    // Escape sequences: \. is a literal dot
    assert_eq!(input().to_string_fmt(r"dd\.ss").unwrap(), "142.18");
}

#[test]
fn format_custom_backslash_escape_inside_quote() {
    // C# TimeSpanFormat.cs FormatCustomized / ParseQuoteString: inside a quoted
    // literal, '\' escapes the next character — '\:' inside quotes emits ':'.
    // E.g. format "'h\:m'" with value 1h2m → "h:m" (not "h\:m").
    assert_eq!(
        TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR + 2 * TimeSpan::TICKS_PER_MINUTE)
            .to_string_fmt(r#"'h\:m'"#)
            .unwrap(),
        "h:m",
    );
}

#[test]
fn format_custom_composite_dd_dot_ss_is_culture_invariant() {
    // Custom format specifiers are not culture-sensitive
    for culture in [Locale::en, Locale::en_GB, Locale::fr, Locale::hr] {
        assert_eq!(
            input()
                .to_string_fmt_with_culture(r"dd\.ss", culture)
                .unwrap(),
            "142.18",
            "culture={culture:?}",
        );
    }
}

#[test]
fn format_custom_dddddd_dot_ss() {
    assert_eq!(input().to_string_fmt(r"dddddd\.ss").unwrap(), "000142.18");
}

// ── Standard format "c" / "t" / "T" (constant, culture-invariant) ─────────────
//
// All three produce identical output; culture is ignored.

#[test]
fn format_constant_large_value() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            input().to_string_fmt(fmt).unwrap(),
            "142.21:21:18.9101112",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_zero() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::ZERO.to_string_fmt(fmt).unwrap(),
            "00:00:00",
            "format={fmt:?}"
        );
    }
}

#[test]
fn format_constant_one_tick() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::from_ticks(1).to_string_fmt(fmt).unwrap(),
            "00:00:00.0000001",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_minus_one_tick() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::from_ticks(-1).to_string_fmt(fmt).unwrap(),
            "-00:00:00.0000001",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_max_value() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::MAX_VALUE.to_string_fmt(fmt).unwrap(),
            "10675199.02:48:05.4775807",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_min_value() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::MIN_VALUE.to_string_fmt(fmt).unwrap(),
            "-10675199.02:48:05.4775808",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_hms() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            hms(1, 2, 3).to_string_fmt(fmt).unwrap(),
            "01:02:03",
            "format={fmt:?}"
        );
        assert_eq!(
            neg(hms(1, 2, 3)).to_string_fmt(fmt).unwrap(),
            "-01:02:03",
            "format={fmt:?}",
        );
        assert_eq!(
            hms(12, 34, 56).to_string_fmt(fmt).unwrap(),
            "12:34:56",
            "format={fmt:?}"
        );
    }
}

#[test]
fn format_constant_dhms_overflow_hours() {
    // 12 days + 34 hours normalises to 13 days 10 hours
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            dhms(12, 34, 56, 23).to_string_fmt(fmt).unwrap(),
            "13.10:56:23",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_dhmsm() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            dhmsm(12, 34, 56, 23, 45).to_string_fmt(fmt).unwrap(),
            "13.10:56:23.0450000",
            "format={fmt:?}",
        );
        assert_eq!(
            dhmsm(0, 23, 59, 59, 999).to_string_fmt(fmt).unwrap(),
            "23:59:59.9990000",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_is_culture_invariant() {
    // "c"/"t"/"T" output must not change with culture
    for fmt in ["c", "t", "T"] {
        for culture in [Locale::en, Locale::en_GB, Locale::fr, Locale::hr] {
            assert_eq!(
                input().to_string_fmt_with_culture(fmt, culture).unwrap(),
                "142.21:21:18.9101112",
                "format={fmt:?} culture={culture:?}",
            );
        }
    }
}

// ── Display (equivalent to C# null format, which also uses "c") ───────────────

#[test]
fn format_display_equals_c_format() {
    assert_eq!(input().to_string(), input().to_string_fmt("c").unwrap());
    assert_eq!(TimeSpan::ZERO.to_string(), "00:00:00");
    assert_eq!(TimeSpan::MAX_VALUE.to_string(), "10675199.02:48:05.4775807");
    assert_eq!(
        TimeSpan::MIN_VALUE.to_string(),
        "-10675199.02:48:05.4775808"
    );
}

// ── Standard format "g" (general short, culture-sensitive) ────────────────────

#[test]
fn format_g_invariant_large_value() {
    assert_eq!(
        input().to_string_fmt_with_culture("g", Locale::en).unwrap(),
        "142:21:21:18.9101112",
    );
}

#[test]
fn format_g_invariant_common_values() {
    let cases: &[(TimeSpan, &str)] = &[
        (TimeSpan::ZERO, "0:00:00"),
        (TimeSpan::from_ticks(1), "0:00:00.0000001"),
        (TimeSpan::from_ticks(-1), "-0:00:00.0000001"),
        (TimeSpan::MAX_VALUE, "10675199:2:48:05.4775807"),
        (TimeSpan::MIN_VALUE, "-10675199:2:48:05.4775808"),
        (hms(1, 2, 3), "1:02:03"),
        (neg(hms(1, 2, 3)), "-1:02:03"),
        (hms(12, 34, 56), "12:34:56"),
        (dhms(12, 34, 56, 23), "13:10:56:23"),
        (dhmsm(12, 34, 56, 23, 45), "13:10:56:23.045"),
        (dhmsm(0, 23, 59, 59, 999), "23:59:59.999"),
    ];
    for (ts, expected) in cases {
        assert_eq!(
            ts.to_string_fmt_with_culture("g", Locale::en).unwrap(),
            *expected,
            "TimeSpan({:?})",
            ts.ticks(),
        );
    }
}

#[test]
fn format_g_fr_fr_uses_comma_separator() {
    let cases: &[(TimeSpan, &str)] = &[
        (input(), "142:21:21:18,9101112"),
        (TimeSpan::ZERO, "0:00:00"),
        (TimeSpan::from_ticks(1), "0:00:00,0000001"),
        (TimeSpan::from_ticks(-1), "-0:00:00,0000001"),
        (TimeSpan::MAX_VALUE, "10675199:2:48:05,4775807"),
        (TimeSpan::MIN_VALUE, "-10675199:2:48:05,4775808"),
        (hms(1, 2, 3), "1:02:03"),
        (neg(hms(1, 2, 3)), "-1:02:03"),
        (hms(12, 34, 56), "12:34:56"),
        (dhms(12, 34, 56, 23), "13:10:56:23"),
        (dhmsm(12, 34, 56, 23, 45), "13:10:56:23,045"),
        (dhmsm(0, 23, 59, 59, 999), "23:59:59,999"),
    ];
    for (ts, expected) in cases {
        assert_eq!(
            ts.to_string_fmt_with_culture("g", Locale::fr).unwrap(),
            *expected,
            "TimeSpan({:?})",
            ts.ticks(),
        );
    }
}

// ── Standard format "G" (general long, culture-sensitive) ─────────────────────

#[test]
fn format_g_upper_invariant_large_value() {
    assert_eq!(
        input().to_string_fmt_with_culture("G", Locale::en).unwrap(),
        "142:21:21:18.9101112",
    );
}

#[test]
fn format_g_upper_invariant_common_values() {
    let cases: &[(TimeSpan, &str)] = &[
        (TimeSpan::ZERO, "0:00:00:00.0000000"),
        (TimeSpan::from_ticks(1), "0:00:00:00.0000001"),
        (TimeSpan::from_ticks(-1), "-0:00:00:00.0000001"),
        (TimeSpan::MAX_VALUE, "10675199:02:48:05.4775807"),
        (TimeSpan::MIN_VALUE, "-10675199:02:48:05.4775808"),
        (hms(1, 2, 3), "0:01:02:03.0000000"),
        (neg(hms(1, 2, 3)), "-0:01:02:03.0000000"),
        (hms(12, 34, 56), "0:12:34:56.0000000"),
        (dhms(12, 34, 56, 23), "13:10:56:23.0000000"),
        (dhmsm(12, 34, 56, 23, 45), "13:10:56:23.0450000"),
        (dhmsm(0, 23, 59, 59, 999), "0:23:59:59.9990000"),
    ];
    for (ts, expected) in cases {
        assert_eq!(
            ts.to_string_fmt_with_culture("G", Locale::en).unwrap(),
            *expected,
            "TimeSpan({:?})",
            ts.ticks(),
        );
    }
}

#[test]
fn format_g_upper_fr_fr_uses_comma_separator() {
    let cases: &[(TimeSpan, &str)] = &[
        (input(), "142:21:21:18,9101112"),
        (TimeSpan::ZERO, "0:00:00:00,0000000"),
        (TimeSpan::from_ticks(1), "0:00:00:00,0000001"),
        (TimeSpan::from_ticks(-1), "-0:00:00:00,0000001"),
        (TimeSpan::MAX_VALUE, "10675199:02:48:05,4775807"),
        (TimeSpan::MIN_VALUE, "-10675199:02:48:05,4775808"),
        (hms(1, 2, 3), "0:01:02:03,0000000"),
        (neg(hms(1, 2, 3)), "-0:01:02:03,0000000"),
        (hms(12, 34, 56), "0:12:34:56,0000000"),
        (dhms(12, 34, 56, 23), "13:10:56:23,0000000"),
        (dhmsm(12, 34, 56, 23, 45), "13:10:56:23,0450000"),
        (dhmsm(0, 23, 59, 59, 999), "0:23:59:59,9990000"),
    ];
    for (ts, expected) in cases {
        assert_eq!(
            ts.to_string_fmt_with_culture("G", Locale::fr).unwrap(),
            *expected,
            "TimeSpan({:?})",
            ts.ticks(),
        );
    }
}

#[test]
fn format_custom_unclosed_quote() {
    // C# TimeSpanFormat.cs ParseQuoteString: reaching end of format without closing quote
    // throws FormatException. Lines ~210-230.
    assert_eq!(
        input().to_string_fmt("'abc").unwrap_err().to_string(),
        r#"quoted literal is not closed
  "'abc"
   ^"#
    );
    assert_eq!(
        input().to_string_fmt(r#""abc"#).unwrap_err().to_string(),
        r#"quoted literal is not closed
  ""abc"
   ^"#
    );
}

#[test]
fn format_custom_invalid_percent() {
    // C# TimeSpanFormat.cs FormatCustomized: "%%" or lone "%" at end throws FormatException.
    // Lines ~180-195.
    assert_eq!(
        input().to_string_fmt("%%").unwrap_err().to_string(),
        r#"'%' must be followed by a single specifier (d h m s f F)
  "%%"
   ^"#
    );
    assert_eq!(
        input().to_string_fmt("%").unwrap_err().to_string(),
        r#"'%' must be followed by a single specifier (d h m s f F)
  "%"
   ^"#
    );
}

#[test]
fn format_custom_trailing_escape() {
    // C# TimeSpanFormat.cs FormatCustomized: trailing '\' with no following char throws FormatException.
    // Lines ~200-205.
    assert_eq!(
        input().to_string_fmt(r"\").unwrap_err().to_string(),
        r#"trailing '\' must be followed by a character to escape
  "\"
   ^"#
    );
}

// ── Docs: standard-timespan-format-strings ────────────────────────────────────
//
// Examples drawn from:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings

// C# doc "c" section arithmetic examples — format "c" output
// new TimeSpan(7, 45, 16) → 07:45:16
// new TimeSpan(18, 12, 38) → 18:12:38
// subtraction → -10:27:22
// addition → 1.01:57:54
#[test]
fn doc_standard_c_format_arithmetic() {
    let interval1 = hms(7, 45, 16);
    let interval2 = hms(18, 12, 38);
    let diff = interval1 - interval2;
    let sum = interval1 + interval2;
    assert_eq!(interval1.to_string_fmt("c").unwrap(), "07:45:16");
    assert_eq!(interval2.to_string_fmt("c").unwrap(), "18:12:38");
    assert_eq!(diff.to_string_fmt("c").unwrap(), "-10:27:22");
    assert_eq!(sum.to_string_fmt("c").unwrap(), "1.01:57:54");
}

// new TimeSpan(0, 0, 1, 14, 365) + TimeSpan.FromTicks(2143756) with "c"
// = 00:01:14.3650000 + 00:00:00.2143756 = 00:01:14.5793756
#[test]
fn doc_standard_c_format_from_ticks() {
    let interval1 = dhmsm(0, 0, 1, 14, 365);
    let interval2 = TimeSpan::from_ticks(2143756);
    let sum = interval1 + interval2;
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
    let interval1 = hms(7, 45, 16);
    let interval2 = hms(18, 12, 38);
    let diff = interval1 - interval2;
    let sum = interval1 + interval2;
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
    let interval1 = dhmsm(0, 0, 1, 14, 36);
    let interval2 = TimeSpan::from_ticks(2143756);
    let sum = interval1 + interval2;
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
    let interval1 = hms(7, 45, 16);
    let interval2 = hms(18, 12, 38);
    let diff = interval1 - interval2;
    let sum = interval1 + interval2;
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
    let interval1 = dhmsm(0, 0, 1, 14, 36);
    let interval2 = TimeSpan::from_ticks(2143756);
    let sum = interval1 + interval2;
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
    let ts = dhmsm(1, 12, 24, 2, 0);
    assert_eq!(ts.to_string_fmt("%d").unwrap(), "1");
}

#[test]
fn doc_custom_intro_format_dd_dot_hh_colon_mm_colon_ss() {
    let ts = dhmsm(1, 12, 24, 2, 0);
    assert_eq!(ts.to_string_fmt(r"dd\.hh\:mm\:ss").unwrap(), "01.12:24:02");
}

// "hh" formatting: new TimeSpan(14, 3, 17) with d\.hh\:mm\:ss → "0.14:03:17"
//                  new TimeSpan(3, 4, 3, 17) with d\.hh\:mm\:ss → "3.04:03:17"
#[test]
fn doc_custom_hh_format_d_dot_hh_colon_mm_colon_ss() {
    // new TimeSpan(14, 3, 17) = 14h 3m 17s → no days
    let ts1 = hms(14, 3, 17);
    assert_eq!(ts1.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "0.14:03:17");
    // new TimeSpan(3, 4, 3, 17) = 3d 4h 3m 17s
    let ts2 = dhms(3, 4, 3, 17);
    assert_eq!(ts2.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "3.04:03:17");
}

// "mm" formatting: (arriveTime - departTime) with hh\:mm → "05:16"
// departTime = new TimeSpan(11, 12, 00), arriveTime = new TimeSpan(16, 28, 00)
#[test]
fn doc_custom_mm_format_travel_time() {
    let depart = hms(11, 12, 0);
    let arrive = hms(16, 28, 0);
    let elapsed = arrive - depart;
    assert_eq!(elapsed.to_string_fmt(r"hh\:mm").unwrap(), "05:16");
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

// "F" formatting examples from the doc
// TimeSpan.Parse("0:0:3.669"):  %F → "6"
// TimeSpan.Parse("0:0:3.091"):  ss\.F → "03."  (zero tenths → nothing after dot but dot is literal)
#[test]
fn doc_custom_uppercase_f_format() {
    // 3.669s → 3s + 669ms → tenths digit = 6
    let ts1 = dhmsm(0, 0, 0, 3, 669);
    assert_eq!(ts1.to_string_fmt("%F").unwrap(), "6");
    // 3.091s → 3s + 91ms → tenths digit = 0 → F outputs nothing
    // but combined with "ss\.F", the dot is a literal so output is "03."
    let ts2 = dhmsm(0, 0, 0, 3, 91);
    assert_eq!(ts2.to_string_fmt(r"ss\.F").unwrap(), "03.");
}

// "FF" formatting examples from the doc
// TimeSpan.Parse("0:0:3.697"):  FF → "69"
// TimeSpan.Parse("0:0:3.809"):  ss\.FF → "03.8"  (trailing zero trimmed)
#[test]
fn doc_custom_uppercase_ff_format() {
    let ts1 = dhmsm(0, 0, 0, 3, 697);
    assert_eq!(ts1.to_string_fmt("FF").unwrap(), "69");
    // 3.809s → hundredths = 80 → trim trailing zero → "8"
    let ts2 = dhmsm(0, 0, 0, 3, 809);
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
    let ts = hms(0, 32, 45);
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
    let ts1 = hms(4, 3, 17);
    assert_eq!(ts1.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "0.04:03:17");
    // new TimeSpan(3, 4, 3, 17) = 3d 4h 3m 17s
    let ts2 = dhms(3, 4, 3, 17);
    assert_eq!(ts2.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "3.04:03:17");
}

// "dd"-"dddddddd" — formatting examples
// new TimeSpan(0, 23, 17, 47) with dd\.hh\:mm\:ss → "00.23:17:47"
// new TimeSpan(365, 21, 19, 45) with dd\.hh\:mm\:ss → "365.21:19:45"
// new TimeSpan(365, 21, 19, 45) with dddd\.hh\:mm\:ss → "0365.21:19:45"
#[test]
fn doc_custom_dd_format_padded_days() {
    let ts1 = dhms(0, 23, 17, 47);
    let ts2 = dhms(365, 21, 19, 45);
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
    let ts1 = hms(14, 3, 17);
    assert_eq!(ts1.to_string_fmt(r"d\.h\:mm\:ss").unwrap(), "0.14:03:17");
    let ts2 = dhms(3, 4, 3, 17);
    assert_eq!(ts2.to_string_fmt(r"d\.h\:mm\:ss").unwrap(), "3.4:03:17");
}

// "m" custom specifier — formatting example
// new TimeSpan(0, 6, 32) with m\:ss → "6:32"
// C# doc also shows new TimeSpan(3, 4, 3, 17) with m\:ss but gives "18:44"
// (this is odd; likely a bug in VB example; C# shows ts2 = new TimeSpan(0, 18, 44))
// We test the straightforward case.
#[test]
fn doc_custom_m_format_m_colon_ss() {
    let ts1 = hms(0, 6, 32);
    assert_eq!(ts1.to_string_fmt(r"m\:ss").unwrap(), "6:32");
}

// "s" custom specifier — format example
// endTime - startTime = 6s 3ms
// with s\:fff → "6:003"
#[test]
fn doc_custom_s_format_s_colon_fff() {
    // startTime = new TimeSpan(0, 12, 30, 15, 0), endTime = new TimeSpan(0, 12, 30, 21, 3)
    // diff = 6s + 3ms
    let diff = dhmsm(0, 0, 0, 6, 3);
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

// ── Coverage gaps identified by C# docs agent ────────────────────────────────
//
// These cases appeared in the Microsoft documentation examples but were not
// yet present in the test suite.
// Ref: https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings

#[test]
fn format_custom_percent_d_zero() {
    // C# docs: new TimeSpan(0,0,0,0,0).ToString("%d") → "0"
    assert_eq!(TimeSpan::ZERO.to_string_fmt("%d").unwrap(), "0");
}

#[test]
fn format_custom_negative_drops_sign() {
    // Custom format strings have no sign specifier — negative TimeSpans output
    // the absolute component values. C# docs: (-TimeSpan.FromHours(1)).ToString(@"hh\:mm\:ss")
    // → "01:00:00" (no minus sign).
    assert_eq!(
        (-hms(1, 0, 0)).to_string_fmt(r"hh\:mm\:ss").unwrap(),
        "01:00:00",
    );
}

#[test]
fn format_g_one_millisecond() {
    // C# docs: new TimeSpan(0,0,0,0,1).ToString("g") → "0:00:00.001"
    // (milliseconds in the short general format)
    assert_eq!(
        dhmsm(0, 0, 0, 0, 1).to_string_fmt("g").unwrap(),
        "0:00:00.001"
    );
}

#[test]
fn format_g_upper_whole_day() {
    // C# docs: new TimeSpan(1,0,0,0,0).ToString("G") → "1:00:00:00.0000000"
    // (long general format always emits all 7 fraction digits)
    assert_eq!(
        dhms(1, 0, 0, 0).to_string_fmt("G").unwrap(),
        "1:00:00:00.0000000"
    );
}
