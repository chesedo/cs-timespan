use crate::TimeSpan;

/// The reason a [`ParseErrorKind::Overflow`] error was triggered.
#[derive(Debug, Clone)]
pub enum OverflowKind {
    /// Hours component was ≥ 24; carries the out-of-range value.
    Hours(u64),
    /// Minutes component was ≥ 60; carries the out-of-range value.
    Minutes(u64),
    /// Seconds component was ≥ 60; carries the out-of-range value.
    Seconds(u64),
    /// Total tick value exceeds [`TimeSpan::MAX_VALUE`].
    TooLarge,
    /// Total tick value is below [`TimeSpan::MIN_VALUE`].
    TooSmall,
}

/// The category of error returned when a time-span string fails to parse.
///
/// Mirrors the two distinct failure modes C# separates into
/// `FormatException` (bad syntax) and `OverflowException` (out of range),
/// with the format case split into more actionable variants.
#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    /// Input is empty or whitespace-only, or reduces to nothing after
    /// stripping a leading `-`.
    Empty,
    /// A non-digit character appeared where only digits are valid, or the
    /// input ran out before all required digits were read.
    NonDigit,
    /// The decimal separator in the input does not match the locale
    /// (e.g. `'.'` when the locale uses `','`).
    WrongSeparator,
    /// The component structure is unrecognised. The payload names the
    /// expected pattern (e.g. `"[-][d.]hh:mm[:ss[.fffffff]]"`).
    InvalidStructure(Box<str>),
    /// The custom format string is itself malformed. The payload describes
    /// what is wrong with the format (e.g. `"duplicate 'h' specifier in format"`).
    InvalidFormat(Box<str>),
    /// The value is syntactically valid but outside the representable range
    /// (`OverflowException`).
    Overflow(OverflowKind),
}

// Compare by variant only — the expected-description payload is informational.
impl PartialEq for ParseErrorKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for ParseErrorKind {}

/// Error returned when a time-span string fails to parse.
///
/// Carries the [`kind`](ParseError::kind) of error, the byte [`pos`](ParseError::pos)
/// where it was detected, and the original input string for display purposes.
#[derive(Debug, Clone)]
pub struct ParseError {
    kind: ParseErrorKind,
    pos: usize,
    input: Box<str>,
}

impl ParseError {
    pub(crate) fn new(kind: ParseErrorKind, pos: usize, input: &str) -> Self {
        Self {
            kind,
            pos,
            input: input.into(),
        }
    }

    /// The category of error.
    #[must_use]
    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    /// Byte index (0-based) of the offending character in the input string.
    #[must_use]
    pub fn pos(&self) -> usize {
        self.pos
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ParseErrorKind::Empty => writeln!(f, "input is empty")?,
            ParseErrorKind::NonDigit => {
                let found = self.input[self.pos..].chars().next();
                match found {
                    Some(ch) => writeln!(f, "unexpected character {ch:?}; expected a digit")?,
                    None => writeln!(f, "unexpected end of input; expected a digit")?,
                }
            }
            ParseErrorKind::WrongSeparator => {
                writeln!(f, "decimal separator does not match the locale")?;
            }
            ParseErrorKind::InvalidStructure(expected) => {
                writeln!(f, "unrecognised input structure; expected {expected}")?;
            }
            ParseErrorKind::InvalidFormat(desc) => writeln!(f, "invalid custom format: {desc}")?,
            ParseErrorKind::Overflow(reason) => match reason {
                OverflowKind::Hours(h) => {
                    writeln!(f, "hours value {h} is out of range; must be 0-23")?;
                }
                OverflowKind::Minutes(m) => {
                    writeln!(f, "minutes value {m} is out of range; must be 0-59")?;
                }
                OverflowKind::Seconds(s) => {
                    writeln!(f, "seconds value {s} is out of range; must be 0-59")?;
                }
                OverflowKind::TooLarge => {
                    writeln!(f, "TimeSpan value exceeds the maximum representable range")?;
                }
                OverflowKind::TooSmall => {
                    writeln!(f, "TimeSpan value is below the minimum representable range")?;
                }
            },
        }
        writeln!(f, "  \"{}\"", self.input)?;
        write!(f, "   {}^", " ".repeat(self.pos))
    }
}

impl std::error::Error for ParseError {}

impl PartialEq for ParseError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

/// Mirrors `System.Globalization.TimeSpanStyles`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeSpanStyles {
    #[default]
    None,
    /// Treat the parsed interval as negative even without a leading `-`.
    AssumeNegative,
}

// ── Builder ───────────────────────────────────────────────────────────────────

struct Builder<'a> {
    neg: bool,
    original: &'a str,
    days: Option<&'a str>,
    hours: Option<&'a str>,
    minutes: Option<&'a str>,
    seconds: Option<&'a str>,
    frac: Option<&'a str>,
}

impl<'a> Builder<'a> {
    fn new(neg: bool, original: &'a str) -> Self {
        Self {
            neg,
            original,
            days: None,
            hours: None,
            minutes: None,
            seconds: None,
            frac: None,
        }
    }

    // C# TryTimeToTicks (TimeSpanParse.cs) rejects hours >= 24 / minutes >= 60 /
    // seconds >= 60 with OverflowException unconditionally — standard and custom
    // format parsing both route through it, so there's no lenient/normalizing path.
    fn build(self) -> Result<TimeSpan, ParseError> {
        let days = parse_component_uint(self.days, self.original, self.neg)?;
        let h = parse_component_uint(self.hours, self.original, self.neg)?;
        let m = parse_component_uint(self.minutes, self.original, self.neg)?;
        let sv = parse_component_uint(self.seconds, self.original, self.neg)?;
        let frac = parse_component_frac(self.frac, self.original, self.neg)?;
        if h >= 24 {
            let pos = self.hours.map_or(0, |s| offset_of(self.original, s));
            return Err(overflow(OverflowKind::Hours(h), pos, self.original));
        }
        if m >= 60 {
            let pos = self.minutes.map_or(0, |s| offset_of(self.original, s));
            return Err(overflow(OverflowKind::Minutes(m), pos, self.original));
        }
        if sv >= 60 {
            let pos = self.seconds.map_or(0, |s| offset_of(self.original, s));
            return Err(overflow(OverflowKind::Seconds(sv), pos, self.original));
        }
        // h ≤ 23, m ≤ 59, sv ≤ 59: safe to narrow to u32 for build_ticks
        #[allow(clippy::cast_possible_truncation)]
        build_ticks(
            self.neg,
            days,
            h as u32,
            m as u32,
            sv as u32,
            frac,
            self.original,
        )
    }
}

fn parse_component_uint(s: Option<&str>, original: &str, neg: bool) -> Result<u64, ParseError> {
    let s = match s {
        None | Some("") => return Ok(0),
        Some(s) => s,
    };
    if let Some(i) = s.bytes().position(|b| !b.is_ascii_digit()) {
        return Err(ParseError::new(
            ParseErrorKind::NonDigit,
            offset_of(original, s) + i,
            original,
        ));
    }
    s.bytes()
        .try_fold(0u64, |acc, b| {
            acc.checked_mul(10)?.checked_add(u64::from(b - b'0'))
        })
        .ok_or_else(|| {
            let dir = if neg {
                OverflowKind::TooSmall
            } else {
                OverflowKind::TooLarge
            };
            overflow(dir, offset_of(original, s), original)
        })
}

fn parse_component_frac(s: Option<&str>, original: &str, neg: bool) -> Result<u32, ParseError> {
    let Some(s) = s else {
        return Ok(0);
    };
    if s.is_empty() {
        return Err(ParseError::new(
            ParseErrorKind::NonDigit,
            offset_of(original, s),
            original,
        ));
    }
    if let Some(i) = s.bytes().position(|b| !b.is_ascii_digit()) {
        return Err(ParseError::new(
            ParseErrorKind::NonDigit,
            offset_of(original, s) + i,
            original,
        ));
    }
    let total = s.len();
    #[allow(clippy::cast_possible_truncation)] // total <= 7 keeps exponent and result in u32 range
    if total <= 7 {
        let v = s
            .bytes()
            .fold(0u32, |acc, b| acc * 10 + u32::from(b - b'0'));
        return Ok(v * 10u32.pow(7 - total as u32));
    }
    // C# NormalizeAndValidateFraction (TimeSpanParse.cs line 148): fractions longer than
    // 7 digits are accepted only when leading zeros push the significant digits into range.
    // Fractions with no leading zeros and len > 7 always exceed MaxFraction (9_999_999).
    let zeroes = s.bytes().take_while(|&b| b == b'0').count();
    if zeroes == 0 {
        let dir = if neg {
            OverflowKind::TooSmall
        } else {
            OverflowKind::TooLarge
        };
        return Err(overflow(dir, offset_of(original, s), original));
    }
    if zeroes > 7 {
        return Ok(0);
    }
    let num = s[zeroes..]
        .bytes()
        .fold(0u64, |acc, b| acc * 10 + u64::from(b - b'0'));
    #[allow(clippy::cast_possible_truncation)] // total - 7 < 8, fits u32
    let power = 10u64.pow((total - 7) as u32);
    #[allow(clippy::cast_possible_truncation)] // rounded result ≤ 9_999_999, fits u32
    let result = ((num + power / 2) / power) as u32;
    Ok(result)
}

// ── Low-level helpers ─────────────────────────────────────────────────────────

/// Byte offset of `sub` within `original`. Both must originate from the same allocation.
fn offset_of(original: &str, sub: &str) -> usize {
    (sub.as_ptr() as usize).saturating_sub(original.as_ptr() as usize)
}

// h/m/s are always range-checked to ≤ 23/59/59 by the caller before this runs, so all
// arithmetic stays in u64 — no u128 needed, which is significantly cheaper on x86-64.
fn build_ticks(
    neg: bool,
    days: u64,
    h: u32,
    m: u32,
    s: u32,
    frac: u32,
    original: &str,
) -> Result<TimeSpan, ParseError> {
    // `neg` fully determines direction here: the two range checks below only ever
    // reject on the side matching the input's own sign.
    let ovf = || {
        let dir = if neg {
            OverflowKind::TooSmall
        } else {
            OverflowKind::TooLarge
        };
        overflow(dir, 0, original)
    };
    // h ≤ 23, m ≤ 59, s ≤ 59, frac ≤ 9_999_999: sub ≤ 864_000_000_000 — well within u64
    let sub = u64::from(h) * TimeSpan::TICKS_PER_HOUR.unsigned_abs()
        + u64::from(m) * TimeSpan::TICKS_PER_MINUTE.unsigned_abs()
        + u64::from(s) * TimeSpan::TICKS_PER_SECOND.unsigned_abs()
        + u64::from(frac);
    let ticks = days
        .checked_mul(TimeSpan::TICKS_PER_DAY.unsigned_abs())
        .and_then(|d| d.checked_add(sub))
        .ok_or_else(ovf)?;
    if neg {
        let abs_min = i64::MIN.unsigned_abs(); // = i64::MAX + 1
        match ticks.cmp(&abs_min) {
            std::cmp::Ordering::Greater => Err(ovf()),
            std::cmp::Ordering::Equal => Ok(TimeSpan::from_ticks(i64::MIN)),
            std::cmp::Ordering::Less => {
                #[allow(clippy::cast_possible_wrap)] // ticks < abs_min ≤ i64::MAX + 1
                Ok(TimeSpan::from_ticks(-(ticks as i64)))
            }
        }
    } else {
        if ticks > i64::MAX.unsigned_abs() {
            return Err(ovf());
        }
        #[allow(clippy::cast_possible_wrap)] // ticks ≤ i64::MAX
        Ok(TimeSpan::from_ticks(ticks as i64))
    }
}

// ── Expected-structure descriptions ──────────────────────────────────────────

const LENIENT_EXPECTED: &str = "[-][d.]h:mm[:ss[.FFFFFFF]] or [-]d:h:mm:ss[.FFFFFFF]";
const CONSTANT_EXPECTED: &str = "[-][d.]hh:mm[:ss[.fffffff]]";
const G_EXPECTED: &str = "[-][d:]h:mm[:ss[.FFFFFFF]]";
const G_UPPER_EXPECTED: &str = "[-]d:hh:mm:ss.fffffff";

fn invalid_structure(expected: &str, pos: usize, input: &str) -> ParseError {
    ParseError::new(
        ParseErrorKind::InvalidStructure(expected.into()),
        pos,
        input,
    )
}

fn invalid_format(desc: &str, fmt: &str, fmt_pos: usize) -> ParseError {
    ParseError::new(ParseErrorKind::InvalidFormat(desc.into()), fmt_pos, fmt)
}

fn overflow(kind: OverflowKind, pos: usize, input: &str) -> ParseError {
    ParseError::new(ParseErrorKind::Overflow(kind), pos, input)
}

// ── Lenient parser (parse / parse_with_culture) ───────────────────────────────

/// Lenient: `[-]{d | [d.]h:mm[:ss[.FFFFFFF]] | d:h:mm:ss[.FFFFFFF]}`
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_lenient(input: &str, sep: char) -> Result<TimeSpan, ParseError> {
    let (neg, s) = strip_neg(input.trim());
    if s.is_empty() {
        return Err(ParseError::new(ParseErrorKind::Empty, 0, input));
    }

    let mut it = s.split(':');
    let head = it.next().unwrap(); // split always yields at least one item
    let p1 = it.next();
    let p2 = it.next();
    let p3 = it.next();
    if let Some(extra) = it.next() {
        return Err(invalid_structure(
            LENIENT_EXPECTED,
            offset_of(input, extra),
            input,
        ));
    }

    // No colons → bare days only (d.anything requires a colon for the time part).
    if p1.is_none() {
        if head.contains('.') {
            let pos = offset_of(input, head) + head.find('.').unwrap();
            return Err(invalid_structure(LENIENT_EXPECTED, pos, input));
        }
        let mut b = Builder::new(neg, input);
        b.days = Some(head);
        return b.build();
    }

    // The first segment may carry a days prefix: `d.h` or just `h`.
    let (days_str, first_s) = if let Some((d, h)) = head.split_once('.') {
        if d.is_empty() || h.is_empty() {
            let pos = offset_of(input, head) + head.find('.').unwrap();
            return Err(invalid_structure(LENIENT_EXPECTED, pos, input));
        }
        (Some(d), h)
    } else {
        if head.is_empty() {
            return Err(invalid_structure(LENIENT_EXPECTED, 0, input));
        }
        (None, head)
    };

    // The fractional part is always in the last colon-split component (p3 > p2 > p1).
    // Pre-parse it once so the match arms only deal with structure.
    let last = p3.or(p2).or(p1).unwrap(); // p1 always Some — early-returned above
    let (last_int, frac_s) = if let Some((i, f)) = last.split_once(sep) {
        (i, Some(f))
    } else {
        let other_sep = if sep == '.' { ',' } else { '.' };
        if last.contains(other_sep) {
            let pos = offset_of(input, last) + last.find(other_sep).unwrap();
            return Err(ParseError::new(ParseErrorKind::WrongSeparator, pos, input));
        }
        (last, None)
    };
    if last_int.is_empty() && frac_s.is_none() {
        return Err(invalid_structure(
            LENIENT_EXPECTED,
            offset_of(input, last),
            input,
        ));
    }

    let mut b = Builder::new(neg, input);
    b.frac = frac_s;
    b.days = days_str; // dot-prefix days; overridden by colon-prefix cases below

    // p1/p2/p3 still carry the raw (pre-split) strings; Some(_) discards the last
    // component since last_int already holds its parsed integer portion.
    match (days_str.is_some(), p1, p2, p3) {
        // h:m[.frac] or d.h:m[.frac] — one colon
        (_, Some(_), None, None) => {
            b.hours = Some(first_s);
            b.minutes = Some(last_int);
        }
        // d.h:m:s[.frac] — two colons with days-dot
        (true, Some(mid), Some(_), None) => {
            if mid.is_empty() {
                return Err(invalid_structure(
                    LENIENT_EXPECTED,
                    offset_of(input, mid),
                    input,
                ));
            }
            b.hours = Some(first_s);
            b.minutes = Some(mid);
            b.seconds = Some(last_int);
        }
        // h:m:s[.frac] or d:h:m[.frac] when first > 23 — two colons, no days-dot
        (false, Some(mid), Some(_), None) => {
            if mid.is_empty() {
                return Err(invalid_structure(
                    LENIENT_EXPECTED,
                    offset_of(input, mid),
                    input,
                ));
            }
            let first_val = parse_component_uint(Some(first_s), input, neg)?;
            if first_val > 23 {
                b.days = Some(first_s);
                b.hours = Some(mid);
                b.minutes = Some(last_int);
            } else {
                b.hours = Some(first_s);
                b.minutes = Some(mid);
                b.seconds = Some(last_int);
            }
        }
        // d:h:m:s[.frac] — three colons, no days-dot
        (false, Some(mid1), Some(mid2), Some(_)) => {
            if mid1.is_empty() || mid2.is_empty() {
                let pos = if mid1.is_empty() {
                    offset_of(input, mid1)
                } else {
                    offset_of(input, mid2)
                };
                return Err(invalid_structure(LENIENT_EXPECTED, pos, input));
            }
            b.days = Some(first_s);
            b.hours = Some(mid1);
            b.minutes = Some(mid2);
            b.seconds = Some(last_int);
        }
        // d.h:m:s:?[.frac] — dot-prefixed days combined with three colons: one time
        // component too many for the "d." variant (which allows at most h:mm[:ss]).
        (true, Some(_), Some(_), Some(extra)) => {
            return Err(invalid_structure(
                LENIENT_EXPECTED,
                offset_of(input, extra),
                input,
            ));
        }
        // p1 is always Some at this point (bare-days with no colon is handled by an
        // earlier early return, same invariant `last`'s .unwrap() above relies on);
        // the type system just can't see that, so match still needs to cover it.
        _ => unreachable!("p1 is always Some here"),
    }

    b.build()
}

// ── parse_exact ───────────────────────────────────────────────────────────────

pub(crate) fn parse_exact(s: &str, fmt: &str, sep: char) -> Result<TimeSpan, ParseError> {
    match fmt {
        "c" | "t" | "T" => parse_constant(s),
        "g" => parse_g(s, sep),
        "G" => parse_g_upper(s, sep),
        "" => Err(invalid_format("empty format string", "", 0)),
        _ => parse_custom(s, fmt),
    }
}

fn strip_neg(s: &str) -> (bool, &str) {
    if let Some(r) = s.strip_prefix('-') {
        (true, r)
    } else {
        (false, s)
    }
}

/// "c"/"t"/"T": `[-][d.]hh:mm[:ss[.fffffff]]`
fn parse_constant(input: &str) -> Result<TimeSpan, ParseError> {
    let (neg, s) = strip_neg(input.trim());
    if s.is_empty() {
        return Err(ParseError::new(ParseErrorKind::Empty, 0, input));
    }

    let mut it = s.splitn(3, ':');
    let day_hour = it.next().unwrap(); // splitn always yields at least one item for non-empty s
    let (days_str, hours_s) = match day_hour.split_once('.') {
        Some((d, h)) => (Some(d), h),
        None => (None, day_hour),
    };

    let min = it
        .next()
        .ok_or_else(|| invalid_structure(CONSTANT_EXPECTED, 0, input))?;

    // C# ParseTime (TimeSpanParse.cs line 1384): `if (_ch == ':')` makes the
    // second colon and seconds optional — "hh:mm" is a valid "c" input.
    let (sec_s, frac_s) = match it.next() {
        None => ("", None),
        Some(sf) => match sf.split_once('.') {
            // Trailing dot with no fractional digits is accepted by C# (zero fraction).
            Some((s, "")) => (s, None),
            // C# reads exactly 7 frac digits; >7 leaves trailing chars → format error.
            Some((_s, f)) if f.len() > 7 => {
                return Err(invalid_structure(
                    CONSTANT_EXPECTED,
                    offset_of(input, f) + 7,
                    input,
                ));
            }
            Some((s, f)) => (s, Some(f)),
            None => {
                if sf.is_empty() {
                    return Err(invalid_structure(
                        CONSTANT_EXPECTED,
                        offset_of(input, sf),
                        input,
                    ));
                }
                (sf, None)
            }
        },
    };

    let mut b = Builder::new(neg, input);
    b.days = days_str;
    b.hours = Some(hours_s);
    b.minutes = Some(min);
    b.seconds = Some(sec_s);
    b.frac = frac_s;
    b.build()
}

/// "g": documented output format is `[-][d:]h:mm:ss[.FFFFFFF]`, but the C# runtime
/// `ParseExact` also accepts reduced forms: bare `d` and `h:mm` without seconds.
/// See: <https://github.com/dotnet/runtime/blob/main/src/libraries/System.Runtime/tests/System.Runtime.Tests/System/TimeSpanTests.cs>
fn parse_g(input: &str, sep: char) -> Result<TimeSpan, ParseError> {
    let (neg, s) = strip_neg(input.trim());
    if s.is_empty() {
        return Err(ParseError::new(ParseErrorKind::Empty, 0, input));
    }

    let mut it = s.split(':');
    let p0 = it.next().unwrap(); // split always yields at least one item
    let p1 = it.next();
    let p2 = it.next();
    let p3 = it.next();
    if let Some(extra) = it.next() {
        return Err(invalid_structure(
            G_EXPECTED,
            offset_of(input, extra),
            input,
        ));
    }

    let mut b = Builder::new(neg, input);

    // bare d — no colons
    if p1.is_none() {
        b.days = Some(p0);
        return b.build();
    }

    // h:mm — one colon, no seconds
    if p2.is_none() {
        if let Some(i) = p0.find(sep) {
            return Err(invalid_structure(
                G_EXPECTED,
                offset_of(input, p0) + i,
                input,
            ));
        }
        b.hours = Some(p0);
        b.minutes = p1;
        return b.build();
    }

    // p1 and p2 are both Some by the early returns above.
    // p3 signals days; seconds[.frac] is always in the rightmost component.
    let last = p3.or(p2).unwrap();
    let (sec_s, frac_s) = match last.split_once(sep) {
        Some((s, f)) => (s, Some(f)),
        None => (last, None),
    };
    if sec_s.is_empty() && frac_s.is_none() {
        return Err(invalid_structure(
            G_EXPECTED,
            offset_of(input, sec_s),
            input,
        ));
    }
    b.frac = frac_s;
    b.seconds = Some(sec_s);
    if p3.is_some() {
        // d:h:mm:ss — p0=d, p1=h, p2=mm
        b.days = Some(p0);
        b.hours = p1;
        b.minutes = p2;
    } else {
        // h:mm:ss — p0=h, p1=mm (p2 was last)
        if let Some(i) = p0.find(sep) {
            return Err(invalid_structure(
                G_EXPECTED,
                offset_of(input, p0) + i,
                input,
            ));
        }
        b.hours = Some(p0);
        b.minutes = p1;
    }

    b.build()
}

/// "G": `[-]d:hh:mm:ss.fffffff` (fractional part required)
fn parse_g_upper(input: &str, sep: char) -> Result<TimeSpan, ParseError> {
    let (neg, s) = strip_neg(input.trim());
    if s.is_empty() {
        return Err(ParseError::new(ParseErrorKind::Empty, 0, input));
    }

    let mut it = s.split(':');
    let days = it.next().unwrap(); // split always yields at least one item for non-empty s
    let h = it
        .next()
        .ok_or_else(|| invalid_structure(G_UPPER_EXPECTED, 0, input))?;
    // Missing-component errors point at the end of the (sign-stripped) input, since
    // that's where the required-but-absent component would have started.
    let min = it
        .next()
        .ok_or_else(|| invalid_structure(G_UPPER_EXPECTED, offset_of(input, s) + s.len(), input))?;
    let sec_frac = it
        .next()
        .ok_or_else(|| invalid_structure(G_UPPER_EXPECTED, offset_of(input, s) + s.len(), input))?;
    if let Some(extra) = it.next() {
        return Err(invalid_structure(
            G_UPPER_EXPECTED,
            offset_of(input, extra),
            input,
        ));
    }
    let (sec_s, frac_s) = sec_frac
        .split_once(sep)
        .ok_or_else(|| invalid_structure(G_UPPER_EXPECTED, offset_of(input, sec_frac), input))?;

    let mut b = Builder::new(neg, input);
    b.days = Some(days);
    b.hours = Some(h);
    b.minutes = Some(min);
    b.seconds = Some(sec_s);
    b.frac = Some(frac_s);
    b.build()
}

/// Custom format specifier parsing.
#[allow(clippy::too_many_lines)]
fn parse_custom(input: &str, fmt: &str) -> Result<TimeSpan, ParseError> {
    // C# TryParseExactTimeSpan (TimeSpanParse.cs line 1228): only dispatches to
    // TryParseByFormat when format.Length >= 2; a single non-standard letter is invalid.
    if fmt.chars().count() < 2 {
        const VALID: &str = "valid specifiers: d, h, m, s, f, F";
        let msg = match fmt {
            "d" | "h" | "m" | "s" | "f" | "F" => {
                format!(
                    "'{fmt}' must be prefixed with '%' when used alone (e.g. '%{fmt}'); {VALID}"
                )
            }
            _ => format!("'{fmt}' is not a known format specifier; {VALID}"),
        };
        return Err(invalid_format(&msg, fmt, 0));
    }
    let mut it = fmt.chars().peekable();
    let mut inp = input;
    let mut b = Builder::new(false, input);
    let mut fmt_pos = 0usize;

    while let Some(mut ch) = it.next() {
        let ch_start = fmt_pos;
        fmt_pos += ch.len_utf8();

        // C# TryParseByFormat: '%' consumes itself then re-enters the full switch
        // (including ParseRepeatPattern) on the next character — it is a transparent
        // pass-through. '%' at end-of-format or '%%' are both errors.
        if ch == '%' {
            ch = it.next().ok_or_else(|| {
                invalid_format(
                    "'%' at end of format must be followed by a specifier",
                    fmt,
                    ch_start,
                )
            })?;
            fmt_pos += ch.len_utf8();
            if ch == '%' {
                return Err(invalid_format(
                    "'%%' is not valid; '%' must be followed by a single specifier character",
                    fmt,
                    ch_start,
                ));
            }
        }
        match ch {
            'd' | 'h' | 'm' | 's' | 'f' | 'F' => {
                let spec_start = ch_start;
                let mut n = 1;
                while it.peek() == Some(&ch) {
                    it.next();
                    fmt_pos += ch.len_utf8();
                    n += 1;
                }
                let max = match ch {
                    'd' => 8,
                    'h' | 'm' | 's' => 2,
                    _ => 7,
                };
                if n > max {
                    return Err(invalid_format(
                        &format!("'{ch}' repeated {n} times; maximum is {max}"),
                        fmt,
                        spec_start,
                    ));
                }
                apply_spec(
                    ch,
                    n,
                    spec_start,
                    &fmt[spec_start..fmt_pos],
                    &mut inp,
                    &mut b,
                    input,
                    fmt,
                )?;
            }
            '\\' => {
                let expected = it.next().ok_or_else(|| {
                    invalid_format(
                        "'\\' in format must be followed by a character",
                        fmt,
                        ch_start,
                    )
                })?;
                fmt_pos += expected.len_utf8();
                let got = inp
                    .chars()
                    .next()
                    .ok_or_else(|| invalid_structure(fmt, offset_of(input, inp), input))?;
                if got != expected {
                    return Err(invalid_structure(fmt, offset_of(input, inp), input));
                }
                inp = &inp[got.len_utf8()..];
            }
            q @ ('\'' | '"') => {
                let mut lit = String::new();
                let mut closed = false;
                let quote_start = ch_start;
                // C# DateTimeParse.TryParseQuoteString (DateTimeParse.cs line 4600):
                // '\' inside a quoted literal escapes the next character.
                while let Some(c) = it.next() {
                    fmt_pos += c.len_utf8();
                    if c == '\\' {
                        let bs_pos = fmt_pos - c.len_utf8();
                        let escaped = it.next().ok_or_else(|| {
                            invalid_format(
                                "'\\' in format must be followed by a character",
                                fmt,
                                bs_pos,
                            )
                        })?;
                        fmt_pos += escaped.len_utf8();
                        lit.push(escaped);
                    } else if c == q {
                        closed = true;
                        break;
                    } else {
                        lit.push(c);
                    }
                }
                if !closed {
                    return Err(invalid_format(
                        &format!("unclosed {q:?} in format string"),
                        fmt,
                        quote_start,
                    ));
                }
                if !inp.starts_with(lit.as_str()) {
                    return Err(invalid_structure(fmt, offset_of(input, inp), input));
                }
                inp = &inp[lit.len()..];
            }
            _ => {
                return Err(invalid_format(
                    &format!(
                        "unrecognised character {ch:?} in format string; valid specifiers: d, h, m, s, f, F \u{2014} use '\\{ch}' to include it as a literal"
                    ),
                    fmt,
                    ch_start,
                ));
            }
        }
    }

    if !inp.is_empty() {
        return Err(invalid_structure(fmt, offset_of(input, inp), input));
    }
    b.build()
}

#[allow(clippy::too_many_arguments)]
fn apply_spec<'a>(
    ch: char,
    n: usize,
    fmt_pos: usize,
    spec: &str,
    inp: &mut &'a str,
    b: &mut Builder<'a>,
    original: &str,
    fmt: &str,
) -> Result<(), ParseError> {
    macro_rules! dup {
        ($field:expr) => {
            if $field.is_some() {
                return Err(invalid_format(
                    &format!("duplicate '{spec}' specifier in format"),
                    fmt,
                    fmt_pos,
                ));
            }
        };
    }
    match ch {
        'd' => {
            dup!(b.days);
            b.days = Some(if n == 1 {
                read_greedy_str(inp, 8, original, fmt)?
            } else {
                read_exact_str(inp, n, original, fmt)?
            });
        }
        'h' => {
            dup!(b.hours);
            b.hours = Some(if n == 1 {
                read_greedy_str(inp, 2, original, fmt)?
            } else {
                read_exact_str(inp, n, original, fmt)?
            });
        }
        'm' => {
            dup!(b.minutes);
            b.minutes = Some(if n == 1 {
                read_greedy_str(inp, 2, original, fmt)?
            } else {
                read_exact_str(inp, n, original, fmt)?
            });
        }
        's' => {
            dup!(b.seconds);
            b.seconds = Some(if n == 1 {
                read_greedy_str(inp, 2, original, fmt)?
            } else {
                read_exact_str(inp, n, original, fmt)?
            });
        }
        'f' => {
            dup!(b.frac);
            b.frac = Some(read_exact_str(inp, n, original, fmt)?);
        }
        'F' => {
            dup!(b.frac);
            // C# TryParseByFormat (TimeSpanParse.cs line 1317): ParseExactDigits
            // return value is ignored for 'F' — zero digits is valid (frac = 0).
            let count = inp.bytes().take(n).take_while(u8::is_ascii_digit).count();
            if count > 0 {
                b.frac = Some(&inp[..count]);
            }
            *inp = &inp[count..];
        }
        _ => {
            return Err(invalid_structure(fmt, offset_of(original, inp), original));
        }
    }
    Ok(())
}

fn read_greedy_str<'a>(
    inp: &mut &'a str,
    max: usize,
    original: &str,
    fmt: &str,
) -> Result<&'a str, ParseError> {
    let n = inp.bytes().take(max).take_while(u8::is_ascii_digit).count();
    if n == 0 {
        return Err(invalid_structure(fmt, offset_of(original, inp), original));
    }
    let s = &inp[..n];
    *inp = &inp[n..];
    Ok(s)
}

fn read_exact_str<'a>(
    inp: &mut &'a str,
    n: usize,
    original: &str,
    fmt: &str,
) -> Result<&'a str, ParseError> {
    let bytes = inp.as_bytes();
    if bytes.len() < n {
        return Err(invalid_structure(fmt, offset_of(original, inp), original));
    }
    // Validate on raw bytes before slicing `inp` as a `&str`: a multi-byte UTF-8
    // char in the first `n` bytes would make `n` land mid-char, which panics on
    // string slicing. Once every byte in `bytes[..n]` is confirmed ASCII, `n` is
    // guaranteed to be a valid char boundary (ASCII bytes are always one char).
    if let Some(i) = bytes[..n].iter().position(|b| !b.is_ascii_digit()) {
        return Err(ParseError::new(
            ParseErrorKind::NonDigit,
            offset_of(original, inp) + i,
            original,
        ));
    }
    let s = &inp[..n];
    *inp = &inp[n..];
    Ok(s)
}
