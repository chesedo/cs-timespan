pub use num_format::Locale;

mod fmt;
mod parse;
pub use fmt::{FormatError, FormatErrorKind};
pub use parse::{ParseError, TimeSpanStyles};

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

    // ── Raw construction ───────────────────────────────────────────────────────
    /// Creates a `TimeSpan` from a raw tick count (1 tick = 100 ns).
    ///
    /// ```
    /// use cs_timespan::TimeSpan;
    /// let one_second = TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND);
    /// assert_eq!(one_second.to_string(), "00:00:01");
    /// ```
    pub const fn from_ticks(ticks: i64) -> Self {
        TimeSpan { ticks }
    }

    /// Returns the total number of ticks (1 tick = 100 ns).
    pub const fn ticks(self) -> i64 {
        self.ticks
    }

    // ── Lenient parsing (mirrors Parse / TryParse) ─────────────────────────────
    /// Parses a time interval string using the invariant culture (`.` decimal separator).
    ///
    /// Accepts the same flexible formats as C# `TimeSpan.Parse`: `h:mm`,
    /// `h:mm:ss`, `d.hh:mm:ss`, `d:h:mm:ss`, with optional fractional seconds.
    ///
    /// ```
    /// use cs_timespan::{ParseError, TimeSpan};
    ///
    /// assert_eq!(TimeSpan::parse("1:02:03").unwrap().ticks(), 37_230_000_000);
    /// assert_eq!(TimeSpan::parse("1.02:03:04").unwrap().ticks(), 937_840_000_000);
    ///
    /// // Leading/trailing whitespace is accepted
    /// assert!(TimeSpan::parse("  01:30:00  ").is_ok());
    ///
    /// // Bad syntax → various ParseError variants; value out of range → Overflow
    /// assert_eq!(TimeSpan::parse("garbage"), Err(ParseError::InvalidCharacter));
    /// assert_eq!(TimeSpan::parse("00:00:60"), Err(ParseError::Overflow));
    /// ```
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        Self::parse_with_culture(s, Locale::en)
    }

    /// Parses using the decimal separator of the given locale.
    ///
    /// ```
    /// use cs_timespan::{ParseError, TimeSpan, Locale};
    ///
    /// // Croatian locale uses ',' as the decimal separator
    /// assert!(TimeSpan::parse_with_culture("6:12:14:45,348", Locale::hr).is_ok());
    ///
    /// // A '.' separator is invalid for that locale
    /// assert_eq!(
    ///     TimeSpan::parse_with_culture("6:12:14:45.348", Locale::hr),
    ///     Err(ParseError::WrongSeparator),
    /// );
    /// ```
    pub fn parse_with_culture(s: &str, locale: Locale) -> Result<Self, ParseError> {
        parse::parse_lenient(s, decimal_sep(locale))
    }

    // ── Strict parsing (mirrors ParseExact / TryParseExact) ───────────────────
    pub fn parse_exact(s: &str, fmt: &str) -> Result<Self, ParseError> {
        Self::parse_exact_with_culture(s, fmt, Locale::en)
    }

    pub fn parse_exact_any(s: &str, formats: &[&str]) -> Result<Self, ParseError> {
        Self::parse_exact_any_with_culture(s, formats, Locale::en)
    }

    pub fn parse_exact_with_culture(
        s: &str,
        fmt: &str,
        locale: Locale,
    ) -> Result<Self, ParseError> {
        parse::parse_exact(s, fmt, decimal_sep(locale))
    }

    pub fn parse_exact_any_with_culture(
        s: &str,
        formats: &[&str],
        locale: Locale,
    ) -> Result<Self, ParseError> {
        let mut last = ParseError::InvalidStructure;
        for fmt in formats {
            match Self::parse_exact_with_culture(s, fmt, locale) {
                Ok(ts) => return Ok(ts),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn parse_exact_with_styles(
        s: &str,
        fmt: &str,
        locale: Locale,
        styles: TimeSpanStyles,
    ) -> Result<Self, ParseError> {
        let ts = parse::parse_exact(s, fmt, decimal_sep(locale))?;
        if styles == TimeSpanStyles::AssumeNegative && ts.ticks > 0 {
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
            let nanos = delta.subsec_nanos() as i64;
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
        let nanos = ts.ticks as u128 * 100;
        let secs = (nanos / 1_000_000_000) as u64;
        let subsec_nanos = (nanos % 1_000_000_000) as u32;
        Ok(std::time::Duration::new(secs, subsec_nanos))
    }
}
