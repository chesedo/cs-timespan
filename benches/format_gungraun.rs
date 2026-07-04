use std::hint::black_box;

use cs_timespan::{Locale, TimeSpan};
use gungraun::{
    Callgrind, EventKind, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main,
};

#[library_benchmark]
#[bench::zero(TimeSpan::ZERO)]
#[bench::large(TimeSpan::from_ticks(123_456_789_101_112))]
#[bench::max(TimeSpan::MAX_VALUE)]
#[bench::min(TimeSpan::MIN_VALUE)]
fn format_constant(ts: TimeSpan) -> String {
    black_box(ts.to_string_fmt("c").unwrap())
}

#[library_benchmark]
#[bench::zero(TimeSpan::ZERO)]
#[bench::large(TimeSpan::from_ticks(123_456_789_101_112))]
#[bench::max(TimeSpan::MAX_VALUE)]
#[bench::min(TimeSpan::MIN_VALUE)]
fn format_display(ts: TimeSpan) -> String {
    black_box(ts.to_string())
}

#[library_benchmark]
#[bench::zero(TimeSpan::ZERO)]
#[bench::large(TimeSpan::from_ticks(123_456_789_101_112))]
#[bench::max(TimeSpan::MAX_VALUE)]
#[bench::min(TimeSpan::MIN_VALUE)]
fn format_g(ts: TimeSpan) -> String {
    black_box(ts.to_string_fmt_with_culture("g", Locale::en).unwrap())
}

#[library_benchmark]
#[bench::zero(TimeSpan::ZERO)]
#[bench::large(TimeSpan::from_ticks(123_456_789_101_112))]
#[bench::max(TimeSpan::MAX_VALUE)]
#[bench::min(TimeSpan::MIN_VALUE)]
fn format_g_upper(ts: TimeSpan) -> String {
    black_box(ts.to_string_fmt_with_culture("G", Locale::en).unwrap())
}

#[library_benchmark]
#[bench::zero(TimeSpan::ZERO)]
#[bench::large(TimeSpan::from_ticks(123_456_789_101_112))]
#[bench::max(TimeSpan::MAX_VALUE)]
#[bench::min(TimeSpan::MIN_VALUE)]
fn format_custom(ts: TimeSpan) -> String {
    black_box(ts.to_string_fmt(r"dddddd\.ss").unwrap())
}

library_benchmark_group!(
    name = format;
    benchmarks =
        format_constant,
        format_display,
        format_g,
        format_g_upper,
        format_custom,
);

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 0.5f64)]));
    library_benchmark_groups = format
);
