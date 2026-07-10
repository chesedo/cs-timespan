//! A Rust implementation of C#'s [`System.TimeSpan`], for working with
//! serialized C# time intervals.
//!
//! Internally stores a signed tick count where 1 tick = 100 nanoseconds,
//! identical to the C# representation.
//!
//! # Is this crate for you?
//!
//! This crate exists for one reason: exact compatibility with C#'s `System.TimeSpan`.
//!
//! - **Just doing arithmetic with durations?** Use [`std::time::Duration`] or
//!   [`chrono::TimeDelta`](https://docs.rs/chrono/latest/chrono/struct.TimeDelta.html).
//! - **Need to parse or format human-readable duration strings (not C# format)?** Use
//!   [`humantime`](https://docs.rs/humantime) (`"1h 30m"`) or [`jiff`](https://docs.rs/jiff)
//!   (ISO 8601 `PT1H30M`).
//! - **Migrating C# code or exchanging data with a .NET system?** This crate is for you —
//!   it parses and formats `System.TimeSpan` strings exactly as .NET does.
//!
//! # Parsing
//!
//! [`TimeSpan::parse`] is lenient (mirrors `TimeSpan.Parse`);
//! [`TimeSpan::parse_exact`] requires an exact format match (mirrors
//! `TimeSpan.ParseExact`):
//!
//! ```
//! use cs_timespan::TimeSpan;
//!
//! // Lenient parse accepts multiple formats for the same value
//! let ts = TimeSpan::parse("1:2:3:4").unwrap();
//! assert_eq!(ts.ticks(), 937_840_000_000);
//!
//! // parse_exact requires the input to match the format precisely
//! let ts2 = TimeSpan::parse_exact("1.02:03:04", "c").unwrap();
//! assert_eq!(ts, ts2);
//! ```
//!
//! # Formatting
//!
//! [`Display`][std::fmt::Display] uses the constant `"c"` format.
//! [`TimeSpan::to_string_fmt`] accepts any standard or custom format string:
//!
//! ```
//! use cs_timespan::TimeSpan;
//!
//! let ts = TimeSpan::from_ticks(937_845_678_900);
//! assert_eq!(ts.to_string(),               "1.02:03:04.5678900");
//! assert_eq!(ts.to_string_fmt("g").unwrap(), "1:2:03:04.56789");
//! ```
//!
//! # Arithmetic
//!
//! Standard Rust operators work on [`TimeSpan`] values:
//!
//! ```
//! use cs_timespan::TimeSpan;
//!
//! let hour = TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR);
//! let half = TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR / 2);
//!
//! assert_eq!((hour + half).to_string(), "01:30:00");
//! assert_eq!((hour - half).to_string(), "00:30:00");
//! assert_eq!((hour * 3).to_string(),    "03:00:00");
//! assert_eq!((hour / 2).to_string(),    "00:30:00");
//! assert_eq!((-hour).to_string(),       "-01:00:00");
//!
//! // Ratio between two spans (returns f64)
//! let ratio = hour / half;
//! assert_eq!(ratio, 2.0);
//! ```
//!
//! # Format strings
//!
//! This crate supports the same standard and custom format specifiers as C#.
//! Refer to the Microsoft documentation for the full reference:
//!
//! - [Standard TimeSpan format strings] — `"c"`, `"g"`, `"G"`
//! - [Custom TimeSpan format strings] — `d`, `h`, `m`, `s`, `f`/`F`, `%x`, `\x`
//!
//! # Locale support
//!
//! Methods with a `_with_culture` suffix accept a [`Locale`] to control the
//! decimal separator used in fractional seconds. The default (invariant) culture
//! uses `.`:
//!
//! ```
//! use cs_timespan::{TimeSpan, Locale};
//!
//! // Croatian locale uses ',' as the decimal separator
//! let ts = TimeSpan::parse_with_culture("6:12:14:45,348", Locale::hr).unwrap();
//! assert_eq!(ts, TimeSpan::parse_with_culture("6:12:14:45.348", Locale::en).unwrap());
//! ```
//!
//! # Conversions
//!
//! [`TimeSpan`] converts to and from [`std::time::Duration`]. Negative values
//! cannot be represented as `Duration`; that direction returns [`NegativeTimeSpan`].
//!
//! With the optional `chrono` feature, conversions to and from
//! [`chrono::TimeDelta`](https://docs.rs/chrono/latest/chrono/struct.TimeDelta.html) are also available.
//!
//! [`System.TimeSpan`]: https://learn.microsoft.com/en-us/dotnet/api/system.timespan
//! [Standard TimeSpan format strings]: https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings
//! [Custom TimeSpan format strings]: https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings

pub use num_format::Locale;

mod fmt;
mod parse;
pub use fmt::{FormatError, FormatErrorKind};
pub use parse::{OverflowKind, ParseError, ParseErrorKind, TimeSpanStyles};

/// A C# `System.TimeSpan`-compatible time interval type for Rust.
///
/// Internally stores a tick count where 1 tick = 100 nanoseconds,
/// matching the C# representation exactly.
///
/// # Examples
///
/// ```
/// use cs_timespan::TimeSpan;
///
/// // Parse the constant "c" format
/// let ts = TimeSpan::parse("1.02:03:04.5678900").unwrap();
/// assert_eq!(ts.ticks(), 937_845_678_900);
///
/// // Round-trip through Display (which also uses the "c" format)
/// assert_eq!(ts.to_string(), "1.02:03:04.5678900");
///
/// // Use a locale-sensitive format with a French locale (comma separator)
/// use cs_timespan::Locale;
/// assert_eq!(
///     ts.to_string_fmt_with_culture("g", Locale::fr).unwrap(),
///     "1:2:03:04,56789",
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TimeSpan {
    ticks: i64,
}

/// Error returned when a negative [`TimeSpan`] is converted to [`std::time::Duration`].
///
/// `std::time::Duration` is unsigned; negative intervals have no representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegativeTimeSpan;

impl std::fmt::Display for NegativeTimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("cannot convert negative TimeSpan to Duration")
    }
}

impl std::error::Error for NegativeTimeSpan {}

/// Error returned when constructing a [`TimeSpan`] from a floating-point value fails.
///
/// Mirrors the `ArgumentException`/`OverflowException` thrown by C#'s
/// `TimeSpan.FromDays(double)` (and the other `From*(double)` factories).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatError {
    /// The value was NaN.
    Nan,
    /// The value is outside the range representable by `TimeSpan`.
    Overflow,
}

impl std::fmt::Display for FloatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatError::Nan => f.write_str("value cannot be NaN"),
            FloatError::Overflow => {
                f.write_str("value is outside the range representable by TimeSpan")
            }
        }
    }
}

impl std::error::Error for FloatError {}

/// Error returned when constructing a [`TimeSpan`] from integer units overflows
/// the range representable by `TimeSpan`.
///
/// Mirrors the `ArgumentOutOfRangeException` thrown by C#'s `TimeSpan.FromDays(int)`
/// (and the other integer `From*` factories).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSpanOverflow;

impl std::fmt::Display for TimeSpanOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("value is outside the range representable by TimeSpan")
    }
}

impl std::error::Error for TimeSpanOverflow {}

impl TimeSpan {
    // ── Nanosecond-unit constants ───────────────────────────────────────────────
    pub const NANOSECONDS_PER_TICK: i64 = 100;

    // ── Tick-unit constants ────────────────────────────────────────────────────
    pub const TICKS_PER_MICROSECOND: i64 = 10;
    pub const TICKS_PER_MILLISECOND: i64 = 10_000;
    pub const TICKS_PER_SECOND: i64 = 10_000_000;
    pub const TICKS_PER_MINUTE: i64 = 600_000_000;
    pub const TICKS_PER_HOUR: i64 = 36_000_000_000;
    pub const TICKS_PER_DAY: i64 = 864_000_000_000;

    // Microsecond-unit constants, used only by TimeSpanBuilder's overflow-safe sum.
    const MICROSECONDS_PER_MILLISECOND: i128 = 1_000;
    const MICROSECONDS_PER_SECOND: i128 = 1_000_000;
    const MICROSECONDS_PER_MINUTE: i128 = 60_000_000;
    const MICROSECONDS_PER_HOUR: i128 = 3_600_000_000;
    const MICROSECONDS_PER_DAY: i128 = 86_400_000_000;

    // ── Boundary constants ─────────────────────────────────────────────────────
    pub const ZERO: TimeSpan = TimeSpan { ticks: 0 };
    pub const MAX_VALUE: TimeSpan = TimeSpan { ticks: i64::MAX };
    pub const MIN_VALUE: TimeSpan = TimeSpan { ticks: i64::MIN };

    // Mirrors C#'s MinMilliseconds/MaxMilliseconds: i64::MIN/MAX ticks converted
    // to whole milliseconds, used to clamp total_milliseconds() below.
    #[allow(clippy::cast_precision_loss)] // exact: magnitude fits f64's mantissa
    const MIN_MILLISECONDS: f64 = (i64::MIN / Self::TICKS_PER_MILLISECOND) as f64;
    #[allow(clippy::cast_precision_loss)]
    const MAX_MILLISECONDS: f64 = (i64::MAX / Self::TICKS_PER_MILLISECOND) as f64;

    // ── Raw construction ───────────────────────────────────────────────────────
    /// Creates a `TimeSpan` from a raw tick count (1 tick = 100 ns).
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// let one_second = TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND);
    /// assert_eq!(one_second.to_string(), "00:00:01");
    /// ```
    #[must_use]
    pub const fn from_ticks(ticks: i64) -> Self {
        TimeSpan { ticks }
    }

    /// Returns the total number of ticks (1 tick = 100 ns).
    #[must_use]
    pub const fn ticks(self) -> i64 {
        self.ticks
    }

    // ── Float factory methods (mirror FromDays(double) / FromHours(double) / ...) ──
    // Named with an _f64 suffix (rather than plain from_days/from_hours/...) to avoid
    // clashing with the integer factories of the same conceptual name added separately
    // -- Rust has no overloading. Matches std::time::Duration's own from_secs/
    // from_secs_f64 precedent for the same integer-vs-float split.
    // i64::MAX rounds up to 2^63 when cast to f64 (i64::MIN is exact, being a power of
    // two); harmless here since it only widens the boundary check, and the saturating
    // "as i64" cast below still clamps to the correct value at that boundary (see
    // from_days_f64_max_value_boundary/from_days_f64_min_value_boundary tests).
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)] // bounds-checked against i64::MIN/MAX above
    fn interval(value: f64, scale: f64) -> Result<Self, FloatError> {
        if value.is_nan() {
            return Err(FloatError::Nan);
        }
        let ticks = value * scale;
        if ticks > i64::MAX as f64 || ticks < i64::MIN as f64 {
            return Err(FloatError::Overflow);
        }
        Ok(Self::from_ticks(ticks as i64))
    }

    // Mirrors C#'s TimeSpan.IntervalFromDoubleTicks (TimeSpan.cs#L645-L656), used by
    // the Multiply(double)/Divide(double) operators below. `ticks` is the rounded
    // tick count, but it can still be NaN here even though the caller's `factor`/
    // `divisor` isn't (e.g. `TimeSpan::ZERO.divide(0.0)` computes `0.0 / 0.0`), so
    // the NaN check below is still load-bearing, not just a range check.
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)] // bounds-checked against i64::MIN/MAX above
    fn interval_from_double_ticks(ticks: f64) -> Result<Self, FloatError> {
        if ticks.is_nan() || ticks > i64::MAX as f64 || ticks < i64::MIN as f64 {
            return Err(FloatError::Overflow);
        }
        Ok(Self::from_ticks(ticks as i64))
    }

    /// Creates a `TimeSpan` from a fractional number of days.
    ///
    /// Fractional ticks are truncated toward zero, not rounded.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `value` is NaN, or
    /// [`FloatError::Overflow`] if it's outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_days_f64(1.0).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY));
    /// ```
    #[allow(clippy::cast_precision_loss)] // TICKS_PER_DAY magnitude fits f64's mantissa
    pub fn from_days_f64(value: f64) -> Result<Self, FloatError> {
        Self::interval(value, Self::TICKS_PER_DAY as f64)
    }

    /// Creates a `TimeSpan` from a fractional number of hours.
    ///
    /// Fractional ticks are truncated toward zero, not rounded.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `value` is NaN, or
    /// [`FloatError::Overflow`] if it's outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_hours_f64(1.0).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR));
    /// ```
    #[allow(clippy::cast_precision_loss)]
    pub fn from_hours_f64(value: f64) -> Result<Self, FloatError> {
        Self::interval(value, Self::TICKS_PER_HOUR as f64)
    }

    /// Creates a `TimeSpan` from a fractional number of minutes.
    ///
    /// Fractional ticks are truncated toward zero, not rounded.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `value` is NaN, or
    /// [`FloatError::Overflow`] if it's outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_minutes_f64(1.0).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_MINUTE));
    /// ```
    #[allow(clippy::cast_precision_loss)]
    pub fn from_minutes_f64(value: f64) -> Result<Self, FloatError> {
        Self::interval(value, Self::TICKS_PER_MINUTE as f64)
    }

    /// Creates a `TimeSpan` from a fractional number of seconds.
    ///
    /// Fractional ticks are truncated toward zero, not rounded.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `value` is NaN, or
    /// [`FloatError::Overflow`] if it's outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_seconds_f64(1.0).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND));
    /// ```
    #[allow(clippy::cast_precision_loss)]
    pub fn from_seconds_f64(value: f64) -> Result<Self, FloatError> {
        Self::interval(value, Self::TICKS_PER_SECOND as f64)
    }

    /// Creates a `TimeSpan` from a fractional number of milliseconds.
    ///
    /// Fractional ticks are truncated toward zero, not rounded.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `value` is NaN, or
    /// [`FloatError::Overflow`] if it's outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_milliseconds_f64(1.0).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_MILLISECOND));
    /// ```
    #[allow(clippy::cast_precision_loss)]
    pub fn from_milliseconds_f64(value: f64) -> Result<Self, FloatError> {
        Self::interval(value, Self::TICKS_PER_MILLISECOND as f64)
    }

    /// Creates a `TimeSpan` from a fractional number of microseconds.
    ///
    /// Fractional ticks are truncated toward zero, not rounded.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `value` is NaN, or
    /// [`FloatError::Overflow`] if it's outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_microseconds_f64(15.0).unwrap().ticks(), 150);
    /// ```
    #[allow(clippy::cast_precision_loss)]
    pub fn from_microseconds_f64(value: f64) -> Result<Self, FloatError> {
        Self::interval(value, Self::TICKS_PER_MICROSECOND as f64)
    }

    // ── Float multiply/divide (mirror TimeSpan.Multiply(double)/Divide(double)) ──
    // TimeSpan.cs#L689-L691 (Multiply/Divide) forward to the operators at
    // L908-L934, which round via `Math.Round` — defaulting to
    // `MidpointRounding.ToEven` — hence `round_ties_even()` below rather than
    // `round()` (which breaks ties away from zero and would diverge from C#
    // on exact half-tick results).
    // `Result` is used instead of throwing, per this crate's established
    // convention for anything that can hit NaN/overflow (see `from_days_f64`
    // and friends above).

    /// Multiplies this `TimeSpan` by `factor`, rounding to the nearest tick.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `factor` is NaN, or
    /// [`FloatError::Overflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::from_ticks(2 * TimeSpan::TICKS_PER_HOUR + 30 * TimeSpan::TICKS_PER_MINUTE);
    /// assert_eq!(ts.multiply(2.0), Ok(TimeSpan::from_ticks(5 * TimeSpan::TICKS_PER_HOUR)));
    /// ```
    pub fn multiply(self, factor: f64) -> Result<Self, FloatError> {
        if factor.is_nan() {
            return Err(FloatError::Nan);
        }
        #[allow(clippy::cast_precision_loss)]
        let ticks = (self.ticks as f64 * factor).round_ties_even();
        Self::interval_from_double_ticks(ticks)
    }

    /// Divides this `TimeSpan` by `divisor`, rounding to the nearest tick.
    ///
    /// # Errors
    ///
    /// Returns [`FloatError::Nan`] if `divisor` is NaN, or
    /// [`FloatError::Overflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::from_ticks(2 * TimeSpan::TICKS_PER_HOUR + 30 * TimeSpan::TICKS_PER_MINUTE);
    /// assert_eq!(ts.divide(0.5), Ok(TimeSpan::from_ticks(5 * TimeSpan::TICKS_PER_HOUR)));
    /// ```
    pub fn divide(self, divisor: f64) -> Result<Self, FloatError> {
        if divisor.is_nan() {
            return Err(FloatError::Nan);
        }
        #[allow(clippy::cast_precision_loss)]
        let ticks = (self.ticks as f64 / divisor).round_ties_even();
        Self::interval_from_double_ticks(ticks)
    }

    // ── Integer factory methods (mirror FromDays(int) / FromHours(int) / ...) ──
    /// Creates a `TimeSpan` from an exact number of days.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_days(1).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY));
    /// ```
    pub fn from_days(days: i32) -> Result<Self, TimeSpanOverflow> {
        Self::builder().days(days).build()
    }

    /// Creates a `TimeSpan` from an exact number of hours.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_hours(1).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR));
    /// ```
    pub fn from_hours(hours: i32) -> Result<Self, TimeSpanOverflow> {
        Self::builder().hours(hours).build()
    }

    /// Creates a `TimeSpan` from an exact number of minutes.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_minutes(1).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_MINUTE));
    /// ```
    pub fn from_minutes(minutes: i64) -> Result<Self, TimeSpanOverflow> {
        Self::builder().minutes(minutes).build()
    }

    /// Creates a `TimeSpan` from an exact number of seconds.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_seconds(1).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND));
    /// ```
    pub fn from_seconds(seconds: i64) -> Result<Self, TimeSpanOverflow> {
        Self::builder().seconds(seconds).build()
    }

    /// Creates a `TimeSpan` from an exact number of milliseconds.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_milliseconds(1).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_MILLISECOND));
    /// ```
    pub fn from_milliseconds(milliseconds: i64) -> Result<Self, TimeSpanOverflow> {
        Self::builder().milliseconds(milliseconds).build()
    }

    /// Creates a `TimeSpan` from an exact number of microseconds.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::from_microseconds(1).unwrap(), TimeSpan::from_ticks(TimeSpan::TICKS_PER_MICROSECOND));
    /// ```
    pub fn from_microseconds(microseconds: i64) -> Result<Self, TimeSpanOverflow> {
        Self::builder().microseconds(microseconds).build()
    }

    /// Starts a [`TimeSpanBuilder`] for constructing a `TimeSpan` from a
    /// combination of days, hours, minutes, seconds, milliseconds, and
    /// microseconds. Mirrors C#'s multi-parameter `FromDays`/`FromHours`/...
    /// overloads, which Rust has no direct equivalent for (no default arguments).
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().days(1).hours(2).minutes(30).build().unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY + 2 * TimeSpan::TICKS_PER_HOUR + 30 * TimeSpan::TICKS_PER_MINUTE));
    /// ```
    #[must_use]
    pub fn builder() -> TimeSpanBuilder {
        TimeSpanBuilder::default()
    }

    // ── Component properties (mirror Days / Hours / Minutes / ...) ────────────
    /// Returns the whole-day component of the time interval.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// assert_eq!(TimeSpan::parse("1.02:03:04").unwrap().days(), 1);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // ticks / TICKS_PER_DAY always fits i32
    pub const fn days(self) -> i32 {
        (self.ticks / Self::TICKS_PER_DAY) as i32
    }

    /// Returns the hours component (-23 to 23) of the time interval.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// assert_eq!(TimeSpan::parse("1.02:03:04").unwrap().hours(), 2);
    /// ```
    #[must_use]
    pub const fn hours(self) -> i32 {
        (self.ticks / Self::TICKS_PER_HOUR % 24) as i32
    }

    /// Returns the minutes component (-59 to 59) of the time interval.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// assert_eq!(TimeSpan::parse("1.02:03:04").unwrap().minutes(), 3);
    /// ```
    #[must_use]
    pub const fn minutes(self) -> i32 {
        (self.ticks / Self::TICKS_PER_MINUTE % 60) as i32
    }

    /// Returns the seconds component (-59 to 59) of the time interval.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// assert_eq!(TimeSpan::parse("1.02:03:04").unwrap().seconds(), 4);
    /// ```
    #[must_use]
    pub const fn seconds(self) -> i32 {
        (self.ticks / Self::TICKS_PER_SECOND % 60) as i32
    }

    /// Returns the milliseconds component (-999 to 999) of the time interval.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// assert_eq!(TimeSpan::parse("1.02:03:04.005006700").unwrap().milliseconds(), 5);
    /// ```
    #[must_use]
    pub const fn milliseconds(self) -> i32 {
        (self.ticks / Self::TICKS_PER_MILLISECOND % 1000) as i32
    }

    /// Returns the microseconds component (-999 to 999) of the time interval.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// assert_eq!(TimeSpan::parse("1.02:03:04.005006700").unwrap().microseconds(), 6);
    /// ```
    #[must_use]
    pub const fn microseconds(self) -> i32 {
        (self.ticks / Self::TICKS_PER_MICROSECOND % 1000) as i32
    }

    /// Returns the nanoseconds component (-900 to 900, in multiples of 100) of the time interval.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// assert_eq!(TimeSpan::parse("1.02:03:04.005006700").unwrap().nanoseconds(), 700);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // (ticks % 10) * 100 is at most 900
    pub const fn nanoseconds(self) -> i32 {
        (self.ticks % 10 * 100) as i32
    }

    // ── Total properties (mirror TotalDays / TotalHours / ...) ────────────────
    /// Returns the total number of days, as a fractional value.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// // 36 hours = 1.5 days
    /// let ts = TimeSpan::from_ticks(36 * TimeSpan::TICKS_PER_HOUR);
    /// assert_eq!(ts.total_days(), 1.5);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // matches C#'s (double)_ticks precision loss
    pub fn total_days(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_DAY as f64
    }

    /// Returns the total number of hours, as a fractional value.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// // 90 minutes = 1.5 hours
    /// let ts = TimeSpan::from_ticks(90 * TimeSpan::TICKS_PER_MINUTE);
    /// assert_eq!(ts.total_hours(), 1.5);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_hours(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_HOUR as f64
    }

    /// Returns the total number of minutes, as a fractional value.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// // 90 seconds = 1.5 minutes
    /// let ts = TimeSpan::from_ticks(90 * TimeSpan::TICKS_PER_SECOND);
    /// assert_eq!(ts.total_minutes(), 1.5);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_minutes(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_MINUTE as f64
    }

    /// Returns the total number of seconds, as a fractional value.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// // 1500 milliseconds = 1.5 seconds
    /// let ts = TimeSpan::from_ticks(1500 * TimeSpan::TICKS_PER_MILLISECOND);
    /// assert_eq!(ts.total_seconds(), 1.5);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_seconds(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_SECOND as f64
    }

    /// Returns the total number of milliseconds, as a fractional value, clamped
    /// to the range representable by `i64::MIN`/`i64::MAX` ticks.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::from_ticks(15 * TimeSpan::TICKS_PER_MILLISECOND);
    /// assert_eq!(ts.total_milliseconds(), 15.0);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_milliseconds(self) -> f64 {
        let temp = self.ticks as f64 / Self::TICKS_PER_MILLISECOND as f64;
        temp.clamp(Self::MIN_MILLISECONDS, Self::MAX_MILLISECONDS)
    }

    /// Returns the total number of microseconds, as a fractional value.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// // 150 ticks * 100ns = 15 microseconds
    /// let ts = TimeSpan::from_ticks(150);
    /// assert_eq!(ts.total_microseconds(), 15.0);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_microseconds(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_MICROSECOND as f64
    }

    /// Returns the total number of nanoseconds, as a fractional value.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// // 5 ticks * 100ns = 500 nanoseconds
    /// let ts = TimeSpan::from_ticks(5);
    /// assert_eq!(ts.total_nanoseconds(), 500.0);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_nanoseconds(self) -> f64 {
        self.ticks as f64 * 100.0
    }

    /// Returns the absolute value of this `TimeSpan`.
    ///
    /// Mirrors C#'s `TimeSpan.Duration()`.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(
    ///     TimeSpan::from_ticks(-5).duration(),
    ///     Ok(TimeSpan::from_ticks(5)),
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] for [`TimeSpan::MIN_VALUE`], whose absolute
    /// value is outside the representable range.
    pub const fn duration(self) -> Result<Self, TimeSpanOverflow> {
        match self.ticks.checked_abs() {
            Some(ticks) => Ok(Self::from_ticks(ticks)),
            None => Err(TimeSpanOverflow),
        }
    }

    // ── Lenient parsing (mirrors Parse / TryParse) ─────────────────────────────
    /// Parses a time interval string using the invariant culture (`.` decimal separator).
    ///
    /// Accepts the same flexible formats as C# `TimeSpan.Parse`: `h:mm`,
    /// `h:mm:ss`, `d.hh:mm:ss`, `d:h:mm:ss`, with optional fractional seconds.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(TimeSpan::parse("1:02:03").unwrap().ticks(), 37_230_000_000);
    /// assert_eq!(TimeSpan::parse("1.02:03:04").unwrap().ticks(), 937_840_000_000);
    ///
    /// // Leading/trailing whitespace is accepted
    /// assert!(TimeSpan::parse("  01:30:00  ").is_ok());
    ///
    /// // Bad syntax or out-of-range value produce descriptive errors
    /// assert!(TimeSpan::parse("garbage").unwrap_err().to_string().contains("expected a digit"));
    /// assert!(TimeSpan::parse("00:00:60").unwrap_err().to_string().contains("out of range"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the string is empty, malformed, or produces a value outside
    /// the representable range.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        Self::parse_with_culture(s, Locale::en)
    }

    /// Parses using the decimal separator of the given locale.
    ///
    /// ```
    /// use cs_timespan::{TimeSpan, Locale};
    ///
    /// // Croatian locale uses ',' as the decimal separator
    /// assert!(TimeSpan::parse_with_culture("6:12:14:45,348", Locale::hr).is_ok());
    ///
    /// // A '.' separator is invalid for that locale
    /// assert!(
    ///     TimeSpan::parse_with_culture("6:12:14:45.348", Locale::hr)
    ///         .unwrap_err()
    ///         .to_string()
    ///         .contains("decimal separator"),
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the string is empty, malformed, uses the wrong decimal
    /// separator for the locale, or produces a value outside the representable range.
    pub fn parse_with_culture(s: &str, locale: Locale) -> Result<Self, ParseError> {
        parse::parse_lenient(s, decimal_sep(locale))
    }

    // ── Strict parsing (mirrors ParseExact / TryParseExact) ───────────────────
    /// Parses a time interval string using a specific format, with the invariant
    /// culture (`.` decimal separator).
    ///
    /// Mirrors [`TimeSpan.ParseExact`]. For supported format strings see
    /// [Standard TimeSpan format strings] and [Custom TimeSpan format strings].
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// assert_eq!(
    ///     TimeSpan::parse_exact("1.02:03:04", "c").unwrap().ticks(),
    ///     937_840_000_000,
    /// );
    /// assert_eq!(
    ///     TimeSpan::parse_exact("12.05:02:03", r"d\.hh\:mm\:ss").unwrap().ticks(),
    ///     TimeSpan::parse("12.05:02:03").unwrap().ticks(),
    /// );
    /// ```
    ///
    /// [`TimeSpan.ParseExact`]: https://learn.microsoft.com/en-us/dotnet/api/system.timespan.parseexact
    /// [Standard TimeSpan format strings]: https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings
    /// [Custom TimeSpan format strings]: https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the string does not match the format, or the value is
    /// outside the representable range.
    pub fn parse_exact(s: &str, fmt: &str) -> Result<Self, ParseError> {
        Self::parse_exact_with_culture(s, fmt, Locale::en)
    }

    /// Tries each format string in order and returns the first successful parse,
    /// using the invariant culture.
    ///
    /// Mirrors `TimeSpan.ParseExact` with an array of formats. Each format
    /// follows the same syntax as [`parse_exact`](Self::parse_exact).
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] (from the last format tried) if no format matches.
    pub fn parse_exact_any(s: &str, formats: &[&str]) -> Result<Self, ParseError> {
        Self::parse_exact_any_with_culture(s, formats, Locale::en)
    }

    /// Parses using a specific format and locale decimal separator.
    ///
    /// `fmt` follows the same syntax as [`parse_exact`](Self::parse_exact).
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the string does not match the format, or the value is
    /// outside the representable range.
    pub fn parse_exact_with_culture(
        s: &str,
        fmt: &str,
        locale: Locale,
    ) -> Result<Self, ParseError> {
        parse::parse_exact(s, fmt, decimal_sep(locale))
    }

    /// Tries each format string in order and returns the first successful parse,
    /// using the given locale decimal separator.
    ///
    /// Each format follows the same syntax as [`parse_exact`](Self::parse_exact).
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] (from the last format tried) if no format matches.
    pub fn parse_exact_any_with_culture(
        s: &str,
        formats: &[&str],
        locale: Locale,
    ) -> Result<Self, ParseError> {
        let mut last = ParseError::new(ParseErrorKind::InvalidStructure("".into()), 0, s);
        for fmt in formats {
            match Self::parse_exact_with_culture(s, fmt, locale) {
                Ok(ts) => return Ok(ts),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    /// Parses using a specific format, locale, and [`TimeSpanStyles`].
    ///
    /// `fmt` follows the same syntax as [`parse_exact`](Self::parse_exact).
    /// [`TimeSpanStyles::AssumeNegative`] negates a positive result, mirroring
    /// the C# overload that accepts `TimeSpanStyles`.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the string does not match the format, or the value is
    /// outside the representable range.
    ///
    /// ```
    /// use cs_timespan::{TimeSpan, Locale, TimeSpanStyles};
    ///
    /// // Without a leading '-' in the input, AssumeNegative flips the sign.
    /// let ts = TimeSpan::parse_exact_with_styles("1:02:03", r"h\:mm\:ss", Locale::en, TimeSpanStyles::AssumeNegative);
    /// assert_eq!(ts.unwrap().to_string(), "-01:02:03");
    /// ```
    pub fn parse_exact_with_styles(
        s: &str,
        fmt: &str,
        locale: Locale,
        styles: TimeSpanStyles,
    ) -> Result<Self, ParseError> {
        let ts = parse::parse_exact(s, fmt, decimal_sep(locale))?;
        // C# ignores AssumeNegative for the five standard formats; it only applies
        // to custom multi-char formats (TryParseByFormat in TimeSpanParse.cs).
        let custom_fmt = !matches!(fmt, "c" | "t" | "T" | "g" | "G");
        if styles == TimeSpanStyles::AssumeNegative && custom_fmt && ts.ticks > 0 {
            Ok(-ts)
        } else {
            Ok(ts)
        }
    }

    // ── Formatting ─────────────────────────────────────────────────────────────
    /// Formats the time span using a format string with the invariant `.` separator.
    ///
    /// Standard specifiers: `"c"`/`"t"`/`"T"` (constant), `"g"` (general short),
    /// `"G"` (general long). Custom specifiers: `d`, `h`, `m`, `s`, `f`/`F`
    /// for fractional seconds, `%x` for a single specifier, `\x` for a literal.
    /// For the full reference see [Standard TimeSpan format strings] and
    /// [Custom TimeSpan format strings].
    ///
    /// ```
    /// use cs_timespan::{TimeSpan, FormatErrorKind};
    /// let ts = TimeSpan::from_ticks(1_234_567_890_123);
    ///
    /// assert_eq!(ts.to_string_fmt("c").unwrap(),          "1.10:17:36.7890123");
    /// assert_eq!(ts.to_string_fmt(r"d\.hh\:mm").unwrap(), "1.10:17");
    /// assert_eq!(ts.to_string_fmt("hh").unwrap(),         "10");
    /// assert_eq!(ts.to_string_fmt("x").unwrap_err().kind, FormatErrorKind::InvalidStandardFormat('x'));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`FormatError`] if the format string contains an unrecognised specifier or
    /// other invalid syntax.
    ///
    /// [Standard TimeSpan format strings]: https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings
    /// [Custom TimeSpan format strings]: https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-timespan-format-strings
    pub fn to_string_fmt(&self, fmt: &str) -> Result<String, FormatError> {
        self.to_string_fmt_with_culture(fmt, Locale::en)
    }

    /// Formats using the decimal separator of the given locale.
    ///
    /// `fmt` follows the same syntax as [`to_string_fmt`](Self::to_string_fmt).
    /// Only the `"g"` and `"G"` standard formats and the `f`/`F` custom
    /// specifiers are affected; `"c"`/`"t"`/`"T"` always use `.`.
    ///
    /// ```
    /// use cs_timespan::{TimeSpan, Locale};
    /// let ts = TimeSpan::from_ticks(1_234_567_890_123);
    ///
    /// // French locale uses ',' as the decimal separator in "g" and "G"
    /// assert_eq!(ts.to_string_fmt_with_culture("g", Locale::fr).unwrap(), "1:10:17:36,7890123");
    /// assert_eq!(ts.to_string_fmt_with_culture("G", Locale::fr).unwrap(), "1:10:17:36,7890123");
    ///
    /// // "c" is always invariant regardless of locale
    /// assert_eq!(ts.to_string_fmt_with_culture("c", Locale::fr).unwrap(), "1.10:17:36.7890123");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`FormatError`] if the format string contains an unrecognised specifier or
    /// other invalid syntax.
    pub fn to_string_fmt_with_culture(
        &self,
        fmt: &str,
        locale: Locale,
    ) -> Result<String, FormatError> {
        fmt::format_timespan(self.ticks, fmt, decimal_sep(locale))
    }

    /// Adds two `TimeSpan`s, returning [`TimeSpanOverflow`] instead of panicking
    /// if the result is outside the representable range.
    ///
    /// Mirrors the `OverflowException` thrown by C#'s `TimeSpan.operator+`. See
    /// also the `+` operator (`Add`), which panics on overflow instead.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    pub const fn checked_add(self, rhs: Self) -> Result<Self, TimeSpanOverflow> {
        match self.ticks.checked_add(rhs.ticks) {
            Some(ticks) => Ok(Self::from_ticks(ticks)),
            None => Err(TimeSpanOverflow),
        }
    }

    /// Subtracts two `TimeSpan`s, returning [`TimeSpanOverflow`] instead of
    /// panicking if the result is outside the representable range.
    ///
    /// Mirrors the `OverflowException` thrown by C#'s `TimeSpan.operator-`. See
    /// also the binary `-` operator (`Sub`), which panics on overflow instead.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the result is outside the representable range.
    pub const fn checked_sub(self, rhs: Self) -> Result<Self, TimeSpanOverflow> {
        match self.ticks.checked_sub(rhs.ticks) {
            Some(ticks) => Ok(Self::from_ticks(ticks)),
            None => Err(TimeSpanOverflow),
        }
    }

    /// Negates the `TimeSpan`, returning [`TimeSpanOverflow`] instead of
    /// panicking for [`TimeSpan::MIN_VALUE`], which has no positive counterpart.
    ///
    /// Mirrors the `OverflowException` thrown by C#'s unary `TimeSpan.operator-`.
    /// See also the unary `-` operator (`Neg`), which panics on overflow instead.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] for [`TimeSpan::MIN_VALUE`], whose negation is
    /// outside the representable range.
    pub const fn checked_neg(self) -> Result<Self, TimeSpanOverflow> {
        match self.ticks.checked_neg() {
            Some(ticks) => Ok(Self::from_ticks(ticks)),
            None => Err(TimeSpanOverflow),
        }
    }
}

/// Builds a [`TimeSpan`] from a combination of days, hours, minutes, seconds,
/// milliseconds, and microseconds. Created via [`TimeSpan::builder`].
///
/// Unset fields default to zero, mirroring C#'s optional trailing parameters
/// on the multi-parameter `FromDays`/`FromHours`/`FromMinutes`/`FromSeconds`/
/// `FromMilliseconds` overloads.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeSpanBuilder {
    days: i32,
    hours: i32,
    minutes: i64,
    seconds: i64,
    milliseconds: i64,
    microseconds: i64,
}

impl TimeSpanBuilder {
    /// Sets the number of days.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().days(1).build().unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(TimeSpan::TICKS_PER_DAY));
    /// ```
    #[must_use]
    pub fn days(mut self, days: i32) -> Self {
        self.days = days;
        self
    }

    /// Sets the number of hours.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().hours(1).build().unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(TimeSpan::TICKS_PER_HOUR));
    /// ```
    #[must_use]
    pub fn hours(mut self, hours: i32) -> Self {
        self.hours = hours;
        self
    }

    /// Sets the number of minutes.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().minutes(1).build().unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(TimeSpan::TICKS_PER_MINUTE));
    /// ```
    #[must_use]
    pub fn minutes(mut self, minutes: i64) -> Self {
        self.minutes = minutes;
        self
    }

    /// Sets the number of seconds.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().seconds(1).build().unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND));
    /// ```
    #[must_use]
    pub fn seconds(mut self, seconds: i64) -> Self {
        self.seconds = seconds;
        self
    }

    /// Sets the number of milliseconds.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().milliseconds(1).build().unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(TimeSpan::TICKS_PER_MILLISECOND));
    /// ```
    #[must_use]
    pub fn milliseconds(mut self, milliseconds: i64) -> Self {
        self.milliseconds = milliseconds;
        self
    }

    /// Sets the number of microseconds.
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    ///
    /// let ts = TimeSpan::builder().microseconds(1).build().unwrap();
    /// assert_eq!(ts, TimeSpan::from_ticks(TimeSpan::TICKS_PER_MICROSECOND));
    /// ```
    #[must_use]
    pub fn microseconds(mut self, microseconds: i64) -> Self {
        self.microseconds = microseconds;
        self
    }

    /// Combines the fields into a `TimeSpan`, summing via microseconds (matching
    /// C#'s `Int128`-based accumulation) to avoid intermediate overflow before
    /// the final bounds check against `TimeSpan`'s representable range.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSpanOverflow`] if the combined value is outside the
    /// representable range.
    #[allow(clippy::cast_possible_truncation)] // bounds-checked against i64::MIN/MAX above
    pub fn build(self) -> Result<TimeSpan, TimeSpanOverflow> {
        let total_microseconds: i128 = i128::from(self.days) * TimeSpan::MICROSECONDS_PER_DAY
            + i128::from(self.hours) * TimeSpan::MICROSECONDS_PER_HOUR
            + i128::from(self.minutes) * TimeSpan::MICROSECONDS_PER_MINUTE
            + i128::from(self.seconds) * TimeSpan::MICROSECONDS_PER_SECOND
            + i128::from(self.milliseconds) * TimeSpan::MICROSECONDS_PER_MILLISECOND
            + i128::from(self.microseconds);

        let max_microseconds = i128::from(i64::MAX) / i128::from(TimeSpan::TICKS_PER_MICROSECOND);
        let min_microseconds = i128::from(i64::MIN) / i128::from(TimeSpan::TICKS_PER_MICROSECOND);
        if total_microseconds > max_microseconds || total_microseconds < min_microseconds {
            return Err(TimeSpanOverflow);
        }

        let ticks = (total_microseconds * i128::from(TimeSpan::TICKS_PER_MICROSECOND)) as i64;
        Ok(TimeSpan::from_ticks(ticks))
    }
}

/// Default `Display` uses the invariant `"c"` format: `[-][d.]hh:mm:ss[.fffffff]`
impl std::fmt::Display for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // "c" format is built from Components directly — no error path.
        f.write_str(&fmt::format_constant(self.ticks))
    }
}

fn decimal_sep(locale: Locale) -> char {
    locale.decimal().chars().next().unwrap_or('.')
}

#[cfg(feature = "chrono")]
mod chrono_impls {
    use super::TimeSpan;
    use chrono::TimeDelta;

    /// ```
    /// use cs_timespan::TimeSpan;
    /// use chrono::TimeDelta;
    ///
    /// assert_eq!(TimeSpan::from(TimeDelta::seconds(1)), TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND));
    /// ```
    impl From<TimeDelta> for TimeSpan {
        fn from(delta: TimeDelta) -> Self {
            // num_seconds() and subsec_nanos() together give signed components.
            // For e.g. -1.5 s: num_seconds()=-1, subsec_nanos()=-500_000_000.
            let secs = delta.num_seconds();
            let nanos = i64::from(delta.subsec_nanos());
            TimeSpan::from_ticks(secs * TimeSpan::TICKS_PER_SECOND + nanos / 100)
        }
    }

    /// ```
    /// use cs_timespan::TimeSpan;
    /// use chrono::TimeDelta;
    ///
    /// assert_eq!(TimeDelta::from(TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND)), TimeDelta::seconds(1));
    /// ```
    impl From<TimeSpan> for TimeDelta {
        fn from(ts: TimeSpan) -> Self {
            // TimeSpan's range (±29 k years) is contained within TimeDelta's range
            // (±292 billion years), so no saturation is needed.
            let secs = ts.ticks / TimeSpan::TICKS_PER_SECOND;
            let subsec_nanos = (ts.ticks % TimeSpan::TICKS_PER_SECOND) * 100;
            TimeDelta::seconds(secs) + TimeDelta::nanoseconds(subsec_nanos)
        }
    }
}

/// ```
/// use cs_timespan::TimeSpan;
/// use std::time::Duration;
///
/// assert_eq!(TimeSpan::from(Duration::from_secs(1)), TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND));
/// ```
impl From<std::time::Duration> for TimeSpan {
    fn from(d: std::time::Duration) -> Self {
        // 1 tick = 100 ns; saturate to MAX_VALUE if Duration exceeds TimeSpan's range.
        let ticks = d.as_nanos() / 100;
        if ticks > i64::MAX as u128 {
            TimeSpan::MAX_VALUE
        } else {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            // guarded: ticks ≤ i64::MAX
            TimeSpan::from_ticks(ticks as i64)
        }
    }
}

/// ```
/// use cs_timespan::TimeSpan;
/// use std::time::Duration;
///
/// assert_eq!(Duration::try_from(TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND)), Ok(Duration::from_secs(1)));
/// ```
impl TryFrom<TimeSpan> for std::time::Duration {
    type Error = NegativeTimeSpan;

    fn try_from(ts: TimeSpan) -> Result<Self, Self::Error> {
        if ts.ticks < 0 {
            return Err(NegativeTimeSpan);
        }
        // ticks ≥ 0, so cast to u128 is lossless; nanos fits u64/u32 by modulo bounds
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let nanos = ts.ticks as u128 * 100;
        #[allow(clippy::cast_possible_truncation)]
        let secs = (nanos / 1_000_000_000) as u64;
        #[allow(clippy::cast_possible_truncation)]
        let subsec_nanos = (nanos % 1_000_000_000) as u32;
        Ok(std::time::Duration::new(secs, subsec_nanos))
    }
}

// ── Arithmetic ────────────────────────────────────────────────────────────────

/// Panics on overflow; see [`checked_add`](TimeSpan::checked_add) for a
/// non-panicking alternative.
impl std::ops::Add for TimeSpan {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("TimeSpan add overflowed")
    }
}

/// Panics on overflow; see [`checked_sub`](TimeSpan::checked_sub) for a
/// non-panicking alternative.
impl std::ops::Sub for TimeSpan {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).expect("TimeSpan sub overflowed")
    }
}

/// Panics on overflow (only possible for [`TimeSpan::MIN_VALUE`]); see
/// [`checked_neg`](TimeSpan::checked_neg) for a non-panicking alternative.
impl std::ops::Neg for TimeSpan {
    type Output = Self;
    fn neg(self) -> Self {
        self.checked_neg().expect("TimeSpan neg overflowed")
    }
}

impl std::ops::AddAssign for TimeSpan {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::SubAssign for TimeSpan {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::Mul<i64> for TimeSpan {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Self::from_ticks(self.ticks * rhs)
    }
}

impl std::ops::Mul<TimeSpan> for i64 {
    type Output = TimeSpan;
    fn mul(self, rhs: TimeSpan) -> TimeSpan {
        TimeSpan::from_ticks(self * rhs.ticks)
    }
}

impl std::ops::MulAssign<i64> for TimeSpan {
    fn mul_assign(&mut self, rhs: i64) {
        self.ticks *= rhs;
    }
}

// TimeSpan.cs#L908-L922 (operator *) and L925-L934 (operator /): both throw
// ArgumentException (NaN) or OverflowException (out of range); unlike the `i64`
// operators above, these can fail, so `Output` is a `Result` instead of panicking.
// Applies equally to the `Div<f64>` impl below.
impl std::ops::Mul<f64> for TimeSpan {
    type Output = Result<Self, FloatError>;

    fn mul(self, rhs: f64) -> Self::Output {
        self.multiply(rhs)
    }
}

impl std::ops::Mul<TimeSpan> for f64 {
    type Output = Result<TimeSpan, FloatError>;

    fn mul(self, rhs: TimeSpan) -> Self::Output {
        rhs.multiply(self)
    }
}

impl std::ops::Div<i64> for TimeSpan {
    type Output = Self;
    fn div(self, rhs: i64) -> Self {
        Self::from_ticks(self.ticks / rhs)
    }
}

impl std::ops::Div<f64> for TimeSpan {
    type Output = Result<Self, FloatError>;

    fn div(self, rhs: f64) -> Self::Output {
        self.divide(rhs)
    }
}

impl std::ops::DivAssign<i64> for TimeSpan {
    fn div_assign(&mut self, rhs: i64) {
        self.ticks /= rhs;
    }
}

impl std::ops::Div<TimeSpan> for TimeSpan {
    type Output = f64;
    #[allow(clippy::cast_precision_loss)] // intentional: best-effort ratio, no exact integer guarantee
    fn div(self, rhs: TimeSpan) -> f64 {
        self.ticks as f64 / rhs.ticks as f64
    }
}
