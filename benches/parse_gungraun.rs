use std::hint::black_box;

use cs_timespan::{Locale, ParseError, TimeSpan};
use gungraun::{
    Callgrind, EventKind, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main,
};

#[library_benchmark]
#[bench::hms("12:24:02")]
#[bench::hms_frac("12:24:02.999")]
#[bench::d_dot_hms_frac("1.12:24:02.999")]
#[bench::d_colon_hms_frac("6:12:14:45.348")]
#[bench::bare_days("10675199")]
#[bench::negative("-01.07:45:16.9990000")]
fn parse_lenient(s: &str) -> Result<TimeSpan, ParseError> {
    black_box(TimeSpan::parse(black_box(s)))
}

#[library_benchmark]
#[bench::hms_comma("12:24:02,999")]
#[bench::full_comma("6:12:14:45,348")]
fn parse_lenient_culture(s: &str) -> Result<TimeSpan, ParseError> {
    black_box(TimeSpan::parse_with_culture(black_box(s), Locale::hr))
}

#[library_benchmark]
#[bench::hms("12:24:02")]
#[bench::d_dot_hms("1.12:24:02")]
#[bench::with_frac("1.12:24:02.9990000")]
#[bench::negative("-01.07:45:16.9990000")]
#[bench::max("10675199.02:48:05.4775807")]
fn parse_exact_c(s: &str) -> Result<TimeSpan, ParseError> {
    black_box(TimeSpan::parse_exact(black_box(s), "c"))
}

#[library_benchmark]
#[bench::days_only("42")]
#[bench::hm("12:34")]
#[bench::hms_frac("12:24:02.999")]
#[bench::d_hms_frac("1:12:24:02.999")]
#[bench::negative("-01:07:45:16.999")]
fn parse_exact_g(s: &str) -> Result<TimeSpan, ParseError> {
    black_box(TimeSpan::parse_exact(black_box(s), "g"))
}

#[library_benchmark]
#[bench::zero("0:00:00:00.0000000")]
#[bench::common("1:12:24:02.9990000")]
#[bench::max("10675199:02:48:05.4775807")]
#[bench::negative("-1:07:45:16.9990000")]
fn parse_exact_g_upper(s: &str) -> Result<TimeSpan, ParseError> {
    black_box(TimeSpan::parse_exact(black_box(s), "G"))
}

// Representative of the fast path affected by PR #35 (removal of the u128
// lenient-parsing path from custom-format parse_exact).
#[library_benchmark]
#[bench::dd_h_m_s("12.23:32:43", r"dd\.h\:m\:s")]
#[bench::ddd_h_m_s_fff("012.23:32:43.893", r"ddd\.h\:m\:s\.fff")]
#[bench::d_hh_mm_ss("12.05:02:03", r"d\.hh\:mm\:ss")]
#[bench::literal("12:34 minutes", r#"mm\:ss\ "minutes""#)]
fn parse_exact_custom(s: &str, fmt: &str) -> Result<TimeSpan, ParseError> {
    black_box(TimeSpan::parse_exact(black_box(s), black_box(fmt)))
}

#[library_benchmark]
#[bench::first_match("12:24:02")]
#[bench::last_match("1:12:24:02.9990000")]
fn parse_exact_any(s: &str) -> Result<TimeSpan, ParseError> {
    const FORMATS: &[&str] = &["c", "g", "G"];
    black_box(TimeSpan::parse_exact_any(black_box(s), FORMATS))
}

library_benchmark_group!(
    name = parse;
    benchmarks =
        parse_lenient,
        parse_lenient_culture,
        parse_exact_c,
        parse_exact_g,
        parse_exact_g_upper,
        parse_exact_custom,
        parse_exact_any,
);

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 0.5f64)]));
    library_benchmark_groups = parse
);
