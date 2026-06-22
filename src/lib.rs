use std::fmt::Write as FmtWrite;

pub use num_format::Locale;

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
///     ts.to_string_fmt_with_culture("g", Locale::fr),
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

/// Mirrors the two distinct failure modes C# separates into
/// `FormatException` (bad syntax) and `OverflowException` (out of range),
/// with the format case split into more actionable variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Input is empty or whitespace-only, or reduces to nothing after
    /// stripping a leading `-`.
    Empty,
    /// A non-digit character appeared where only digits are valid, or the
    /// input contains a null byte.
    InvalidCharacter,
    /// The decimal separator in the input does not match the locale
    /// (e.g. `'.'` when the locale uses `','`).
    WrongSeparator,
    /// The component structure is unrecognised: wrong number of colons or
    /// parts, missing required separator, duplicate specifiers, etc.
    InvalidStructure,
    /// The value is syntactically valid but outside the representable range
    /// (`OverflowException`).
    Overflow,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("input is empty"),
            Self::InvalidCharacter => f.write_str("input contains an invalid character"),
            Self::WrongSeparator => f.write_str("decimal separator does not match the locale"),
            Self::InvalidStructure => f.write_str("input has an unrecognised component structure"),
            Self::Overflow => f.write_str("TimeSpan value is outside the representable range"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Mirrors `System.Globalization.TimeSpanStyles`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeSpanStyles {
    #[default]
    None,
    /// Treat the parsed interval as negative even without a leading `-`.
    AssumeNegative,
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
        parse_impl::parse_lenient(s, decimal_sep(locale))
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
        parse_impl::parse_exact(s, fmt, decimal_sep(locale))
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
        let ts = parse_impl::parse_exact(s, fmt, decimal_sep(locale))?;
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
    /// use cs_timespan::TimeSpan;
    /// let ts = TimeSpan::from_ticks(1_234_567_890_123);
    ///
    /// assert_eq!(ts.to_string_fmt("c"),          "1.10:17:36.7890123");
    /// assert_eq!(ts.to_string_fmt(r"d\.hh\:mm"), "1.10:17");
    /// assert_eq!(ts.to_string_fmt("hh"),         "10");
    /// ```
    pub fn to_string_fmt(&self, fmt: &str) -> String {
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
    /// assert_eq!(ts.to_string_fmt_with_culture("g", Locale::fr), "1:10:17:36,7890123");
    /// assert_eq!(ts.to_string_fmt_with_culture("G", Locale::fr), "1:10:17:36,7890123");
    ///
    /// // "c" is always invariant regardless of locale
    /// assert_eq!(ts.to_string_fmt_with_culture("c", Locale::fr), "1.10:17:36.7890123");
    /// ```
    pub fn to_string_fmt_with_culture(&self, fmt: &str, locale: Locale) -> String {
        let sep = decimal_sep(locale);
        let c = self.to_components();
        match fmt {
            "c" | "t" | "T" => format_constant(&c),
            "g" => format_general_short(&c, sep),
            "G" => format_general_long(&c, sep),
            _ => format_custom(&c, fmt),
        }
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


/// `"c"` / `"t"` / `"T"`: `[-][d.]hh:mm:ss[.fffffff]` — culture-invariant.
fn format_constant(c: &Components) -> String {

    let mut out = String::new();
    if c.negative { out.push('-'); }
    if c.days > 0 { write!(out, "{}.", c.days).unwrap(); }
    write!(out, "{:02}:{:02}:{:02}", c.hours, c.minutes, c.seconds).unwrap();
    if c.sub_sec_ticks > 0 { write!(out, ".{:07}", c.sub_sec_ticks).unwrap(); }
    out
}

/// `"g"`: `[-][d:]h:mm:ss[.FFFFFFF]` — culture-sensitive decimal separator.
fn format_general_short(c: &Components, sep: char) -> String {

    let mut out = String::new();
    if c.negative { out.push('-'); }
    if c.days > 0 { write!(out, "{}:", c.days).unwrap(); }
    write!(out, "{}:{:02}:{:02}", c.hours, c.minutes, c.seconds).unwrap();
    if c.sub_sec_ticks > 0 {
        // FFFFFFF — trim trailing zeros
        write!(out, "{}{}", sep, fmt_frac(c.sub_sec_ticks, 7, true)).unwrap();
    }
    out
}

/// `"G"`: `[-]d:hh:mm:ss.fffffff` — culture-sensitive decimal separator.
fn format_general_long(c: &Components, sep: char) -> String {

    let mut out = String::new();
    if c.negative { out.push('-'); }
    write!(
        out,
        "{}:{:02}:{:02}:{:02}{}{}",
        c.days, c.hours, c.minutes, c.seconds, sep, fmt_frac(c.sub_sec_ticks, 7, false),
    ).unwrap();
    out
}

fn format_custom(c: &Components, fmt: &str) -> String {
    let chars: Vec<char> = fmt.chars().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            // `%x` — single specifier written with explicit percent prefix
            '%' if i + 1 < chars.len() => {
                i += 1;
                out.push_str(&format_specifier(c, chars[i], 1));
                i += 1;
            }
            // `d`, `h`, `m`, `s`, `f`, `F` — run of identical specifier chars
            ch @ ('d' | 'h' | 'm' | 's' | 'f' | 'F') => {
                let n = run_length(&chars, i, ch);
                out.push_str(&format_specifier(c, ch, n));
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

/// Emit one component according to its specifier character and repeat count `n`.
fn format_specifier(c: &Components, ch: char, n: usize) -> String {

    match ch {
        'd' => {
            let s = c.days.to_string();
            if s.len() < n {
                let mut out = String::new();
                write!(out, "{:0>width$}", s, width = n).unwrap();
                out
            } else {
                s
            }
        }
        'h' => fmt_component(n, c.hours),
        'm' => fmt_component(n, c.minutes),
        's' => fmt_component(n, c.seconds),
        'f' => fmt_frac(c.sub_sec_ticks, n, false),
        'F' => fmt_frac(c.sub_sec_ticks, n, true),
        _ => todo!("unknown custom specifier: {}", ch),
    }
}

/// `n == 1` → no leading zero; `n > 1` → zero-padded to 2 digits.
fn fmt_component(n: usize, val: u32) -> String {

    let mut out = String::new();
    if n == 1 { write!(out, "{}", val).unwrap(); } else { write!(out, "{:02}", val).unwrap(); }
    out
}

fn decimal_sep(locale: Locale) -> char {
    locale.decimal().chars().next().unwrap_or('.')
}

fn fmt_frac(sub_sec_ticks: u32, n: usize, trim: bool) -> String {

    let mut full = String::new();
    write!(full, "{:07}", sub_sec_ticks).unwrap();
    let s = &full[..n];
    if trim { s.trim_end_matches('0').to_string() } else { s.to_string() }
}

fn run_length(chars: &[char], start: usize, ch: char) -> usize {
    chars[start..].iter().take_while(|&&c| c == ch).count()
}

mod parse_impl {
    use super::{ParseError, TimeSpan};

    // ── Builder ───────────────────────────────────────────────────────────────

    #[derive(Default)]
    struct Builder<'a> {
        neg: bool,
        days: Option<&'a str>,
        hours: Option<&'a str>,
        minutes: Option<&'a str>,
        seconds: Option<&'a str>,
        frac: Option<&'a str>,
    }

    impl<'a> Builder<'a> {
        fn new(neg: bool) -> Self {
            Self { neg, ..Default::default() }
        }

        fn build(self) -> Result<TimeSpan, ParseError> {
            let days = parse_component_uint(self.days)?;
            let h    = parse_component_uint(self.hours)? as u32;
            let m    = parse_component_uint(self.minutes)? as u32;
            let sv   = parse_component_uint(self.seconds)? as u32;
            let frac = parse_component_frac(self.frac)?;
            if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
            build_ticks(self.neg, days, h, m, sv, frac)
        }
    }

    fn parse_component_uint(s: Option<&str>) -> Result<u64, ParseError> {
        match s {
            None | Some("") => Ok(0),
            Some(s) => parse_uint(s).map_err(Into::into),
        }
    }

    fn parse_component_frac(s: Option<&str>) -> Result<u32, ParseError> {
        match s {
            None => Ok(0),
            Some(s) => parse_frac(s).map_err(Into::into),
        }
    }

    // ── Low-level validators ──────────────────────────────────────────────────

    #[derive(Debug, PartialEq, Eq)]
    enum UintError { Empty, NonDigit, Overflow }

    impl From<UintError> for ParseError {
        fn from(e: UintError) -> Self {
            match e {
                UintError::Empty    => ParseError::InvalidStructure,
                UintError::NonDigit => ParseError::InvalidCharacter,
                UintError::Overflow => ParseError::Overflow,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum FracError { Empty, NonDigit, TooLong }

    impl From<FracError> for ParseError {
        fn from(e: FracError) -> Self {
            match e {
                FracError::Empty    => ParseError::InvalidStructure,
                FracError::NonDigit => ParseError::InvalidCharacter,
                FracError::TooLong  => ParseError::Overflow,
            }
        }
    }

    fn parse_uint(s: &str) -> Result<u64, UintError> {
        if s.is_empty() { return Err(UintError::Empty); }
        if !s.bytes().all(|b| b.is_ascii_digit()) { return Err(UintError::NonDigit); }
        s.parse::<u64>().map_err(|_| UintError::Overflow)
    }

    fn parse_frac(s: &str) -> Result<u32, FracError> {
        if s.is_empty() { return Err(FracError::Empty); }
        if !s.bytes().all(|b| b.is_ascii_digit()) { return Err(FracError::NonDigit); }
        if s.len() > 7 { return Err(FracError::TooLong); }
        let v = s.parse::<u32>().unwrap();
        Ok(v * 10u32.pow(7 - s.len() as u32))
    }

    fn build_ticks(neg: bool, days: u64, h: u32, m: u32, s: u32, frac: u32) -> Result<TimeSpan, ParseError> {
        let ticks = (days as u128)
            .checked_mul(TimeSpan::TICKS_PER_DAY as u128)
            .and_then(|t| t.checked_add(h as u128 * TimeSpan::TICKS_PER_HOUR as u128))
            .and_then(|t| t.checked_add(m as u128 * TimeSpan::TICKS_PER_MINUTE as u128))
            .and_then(|t| t.checked_add(s as u128 * TimeSpan::TICKS_PER_SECOND as u128))
            .and_then(|t| t.checked_add(frac as u128))
            .ok_or(ParseError::Overflow)?;
        if neg {
            const ABS_MIN: u128 = (i64::MAX as u128) + 1;
            if ticks > ABS_MIN {
                return Err(ParseError::Overflow);
            } else if ticks == ABS_MIN {
                return Ok(TimeSpan::from_ticks(i64::MIN));
            }
            Ok(TimeSpan::from_ticks(-(ticks as i64)))
        } else {
            if ticks > i64::MAX as u128 {
                return Err(ParseError::Overflow);
            }
            Ok(TimeSpan::from_ticks(ticks as i64))
        }
    }

    // ── Lenient parser (parse / parse_with_culture) ───────────────────────────

    pub fn parse_lenient(input: &str, sep: char) -> Result<TimeSpan, ParseError> {
        let s = input.trim();
        if s.is_empty() { return Err(ParseError::Empty); }
        if s.contains('\x00') { return Err(ParseError::InvalidCharacter); }

        let (neg, s) = strip_neg(s);
        if s.is_empty() { return Err(ParseError::Empty); }

        // A '.' before the first ':' is the days separator (always '.', culture-independent).
        let first_colon = s.find(':');
        let days_dot = s.find('.').filter(|&d| first_colon.map_or(true, |c| d < c));

        let (days_str, time_s) = if let Some(dot) = days_dot {
            (Some(&s[..dot]), &s[dot + 1..])
        } else {
            (None, s)
        };

        let n_colons = time_s.bytes().filter(|&b| b == b':').count();

        if n_colons == 0 {
            if days_dot.is_some() { return Err(ParseError::InvalidStructure); }
            let mut b = Builder::new(neg);
            b.days = Some(time_s);
            return b.build();
        }

        let parts: Vec<&str> = time_s.splitn(n_colons + 1, ':').collect();

        // All but the last component must be non-empty digit strings.
        for p in &parts[..parts.len() - 1] {
            if p.is_empty() { return Err(ParseError::InvalidStructure); }
            if !p.bytes().all(|b| b.is_ascii_digit()) { return Err(ParseError::InvalidCharacter); }
        }

        // Last component: optional `integer[sep fraction]` or `[sep fraction]`.
        let last = *parts.last().unwrap();
        let (last_int_s, frac_s) = if let Some(d) = last.find(sep) {
            (&last[..d], Some(&last[d + sep.len_utf8()..]))
        } else {
            if sep != '.' && last.contains('.') { return Err(ParseError::WrongSeparator); }
            (last, None)
        };

        if !last_int_s.is_empty() && !last_int_s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidCharacter);
        }
        // Trailing colon with no integer and no fraction is structurally invalid.
        if last_int_s.is_empty() && frac_s.is_none() { return Err(ParseError::InvalidStructure); }

        let total = parts.len();
        let mut b = Builder::new(neg);
        b.frac = frac_s;

        match (days_dot.is_some(), total) {
            // d.h:m[.frac]
            (true, 2) => {
                b.days    = days_str;
                b.hours   = Some(parts[0]);
                b.minutes = Some(last_int_s);
            }
            // d.h:m:s[.frac]
            (true, 3) => {
                b.days    = days_str;
                b.hours   = Some(parts[0]);
                b.minutes = Some(parts[1]);
                b.seconds = Some(last_int_s);
            }
            // h:m[.frac]
            (false, 2) => {
                b.hours   = Some(parts[0]);
                b.minutes = Some(last_int_s);
            }
            // h:m:s[.frac]  or  d:h:m[.frac] when first component > 23
            (false, 3) => {
                let first_val = parse_component_uint(Some(parts[0]))?;
                if first_val > 23 {
                    b.days    = Some(parts[0]);
                    b.hours   = Some(parts[1]);
                    b.minutes = Some(last_int_s);
                } else {
                    b.hours   = Some(parts[0]);
                    b.minutes = Some(parts[1]);
                    b.seconds = Some(last_int_s);
                }
            }
            // d:h:m:s[.frac]
            (false, 4) => {
                b.days    = Some(parts[0]);
                b.hours   = Some(parts[1]);
                b.minutes = Some(parts[2]);
                b.seconds = Some(last_int_s);
            }
            _ => return Err(ParseError::InvalidStructure),
        }

        b.build()
    }

    // ── parse_exact ───────────────────────────────────────────────────────────

    pub fn parse_exact(s: &str, fmt: &str, sep: char) -> Result<TimeSpan, ParseError> {
        match fmt {
            "c" | "t" | "T" => parse_constant(s),
            "g" => parse_g(s, sep),
            "G" => parse_g_upper(s, sep),
            "" => Err(ParseError::InvalidStructure),
            _ => parse_custom(s, fmt),
        }
    }

    fn strip_neg(s: &str) -> (bool, &str) {
        if let Some(r) = s.strip_prefix('-') { (true, r) } else { (false, s) }
    }

    fn split_at_sep<'a>(s: &'a str, sep: char) -> (&'a str, Option<&'a str>) {
        match s.find(sep) {
            Some(d) => (&s[..d], Some(&s[d + sep.len_utf8()..])),
            None    => (s, None),
        }
    }

    /// "c"/"t"/"T": `[-][d.]hh:mm:ss[.fffffff]`
    fn parse_constant(s: &str) -> Result<TimeSpan, ParseError> {
        if s.trim().is_empty() { return Err(ParseError::Empty); }
        let (neg, s) = strip_neg(s.trim());
        if s.is_empty() { return Err(ParseError::Empty); }

        if s.bytes().filter(|&b| b == b':').count() != 2 {
            return Err(ParseError::InvalidStructure);
        }

        let first_colon = s.find(':').unwrap();
        let (days_str, time_s) = if let Some(dot) = s[..first_colon].find('.') {
            (Some(&s[..dot]), &s[dot + 1..])
        } else {
            (None, s)
        };

        let parts: Vec<&str> = time_s.splitn(3, ':').collect();
        let (sec_s, frac_s) = split_at_sep(parts[2], '.');
        if sec_s.is_empty() && frac_s.is_none() { return Err(ParseError::InvalidStructure); }

        let mut b = Builder::new(neg);
        b.days    = days_str;
        b.hours   = Some(parts[0]);
        b.minutes = Some(parts[1]);
        b.seconds = Some(sec_s);
        b.frac    = frac_s;
        b.build()
    }

    /// "g": `[-][d:]h:mm:ss[.FFFFFFF]`
    fn parse_g(s: &str, sep: char) -> Result<TimeSpan, ParseError> {
        if s.trim().is_empty() { return Err(ParseError::Empty); }
        let (neg, s) = strip_neg(s.trim());
        if s.is_empty() { return Err(ParseError::Empty); }

        // Separator before the first colon would mean a days-dot — invalid for "g".
        let first_colon = s.find(':');
        if let Some(dot) = s.find(sep) {
            if first_colon.map_or(true, |c| dot < c) {
                return Err(ParseError::InvalidStructure);
            }
        }

        let n_colons = s.bytes().filter(|&b| b == b':').count();
        let parts: Vec<&str> = s.splitn(n_colons + 1, ':').collect();
        let mut b = Builder::new(neg);

        match n_colons {
            0 => { b.days = Some(parts[0]); }
            1 => {
                b.hours   = Some(parts[0]);
                b.minutes = Some(parts[1]);
            }
            2 => {
                let (sec_s, frac_s) = split_at_sep(parts[2], sep);
                if sec_s.is_empty() && frac_s.is_none() { return Err(ParseError::InvalidStructure); }
                b.hours   = Some(parts[0]);
                b.minutes = Some(parts[1]);
                b.seconds = Some(sec_s);
                b.frac    = frac_s;
            }
            3 => {
                let (sec_s, frac_s) = split_at_sep(parts[3], sep);
                if sec_s.is_empty() && frac_s.is_none() { return Err(ParseError::InvalidStructure); }
                b.days    = Some(parts[0]);
                b.hours   = Some(parts[1]);
                b.minutes = Some(parts[2]);
                b.seconds = Some(sec_s);
                b.frac    = frac_s;
            }
            _ => return Err(ParseError::InvalidStructure),
        }

        b.build()
    }

    /// "G": `[-]d:hh:mm:ss.fffffff` (fractional part required)
    fn parse_g_upper(s: &str, sep: char) -> Result<TimeSpan, ParseError> {
        if s.trim().is_empty() { return Err(ParseError::Empty); }
        let (neg, s) = strip_neg(s.trim());
        if s.is_empty() { return Err(ParseError::Empty); }

        if s.bytes().filter(|&b| b == b':').count() != 3 {
            return Err(ParseError::InvalidStructure);
        }

        let parts: Vec<&str> = s.splitn(4, ':').collect();
        let dot = parts[3].find(sep).ok_or(ParseError::InvalidStructure)?;

        let mut b = Builder::new(neg);
        b.days    = Some(parts[0]);
        b.hours   = Some(parts[1]);
        b.minutes = Some(parts[2]);
        b.seconds = Some(&parts[3][..dot]);
        b.frac    = Some(&parts[3][dot + sep.len_utf8()..]);
        b.build()
    }

    /// Custom format specifier parsing.
    fn parse_custom(input: &str, fmt: &str) -> Result<TimeSpan, ParseError> {
        let fmt_chars: Vec<char> = fmt.chars().collect();
        let mut inp = input;
        let mut fi = 0;
        let mut b = Builder::new(false);

        while fi < fmt_chars.len() {
            match fmt_chars[fi] {
                '%' if fi + 1 < fmt_chars.len() => {
                    fi += 1;
                    let ch = fmt_chars[fi];
                    fi += 1;
                    apply_spec(ch, 1, &mut inp, &mut b)?;
                }
                '%' => return Err(ParseError::InvalidStructure),
                ch @ ('d' | 'h' | 'm' | 's' | 'f' | 'F') => {
                    let n = fmt_chars[fi..].iter().take_while(|&&c| c == ch).count();
                    fi += n;
                    let max = match ch { 'd' => 8, 'h' | 'm' | 's' => 2, _ => 7 };
                    if n > max { return Err(ParseError::InvalidStructure); }
                    apply_spec(ch, n, &mut inp, &mut b)?;
                }
                '\\' if fi + 1 < fmt_chars.len() => {
                    fi += 1;
                    let expected = fmt_chars[fi];
                    fi += 1;
                    let ch = inp.chars().next().ok_or(ParseError::InvalidStructure)?;
                    if ch != expected { return Err(ParseError::InvalidStructure); }
                    inp = &inp[ch.len_utf8()..];
                }
                '\'' | '"' => {
                    let q = fmt_chars[fi];
                    fi += 1;
                    let start = fi;
                    while fi < fmt_chars.len() && fmt_chars[fi] != q { fi += 1; }
                    if fi >= fmt_chars.len() { return Err(ParseError::InvalidStructure); }
                    let lit: String = fmt_chars[start..fi].iter().collect();
                    fi += 1;
                    if !inp.starts_with(lit.as_str()) { return Err(ParseError::InvalidStructure); }
                    inp = &inp[lit.len()..];
                }
                _ => return Err(ParseError::InvalidStructure),
            }
        }

        if !inp.is_empty() { return Err(ParseError::InvalidStructure); }
        b.build()
    }

    fn apply_spec<'a>(
        ch: char, n: usize,
        inp: &mut &'a str,
        b: &mut Builder<'a>,
    ) -> Result<(), ParseError> {
        macro_rules! dup { ($field:expr) => { if $field.is_some() { return Err(ParseError::InvalidStructure); } }; }
        match ch {
            'd' => { dup!(b.days);    b.days    = Some(if n == 1 { read_greedy_str(inp, 8)? } else { read_exact_str(inp, n)? }); }
            'h' => { dup!(b.hours);   b.hours   = Some(if n == 1 { read_greedy_str(inp, 2)? } else { read_exact_str(inp, n)? }); }
            'm' => { dup!(b.minutes); b.minutes = Some(if n == 1 { read_greedy_str(inp, 2)? } else { read_exact_str(inp, n)? }); }
            's' => { dup!(b.seconds); b.seconds = Some(if n == 1 { read_greedy_str(inp, 2)? } else { read_exact_str(inp, n)? }); }
            'f' | 'F' => { dup!(b.frac); b.frac = Some(read_frac_str(inp, n, ch == 'F')?); }
            _ => return Err(ParseError::InvalidStructure),
        }
        Ok(())
    }

    fn read_greedy_str<'a>(inp: &mut &'a str, max: usize) -> Result<&'a str, ParseError> {
        let n = inp.bytes().take(max).take_while(|b| b.is_ascii_digit()).count();
        if n == 0 { return Err(ParseError::InvalidStructure); }
        let s = &inp[..n];
        *inp = &inp[n..];
        Ok(s)
    }

    fn read_exact_str<'a>(inp: &mut &'a str, n: usize) -> Result<&'a str, ParseError> {
        if inp.len() < n { return Err(ParseError::InvalidStructure); }
        if !inp[..n].bytes().all(|b| b.is_ascii_digit()) { return Err(ParseError::InvalidCharacter); }
        let s = &inp[..n];
        *inp = &inp[n..];
        Ok(s)
    }

    fn read_frac_str<'a>(inp: &mut &'a str, n: usize, greedy: bool) -> Result<&'a str, ParseError> {
        if greedy {
            let count = inp.bytes().take(n).take_while(|b| b.is_ascii_digit()).count();
            if count == 0 { return Err(ParseError::InvalidStructure); }
            let s = &inp[..count];
            *inp = &inp[count..];
            Ok(s)
        } else {
            read_exact_str(inp, n)
        }
    }
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
