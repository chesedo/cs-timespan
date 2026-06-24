use std::fmt::Write as FmtWrite;

use crate::TimeSpan;

struct Components {
    negative: bool,
    days: u64,
    hours: u32,
    minutes: u32,
    seconds: u32,
    /// Fractional-second ticks: 0..=9_999_999 (one tick = 100 ns)
    sub_sec_ticks: u32,
}

fn to_components(ticks: i64) -> Components {
    let abs = ticks.unsigned_abs();
    let days = abs / TimeSpan::TICKS_PER_DAY as u64;
    let r = abs % TimeSpan::TICKS_PER_DAY as u64;
    let hours = (r / TimeSpan::TICKS_PER_HOUR as u64) as u32;
    let r = r % TimeSpan::TICKS_PER_HOUR as u64;
    let minutes = (r / TimeSpan::TICKS_PER_MINUTE as u64) as u32;
    let r = r % TimeSpan::TICKS_PER_MINUTE as u64;
    let seconds = (r / TimeSpan::TICKS_PER_SECOND as u64) as u32;
    let sub_sec_ticks = (r % TimeSpan::TICKS_PER_SECOND as u64) as u32;
    Components {
        negative: ticks < 0,
        days,
        hours,
        minutes,
        seconds,
        sub_sec_ticks,
    }
}

pub(crate) fn format_timespan(ticks: i64, fmt: &str, sep: char) -> String {
    let c = to_components(ticks);
    match fmt {
        "c" | "t" | "T" => format_constant(&c),
        "g" => format_general_short(&c, sep),
        "G" => format_general_long(&c, sep),
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
        write!(out, "{}.", c.days).unwrap();
    }
    write!(out, "{:02}:{:02}:{:02}", c.hours, c.minutes, c.seconds).unwrap();
    if c.sub_sec_ticks > 0 {
        write!(out, ".{:07}", c.sub_sec_ticks).unwrap();
    }
    out
}

/// `"g"`: `[-][d:]h:mm:ss[.FFFFFFF]` — culture-sensitive decimal separator.
fn format_general_short(c: &Components, sep: char) -> String {
    let mut out = String::new();
    if c.negative {
        out.push('-');
    }
    if c.days > 0 {
        write!(out, "{}:", c.days).unwrap();
    }
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
    if c.negative {
        out.push('-');
    }
    write!(
        out,
        "{}:{:02}:{:02}:{:02}{}{}",
        c.days,
        c.hours,
        c.minutes,
        c.seconds,
        sep,
        fmt_frac(c.sub_sec_ticks, 7, false),
    )
    .unwrap();
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
    if n == 1 {
        write!(out, "{}", val).unwrap();
    } else {
        write!(out, "{:02}", val).unwrap();
    }
    out
}

fn fmt_frac(sub_sec_ticks: u32, n: usize, trim: bool) -> String {
    let mut full = String::new();
    write!(full, "{:07}", sub_sec_ticks).unwrap();
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
