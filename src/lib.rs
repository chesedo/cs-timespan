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
        parse_impl::parse_lenient(s, Culture::Invariant)
    }

    pub fn parse_with_culture(s: &str, culture: Culture) -> Result<Self, ParseError> {
        parse_impl::parse_lenient(s, culture)
    }

    pub fn try_parse(s: &str) -> Option<Self> {
        Self::parse(s).ok()
    }

    pub fn try_parse_with_culture(s: &str, culture: Culture) -> Option<Self> {
        Self::parse_with_culture(s, culture).ok()
    }

    // ── Strict parsing (mirrors ParseExact / TryParseExact) ───────────────────
    pub fn parse_exact(s: &str, fmt: &str) -> Result<Self, ParseError> {
        parse_impl::parse_exact(s, fmt, Culture::Invariant)
    }

    pub fn parse_exact_any(s: &str, formats: &[&str]) -> Result<Self, ParseError> {
        let mut last = ParseError::InvalidFormat;
        for fmt in formats {
            match Self::parse_exact(s, fmt) {
                Ok(ts) => return Ok(ts),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn parse_exact_with_culture(
        s: &str,
        fmt: &str,
        culture: Culture,
    ) -> Result<Self, ParseError> {
        parse_impl::parse_exact(s, fmt, culture)
    }

    pub fn parse_exact_any_with_culture(
        s: &str,
        formats: &[&str],
        culture: Culture,
    ) -> Result<Self, ParseError> {
        let mut last = ParseError::InvalidFormat;
        for fmt in formats {
            match Self::parse_exact_with_culture(s, fmt, culture) {
                Ok(ts) => return Ok(ts),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    pub fn parse_exact_with_styles(
        s: &str,
        _fmt: &str,
        _culture: Culture,
        _styles: TimeSpanStyles,
    ) -> Result<Self, ParseError> {
        todo!()
    }

    pub fn try_parse_exact(s: &str, fmt: &str) -> Option<Self> {
        Self::parse_exact(s, fmt).ok()
    }

    pub fn try_parse_exact_any(s: &str, formats: &[&str]) -> Option<Self> {
        Self::parse_exact_any(s, formats).ok()
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
    let mut out = String::new();
    if c.negative {
        out.push('-');
    }
    if c.days > 0 {
        out.push_str(&c.days.to_string());
        out.push(':');
    }
    out.push_str(&format!("{}:{:02}:{:02}", c.hours, c.minutes, c.seconds));
    if c.sub_sec_ticks > 0 {
        out.push(decimal_sep(culture));
        // FFFFFFF — trim trailing zeros
        out.push_str(&fmt_frac(c.sub_sec_ticks, 7, true));
    }
    out
}

/// `"G"`: `[-]d:hh:mm:ss.fffffff` — culture-sensitive decimal separator.
fn format_general_long(c: &Components, culture: Culture) -> String {
    let mut out = String::new();
    if c.negative {
        out.push('-');
    }
    out.push_str(&format!(
        "{}:{:02}:{:02}:{:02}{}{}",
        c.days,
        c.hours,
        c.minutes,
        c.seconds,
        decimal_sep(culture),
        fmt_frac(c.sub_sec_ticks, 7, false),
    ));
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
            if s.len() < n { format!("{:0>width$}", s, width = n) } else { s }
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
    if n == 1 { val.to_string() } else { format!("{:02}", val) }
}

fn decimal_sep(culture: Culture) -> char {
    match culture {
        Culture::Invariant => '.',
        Culture::HrHR | Culture::FrFR => ',',
    }
}

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

mod parse_impl {
    use super::{decimal_sep, Culture, ParseError, TimeSpan};

    fn parse_uint(s: &str) -> Result<u64, ParseError> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }
        s.parse::<u64>().map_err(|_| ParseError::Overflow)
    }

    fn parse_frac(s: &str) -> Result<u32, ParseError> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }
        if s.len() > 7 {
            return Err(ParseError::Overflow);
        }
        let v = s.parse::<u32>().unwrap();
        Ok(v * 10u32.pow(7 - s.len() as u32))
    }

    fn build(neg: bool, days: u64, h: u32, m: u32, s: u32, frac: u32) -> Result<TimeSpan, ParseError> {
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

    pub fn parse_lenient(input: &str, culture: Culture) -> Result<TimeSpan, ParseError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(ParseError::InvalidFormat);
        }
        if s.contains('\x00') {
            return Err(ParseError::InvalidFormat);
        }

        let sep = decimal_sep(culture);

        let (neg, s) = if let Some(r) = s.strip_prefix('-') {
            (true, r)
        } else {
            (false, s)
        };
        if s.is_empty() {
            return Err(ParseError::InvalidFormat);
        }

        // A '.' before the first ':' is the days separator (always '.', culture-independent).
        let first_colon = s.find(':');
        let days_dot = s.find('.').filter(|&d| first_colon.map_or(true, |c| d < c));

        let (days, time_s) = if let Some(dot) = days_dot {
            (parse_uint(&s[..dot])?, &s[dot + 1..])
        } else {
            (0u64, s)
        };

        let n_colons = time_s.bytes().filter(|&b| b == b':').count();

        if n_colons == 0 {
            // Bare integer: days (when no dot prefix) or invalid (when dot prefix without time)
            if days_dot.is_some() {
                return Err(ParseError::InvalidFormat);
            }
            let d = parse_uint(time_s)?;
            return build(neg, d, 0, 0, 0, 0);
        }

        // Split into colon-separated parts; last part may carry a fractional suffix.
        let parts: Vec<&str> = time_s.splitn(n_colons + 1, ':').collect();

        // All but last must be pure non-empty digit strings.
        for p in &parts[..parts.len() - 1] {
            if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ParseError::InvalidFormat);
            }
        }

        // Last part: optional "integer[sep fraction]" or "[sep fraction]" (empty seconds).
        let last = *parts.last().unwrap();
        let (last_int_s, frac) = if let Some(d) = last.find(sep) {
            let frac_part = &last[d + sep.len_utf8()..];
            (&last[..d], parse_frac(frac_part)?)
        } else {
            // Reject wrong-culture decimal separator in the last segment.
            if sep != '.' && last.contains('.') {
                return Err(ParseError::InvalidFormat);
            }
            (last, 0u32)
        };

        // Validate the integer portion of the last segment.
        if !last_int_s.is_empty() && !last_int_s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }
        // Trailing colon with no content (and no fraction) is invalid.
        if last_int_s.is_empty() && !last.starts_with(sep) {
            return Err(ParseError::InvalidFormat);
        }

        let mut vals: Vec<u64> = parts[..parts.len() - 1]
            .iter()
            .map(|p| parse_uint(p))
            .collect::<Result<_, _>>()?;
        let last_val = if last_int_s.is_empty() { 0 } else { parse_uint(last_int_s)? };
        vals.push(last_val);

        let total = vals.len();
        let mut d = days;

        let (h, m, sv) = match (days_dot.is_some(), total) {
            // d.h:m
            (true, 2) => {
                let (h, m) = (vals[0] as u32, vals[1] as u32);
                if h >= 24 || m >= 60 { return Err(ParseError::Overflow); }
                (h, m, 0u32)
            }
            // d.h:m:s
            (true, 3) => {
                let (h, m, sv) = (vals[0] as u32, vals[1] as u32, vals[2] as u32);
                if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
                (h, m, sv)
            }
            // h:m
            (false, 2) => {
                let (h, m) = (vals[0] as u32, vals[1] as u32);
                if h >= 24 { return Err(ParseError::Overflow); }
                if m >= 60 { return Err(ParseError::Overflow); }
                (h, m, 0u32)
            }
            // h:m:s  or  d:h:m (when first > 23)
            (false, 3) => {
                if vals[0] > 23 {
                    d = vals[0];
                    let (h, m) = (vals[1] as u32, vals[2] as u32);
                    if h >= 24 || m >= 60 { return Err(ParseError::Overflow); }
                    (h, m, 0u32)
                } else {
                    let (h, m, sv) = (vals[0] as u32, vals[1] as u32, vals[2] as u32);
                    if m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
                    (h, m, sv)
                }
            }
            // d:h:m:s
            (false, 4) => {
                d = vals[0];
                let (h, m, sv) = (vals[1] as u32, vals[2] as u32, vals[3] as u32);
                if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
                (h, m, sv)
            }
            _ => return Err(ParseError::InvalidFormat),
        };

        build(neg, d, h, m, sv, frac)
    }

    // ── parse_exact ───────────────────────────────────────────────────────────

    pub fn parse_exact(s: &str, fmt: &str, _culture: Culture) -> Result<TimeSpan, ParseError> {
        match fmt {
            "c" | "t" | "T" => parse_constant(s),
            "g" => parse_g(s),
            "G" => parse_g_upper(s),
            "" => Err(ParseError::InvalidFormat),
            _ => parse_custom(s, fmt),
        }
    }

    fn strip_neg(s: &str) -> (bool, &str) {
        if let Some(r) = s.strip_prefix('-') { (true, r) } else { (false, s) }
    }

    /// "c"/"t"/"T": `[-][d.]hh:mm:ss[.fffffff]`
    fn parse_constant(s: &str) -> Result<TimeSpan, ParseError> {
        if s.trim().is_empty() { return Err(ParseError::InvalidFormat); }
        let (neg, s) = strip_neg(s.trim());
        if s.is_empty() { return Err(ParseError::InvalidFormat); }

        if s.bytes().filter(|&b| b == b':').count() != 2 {
            return Err(ParseError::InvalidFormat);
        }

        let first_colon = s.find(':').unwrap();
        let (days, s) = if let Some(dot) = s[..first_colon].find('.') {
            (parse_uint(&s[..dot])?, &s[dot + 1..])
        } else {
            (0u64, s)
        };

        let c1 = s.find(':').unwrap();
        let c2 = s[c1 + 1..].find(':').unwrap() + c1 + 1;
        let (sv_s, frac) = if let Some(dot) = s[c2 + 1..].find('.') {
            (&s[c2 + 1..c2 + 1 + dot], parse_frac(&s[c2 + 2 + dot..])?)
        } else {
            (&s[c2 + 1..], 0u32)
        };

        let h = parse_uint(&s[..c1])? as u32;
        let m = parse_uint(&s[c1 + 1..c2])? as u32;
        let sv = parse_uint(sv_s)? as u32;
        if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
        build(neg, days, h, m, sv, frac)
    }

    /// "g": `[-][d:]h:mm:ss[.FFFFFFF]`
    fn parse_g(s: &str) -> Result<TimeSpan, ParseError> {
        if s.trim().is_empty() { return Err(ParseError::InvalidFormat); }
        let (neg, s) = strip_neg(s.trim());
        if s.is_empty() { return Err(ParseError::InvalidFormat); }

        // Dot before any colon would be days separator — invalid for "g".
        let first_colon = s.find(':');
        if let Some(dot) = s.find('.') {
            if first_colon.map_or(true, |c| dot < c) {
                return Err(ParseError::InvalidFormat);
            }
        }

        let n_colons = s.bytes().filter(|&b| b == b':').count();
        let parts: Vec<&str> = s.splitn(n_colons + 1, ':').collect();

        match n_colons {
            0 => build(neg, parse_uint(parts[0])?, 0, 0, 0, 0),
            1 => {
                let h = parse_uint(parts[0])? as u32;
                let m = parse_uint(parts[1])? as u32;
                if h >= 24 || m >= 60 { return Err(ParseError::Overflow); }
                build(neg, 0, h, m, 0, 0)
            }
            2 => {
                let h = parse_uint(parts[0])? as u32;
                let m = parse_uint(parts[1])? as u32;
                let (sv, frac) = last_with_frac(parts[2], '.')?;
                if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
                build(neg, 0, h, m, sv, frac)
            }
            3 => {
                let d = parse_uint(parts[0])?;
                let h = parse_uint(parts[1])? as u32;
                let m = parse_uint(parts[2])? as u32;
                let (sv, frac) = last_with_frac(parts[3], '.')?;
                if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
                build(neg, d, h, m, sv, frac)
            }
            _ => Err(ParseError::InvalidFormat),
        }
    }

    /// "G": `[-]d:hh:mm:ss.fffffff` (fractional part required)
    fn parse_g_upper(s: &str) -> Result<TimeSpan, ParseError> {
        if s.trim().is_empty() { return Err(ParseError::InvalidFormat); }
        let (neg, s) = strip_neg(s.trim());
        if s.is_empty() { return Err(ParseError::InvalidFormat); }

        if s.bytes().filter(|&b| b == b':').count() != 3 {
            return Err(ParseError::InvalidFormat);
        }

        let parts: Vec<&str> = s.splitn(4, ':').collect();
        let d = parse_uint(parts[0])?;
        let h = parse_uint(parts[1])? as u32;
        let m = parse_uint(parts[2])? as u32;
        let dot = parts[3].find('.').ok_or(ParseError::InvalidFormat)?;
        let sv = parse_uint(&parts[3][..dot])? as u32;
        let frac = parse_frac(&parts[3][dot + 1..])?;

        if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
        build(neg, d, h, m, sv, frac)
    }

    /// Custom format specifier parsing.
    fn parse_custom(input: &str, fmt: &str) -> Result<TimeSpan, ParseError> {
        let fmt_chars: Vec<char> = fmt.chars().collect();
        let mut inp = input;
        let mut fi = 0;
        let mut days: Option<u64> = None;
        let mut hours: Option<u32> = None;
        let mut minutes: Option<u32> = None;
        let mut seconds: Option<u32> = None;
        let mut frac: Option<u32> = None;

        while fi < fmt_chars.len() {
            match fmt_chars[fi] {
                '%' if fi + 1 < fmt_chars.len() => {
                    fi += 1;
                    let ch = fmt_chars[fi];
                    fi += 1;
                    apply_spec(ch, 1, &mut inp, &mut days, &mut hours, &mut minutes, &mut seconds, &mut frac)?;
                }
                '%' => return Err(ParseError::InvalidFormat),
                ch @ ('d' | 'h' | 'm' | 's' | 'f' | 'F') => {
                    let n = fmt_chars[fi..].iter().take_while(|&&c| c == ch).count();
                    fi += n;
                    let max = match ch { 'd' => 8, 'h' | 'm' | 's' => 2, _ => 7 };
                    if n > max { return Err(ParseError::InvalidFormat); }
                    apply_spec(ch, n, &mut inp, &mut days, &mut hours, &mut minutes, &mut seconds, &mut frac)?;
                }
                '\\' if fi + 1 < fmt_chars.len() => {
                    fi += 1;
                    let expected = fmt_chars[fi];
                    fi += 1;
                    let ch = inp.chars().next().ok_or(ParseError::InvalidFormat)?;
                    if ch != expected { return Err(ParseError::InvalidFormat); }
                    inp = &inp[ch.len_utf8()..];
                }
                '\'' | '"' => {
                    let q = fmt_chars[fi];
                    fi += 1;
                    let start = fi;
                    while fi < fmt_chars.len() && fmt_chars[fi] != q { fi += 1; }
                    if fi >= fmt_chars.len() { return Err(ParseError::InvalidFormat); }
                    let lit: String = fmt_chars[start..fi].iter().collect();
                    fi += 1;
                    if !inp.starts_with(lit.as_str()) { return Err(ParseError::InvalidFormat); }
                    inp = &inp[lit.len()..];
                }
                _ => return Err(ParseError::InvalidFormat),
            }
        }

        if !inp.is_empty() { return Err(ParseError::InvalidFormat); }
        let h = hours.unwrap_or(0);
        let m = minutes.unwrap_or(0);
        let sv = seconds.unwrap_or(0);
        if h >= 24 || m >= 60 || sv >= 60 { return Err(ParseError::Overflow); }
        build(false, days.unwrap_or(0), h, m, sv, frac.unwrap_or(0))
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_spec<'a>(
        ch: char, n: usize,
        inp: &mut &'a str,
        days: &mut Option<u64>, hours: &mut Option<u32>,
        minutes: &mut Option<u32>, seconds: &mut Option<u32>,
        frac: &mut Option<u32>,
    ) -> Result<(), ParseError> {
        macro_rules! dup { ($s:expr) => { if $s.is_some() { return Err(ParseError::InvalidFormat); } }; }
        match ch {
            'd' => { dup!(days);    let v = if n == 1 { rd_grdy(inp, 8)? } else { rd_exact(inp, n)? }; *days = Some(v); }
            'h' => { dup!(hours);   let v = if n == 1 { rd_grdy(inp, 2)? } else { rd_exact(inp, n)? }; *hours = Some(v as u32); }
            'm' => { dup!(minutes); let v = if n == 1 { rd_grdy(inp, 2)? } else { rd_exact(inp, n)? }; *minutes = Some(v as u32); }
            's' => { dup!(seconds); let v = if n == 1 { rd_grdy(inp, 2)? } else { rd_exact(inp, n)? }; *seconds = Some(v as u32); }
            'f' | 'F' => {
                dup!(frac);
                let v = rd_frac(inp, n, ch == 'F')?;
                *frac = Some(v);
            }
            _ => return Err(ParseError::InvalidFormat),
        }
        Ok(())
    }

    fn rd_grdy<'a>(inp: &mut &'a str, max: usize) -> Result<u64, ParseError> {
        let n = inp.bytes().take(max).take_while(|b| b.is_ascii_digit()).count();
        if n == 0 { return Err(ParseError::InvalidFormat); }
        let v = inp[..n].parse::<u64>().map_err(|_| ParseError::Overflow)?;
        *inp = &inp[n..];
        Ok(v)
    }

    fn rd_exact<'a>(inp: &mut &'a str, n: usize) -> Result<u64, ParseError> {
        if inp.len() < n || !inp[..n].bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }
        let v = inp[..n].parse::<u64>().map_err(|_| ParseError::Overflow)?;
        *inp = &inp[n..];
        Ok(v)
    }

    fn rd_frac<'a>(inp: &mut &'a str, n: usize, greedy: bool) -> Result<u32, ParseError> {
        if greedy {
            let count = inp.bytes().take(n).take_while(|b| b.is_ascii_digit()).count();
            if count == 0 { return Err(ParseError::InvalidFormat); }
            let v = inp[..count].parse::<u32>().unwrap();
            *inp = &inp[count..];
            Ok(v * 10u32.pow(7 - count as u32))
        } else {
            rd_exact(inp, n).map(|v| v as u32 * 10u32.pow(7 - n as u32))
        }
    }

    fn last_with_frac(s: &str, sep: char) -> Result<(u32, u32), ParseError> {
        if let Some(dot) = s.find(sep) {
            let int_s = &s[..dot];
            let sv = if int_s.is_empty() { 0u32 } else { parse_uint(int_s)? as u32 };
            Ok((sv, parse_frac(&s[dot + 1..])?))
        } else {
            if s.is_empty() { return Err(ParseError::InvalidFormat); }
            Ok((parse_uint(s)? as u32, 0))
        }
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
