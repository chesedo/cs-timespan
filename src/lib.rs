//! A Rust implementation of C#'s [`System.TimeSpan`], for working with
//! serialized C# time intervals.
//!
//! Internally stores a signed tick count where 1 tick = 100 nanoseconds,
//! identical to the C# representation.
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
//! [`chrono::TimeDelta`] are also available.
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

impl TimeSpan {
    // ── Tick-unit constants ────────────────────────────────────────────────────
    pub const TICKS_PER_MILLISECOND: i64 = 10_000;
    pub const TICKS_PER_SECOND: i64 = 10_000_000;
    pub const TICKS_PER_MINUTE: i64 = 600_000_000;
    pub const TICKS_PER_HOUR: i64 = 36_000_000_000;
    pub const TICKS_PER_DAY: i64 = 864_000_000_000;

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
    #[must_use]
    pub const fn hours(self) -> i32 {
        (self.ticks / Self::TICKS_PER_HOUR % 24) as i32
    }

    /// Returns the minutes component (-59 to 59) of the time interval.
    #[must_use]
    pub const fn minutes(self) -> i32 {
        (self.ticks / Self::TICKS_PER_MINUTE % 60) as i32
    }

    /// Returns the seconds component (-59 to 59) of the time interval.
    #[must_use]
    pub const fn seconds(self) -> i32 {
        (self.ticks / Self::TICKS_PER_SECOND % 60) as i32
    }

    /// Returns the milliseconds component (-999 to 999) of the time interval.
    #[must_use]
    pub const fn milliseconds(self) -> i32 {
        (self.ticks / Self::TICKS_PER_MILLISECOND % 1000) as i32
    }

    /// Returns the microseconds component (-999 to 999) of the time interval.
    #[must_use]
    pub const fn microseconds(self) -> i32 {
        (self.ticks / 10 % 1000) as i32
    }

    /// Returns the nanoseconds component (-900 to 900, in multiples of 100) of the time interval.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // (ticks % 10) * 100 is at most 900
    pub const fn nanoseconds(self) -> i32 {
        (self.ticks % 10 * 100) as i32
    }

    // ── Total properties (mirror TotalDays / TotalHours / ...) ────────────────
    /// Returns the total number of days, as a fractional value.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // matches C#'s (double)_ticks precision loss
    pub fn total_days(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_DAY as f64
    }

    /// Returns the total number of hours, as a fractional value.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_hours(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_HOUR as f64
    }

    /// Returns the total number of minutes, as a fractional value.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_minutes(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_MINUTE as f64
    }

    /// Returns the total number of seconds, as a fractional value.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_seconds(self) -> f64 {
        self.ticks as f64 / Self::TICKS_PER_SECOND as f64
    }

    /// Returns the total number of milliseconds, as a fractional value, clamped
    /// to the range representable by `i64::MIN`/`i64::MAX` ticks.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_milliseconds(self) -> f64 {
        let temp = self.ticks as f64 / Self::TICKS_PER_MILLISECOND as f64;
        temp.clamp(Self::MIN_MILLISECONDS, Self::MAX_MILLISECONDS)
    }

    /// Returns the total number of microseconds, as a fractional value.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_microseconds(self) -> f64 {
        self.ticks as f64 / 10.0
    }

    /// Returns the total number of nanoseconds, as a fractional value.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_nanoseconds(self) -> f64 {
        self.ticks as f64 * 100.0
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
    /// Mirrors `TimeSpan.ParseExact` with an array of formats.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] (from the last format tried) if no format matches.
    pub fn parse_exact_any(s: &str, formats: &[&str]) -> Result<Self, ParseError> {
        Self::parse_exact_any_with_culture(s, formats, Locale::en)
    }

    /// Parses using a specific format and locale decimal separator.
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
    /// [`TimeSpanStyles::AssumeNegative`] negates a positive result, mirroring
    /// the C# overload that accepts `TimeSpanStyles`.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the string does not match the format, or the value is
    /// outside the representable range.
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
            Ok(TimeSpan::from_ticks(-ts.ticks))
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
    ///
    /// ```
    /// use cs_timespan::{TimeSpan, FormatErrorKind};
    /// let ts = TimeSpan::from_ticks(1_234_567_890_123);
    ///
    /// assert_eq!(ts.to_string_fmt("c").unwrap(),          "1.10:17:36.7890123");
    /// assert_eq!(ts.to_string_fmt(r"d\.hh\:mm").unwrap(), "1.10:17");
    /// assert_eq!(ts.to_string_fmt("hh").unwrap(),         "10");
    /// assert_eq!(ts.to_string_fmt("x").unwrap_err().kind, FormatErrorKind::UnknownSpecifier('x'));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`FormatError`] if the format string contains an unrecognised specifier or
    /// other invalid syntax.
    pub fn to_string_fmt(&self, fmt: &str) -> Result<String, FormatError> {
        self.to_string_fmt_with_culture(fmt, Locale::en)
    }

    /// Formats using the decimal separator of the given locale.
    ///
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

    impl From<TimeDelta> for TimeSpan {
        fn from(delta: TimeDelta) -> Self {
            // num_seconds() and subsec_nanos() together give signed components.
            // For e.g. -1.5 s: num_seconds()=-1, subsec_nanos()=-500_000_000.
            let secs = delta.num_seconds();
            let nanos = i64::from(delta.subsec_nanos());
            TimeSpan::from_ticks(secs * TimeSpan::TICKS_PER_SECOND + nanos / 100)
        }
    }

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

impl std::ops::Add for TimeSpan {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from_ticks(self.ticks + rhs.ticks)
    }
}

impl std::ops::Sub for TimeSpan {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::from_ticks(self.ticks - rhs.ticks)
    }
}

impl std::ops::Neg for TimeSpan {
    type Output = Self;
    fn neg(self) -> Self {
        Self::from_ticks(-self.ticks)
    }
}

impl std::ops::AddAssign for TimeSpan {
    fn add_assign(&mut self, rhs: Self) {
        self.ticks += rhs.ticks;
    }
}

impl std::ops::SubAssign for TimeSpan {
    fn sub_assign(&mut self, rhs: Self) {
        self.ticks -= rhs.ticks;
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

impl std::ops::Div<i64> for TimeSpan {
    type Output = Self;
    fn div(self, rhs: i64) -> Self {
        Self::from_ticks(self.ticks / rhs)
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
