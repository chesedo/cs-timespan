# cs-timespan

A Rust implementation of C#'s `System.TimeSpan`, for working with serialized C# time intervals.

Internally stores a signed tick count where **1 tick = 100 nanoseconds**, identical to the C# representation. If you are migrating code from C# to Rust and need to parse or format time intervals exactly as .NET does, this crate is for you.

## Usage

```toml
[dependencies]
cs-timespan = "0.1"

# Optional: conversions to/from chrono::TimeDelta
cs-timespan = { version = "0.1", features = ["chrono"] }
```

## Parsing

`TimeSpan::parse` is lenient (mirrors `TimeSpan.Parse`); `TimeSpan::parse_exact` requires an exact format match (mirrors `TimeSpan.ParseExact`):

```rust
use cs_timespan::TimeSpan;

// Lenient parse accepts multiple formats for the same value
let ts = TimeSpan::parse("1:2:3:4").unwrap();

// parse_exact requires the input to match the format precisely
let ts2 = TimeSpan::parse_exact("1.02:03:04", "c").unwrap();
assert_eq!(ts, ts2);

// Try multiple formats at once
let ts3 = TimeSpan::parse_exact_any("03:45", &[r"hh\:mm", "g"]).unwrap();
```

## Formatting

`Display` uses the constant `"c"` format. `to_string_fmt` accepts any standard or custom format string:

```rust
use cs_timespan::TimeSpan;

let ts = TimeSpan::from_ticks(937_845_678_900);
assert_eq!(ts.to_string(),                 "1.02:03:04.5678900");
assert_eq!(ts.to_string_fmt("g").unwrap(), "1:2:03:04.56789");
assert_eq!(ts.to_string_fmt(r"d\.hh\:mm\:ss").unwrap(), "1.02:03:04");
```

## Arithmetic

Standard Rust operators work on `TimeSpan` values:

```rust
use cs_timespan::TimeSpan;

let hour = TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR);
let half = TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR / 2);

assert_eq!((hour + half).to_string(), "01:30:00");
assert_eq!((hour - half).to_string(), "00:30:00");
assert_eq!((hour * 3).to_string(),    "03:00:00");
assert_eq!((hour / 2).to_string(),    "00:30:00");
assert_eq!((-hour).to_string(),       "-01:00:00");

// Ratio between two spans (returns f64)
let ratio = hour / half; // 2.0
```

## Format strings

This crate supports the same standard and custom format specifiers as C#:

| Format | Description | Example output |
|--------|-------------|----------------|
| `"c"` / `"t"` / `"T"` | Constant, culture-invariant | `1.02:03:04.5678900` |
| `"g"` | General short, culture-sensitive | `1:2:03:04.56789` |
| `"G"` | General long, culture-sensitive | `1:02:03:04.5678900` |
| `d`, `dd`–`dddddddd` | Days component | `1`, `01` |
| `h` / `hh` | Hours component | `2`, `02` |
| `m` / `mm` | Minutes component | `3`, `03` |
| `s` / `ss` | Seconds component | `4`, `04` |
| `f`–`fffffff` | Sub-second digits (exact count) | `5678900` |
| `F`–`FFFFFFF` | Sub-second digits (trailing zeros trimmed) | `56789` |
| `%x` | Single-specifier prefix | `%h` → hours only |
| `\x` | Literal character escape | `\:` → `:` |
| `'...'` / `"..."` | Quoted literal | `'min'` → `min` |

Refer to the Microsoft documentation for the full reference:
- [Standard TimeSpan format strings](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings)
- [Custom TimeSpan format strings](https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings)

## Locale support

Methods with a `_with_culture` suffix accept a `Locale` to control the decimal separator used in fractional seconds:

```rust
use cs_timespan::{TimeSpan, Locale};

// Croatian locale uses ',' as the decimal separator
let ts = TimeSpan::parse_with_culture("6:12:14:45,348", Locale::hr).unwrap();
assert_eq!(ts, TimeSpan::parse_with_culture("6:12:14:45.348", Locale::en).unwrap());

// Format with French locale
let ts = TimeSpan::parse("00:00:01.5").unwrap();
assert_eq!(ts.to_string_fmt_with_culture("g", Locale::fr).unwrap(), "0:00:01,5");
```

## Conversions

`TimeSpan` converts to and from `std::time::Duration`. Negative values cannot be represented as `Duration`:

```rust
use cs_timespan::TimeSpan;
use std::time::Duration;

let ts = TimeSpan::from(Duration::from_secs(90));
assert_eq!(ts.to_string(), "00:01:30");

let d = Duration::try_from(ts).unwrap();
assert_eq!(d, Duration::from_secs(90));
```

With the `chrono` feature, conversions to and from `chrono::TimeDelta` are also available.

## Interoperability notes

- `TimeSpan::from_ticks` / `TimeSpan::ticks()` give direct access to the underlying 100 ns tick count, matching C#'s `TimeSpan.Ticks`.
- `TimeSpan::ZERO`, `TimeSpan::MIN_VALUE`, and `TimeSpan::MAX_VALUE` match the C# constants.
- Tick-unit constants (`TICKS_PER_SECOND`, etc.) are provided for constructing values without a dependency on a time library.
