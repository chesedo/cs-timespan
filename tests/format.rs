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
    TimeSpan::from_ticks(-t.ticks())
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
        r#"unrecognised specifier
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
        r#"'%' must be followed by a single specifier
  "%%"
   ^"#
    );
    assert_eq!(
        input().to_string_fmt("%").unwrap_err().to_string(),
        r#"'%' must be followed by a single specifier
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
        r#"'\' at end of format string
  "\"
   ^"#
    );
}
