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

use cs_timespan::{Culture, TimeSpan};

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
    assert_eq!(input().to_string_fmt("%d"), "142");
}

#[test]
fn format_custom_dd() {
    assert_eq!(input().to_string_fmt("dd"), "142");
}

#[test]
fn format_custom_dddddd_padded() {
    assert_eq!(input().to_string_fmt("dddddd"), "000142");
}

#[test]
fn format_custom_percent_h() {
    assert_eq!(input().to_string_fmt("%h"), "21");
}

#[test]
fn format_custom_hh() {
    assert_eq!(input().to_string_fmt("hh"), "21");
}

#[test]
fn format_custom_percent_m() {
    assert_eq!(input().to_string_fmt("%m"), "21");
}

#[test]
fn format_custom_mm() {
    assert_eq!(input().to_string_fmt("mm"), "21");
}

#[test]
fn format_custom_percent_s() {
    assert_eq!(input().to_string_fmt("%s"), "18");
}

#[test]
fn format_custom_ss() {
    assert_eq!(input().to_string_fmt("ss"), "18");
}

#[test]
fn format_custom_fractional_lowercase_f() {
    // Lowercase f* always emits exactly N digits (no trailing-zero trimming)
    assert_eq!(input().to_string_fmt("%f"), "9");
    assert_eq!(input().to_string_fmt("ff"), "91");
    assert_eq!(input().to_string_fmt("fff"), "910");
    assert_eq!(input().to_string_fmt("ffff"), "9101");
    assert_eq!(input().to_string_fmt("fffff"), "91011");
    assert_eq!(input().to_string_fmt("ffffff"), "910111");
    assert_eq!(input().to_string_fmt("fffffff"), "9101112");
}

#[test]
fn format_custom_fractional_uppercase_f() {
    // Uppercase F* trims trailing zeros
    assert_eq!(input().to_string_fmt("%F"), "9");
    assert_eq!(input().to_string_fmt("FF"), "91");
    assert_eq!(input().to_string_fmt("FFF"), "91"); // "910" → trim trailing 0 → "91"
    assert_eq!(input().to_string_fmt("FFFF"), "9101");
    assert_eq!(input().to_string_fmt("FFFFF"), "91011");
    assert_eq!(input().to_string_fmt("FFFFFF"), "910111");
    assert_eq!(input().to_string_fmt("FFFFFFF"), "9101112");
}

#[test]
fn format_custom_composite_dd_dot_ss() {
    // Escape sequences: \. is a literal dot
    assert_eq!(input().to_string_fmt(r"dd\.ss"), "142.18");
}

#[test]
fn format_custom_composite_dd_dot_ss_is_culture_invariant() {
    // Custom format specifiers are not culture-sensitive
    assert_eq!(
        input().to_string_fmt_with_culture(r"dd\.ss", Culture::FrFR),
        "142.18",
    );
}

#[test]
fn format_custom_dddddd_dot_ss() {
    assert_eq!(input().to_string_fmt(r"dddddd\.ss"), "000142.18");
}

// ── Standard format "c" / "t" / "T" (constant, culture-invariant) ─────────────
//
// All three produce identical output; culture is ignored.

#[test]
fn format_constant_large_value() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            input().to_string_fmt(fmt),
            "142.21:21:18.9101112",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_zero() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(TimeSpan::ZERO.to_string_fmt(fmt), "00:00:00", "format={fmt:?}");
    }
}

#[test]
fn format_constant_one_tick() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::from_ticks(1).to_string_fmt(fmt),
            "00:00:00.0000001",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_minus_one_tick() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::from_ticks(-1).to_string_fmt(fmt),
            "-00:00:00.0000001",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_max_value() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::MAX_VALUE.to_string_fmt(fmt),
            "10675199.02:48:05.4775807",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_min_value() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            TimeSpan::MIN_VALUE.to_string_fmt(fmt),
            "-10675199.02:48:05.4775808",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_hms() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(hms(1, 2, 3).to_string_fmt(fmt), "01:02:03", "format={fmt:?}");
        assert_eq!(
            neg(hms(1, 2, 3)).to_string_fmt(fmt),
            "-01:02:03",
            "format={fmt:?}",
        );
        assert_eq!(hms(12, 34, 56).to_string_fmt(fmt), "12:34:56", "format={fmt:?}");
    }
}

#[test]
fn format_constant_dhms_overflow_hours() {
    // 12 days + 34 hours normalises to 13 days 10 hours
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            dhms(12, 34, 56, 23).to_string_fmt(fmt),
            "13.10:56:23",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_dhmsm() {
    for fmt in ["c", "t", "T"] {
        assert_eq!(
            dhmsm(12, 34, 56, 23, 45).to_string_fmt(fmt),
            "13.10:56:23.0450000",
            "format={fmt:?}",
        );
        assert_eq!(
            dhmsm(0, 23, 59, 59, 999).to_string_fmt(fmt),
            "23:59:59.9990000",
            "format={fmt:?}",
        );
    }
}

#[test]
fn format_constant_is_culture_invariant() {
    // "c"/"t"/"T" output must not change with culture
    for fmt in ["c", "t", "T"] {
        for culture in [Culture::Invariant, Culture::FrFR] {
            assert_eq!(
                input().to_string_fmt_with_culture(fmt, culture),
                "142.21:21:18.9101112",
                "format={fmt:?} culture={culture:?}",
            );
        }
    }
}

// ── Display (equivalent to C# null format, which also uses "c") ───────────────

#[test]
fn format_display_equals_c_format() {
    assert_eq!(input().to_string(), input().to_string_fmt("c"));
    assert_eq!(TimeSpan::ZERO.to_string(), "00:00:00");
    assert_eq!(TimeSpan::MAX_VALUE.to_string(), "10675199.02:48:05.4775807");
    assert_eq!(TimeSpan::MIN_VALUE.to_string(), "-10675199.02:48:05.4775808");
}

// ── Standard format "g" (general short, culture-sensitive) ────────────────────

#[test]
fn format_g_invariant_large_value() {
    assert_eq!(
        input().to_string_fmt_with_culture("g", Culture::Invariant),
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
            ts.to_string_fmt_with_culture("g", Culture::Invariant),
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
            ts.to_string_fmt_with_culture("g", Culture::FrFR),
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
        input().to_string_fmt_with_culture("G", Culture::Invariant),
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
            ts.to_string_fmt_with_culture("G", Culture::Invariant),
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
            ts.to_string_fmt_with_culture("G", Culture::FrFR),
            *expected,
            "TimeSpan({:?})",
            ts.ticks(),
        );
    }
}
