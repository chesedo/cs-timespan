# cs-timespan

A Rust library whose sole intent is to be a drop-in replacement for the C# `System.TimeSpan` struct. If you are migrating code from C# to Rust and need to parse or format time intervals in exactly the same way .NET does, this crate is for you.

## Implementation Progress

### Interoperability

> Construction of time intervals is intentionally delegated to [`std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html) or [`chrono::TimeDelta`](https://docs.rs/chrono/latest/chrono/struct.TimeDelta.html). `std::time::Duration` is unsigned and cannot represent negative intervals; `chrono::TimeDelta` supports negative values and is the closer equivalent to `System.TimeSpan`. Use whichever suits your project, then convert into this crate's types for parsing/formatting.

- [ ] `From<std::time::Duration>` for `TimeSpan` (always succeeds — `Duration` is unsigned)
- [ ] `TryFrom<TimeSpan>` for `std::time::Duration` (fails if the interval is negative)

The following require the `chrono` feature flag:

- [ ] `From<chrono::TimeDelta>` for `TimeSpan` *(feature: `chrono`)*
- [ ] `From<TimeSpan>` for `chrono::TimeDelta` *(feature: `chrono`)*

### Constants

- [ ] `TimeSpan::ZERO`
- [ ] `TimeSpan::MIN_VALUE`
- [ ] `TimeSpan::MAX_VALUE`
- [ ] `TimeSpan::TICKS_PER_MILLISECOND`
- [ ] `TimeSpan::TICKS_PER_SECOND`
- [ ] `TimeSpan::TICKS_PER_MINUTE`
- [ ] `TimeSpan::TICKS_PER_HOUR`
- [ ] `TimeSpan::TICKS_PER_DAY`

### Component Properties

- [ ] `.ticks() -> i64` — raw tick count
- [ ] `.days() -> i32` — whole-day component
- [ ] `.hours() -> i32` — hours component (0–23)
- [ ] `.minutes() -> i32` — minutes component (0–59)
- [ ] `.seconds() -> i32` — seconds component (0–59)
- [ ] `.milliseconds() -> i32` — milliseconds component (0–999)

### Total Properties

- [ ] `.total_days() -> f64`
- [ ] `.total_hours() -> f64`
- [ ] `.total_minutes() -> f64`
- [ ] `.total_seconds() -> f64`
- [ ] `.total_milliseconds() -> f64`

### Arithmetic

- [ ] `.add(rhs: TimeSpan) -> TimeSpan`
- [ ] `.subtract(rhs: TimeSpan) -> TimeSpan`
- [ ] `.negate() -> TimeSpan` — flip sign
- [ ] `.duration() -> TimeSpan` — absolute value
- [ ] `.multiply(factor: f64) -> TimeSpan`
- [ ] `.divide_by_scalar(divisor: f64) -> TimeSpan`
- [ ] `.divide_by_timespan(divisor: TimeSpan) -> f64` — ratio between two intervals
- [ ] Operator `+` (Add)
- [ ] Operator `-` (Sub, binary)
- [ ] Operator `-` (Neg, unary)
- [ ] Operator `*` (Mul, scalar on both sides)
- [ ] Operator `/` (Div by scalar)
- [ ] Operator `/` (Div by TimeSpan)

### Comparison

- [ ] `PartialEq` / `Eq`
- [ ] `PartialOrd` / `Ord`
- [ ] `TimeSpan::compare(t1, t2) -> i32` — static equivalent of `TimeSpan.Compare`

### Parsing — Lenient (`Parse` / `TryParse`)

- [ ] `TimeSpan::parse(s: &str) -> Result<TimeSpan, ParseError>` — invariant culture
- [ ] `TimeSpan::parse_with_culture(s: &str, culture: Culture) -> Result<...>` — culture-aware (affects decimal separator for `g`/`G` input)
- [ ] `TimeSpan::try_parse(s: &str) -> Option<TimeSpan>`
- [ ] `TimeSpan::try_parse_with_culture(s: &str, culture: Culture) -> Option<TimeSpan>`
- [ ] Accept bare integer as days (e.g. `"5"` → 5 days)
- [ ] Accept `hh:mm` (hours:minutes)
- [ ] Accept `hh:mm:ss`
- [ ] Accept `d.hh:mm:ss`
- [ ] Accept `d.hh:mm:ss.fffffff`
- [ ] Accept negative intervals via leading `-`

### Parsing — Strict (`ParseExact` / `TryParseExact`)

- [ ] `TimeSpan::parse_exact(s: &str, fmt: &str) -> Result<TimeSpan, ParseError>`
- [ ] `TimeSpan::parse_exact_any(s: &str, fmts: &[&str]) -> Result<...>` — try multiple formats
- [ ] `TimeSpan::parse_exact_with_culture(s, fmt, culture)`
- [ ] `TimeSpan::parse_exact_any_with_culture(s, fmts, culture)`
- [ ] `TimeSpan::try_parse_exact(s: &str, fmt: &str) -> Option<TimeSpan>`
- [ ] `TimeSpan::try_parse_exact_any(s: &str, fmts: &[&str]) -> Option<TimeSpan>`
- [ ] `TimeSpanStyles::None` — default behaviour
- [ ] `TimeSpanStyles::AssumeNegative` — treat input as negative even without leading `-`

### Standard Format Specifiers (parsing + formatting)

- [ ] `"c"` — constant/invariant: `[-][d.]hh:mm:ss[.fffffff]`
- [ ] `"t"` — alias for `"c"`
- [ ] `"T"` — alias for `"c"`
- [ ] `"g"` — general short, culture-sensitive: `[-][d:]h:mm:ss[.FFFFFFF]`
- [ ] `"G"` — general long, culture-sensitive: `[-]d:hh:mm:ss.fffffff`

### Custom Format Specifiers (parsing + formatting)

- [ ] `d` / `%d` — whole days, no leading zero
- [ ] `dd`–`dddddddd` — whole days, padded to N digits
- [ ] `h` / `%h` — hours component, no leading zero
- [ ] `hh` — hours component, leading zero
- [ ] `m` / `%m` — minutes component, no leading zero
- [ ] `mm` — minutes component, leading zero
- [ ] `s` / `%s` — seconds component, no leading zero
- [ ] `ss` — seconds component, leading zero
- [ ] `f` / `%f` — tenths of a second (exact 1 digit in parsing)
- [ ] `ff` — hundredths of a second (exact 2 digits in parsing)
- [ ] `fff` — milliseconds (exact 3 digits in parsing)
- [ ] `ffff` — ten-thousandths of a second (exact 4 digits)
- [ ] `fffff` — hundred-thousandths of a second (exact 5 digits)
- [ ] `ffffff` — millionths of a second (exact 6 digits)
- [ ] `fffffff` — ten-millionths / ticks (exact 7 digits)
- [ ] `F` / `%F` — tenths of a second, optional in parsing, no trailing zero in output
- [ ] `FF` — hundredths, optional digits in parsing, no trailing zeros
- [ ] `FFF` — milliseconds, optional digits, no trailing zeros
- [ ] `FFFF` — ten-thousandths, optional digits, no trailing zeros
- [ ] `FFFFF` — hundred-thousandths, optional digits, no trailing zeros
- [ ] `FFFFFF` — millionths, optional digits, no trailing zeros
- [ ] `FFFFFFF` — ten-millionths, optional digits, no trailing zeros
- [ ] `\x` — escape character (next char is literal)
- [ ] `'...'` — quoted literal string

### Formatting

- [ ] `Display` impl (default `"c"` format)
- [ ] `.to_string_fmt(fmt: &str) -> String`
- [ ] `.to_string_fmt_with_culture(fmt: &str, culture: Culture) -> String`
- [ ] Culture-sensitive decimal separator for `"g"` / `"G"` formats

### Standard Traits

- [ ] `Clone` / `Copy`
- [ ] `Debug`
- [ ] `Display`
- [ ] `Hash`
- [ ] `Default` (equivalent to `TimeSpan::ZERO`)
