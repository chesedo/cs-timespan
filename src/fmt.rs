use std::fmt::Write as FmtWrite;

use crate::TimeSpan;

/// Error returned when a custom format string is invalid.
///
/// Mirrors the `FormatException` C# throws from `TimeSpan.ToString(string)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// A specifier is repeated more times than allowed
    /// (`d` > 8, `h`/`m`/`s` > 2, `f`/`F` > 7).
    RepeatTooLong,
    /// An unrecognised character appeared in the custom format string.
    UnknownSpecifier,
    /// A quoted literal (`'...'` or `"..."`) is not closed before end of format.
    UnclosedQuote,
    /// `%%` or a lone `%` at end of format string.
    InvalidPercent,
    /// A trailing `\` at end of format string with no character to escape.
    TrailingEscape,
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepeatTooLong => f.write_str("format specifier repeated too many times"),
            Self::UnknownSpecifier => f.write_str("unrecognised character in format string"),
            Self::UnclosedQuote => f.write_str("quoted literal is not closed"),
            Self::InvalidPercent => {
                f.write_str("'%' must be followed by a single specifier, not '%'")
            }
            Self::TrailingEscape => f.write_str("'\\' at end of format string"),
        }
    }
}

impl std::error::Error for FormatError {}

struct Components {
    negative: bool,
    days: u64,
    hours: u32,
    minutes: u32,
    seconds: u32,
    /// Fractional-second ticks: 0..=9_999_999 (one tick = 100 ns)
    sub_sec_ticks: u32,
}

impl Components {
    fn from_ticks(ticks: i64) -> Self {
        let abs = ticks.unsigned_abs();
        let days = abs / TimeSpan::TICKS_PER_DAY as u64;
        let r = abs % TimeSpan::TICKS_PER_DAY as u64;
        let hours = (r / TimeSpan::TICKS_PER_HOUR as u64) as u32;
        let r = r % TimeSpan::TICKS_PER_HOUR as u64;
        let minutes = (r / TimeSpan::TICKS_PER_MINUTE as u64) as u32;
        let r = r % TimeSpan::TICKS_PER_MINUTE as u64;
        let seconds = (r / TimeSpan::TICKS_PER_SECOND as u64) as u32;
        let sub_sec_ticks = (r % TimeSpan::TICKS_PER_SECOND as u64) as u32;
        Self {
            negative: ticks < 0,
            days,
            hours,
            minutes,
            seconds,
            sub_sec_ticks,
        }
    }

    /// `"c"` / `"t"` / `"T"`: `[-][d.]hh:mm:ss[.fffffff]` — culture-invariant.
    fn format_constant(&self) -> String {
        let mut out = String::new();
        if self.negative {
            out.push('-');
        }
        if self.days > 0 {
            write!(out, "{}.", self.days).unwrap(); // write! to String is infallible
        }
        write!(
            out,
            "{:02}:{:02}:{:02}",
            self.hours, self.minutes, self.seconds
        )
        .unwrap(); // write! to String is infallible
        if self.sub_sec_ticks > 0 {
            write!(out, ".{:07}", self.sub_sec_ticks).unwrap(); // write! to String is infallible
        }
        out
    }

    /// `"g"`: `[-][d:]h:mm:ss[.FFFFFFF]` — culture-sensitive decimal separator.
    fn format_general_short(&self, sep: char) -> String {
        let mut out = String::new();
        if self.negative {
            out.push('-');
        }
        if self.days > 0 {
            write!(out, "{}:", self.days).unwrap(); // write! to String is infallible
        }
        write!(
            out,
            "{}:{:02}:{:02}",
            self.hours, self.minutes, self.seconds
        )
        .unwrap(); // write! to String is infallible
        if self.sub_sec_ticks > 0 {
            // FFFFFFF — trim trailing zeros
            write!(out, "{}{}", sep, fmt_frac(self.sub_sec_ticks, 7, true)).unwrap(); // write! to String is infallible
        }
        out
    }

    /// `"G"`: `[-]d:hh:mm:ss.fffffff` — culture-sensitive decimal separator.
    fn format_general_long(&self, sep: char) -> String {
        let mut out = String::new();
        if self.negative {
            out.push('-');
        }
        write!(
            out,
            "{}:{:02}:{:02}:{:02}{}{}",
            self.days,
            self.hours,
            self.minutes,
            self.seconds,
            sep,
            fmt_frac(self.sub_sec_ticks, 7, false),
        )
        .unwrap(); // write! to String is infallible
        out
    }

    fn format_custom(&self, fmt: &str) -> Result<String, FormatError> {
        let chars: Vec<char> = fmt.chars().collect();
        let mut out = String::new();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                // `%x` — single specifier written with explicit percent prefix
                '%' if i + 1 < chars.len() => {
                    i += 1;
                    out.push_str(&self.format_specifier(chars[i], 1));
                    i += 1;
                }
                // `d`, `h`, `m`, `s`, `f`, `F` — run of identical specifier chars
                // C# FormatCustomized (TimeSpanFormat.cs): repeat > max throws FormatException.
                ch @ ('d' | 'h' | 'm' | 's' | 'f' | 'F') => {
                    let n = run_length(&chars, i, ch);
                    let max = match ch {
                        'd' => 8,
                        'h' | 'm' | 's' => 2,
                        _ => 7,
                    };
                    if n > max {
                        return Err(FormatError::RepeatTooLong);
                    }
                    out.push_str(&self.format_specifier(ch, n));
                    i += n;
                }
                // `\x` — escape: next char is a literal
                '\\' if i + 1 < chars.len() => {
                    out.push(chars[i + 1]);
                    i += 2;
                }
                // `'...'` or `"..."` — quoted literal string
                // C# ParseQuoteString (TimeSpanFormat.cs): '\' inside quotes escapes next char.
                '\'' | '"' => {
                    let q = chars[i];
                    i += 1;
                    while i < chars.len() && chars[i] != q {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            i += 1;
                        }
                        out.push(chars[i]);
                        i += 1;
                    }
                    i += 1; // skip closing quote
                }
                _ => return Err(FormatError::UnknownSpecifier),
            }
        }

        Ok(out)
    }

    /// Emit one component according to its specifier character and repeat count `n`.
    fn format_specifier(&self, ch: char, n: usize) -> String {
        match ch {
            'd' => {
                let s = self.days.to_string();
                if s.len() < n {
                    let mut out = String::new();
                    write!(out, "{:0>width$}", s, width = n).unwrap(); // write! to String is infallible
                    out
                } else {
                    s
                }
            }
            'h' => fmt_component(n, self.hours),
            'm' => fmt_component(n, self.minutes),
            's' => fmt_component(n, self.seconds),
            'f' => fmt_frac(self.sub_sec_ticks, n, false),
            'F' => fmt_frac(self.sub_sec_ticks, n, true),
            _ => unreachable!("format_specifier called with invalid char: {ch:?}"),
        }
    }
}

pub(crate) fn format_constant(ticks: i64) -> String {
    Components::from_ticks(ticks).format_constant()
}

pub(crate) fn format_timespan(ticks: i64, fmt: &str, sep: char) -> Result<String, FormatError> {
    let c = Components::from_ticks(ticks);
    Ok(match fmt {
        "c" | "t" | "T" => c.format_constant(),
        "g" => c.format_general_short(sep),
        "G" => c.format_general_long(sep),
        _ => c.format_custom(fmt)?,
    })
}

/// `n == 1` → no leading zero; `n > 1` → zero-padded to 2 digits.
fn fmt_component(n: usize, val: u32) -> String {
    let mut out = String::new();
    if n == 1 {
        write!(out, "{}", val).unwrap(); // write! to String is infallible
    } else {
        write!(out, "{:02}", val).unwrap(); // write! to String is infallible
    }
    out
}

fn fmt_frac(sub_sec_ticks: u32, n: usize, trim: bool) -> String {
    let mut full = String::new();
    write!(full, "{:07}", sub_sec_ticks).unwrap(); // write! to String is infallible
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
