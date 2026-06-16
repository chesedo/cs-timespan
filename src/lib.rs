/// A C# `System.TimeSpan`-compatible time interval type for Rust.
///
/// Internally stores a tick count where 1 tick = 100 nanoseconds,
/// matching the C# representation exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TimeSpan {
    ticks: i64,
}

/// Mirrors the two distinct failure modes C# separates into
/// `FormatException` (bad syntax) and `OverflowException` (out of range).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input string was not in a recognised format (`FormatException`).
    InvalidFormat,
    /// The input was syntactically valid but outside the representable range
    /// (`OverflowException`).
    Overflow,
}

/// Mirrors `System.Globalization.TimeSpanStyles`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeSpanStyles {
    #[default]
    None,
    /// Treat the parsed interval as negative even without a leading `-`.
    AssumeNegative,
}

/// Subset of cultures relevant to `TimeSpan` formatting and parsing.
///
/// Only cultures that affect the decimal/group separator are included,
/// since those are the only axes along which `TimeSpan` output varies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Culture {
    /// `CultureInfo.InvariantCulture` — decimal separator is `.`
    #[default]
    Invariant,
    /// Croatian (`hr-HR`) — decimal separator is `,`
    HrHR,
    /// French (`fr-FR`) — decimal separator is `,`
    FrFR,
}

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
    pub const fn from_ticks(ticks: i64) -> Self {
        TimeSpan { ticks }
    }

    pub const fn ticks(self) -> i64 {
        self.ticks
    }

    // ── Lenient parsing (mirrors Parse / TryParse) ─────────────────────────────
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn parse_with_culture(s: &str, _culture: Culture) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn try_parse(s: &str) -> Option<Self> {
        todo!()
    }

    pub fn try_parse_with_culture(s: &str, _culture: Culture) -> Option<Self> {
        todo!()
    }

    // ── Strict parsing (mirrors ParseExact / TryParseExact) ───────────────────
    pub fn parse_exact(s: &str, _fmt: &str) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn parse_exact_any(s: &str, _formats: &[&str]) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn parse_exact_with_culture(
        s: &str,
        _fmt: &str,
        _culture: Culture,
    ) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn parse_exact_any_with_culture(
        s: &str,
        _formats: &[&str],
        _culture: Culture,
    ) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn parse_exact_with_styles(
        s: &str,
        _fmt: &str,
        _culture: Culture,
        _styles: TimeSpanStyles,
    ) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn try_parse_exact(s: &str, _fmt: &str) -> Option<Self> {
        todo!()
    }

    pub fn try_parse_exact_any(s: &str, _formats: &[&str]) -> Option<Self> {
        todo!()
    }

    // ── Formatting ─────────────────────────────────────────────────────────────
    pub fn to_string_fmt(&self, _fmt: &str) -> String {
        todo!()
    }

    pub fn to_string_fmt_with_culture(&self, _fmt: &str, _culture: Culture) -> String {
        todo!()
    }
}

/// Default `Display` uses the invariant `"c"` format: `[-][d.]hh:mm:ss[.fffffff]`
impl std::fmt::Display for TimeSpan {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(feature = "chrono")]
mod chrono_impls {
    use super::TimeSpan;
    use chrono::TimeDelta;

    impl From<TimeDelta> for TimeSpan {
        fn from(_delta: TimeDelta) -> Self {
            todo!()
        }
    }

    impl From<TimeSpan> for TimeDelta {
        fn from(_ts: TimeSpan) -> Self {
            todo!()
        }
    }
}

impl From<std::time::Duration> for TimeSpan {
    fn from(_d: std::time::Duration) -> Self {
        todo!()
    }
}

impl TryFrom<TimeSpan> for std::time::Duration {
    type Error = ();

    fn try_from(_ts: TimeSpan) -> Result<Self, Self::Error> {
        todo!()
    }
}
