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
    pub fn to_string_fmt(&self, fmt: &str) -> String {
        format_timespan(*self, fmt, Culture::Invariant)
    }

    pub fn to_string_fmt_with_culture(&self, fmt: &str, culture: Culture) -> String {
        format_timespan(*self, fmt, culture)
    }

    fn to_components(self) -> Components {
        let abs = self.ticks.unsigned_abs();
        let days = abs / Self::TICKS_PER_DAY as u64;
        let r = abs % Self::TICKS_PER_DAY as u64;
        let hours = (r / Self::TICKS_PER_HOUR as u64) as u32;
        let r = r % Self::TICKS_PER_HOUR as u64;
        let minutes = (r / Self::TICKS_PER_MINUTE as u64) as u32;
        let r = r % Self::TICKS_PER_MINUTE as u64;
        let seconds = (r / Self::TICKS_PER_SECOND as u64) as u32;
        let sub_sec_ticks = (r % Self::TICKS_PER_SECOND as u64) as u32;
        Components {
            negative: self.ticks < 0,
            days,
            hours,
            minutes,
            seconds,
            sub_sec_ticks,
        }
    }
}

/// Default `Display` uses the invariant `"c"` format: `[-][d.]hh:mm:ss[.fffffff]`
impl std::fmt::Display for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format_constant(&self.to_components()))
    }
}

// ── Internal formatting helpers ────────────────────────────────────────────────

struct Components {
    negative: bool,
    days: u64,
    hours: u32,
    minutes: u32,
    seconds: u32,
    /// Fractional-second ticks: 0..=9_999_999 (one tick = 100 ns)
    sub_sec_ticks: u32,
}

fn format_timespan(ts: TimeSpan, fmt: &str, culture: Culture) -> String {
    let c = ts.to_components();
    match fmt {
        "c" | "t" | "T" => format_constant(&c),
        "g" => format_general_short(&c, culture),
        "G" => format_general_long(&c, culture),
        _ => format_custom(&c, fmt),
    }
}

/// `"c"` / `"t"` / `"T"`: `[-][d.]hh:mm:ss[.fffffff]` — culture-invariant.
fn format_constant(c: &Components) -> String {
    let mut out = String::new();
    if c.negative {
        out.push('-');
    }
    if c.days > 0 {
        out.push_str(&c.days.to_string());
        out.push('.');
    }
    out.push_str(&format!("{:02}:{:02}:{:02}", c.hours, c.minutes, c.seconds));
    if c.sub_sec_ticks > 0 {
        out.push('.');
        out.push_str(&format!("{:07}", c.sub_sec_ticks));
    }
    out
}

/// `"g"`: `[-][d:]h:mm:ss[.FFFFFFF]` — culture-sensitive decimal separator.
fn format_general_short(c: &Components, culture: Culture) -> String {
    todo!("format_general_short")
}

/// `"G"`: `[-]d:hh:mm:ss.fffffff` — culture-sensitive decimal separator.
fn format_general_long(c: &Components, culture: Culture) -> String {
    todo!("format_general_long")
}

fn format_custom(c: &Components, fmt: &str) -> String {
    let chars: Vec<char> = fmt.chars().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            // `%x` — single specifier written with explicit percent
            '%' if i + 1 < chars.len() => {
                i += 1;
                match chars[i] {
                    'd' => { out.push_str(&c.days.to_string()); i += 1; }
                    'h' => { out.push_str(&c.hours.to_string()); i += 1; }
                    'm' => { out.push_str(&c.minutes.to_string()); i += 1; }
                    's' => { out.push_str(&c.seconds.to_string()); i += 1; }
                    'f' => { out.push_str(&fmt_frac(c.sub_sec_ticks, 1, false)); i += 1; }
                    'F' => { out.push_str(&fmt_frac(c.sub_sec_ticks, 1, true)); i += 1; }
                    _ => todo!("custom specifier: %{}", chars[i]),
                }
            }
            // `h` / `hh` — hours component (hh always 2 digits)
            'h' => {
                let n = run_length(&chars, i, 'h');
                if n == 1 {
                    out.push_str(&c.hours.to_string());
                } else {
                    out.push_str(&format!("{:02}", c.hours));
                }
                i += n;
            }
            // `f{n}` — fractional seconds, exactly n digits (1–7), no trimming
            'f' => {
                let n = run_length(&chars, i, 'f');
                out.push_str(&fmt_frac(c.sub_sec_ticks, n, false));
                i += n;
            }
            // `F{n}` — fractional seconds, up to n digits, trailing zeros trimmed
            'F' => {
                let n = run_length(&chars, i, 'F');
                out.push_str(&fmt_frac(c.sub_sec_ticks, n, true));
                i += n;
            }
            // `s` / `ss` — seconds component
            's' => {
                let n = run_length(&chars, i, 's');
                if n == 1 {
                    out.push_str(&c.seconds.to_string());
                } else {
                    out.push_str(&format!("{:02}", c.seconds));
                }
                i += n;
            }
            // `m` / `mm` — minutes component
            'm' => {
                let n = run_length(&chars, i, 'm');
                if n == 1 {
                    out.push_str(&c.minutes.to_string());
                } else {
                    out.push_str(&format!("{:02}", c.minutes));
                }
                i += n;
            }
            // `d{n}` — days padded to at least n digits
            'd' => {
                let n = run_length(&chars, i, 'd');
                let s = c.days.to_string();
                if s.len() < n {
                    out.push_str(&format!("{:0>width$}", s, width = n));
                } else {
                    out.push_str(&s);
                }
                i += n;
            }
            // `\x` — escape: next char is a literal
            '\\' if i + 1 < chars.len() => {
                out.push(chars[i + 1]);
                i += 2;
            }
            // `'...'` or `"..."` — quoted literal string
            '\'' | '"' => {
                let q = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != q {
                    out.push(chars[i]);
                    i += 1;
                }
                i += 1; // skip closing quote
            }
            _ => todo!("custom format char: {:?}", chars[i]),
        }
    }

    out
}

/// Format `sub_sec_ticks` (0..=9_999_999) as `n` fractional-second digits.
/// If `trim` is true, trailing zeros are removed (uppercase `F` behaviour).
fn fmt_frac(sub_sec_ticks: u32, n: usize, trim: bool) -> String {
    // Full 7-digit string, zero-padded
    let full = format!("{:07}", sub_sec_ticks);
    // Take the first `n` digits
    let s = &full[..n];
    if trim {
        s.trim_end_matches('0').to_string()
    } else {
        s.to_string()
    }
}

fn run_length(chars: &[char], start: usize, ch: char) -> usize {
    chars[start..].iter().take_while(|&&c| c == ch).count()
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
