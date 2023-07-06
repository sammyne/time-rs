use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Add;
use std::str;
use std::{ops::Mul, str::FromStr};

use lazy_static::lazy_static;

pub const NANOSECOND: Duration = Duration(1);
pub const MICROSECOND: Duration = Duration(1_000);
pub const MILLISECOND: Duration = Duration(1_000_000);
pub const SECOND: Duration = Duration(1_000_000_000);
pub const MINUTE: Duration = Duration(60_000_000_000);
pub const HOUR: Duration = Duration(3_600_000_000_000);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Duration(pub i64);

impl Duration {
    pub fn nanoseconds(&self) -> i64 {
        self.0
    }

    pub fn seconds(&self) -> f64 {
        let s = self.0 / SECOND.0;
        let ns = self.0 % SECOND.0;

        (s as f64) + (ns as f64) / 1e9
    }

    pub fn string(&self) -> String {
        self.to_string()
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Largest time is 2540400h10m10.000000000s
        if self.0 == i64::MIN {
            return write!(f, "-2562047h47m16.854775808s");
        }

        let mut buf = [0u8; 32];
        let mut w = buf.len();

        let neg = self.0 < 0;
        let mut u = self.0.abs() as u64;

        if u < SECOND.0 as u64 {
            // Special case: if duration is smaller than a second,
            // use smaller units, like 1.2ms
            w -= 1;
            buf[w] = b's';
            w -= 1;

            let prec = if u == 0 {
                return write!(f, "0s");
            } else if u < MICROSECOND.0 as u64 {
                buf[w] = b'n';
                0
            } else if u < MILLISECOND.0 as u64 {
                w -= 1;
                buf[w..(w + 2)].copy_from_slice(b"\xc2\xb5");
                3
            } else {
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
        write!(f, "{out}")
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
    type Output = i64;

    fn mul(self, rhs: Duration) -> Self::Output {
        self * rhs.0
    }
}

impl From<i64> for Duration {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let orig = s;
        let mut s = s.as_bytes();
        let mut d = 0u64;

        let err = format!("time: invalid duration {}", quote(orig));

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
            return Err(err.clone());
        }

        while !s.is_empty() {
            let mut f = 0i64;
            let mut scale = 0f64;

            if !((s[0] == b'.') || ((b'0' <= s[0]) && (s[0] <= b'9'))) {
                return Err(err.clone());
            }

            let pl = s.len();
            let mut v = {
                let (vv, ss) = leading_int(s).map_err(|_| err.clone())?;
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
                return Err(err.clone());
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
                return Err(format!("time: miss unit in duration {}", quote(orig)));
            }
            let u = str::from_utf8(&s[..i]).expect("no UTF-8 unit");
            s = &s[i..];

            let unit = if let Some(v) = UNIT_MAP.get(u) {
                *v
            } else {
                return Err(format!(
                    "time: unknown unit {} in duration {}",
                    quote(u),
                    quote(orig)
                ));
            };
            if v > (i64::MIN as u64) / unit {
                // overflow
                return Err(format!("time: invalid duration {}", quote(orig)));
            }

            v *= unit;
            if f > 0 {
                v += ((f as f64) * (unit as f64 / scale)) as u64;
                if v > (i64::MIN as u64) {
                    return Err(err.clone());
                }
            }
            d += v;
            if d > (i64::MIN as u64) {
                return Err(err.clone());
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
            return Err(err);
        }

        Ok(Self(d as i64))
    }
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

const ERR_LEADING_INT: &'static str = "time: bad [0-9]*";

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

    for j in 0..s.len() {
        let c = s[j];
        if (c < b'0') || (c > b'9') {
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
    for j in 0..s.len() {
        let c = s[j];
        if c < b'0' || c > b'9' {
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
