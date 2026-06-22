use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use cs_timespan::{Locale, TimeSpan};

fn inputs() -> [(&'static str, TimeSpan); 4] {
    [
        ("zero", TimeSpan::ZERO),
        ("large", TimeSpan::from_ticks(123_456_789_101_112)),
        ("max", TimeSpan::MAX_VALUE),
        ("min", TimeSpan::MIN_VALUE),
    ]
}

fn bench_format_constant(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_constant");
    for (name, ts) in inputs() {
        group.bench_with_input(BenchmarkId::new("c", name), &ts, |b, ts| {
            b.iter(|| ts.to_string_fmt("c"))
        });
    }
    group.finish();
}

fn bench_format_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_display");
    for (name, ts) in inputs() {
        group.bench_with_input(BenchmarkId::new("to_string", name), &ts, |b, ts| {
            b.iter(|| ts.to_string())
        });
    }
    group.finish();
}

fn bench_format_g(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_g");
    for (name, ts) in inputs() {
        group.bench_with_input(BenchmarkId::new("g/invariant", name), &ts, |b, ts| {
            b.iter(|| ts.to_string_fmt_with_culture("g", Locale::en))
        });
    }
    group.finish();
}

fn bench_format_g_upper(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_G");
    for (name, ts) in inputs() {
        group.bench_with_input(BenchmarkId::new("G/invariant", name), &ts, |b, ts| {
            b.iter(|| ts.to_string_fmt_with_culture("G", Locale::en))
        });
    }
    group.finish();
}

fn bench_format_custom(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_custom");
    for (name, ts) in inputs() {
        group.bench_with_input(
            BenchmarkId::new(r"dddddd\.ss", name),
            &ts,
            |b, ts| b.iter(|| ts.to_string_fmt(r"dddddd\.ss")),
        );
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(2))
        .warm_up_time(Duration::from_millis(500))
        .sample_size(50);
    targets =
        bench_format_constant,
        bench_format_display,
        bench_format_g,
        bench_format_g_upper,
        bench_format_custom,
);
criterion_main!(benches);
