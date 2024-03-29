use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Add, Div, Neg, Sub};
use std::str;
use std::{ops::Mul, str::FromStr};

use lazy_static::lazy_static;

use crate::DurationParseError;

/// Duration of a nanosecond. There is no definition for units of Day or larger
/// to avoid confusion across daylight savings time zone transitions.
pub const NANOSECOND: Duration = Duration(1);
/// Duration of a microsecond. There is no definition for units of Day or larger
/// to avoid confusion across daylight savings time zone transitions.
pub const MICROSECOND: Duration = Duration(1_000);
/// duration of a millisecond. There is no definition for units of Day or larger
/// to avoid confusion across daylight savings time zone transitions.
pub const MILLISECOND: Duration = Duration(1_000_000);
/// Duration of a second. There is no definition for units of Day or larger
/// to avoid confusion across daylight savings time zone transitions.
pub const SECOND: Duration = Duration(1_000_000_000);
/// Duration of a minute. There is no definition for units of Day or larger
/// to avoid confusion across daylight savings time zone transitions.
pub const MINUTE: Duration = Duration(60_000_000_000);
/// Duration of an hour. There is no definition for units of Day or larger
/// to avoid confusion across daylight savings time zone transitions.
pub const HOUR: Duration = Duration(3_600_000_000_000);

/// A Duration represents the elapsed time between two instants
/// as an int64 nanosecond count. The representation limits the
/// largest representable duration to approximately 290 years.
///
/// # Example
/// Count the number of units in a Duration:
///
/// ```
#[doc = include_str!("../../examples/duration_count_units.rs")]
/// ```
///
/// Convert an integer number of units to a [Duration]:
///
/// ```
#[doc = include_str!("../../examples/duration_from_i64.rs")]
/// ```
///
/// Convert a Duration to a human-readable string.
/// ```
#[doc = include_str!("../../examples/duration_to_string.rs")]
/// ```
#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub struct Duration(pub i64);

impl Duration {
    /// Returns the absolute value of `self`.
    /// As a special case, i64::MIN is converted to i64::MAX.
    pub fn abs(&self) -> Self {
        if self.0 >= 0 {
            *self
        } else if self == &MIN_DURATION {
            MAX_DURATION
        } else {
            Self(-self.0)
        }
    }

    /// Returns the duration as a floating point number of hours.
    ///
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_hours.rs")]
    /// ```
    pub fn hours(&self) -> f64 {
        let hour = self.0 / HOUR.0;
        let nsec = self.0 % HOUR.0;

        (hour as f64) + (nsec as f64) / (60.0 * 60.0 * 1e9)
    }

    /// Returns the duration as an integer microsecond count.
    ///
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_microseconds.rs")]
    /// ```
    pub fn microseconds(&self) -> i64 {
        self.0 / 1000
    }

    /// Returns the duration as an integer millisecond count.
    ///
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_milliseconds.rs")]
    /// ```
    pub fn milliseconds(&self) -> i64 {
        self.0 / 1_000_000
    }

    /// Returns the duration as an integer nanosecond count.
    ///
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_nanoseconds.rs")]
    /// ```
    pub fn nanoseconds(&self) -> i64 {
        self.0
    }

    /// Returns the duration as a floating point number of minutes.
    ///
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_minutes.rs")]
    /// ```
    pub fn minutes(&self) -> f64 {
        let m = self.0 / MINUTE.0;
        let nsec = self.0 % MINUTE.0;

        (m as f64) + (nsec as f64) / (60.0 * 1e9)
    }

    /// Returns the result of rounding `self` to the nearest multiple of `m`.
    /// The rounding behavior for halfway values is to round away from zero.
    /// If the result exceeds the maximum (or minimum)
    /// value that can be stored in a Duration,
    /// Round returns the maximum (or minimum) duration.
    /// If m <= 0, `round` returns `self` unchanged.
    ///
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_round.rs")]
    /// ```
    pub fn round(&self, m: Self) -> Self {
        let (d, m) = (self.0, m.0);

        if m <= 0 {
            return *self;
        }

        let mut r = d % m;
        if d < 0 {
            r = -r;

            if less_than_half(r, m) {
                return Self(d + r);
            }

            if let Some(d1) = (d + r).checked_sub(m) {
                return Self(d1);
            }

            return MIN_DURATION; // overflow
        }

        if less_than_half(r, m) {
            return Self(d - r);
        }

        if let Some(d1) = (d - r).checked_add(m) {
            return Self(d1);
        }

        MAX_DURATION
    }

    /// Returns the duration as a floating point number of seconds.
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_seconds.rs")]
    /// ```
    pub fn seconds(&self) -> f64 {
        let s = self.0 / SECOND.0;
        let ns = self.0 % SECOND.0;

        (s as f64) + (ns as f64) / 1e9
    }

    /// Returns a string representing the duration in the form `72h3m0.5s`.
    /// Leading zero units are omitted. As a special case, durations less than one
    /// second format use a smaller unit (milli-, micro-, or nanoseconds) to ensure
    /// that the leading digit is non-zero. The zero duration formats as 0s.
    #[deprecated(since = "0.1.0", note = "use `to_string` instead")]
    pub fn string(&self) -> String {
        self.to_string()
    }

    /// Returns the result of rounding `self` toward zero to a multiple of `m`.
    /// If `m` <= 0, `truncate` returns `self` unchanged.
    pub fn truncate(&self, m: Self) -> Self {
        if m.0 <= 0 {
            *self
        } else {
            Self(self.0 - self.0 % m.0)
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Display for Duration {
    /// Writes a string representing the duration in the form "72h3m0.5s" to `f`.
    /// Leading zero units are omitted. As a special case, durations less than one
    /// second format use a smaller unit (milli-, micro-, or nanoseconds) to ensure
    /// that the leading digit is non-zero. The zero duration formats as 0s.
    ///
    /// # Example
    /// ```
    #[doc = include_str!("../../examples/duration_to_string.rs")]
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Largest time is 2540400h10m10.000000000s
        if self.0 == i64::MIN {
            return f.pad("-2562047h47m16.854775808s");
        }

        let mut buf = [0u8; 32];
        let mut w = buf.len();

        let neg = self.0 < 0;
        let mut u = self.0.unsigned_abs();

        if u < SECOND.0 as u64 {
            // Special case: if duration is smaller than a second,
            // use smaller units, like 1.2ms
            w -= 1;
            buf[w] = b's';
            w -= 1;

            let prec = if u == 0 {
                return f.pad("0s");
            } else if u < MICROSECOND.0 as u64 {
                // print nanoseconds
                buf[w] = b'n';
                0
            } else if u < MILLISECOND.0 as u64 {
                // print microseconds
                // U+00B5 'µ' micro sign == 0xC2 0xB5
                // Need room for two bytes.
                w -= 1;
                buf[w..(w + 2)].copy_from_slice(b"\xc2\xb5");
                3
            } else {
                // print milliseconds
                buf[w] = b'm';
                6
            };

            let (ww, uu) = fmt_frac(&mut buf[..w], u, prec);
            w = ww;
            u = uu;
            w = fmt_int(&mut buf[..w], u);
        } else {
            w -= 1;
            buf[w] = b's';

            let (ww, uu) = fmt_frac(&mut buf[..w], u, 9);
            w = ww;
            u = uu;

            w = fmt_int(&mut buf[..w], u % 60);
            u /= 60;

            // u is now integer minutes
            if u > 0 {
                w -= 1;
                buf[w] = b'm';
                w = fmt_int(&mut buf[..w], u % 60);
                u /= 60;

                // u is now integer hours
                // Stop at hours because days can be different lengths.
                if u > 0 {
                    w -= 1;
                    buf[w] = b'h';
                    w = fmt_int(&mut buf[..w], u);
                }
            }
        }

        if neg {
            w -= 1;
            buf[w] = b'-';
        }

        let out = unsafe { str::from_utf8_unchecked(&buf[w..]) };
        f.pad(out)
    }
}

impl<D> Div<D> for Duration
where
    D: Into<Duration>,
{
    type Output = i64;

    fn div(self, rhs: D) -> Self::Output {
        self.0 / rhs.into().0
    }
}

impl<D> Mul<D> for Duration
where
    D: Into<Duration>,
{
    type Output = Self;

    fn mul(self, rhs: D) -> Self::Output {
        Self(self.0 * rhs.into().0)
    }
}

impl Mul<Duration> for i64 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Self::Output {
        Duration(self * rhs.0)
    }
}

impl Neg for Duration {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            MIN_DURATION => MIN_DURATION,
            _ => Self(-self.0),
        }
    }
}

impl Sub<Duration> for Duration {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl From<i64> for Duration {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl FromStr for Duration {
    type Err = DurationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.as_bytes();
        let mut d = 0u64;

        let neg = if s.is_empty() {
            false
        } else {
            let c = s[0];
            if c == b'-' || c == b'+' {
                s = &s[1..];
                c == b'-'
            } else {
                false
            }
        };

        if s == b"0" {
            return Ok(Duration(0));
        }
        if s == b"" {
            return Err(DurationParseError::Invalid);
        }

        while !s.is_empty() {
            let mut f = 0i64;
            let mut scale = 0f64;

            if !((s[0] == b'.') || ((b'0' <= s[0]) && (s[0] <= b'9'))) {
                return Err(DurationParseError::Invalid);
            }

            let pl = s.len();
            let mut v = {
                let (vv, ss) = leading_int(s).map_err(|_| DurationParseError::Invalid)?;
                s = ss;
                vv
            };
            let pre = pl != s.len();

            let post = if !s.is_empty() && (s[0] == b'.') {
                s = &s[1..];
                let pl = s.len();
                {
                    let (ff, scale_, ss) = leading_fraction(s);
                    f = ff;
                    scale = scale_;
                    s = ss;
                }
                pl != s.len()
            } else {
                false
            };

            if !pre && !post {
                return Err(DurationParseError::Invalid);
            }

            // consume unit
            let mut i = 0;
            loop {
                if i >= s.len() {
                    break;
                }

                match s[i] {
                    b'.' | b'0'..=b'9' => break,
                    _ => {}
                }
                i += 1;
            }
            if i == 0 {
                return Err(DurationParseError::MissUnit);
            }
            let u = str::from_utf8(&s[..i]).expect("no UTF-8 unit");
            s = &s[i..];

            let unit = if let Some(v) = UNIT_MAP.get(u) {
                *v
            } else {
                return Err(DurationParseError::UnknownUnit {
                    unit: u.to_string(),
                });
            };
            if v > (i64::MIN as u64) / unit {
                // overflow
                return Err(DurationParseError::Invalid);
            }

            v *= unit;
            if f > 0 {
                v += ((f as f64) * (unit as f64 / scale)) as u64;
                if v > (i64::MIN as u64) {
                    return Err(DurationParseError::Invalid);
                }
            }
            d += v;
            if d > (i64::MIN as u64) {
                return Err(DurationParseError::Invalid);
            }
        }

        if neg {
            let mut d = d as i64;
            if d != i64::MIN {
                d = -d;
            }

            return Ok(Self(d));
        }

        if d > (i64::MAX as u64) {
            return Err(DurationParseError::Invalid);
        }

        Ok(Self(d as i64))
    }
}

/// Parses a duration string.
/// A duration string is a possibly signed sequence of
/// decimal numbers, each with optional fraction and a unit suffix,
/// such as "300ms", "-1.5h" or "2h45m".
/// Valid time units are "ns", "us" (or "µs"), "ms", "s", "m", "h".
///
/// We can also use [str::parse] instead thanks to [FromStr] implementation of [Duration].
///
/// # Example
/// ```
#[doc = include_str!("../../examples/parse_duration.rs")]
/// ```
pub fn parse_duration<S>(s: S) -> Result<Duration, DurationParseError>
where
    S: AsRef<str>,
{
    s.as_ref().parse()
}

lazy_static! {
    pub(crate) static ref UNIT_MAP: HashMap<&'static str, u64> = {
        let mut m = HashMap::new();

        m.insert("ns", NANOSECOND.0 as u64);
        m.insert("us", MICROSECOND.0 as u64);
        m.insert("µs", MICROSECOND.0 as u64); // \u{00b5}
        m.insert("μs", MICROSECOND.0 as u64); // \u{03bc}
        m.insert("ms", MILLISECOND.0 as u64);
        m.insert("s", SECOND.0 as u64);
        m.insert("m", MINUTE.0 as u64);
        m.insert("h", HOUR.0 as u64);

        m
    };
}

// private APIs
//const LOWER_HEX: &'static str = "0123456789abcdef";
const RUNE_SELF: char = 0x80 as char;
//const RUNE_ERROR: char = '\u{FFFD}';

const ERR_LEADING_INT: &str = "time: bad [0-9]*";

const MAX_DURATION: Duration = Duration(i64::MAX);

const MIN_DURATION: Duration = Duration(i64::MIN);

/// Formats the fraction of v/10**prec (e.g., ".12345") into the
/// tail of buf, omitting trailing zeros. It omits the decimal
/// point too when the fraction is 0. It returns the index where the
/// output bytes begin and the value v/10**prec.
fn fmt_frac(buf: &mut [u8], v: u64, prec: i32) -> (usize, u64) {
    let mut w = buf.len();
    let mut print = false;
    let mut v = v;

    for _i in 0..prec {
        let digit = v % 10;
        print = print || (digit != 0);
        if print {
            w -= 1;
            buf[w] = (digit as u8) + b'0';
        }
        v /= 10;
    }
    if print {
        w -= 1;
        buf[w] = b'.';
    }

    (w, v)
}

fn fmt_int(buf: &mut [u8], v: u64) -> usize {
    let mut w = buf.len();
    if v == 0 {
        w -= 1;
        buf[w] = b'0';
    } else {
        let mut v = v;
        while v > 0 {
            w -= 1;
            buf[w] = ((v % 10) as u8) + b'0';
            v /= 10;
        }
    }

    w
}

fn leading_fraction(s: &[u8]) -> (i64, f64, &[u8]) {
    let mut i = s.len();
    let mut scale = 1f64;
    let mut overflow = false;
    let mut x = 0i64;

    for (j, c) in s.iter().enumerate() {
        if !c.is_ascii_digit() {
            i = j;
            break;
        }
        if overflow {
            continue;
        }

        if x > i64::MAX / 10 {
            overflow = true;
            continue;
        }

        let y = x * 10 + ((c - b'0') as i64);
        if y < 0 {
            overflow = true;
            continue;
        }
        x = y;
        scale *= 10.0;
    }

    (x, scale, &s[i..])
}

fn leading_int(s: &[u8]) -> Result<(u64, &[u8]), String> {
    let mut i = s.len();
    let mut x = 0u64;
    for (j, c) in s.iter().enumerate() {
        if !c.is_ascii_digit() {
            i = j;
            break;
        }

        if x > (1 << 63) / 10 {
            // overflow
            return Err(ERR_LEADING_INT.to_string());
        }

        x = x * 10 + ((c - b'0') as u64);
        if x > (1 << 63) {
            // overflow
            return Err(ERR_LEADING_INT.to_string());
        }
    }

    Ok((x, &s[i..]))
}

fn less_than_half(x: i64, y: i64) -> bool {
    ((x as u64) << 1) < (y as u64)
}

pub(crate) fn quote<S>(s: S) -> String
where
    S: AsRef<str>,
{
    let s = s.as_ref();
    let mut buf = String::with_capacity(s.len() + 2);

    buf.push('"');
    for c in s.chars() {
        if (c < RUNE_SELF) && (c >= ' ') {
            match c {
                '"' | '\\' => buf.push('\\'),
                _ => {}
            }
            buf.push(c);
            continue;
        }

        buf.push_str(&c.escape_unicode().to_string());
    }

    buf.push('"');

    buf
}

#[cfg(test)]
mod tests;
