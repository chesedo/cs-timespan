use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use cs_timespan::{Locale, TimeSpan};

// Representative inputs for each parse path.
// Each tuple is (label, input_str).
fn lenient_inputs() -> [(&'static str, &'static str); 6] {
    [
        ("hms",              "12:24:02"),
        ("hms_frac",         "12:24:02.999"),
        ("d_dot_hms_frac",   "1.12:24:02.999"),
        ("d_colon_hms_frac", "6:12:14:45.348"),
        ("bare_days",        "10675199"),
        ("negative",         "-01.07:45:16.9990000"),
    ]
}

fn constant_inputs() -> [(&'static str, &'static str); 5] {
    [
        ("hms",          "12:24:02"),
        ("d_dot_hms",    "1.12:24:02"),
        ("with_frac",    "1.12:24:02.9990000"),
        ("negative",     "-01.07:45:16.9990000"),
        ("max",          "10675199.02:48:05.4775807"),
    ]
}

fn g_inputs() -> [(&'static str, &'static str); 5] {
    [
        ("days_only",  "42"),
        ("hm",         "12:34"),
        ("hms_frac",   "12:24:02.999"),
        ("d_hms_frac", "1:12:24:02.999"),
        ("negative",   "-01:07:45:16.999"),
    ]
}

fn g_upper_inputs() -> [(&'static str, &'static str); 4] {
    [
        ("zero",     "0:00:00:00.0000000"),
        ("common",   "1:12:24:02.9990000"),
        ("max",      "10675199:02:48:05.4775807"),
        ("negative", "-1:07:45:16.9990000"),
    ]
}

fn custom_inputs() -> [(&'static str, &'static str); 4] {
    [
        ("dd_h_m_s",     "12.23:32:43"),
        ("ddd_h_m_s_fff","012.23:32:43.893"),
        ("d_hh_mm_ss",   "12.05:02:03"),
        ("literal",      "12:34 minutes"),
    ]
}

// ── parse (lenient) ───────────────────────────────────────────────────────────

fn bench_parse_lenient(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_lenient");
    for (name, s) in lenient_inputs() {
        group.bench_with_input(BenchmarkId::new("invariant", name), s, |b, s| {
            b.iter(|| TimeSpan::parse(s))
        });
    }
    group.finish();
}

fn bench_parse_lenient_culture(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_lenient_culture");
    // Test with a locale whose decimal separator differs from '.'.
    let inputs = [
        ("hms_comma", "12:24:02,999"),
        ("full_comma", "6:12:14:45,348"),
    ];
    for (name, s) in inputs {
        group.bench_with_input(BenchmarkId::new("hr", name), s, |b, s| {
            b.iter(|| TimeSpan::parse_with_culture(s, Locale::hr))
        });
    }
    group.finish();
}

// ── parse_exact — standard formats ────────────────────────────────────────────

fn bench_parse_exact_c(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_exact_c");
    for (name, s) in constant_inputs() {
        group.bench_with_input(BenchmarkId::new("c", name), s, |b, s| {
            b.iter(|| TimeSpan::parse_exact(s, "c"))
        });
    }
    group.finish();
}

fn bench_parse_exact_g(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_exact_g");
    for (name, s) in g_inputs() {
        group.bench_with_input(BenchmarkId::new("g", name), s, |b, s| {
            b.iter(|| TimeSpan::parse_exact(s, "g"))
        });
    }
    group.finish();
}

fn bench_parse_exact_g_upper(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_exact_G");
    for (name, s) in g_upper_inputs() {
        group.bench_with_input(BenchmarkId::new("G", name), s, |b, s| {
            b.iter(|| TimeSpan::parse_exact(s, "G"))
        });
    }
    group.finish();
}

// ── parse_exact — custom format ───────────────────────────────────────────────

fn bench_parse_exact_custom(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_exact_custom");
    let fmts: [(&str, &str); 4] = [
        ("dd_h_m_s",      r"dd\.h\:m\:s"),
        ("ddd_h_m_s_fff", r"ddd\.h\:m\:s\.fff"),
        ("d_hh_mm_ss",    r"d\.hh\:mm\:ss"),
        ("literal",       r#"mm\:ss\ "minutes""#),
    ];
    for ((name, s), (_, fmt)) in custom_inputs().iter().zip(fmts.iter()) {
        group.bench_with_input(BenchmarkId::new(*fmt, *name), s, |b, s| {
            b.iter(|| TimeSpan::parse_exact(s, fmt))
        });
    }
    group.finish();
}

// ── parse_exact_any ───────────────────────────────────────────────────────────

fn bench_parse_exact_any(c: &mut Criterion) {
    const FORMATS: &[&str] = &["c", "g", "G"];
    let mut group = c.benchmark_group("parse_exact_any");
    let inputs = [
        // First format matches immediately.
        ("first_match",  "12:24:02"),
        // Last format matches (tries "c" and "g" first, fails, then "G").
        ("last_match",   "1:12:24:02.9990000"),
    ];
    for (name, s) in inputs {
        group.bench_with_input(BenchmarkId::new("c|g|G", name), s, |b, s| {
            b.iter(|| TimeSpan::parse_exact_any(s, FORMATS))
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(1))
        .sample_size(100)
        .noise_threshold(0.10);
    targets =
        bench_parse_lenient,
        bench_parse_lenient_culture,
        bench_parse_exact_c,
        bench_parse_exact_g,
        bench_parse_exact_g_upper,
        bench_parse_exact_custom,
        bench_parse_exact_any,
);
criterion_main!(benches);
