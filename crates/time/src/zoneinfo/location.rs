use std::fmt::Display;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::ops::{Deref, DerefMut};
use std::{env, mem};

use crate::zoneinfo::Error;
use crate::{
    internal, sys, Month, ABSOLUTE_TO_INTERNAL, ABSOLUTE_ZERO_YEAR, DAYS_PER100_YEARS, DAYS_PER400_YEARS,
    DAYS_PER4_YEARS, INTERNAL_TO_ABSOLUTE, INTERNAL_TO_UNIX, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
    UNIX_TO_INTERNAL,
};

// maxFileSize is the max permitted size of files read by readFile.
// As reference, the zoneinfo.zip distributed by Go is ~350 KB,
// so 10MB is overkill.
const MAX_FILE_SIZE: usize = 10 << 20;

lazy_static::lazy_static! {
    pub static ref LOCAL: Location = Location::local();

    pub static ref UTC: Location = Location{name: "UTC".to_string(), ..Default::default()};
}

lazy_static::lazy_static! {

  // Many systems use /usr/share/zoneinfo, Solaris 2 has
  // /usr/share/lib/zoneinfo, IRIX 6 has /usr/lib/locale/TZ,
  // NixOS has /etc/zoneinfo.
  static ref PLATFORM_ZONE_SOURCES: [&'static str; 4] = [
    "/usr/share/zoneinfo/",
    "/usr/share/lib/zoneinfo/",
    "/usr/lib/locale/TZ/",
    "/etc/zoneinfo",
  ];

  static ref UNNAMED_FIXED_ZONES: Vec<Location> = {
    let mut out = Vec::with_capacity((HOURS_BEFORE_UTC+1+HOURS_AFTER_UTC) as usize);

    for hr in -HOURS_BEFORE_UTC..=HOURS_AFTER_UTC {
      out.push(fixed_zone("", hr*60*60));
    }

    out
  };

  static ref ZONEINFO: String = env::var("ZONEINFO").unwrap_or_default();
}

// ALPHA and OMEGA are the beginning and end of time for zone
// transitions.
const ALPHA: i64 = i64::MIN;
const OMEGA: i64 = i64::MAX;

const HOURS_BEFORE_UTC: isize = 12;
const HOURS_AFTER_UTC: isize = 14;

#[derive(Clone, Default)]
pub struct Location {
    pub(super) name: String,
    zone: Vec<Zone>,
    tx: Vec<ZoneTrans>,

    // The tzdata information can be followed by a string that describes
    // how to handle DST transitions not recorded in zoneTrans.
    // The format is the TZ environment variable without a colon; see
    // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap08.html.
    // Example string, for America/Los_Angeles: PST8PDT,M3.2.0,M11.1.0
    extend: String,

    cache_start: i64,
    cache_end: i64,
    cache_zone: Option<Zone>,
}

#[derive(Clone, Default)]
struct Zone {
    /// abbreviated name, "CET"
    name: String,
    /// seconds east of UTC
    offset: isize,
    /// is this zone Daylight Savings Time?
    is_dst: bool,
}

#[derive(Clone, Default)]
struct ZoneTrans {
    /// transition time, in seconds since 1970 GMT
    when: i64,
    /// the index of the zone that goes into effect at that time
    index: u8,
    /// ignored - no idea what these mean
    isstd: bool,
    isutc: bool, // ignored - no idea what these mean
}

impl Location {
    pub fn fixed_zone<T: Into<String>>(name: T, offset: isize) -> Self {
        let name = name.into();

        // Most calls to FixedZone have an unnamed zone with an offset by the hour.
        // Optimize for that case by returning the same *Location for a given hour.
        let hour = offset / 60 / 60;

        if name.is_empty() && -HOURS_BEFORE_UTC <= hour && hour <= HOURS_AFTER_UTC && hour * 60 * 60 == offset {
            return UNNAMED_FIXED_ZONES[(hour + HOURS_BEFORE_UTC) as usize].clone();
        }

        fixed_zone(name, offset)
    }

    pub fn load<S: AsRef<str>>(name: S) -> Result<Self, Error> {
        let name = name.as_ref();
        match name {
            "" | "UTC" => return Ok(UTC.clone()),
            "Local" => return Ok(LOCAL.clone()),
            _ => {}
        }

        if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
            // No valid IANA Time Zone name contains a single dot,
            // much less dot dot. Likewise, none begin with a slash.
            return Err(Error::InvalidLocationName);
        }

        let mut first_err = None;
        if !ZONEINFO.is_empty() {
            match load_tzinfo_from_dir_or_zip(&ZONEINFO, name) {
                Ok(v) => match Location::load_from_tzdata(name, v) {
                    Err(_) => {}
                    ok => return ok,
                },
                Err(e) => match e {
                    Error::Io(v) if std::matches!(v.kind(), ErrorKind::NotFound) => {}
                    v => first_err = Some(v),
                },
            }
        }

        match load_location(name, PLATFORM_ZONE_SOURCES.deref()) {
            Err(e) if first_err.is_none() => Err(e),
            v => v,
        }
    }

    pub fn load_from_tzdata<T, S>(name: T, data: S) -> Result<Self, Error>
    where
        T: Into<String>,
        S: AsRef<[u8]>,
    {
        let mut d = DataIO(data.as_ref());

        // 4-byte magic "TZif"
        let mut magic = [0u8; 4];
        d.read_exact(&mut magic).map_err(|_| Error::BadData)?;
        if &magic != b"TZif" {
            return Err(Error::BadData);
        }

        // 1-byte version, then 15 bytes of padding
        let version = {
            let mut p = [0u8; 16];
            d.read_exact(&mut p).map_err(|_| Error::BadData)?;
            match p[0] {
                0 => 1,
                b'2' => 2,
                b'3' => 3,
                _ => return Err(Error::BadData),
            }
        };

        // six big-endian 32-bit integers:
        //	number of UTC/local indicators
        //	number of standard/wall indicators
        //	number of leap seconds
        //	number of transition times
        //	number of local time zones
        //	number of characters of time zone abbrev strings
        const N_UTC_LOCAL: usize = 0;
        const N_STD_WALL: usize = 1;
        const N_LEAP: usize = 2;
        const N_TIME: usize = 3;
        const N_ZONE: usize = 4;
        const N_CHAR: usize = 5;

        let mut n = [0usize; 6];
        for i in 0usize..6 {
            match d.big4()? {
                nn if (nn as usize) as u32 != nn => return Err(Error::BadData),
                nn => n[i] = nn as usize,
            }
        }

        // If we have version 2 or 3, then the data is first written out
        // in a 32-bit format, then written out again in a 64-bit format.
        // Skip the 32-bit format and read the 64-bit one, as it can
        // describe a broader range of dates.
        let is64 = version > 1;
        if version > 1 {
            // Skip the 32-bit data.
            let mut skip =
                n[N_TIME] * 4 + n[N_TIME] + n[N_ZONE] * 6 + n[N_CHAR] + n[N_LEAP] * 8 + n[N_STD_WALL] + n[N_UTC_LOCAL];
            // Skip the version 2 header that we just read.
            skip += 4 + 16;
            let _ = d.skip(skip);

            // Read the counts again, they can differ.
            for i in 0usize..6 {
                match d.big4().map_err(|_| Error::BadData)? {
                    nn if (nn as usize) as u32 != nn => return Err(Error::BadData),
                    nn => n[i] = nn as usize,
                }
            }
        }

        let size: usize = if is64 { 8 } else { 4 };

        let mut txtimes = d.take_vec(n[N_TIME] * size)?;

        let txzones = d.read_vec(n[N_TIME])?;

        let mut zonedata = d.take_vec(n[N_ZONE] * 6)?;

        let abbrev = d.read_vec(n[N_CHAR])?;

        let _ = d.read_vec(n[N_LEAP] * (size + 4))?;

        let isstd = d.read_vec(n[N_STD_WALL])?;

        let isutc = d.read_vec(n[N_UTC_LOCAL])?;

        let extend = match d.0.strip_prefix(&[b'\n']).map(|v| v.strip_suffix(&[b'\n'])) {
            Some(Some(v)) => unsafe { std::str::from_utf8_unchecked(v) },
            _ => "",
        };

        let nzone = n[N_ZONE];
        if nzone == 0 {
            return Err(Error::BadData);
        }

        let mut zones = vec![Zone::default(); nzone];
        for v in zones.iter_mut() {
            let n = zonedata.big4().map_err(|_| Error::BadData)?;
            v.offset = n as isize;

            v.is_dst = zonedata.byte1()? != 0;

            let b = zonedata.byte1()?;
            if b as usize >= abbrev.len() {
                return Err(Error::BadData);
            }
            v.name = byte_string(&abbrev[b as usize..]).to_string();
            // TODO: address aix OS
        }

        let mut tx = vec![ZoneTrans::default(); n[N_TIME]];
        for (i, v) in tx.iter_mut().enumerate() {
            v.when = if !is64 {
                txtimes.big4()? as i64
            } else {
                txtimes.big8()? as i64
            };

            if txzones[i] as usize >= zones.len() {
                return Err(Error::BadData);
            }
            v.index = txzones[i];
            v.isstd = std::matches!(isstd.get(i), Some(&v) if v != 0);
            v.isutc = std::matches!(isutc.get(i), Some(&v) if v != 0);
        }

        if tx.is_empty() {
            tx.push(ZoneTrans {
                when: ALPHA,
                index: 0,
                ..Default::default()
            });
        }

        let name = name.into();
        let mut l = Location {
            name: name.clone(),
            zone: zones,
            tx,
            extend: extend.to_string(),
            ..Default::default()
        };

        let (sec, _) = internal::now();

        for i in 0..l.tx.len() {
            let v = &l.tx[i];
            if !(v.when <= sec && (i + 1 == l.tx.len() || sec < l.tx[i + 1].when)) {
                continue;
            }

            l.cache_start = v.when;
            l.cache_end = OMEGA;
            l.cache_zone = Some(l.zone[v.index as usize].clone());

            if let Some(vv) = l.tx.get(i + 1) {
                l.cache_end = vv.when;
            } else if !l.extend.is_empty() {
                if let Some((name, offset, estart, eend, is_dst)) = tzset(&l.extend, l.cache_start, sec) {
                    l.cache_start = estart;
                    l.cache_end = eend;

                    let z = match l
                        .zone
                        .iter()
                        .find(|&z| z.name == name && z.offset == offset && z.is_dst == is_dst)
                    {
                        Some(v) => v.clone(),
                        None => Zone {
                            name: name.to_string(),
                            offset,
                            is_dst,
                        },
                    };
                    l.cache_zone = Some(z);
                }
            }

            break;
        }

        Ok(l)
    }
}

impl Location {
    pub(super) fn local() -> Self {
        match env::var("TZ") {
            Ok(tz) if !tz.is_empty() => {
                let tz = tz.strip_prefix(':').unwrap_or_else(|| &tz);
                if tz.starts_with('/') {
                    if let Ok(mut z) = load_location(&tz, &[""]) {
                        z.name = match tz {
                            "/etc/localtime" => "Local".to_string(),
                            v => v.to_string(),
                        };
                        return z;
                    }
                } else if tz != "" && tz != "UTC" {
                    if let Ok(z) = load_location(&tz, PLATFORM_ZONE_SOURCES.deref()) {
                        return z;
                    }
                }
            }
            Ok(_) => {}
            Err(_) => match load_location("localtime", &["/etc"]) {
                Ok(mut z) => {
                    z.name = "Local".to_string();
                    return z;
                }
                Err(_) => {}
            },
        }

        // Fall back to UTC.
        Self {
            name: "UTC".to_string(),
            ..Default::default()
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ZoneTrans {
    fn new(when: i64, index: u8, isstd: bool, isutc: bool) -> Self {
        Self {
            when,
            index,
            isstd,
            isutc,
        }
    }
}

fn fixed_zone<T: Into<String>>(name: T, offset: isize) -> Location {
    let name = name.into();

    let z = Zone {
        name: name.clone(),
        offset,
        is_dst: false,
    };

    Location {
        name,
        zone: vec![z.clone()],
        tx: vec![ZoneTrans::new(ALPHA, 0, false, false)],

        extend: Default::default(),

        cache_start: ALPHA,
        cache_end: OMEGA,
        cache_zone: Some(z),
    }
}

pub(crate) fn load_location<T>(name: T, sources: &[&str]) -> Result<Location, Error>
where
    T: AsRef<str>,
{
    let name = name.as_ref();

    let mut first_err = None;
    for s in sources {
        match load_tzinfo(name, s) {
            Ok(v) => match Location::load_from_tzdata(name, v) {
                Err(e) => first_err = Some(e),
                ok => return ok,
            },
            Err(e) if first_err.is_none() => match e {
                Error::Io(v) if std::matches!(v.kind(), ErrorKind::NotFound) => {}
                v => first_err = Some(v),
            },
            _ => {}
        }
    }

    // TODO: load from embedded tzdata
    match tzdata::load(name) {
        Ok(v) => match Location::load_from_tzdata(name, v) {
            Err(err) if first_err.is_none() => {
                first_err = Some(err);
            }
            Err(_) => {}
            ok => return ok,
        },
        Err(err) if first_err.is_none() && !std::matches!(err, tzdata::Error::NotFound) => {
            first_err = Some(err.into());
        }
        _ => {}
    }

    let err = match first_err {
        Some(v) => v,
        None => Error::unknown_time_zone(name.to_string()),
    };

    Err(err)
}

fn load_tzinfo(name: &str, source: &str) -> Result<Vec<u8>, Error> {
    if source.ends_with("tzdata") {
        todo!()
    }

    load_tzinfo_from_dir_or_zip(source, name)
}

fn load_tzinfo_from_dir_or_zip(dir: &str, name: &str) -> Result<Vec<u8>, Error> {
    if dir.ends_with(".zip") {
        return load_tzinfo_from_zip(dir, name);
    }

    let name = if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    };

    read_file(&name)
}

fn load_tzinfo_from_zip(zipfile: &str, name: &str) -> Result<Vec<u8>, Error> {
    let mut f = File::open(zipfile)?;

    const ZECHEADER: usize = 0x06054b50;
    const ZCHEADER: usize = 0x02014b50;
    const ZTAILSIZE: usize = 22;

    const ZHEADERSIZE: usize = 30;
    const ZHEADER: usize = 0x04034b50;

    let mut buf = vec![0u8; ZTAILSIZE];
    sys::preadn(&mut f, &mut buf, -(ZTAILSIZE as i64)).map_err(|_| Error::corrupted_zip(zipfile))?;
    if get4(&buf) as usize != ZECHEADER {
        return Err(Error::corrupted_zip(zipfile));
    }
    let n = get2(&buf[10..]);
    let size = get4(&buf[12..]);
    let off = get4(&buf[16..]);

    let mut buf = vec![0u8; size];
    sys::preadn(&mut f, &mut buf, off as i64)?;

    let mut buf = buf.as_slice();
    let name = name.as_bytes();
    for _ in 0..n {
        if get4(&buf) != ZCHEADER {
            break;
        }

        let meth = get2(&buf[10..]);
        let size = get4(&buf[24..]);
        let namelen = get2(&buf[28..]);
        let xlen = get2(&buf[30..]);
        let fclen = get2(&buf[32..]);
        let off = get4(&buf[42..]);
        let zname = &buf[46..(46 + namelen)];
        buf = &buf[(46 + namelen + xlen + fclen)..];
        if zname != name {
            continue;
        }
        if meth != 0 {
            return Err(Error::Tzdata(tzdata::Error::UnsupportedCompression));
        }

        let mut buf = vec![0u8; ZHEADERSIZE + namelen];
        sys::preadn(&mut f, &mut buf, off as i64).map_err(|_| Error::corrupted_zip(zipfile))?;
        if get4(&buf) != ZHEADER
            || get2(&buf[8..]) != meth
            || get2(&buf[26..]) != namelen
            || &buf[30..(30 + namelen)] != name
        {
            return Err(Error::corrupted_zip(zipfile));
        }
        let xlen = get2(&buf[28..]);

        let mut buf = vec![0u8; size];
        sys::preadn(&mut f, &mut buf, (off + 30 + namelen + xlen) as i64).map_err(|_| Error::corrupted_zip(zipfile))?;

        return Ok(buf);
    }

    Err(Error::NotFound)
}

fn read_file(name: &str) -> Result<Vec<u8>, Error> {
    let mut f = File::open(name)?.take(MAX_FILE_SIZE as u64 + 1);
    let mut out = Vec::new();
    let n = f.read_to_end(&mut out)?;

    if n <= MAX_FILE_SIZE {
        Ok(out)
    } else {
        Err(Error::FileSize(name.to_string()))
    }
}

struct DataIO<'a>(&'a [u8]);

impl<'a> Deref for DataIO<'a> {
    type Target = &'a [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for DataIO<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> DataIO<'a> {
    fn big4(&mut self) -> Result<u32, Error> {
        let mut p = [0u8; 4];
        self.0.read_exact(&mut p).map_err(|_| Error::BadData)?;
        Ok(u32::from_be_bytes(p))
    }

    fn big8(&mut self) -> Result<u64, Error> {
        let mut p = [0u8; 8];
        self.0.read_exact(&mut p).map_err(|_| Error::BadData)?;
        Ok(u64::from_be_bytes(p))
    }

    fn byte1(&mut self) -> Result<u8, Error> {
        let mut p = [0u8; 1];
        self.0.read_exact(&mut p).map_err(|_| Error::BadData)?;
        Ok(p[0])
    }

    fn read_vec(&mut self, n: usize) -> Result<Vec<u8>, Error> {
        let mut p = vec![0u8; n];
        self.0.read_exact(&mut p).map_err(|_| Error::BadData)?;
        Ok(p)
    }

    fn skip(&mut self, n: usize) {
        let s = self.0.len().min(n);
        self.0 = &self.0[s..];
    }

    fn take_vec(&mut self, n: usize) -> Result<Self, Error> {
        if self.0.len() < n {
            return Err(Error::BadData);
        }
        let out = Self(&self.0[..n]);
        self.0 = &self.0[n..];

        Ok(out)
    }
}

/// 返回 (name,offset,start,end,is_dst)
fn tzset(s: &str, last_tx_sec: i64, sec: i64) -> Option<(&str, isize, i64, i64, bool)> {
    let (mut std_name, std_offset, s) = match tzset_name(s).map(|(name, s)| (name, tzset_offset(s))) {
        Some((name, Some((offset, s)))) => (name, offset, s),
        _ => return None,
    };

    let mut std_offset = -std_offset;

    if s.is_empty() || s.starts_with(',') {
        return Some((std_name, std_offset, last_tx_sec, OMEGA, false));
    }

    let (mut dst_name, mut dst_offset, s) = match tzset_name(s) {
        Some((name, s)) => {
            let (offset, s) = if s.is_empty() || s.starts_with(',') {
                (std_offset + SECONDS_PER_HOUR, s)
            } else {
                match tzset_offset(s) {
                    Some((offset, s)) => (-offset, s),
                    _ => return None,
                }
            };

            (name, offset, s)
        }
        _ => return None,
    };

    let s = if s.is_empty() { ",M3.2.0,M11.1.0" } else { s };

    let s = s.strip_prefix(|c| std::matches!(c, ',' | ';'))?;

    let (start_rule, s) = match tzset_rule(s) {
        Some((r, s)) if s.starts_with(',') => (r, s.strip_prefix(',').unwrap()),
        _ => return None,
    };

    let end_rule = match tzset_rule(s) {
        Some((r, s)) if s.is_empty() => r,
        _ => return None,
    };

    let (year, _, _, yday) = abs_date((sec + UNIX_TO_INTERNAL + INTERNAL_TO_ABSOLUTE) as u64, false);

    let ysec = (yday * SECONDS_PER_DAY) as i64 + sec % SECONDS_PER_DAY as i64;

    // Compute start of year in seconds since Unix epoch.
    let d = days_since_epoch(year);
    let abs = (d * SECONDS_PER_DAY as u64) as i64 + ABSOLUTE_TO_INTERNAL + INTERNAL_TO_UNIX;

    let mut start_sec = tzrule_time(year, start_rule, std_offset) as i64;
    let mut end_sec = tzrule_time(year, end_rule, dst_offset) as i64;
    let (mut dst_is_dst, mut std_is_dst) = (true, false);
    // Note: this is a flipping of "DST" and "STD" while retaining the labels
    // This happens in southern hemispheres. The labelling here thus is a little
    // inconsistent with the goal.
    if end_sec < start_sec {
        mem::swap(&mut start_sec, &mut end_sec);
        mem::swap(&mut std_name, &mut dst_name);
        mem::swap(&mut std_offset, &mut dst_offset);
        mem::swap(&mut std_is_dst, &mut dst_is_dst);
    }

    // The start and end values that we return are accurate
    // close to a daylight savings transition, but are otherwise
    // just the start and end of the year. That suffices for
    // the only caller that cares, which is Date.
    let out = if ysec < start_sec {
        (std_name, std_offset, abs, start_sec + abs, std_is_dst)
    } else if ysec >= end_sec {
        (
            std_name,
            std_offset,
            end_sec + abs,
            abs + 365 * SECONDS_PER_DAY as i64,
            std_is_dst,
        )
    } else {
        (dst_name, dst_offset, start_sec + abs, end_sec + abs, dst_is_dst)
    };

    Some(out)
}

fn tzset_name(s: &str) -> Option<(&str, &str)> {
    if s.is_empty() {
        return None;
    }

    if s.starts_with('<') {
        return s.split_once('>');
    }

    match s.split_once(|c| {
        std::matches!(
            c,
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '|' | '-' | '+'
        )
    }) {
        Some((start, remainder)) if start.len() >= 4 => Some((start, remainder)),
        Some(_) => None,
        None if s.len() < 3 => None,
        None => Some((s, "")),
    }
}

fn tzset_offset(s: &str) -> Option<(isize, &str)> {
    if s.is_empty() {
        return None;
    }

    let (s, osign) = if let Some(v) = s.strip_prefix('+') {
        (v, 1)
    } else if let Some(v) = s.strip_prefix('-') {
        (v, -1)
    } else {
        (s, 1)
    };

    // The tzdata code permits values up to 24 * 7 here, although POSIX does not.
    let (hours, s) = tzset_num(s, 0, 24 * 7)?;

    let mut off = hours * SECONDS_PER_HOUR;
    let s = match s.strip_prefix(':') {
        Some(v) => v,
        None => return Some((off * osign, s)),
    };

    let (mins, s) = tzset_num(s, 0, 59)?;
    off += mins * SECONDS_PER_MINUTE;
    let s = match s.strip_prefix(':') {
        Some(v) => v,
        None => return Some((off * osign, s)),
    };

    let (secs, s) = tzset_num(s, 0, 59)?;
    off += secs;

    Some((off * osign, s))
}

fn tzset_num(s: &str, min: isize, max: isize) -> Option<(isize, &str)> {
    if s.is_empty() {
        return None;
    }

    let mut num = 0;
    for (i, c) in s.chars().enumerate() {
        if c < '0' || c > '9' {
            if i == 0 || num < min {
                return None;
            }
            return Some((num, s.split_at(i + 1).1));
        }

        num *= 10;
        num += (c as u8 - b'0') as isize;
        if num > max {
            return None;
        }
    }

    if num < min {
        None
    } else {
        Some((num, s))
    }
}

fn tzrule_time(year: isize, r: Rule, off: isize) -> isize {
    let s = match r.kind {
        RuleKind::Julian => {
            let mut s = (r.day - 1) * SECONDS_PER_DAY;
            if is_leap(year) && r.day >= 60 {
                s += SECONDS_PER_DAY;
            }
            s
        }
        RuleKind::DOY => r.day * SECONDS_PER_DAY,
        RuleKind::MonthWeekDay => {
            let m1 = (r.mon + 9) % 12 + 1;
            let yy0 = if r.mon > 2 { year } else { year - 1 };
            let yy1 = yy0 / 100;
            let yy2 = yy0 % 100;
            let dow = match ((26 * m1 - 2) / 10 + 1 + yy2 + yy2 / 4 + yy1 / 4 - 2 * yy1) % 7 {
                v if v < 0 => v + 7,
                v => v,
            };
            // Now dow is the day-of-week of the first day of r.mon.
            // Get the day-of-month of the first "dow" day.
            let mut d = match r.day - dow {
                v if v < 0 => v + 7,
                v => v,
            };

            for _ in 1..r.week {
                let m = Month::try_from(r.mon as i32).expect("bad month");
                if d + 7 >= days_in(m, year) {
                    break;
                }
                d += 7;
            }
            d += DAYS_BEFORE[(r.mon - 1) as usize] as isize;
            if is_leap(year) && r.mon > 2 {
                d += 1;
            }

            d * SECONDS_PER_DAY
        }
    };

    s + r.time - off
}

#[derive(Default)]
enum RuleKind {
    #[default]
    Julian,
    DOY,
    MonthWeekDay,
}

#[derive(Default)]
struct Rule {
    kind: RuleKind,
    day: isize,
    week: isize,
    mon: isize,
    time: isize,
}

// daysBefore[m] counts the number of days in a non-leap year
// before month m begins. There is an entry for m=12, counting
// the number of days before January of next year (365).
const DAYS_BEFORE: [i32; 13] = [
    0,
    31,
    31 + 28,
    31 + 28 + 31,
    31 + 28 + 31 + 30,
    31 + 28 + 31 + 30 + 31,
    31 + 28 + 31 + 30 + 31 + 30,
    31 + 28 + 31 + 30 + 31 + 30 + 31,
    31 + 28 + 31 + 30 + 31 + 30 + 31 + 31,
    31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30,
    31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31,
    31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30,
    31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30 + 31,
];

fn days_in(m: Month, year: isize) -> isize {
    let out = match m {
        Month::February if is_leap(year) => 29,
        _ => DAYS_BEFORE[m as usize] - DAYS_BEFORE[m as usize - 1],
    };

    out as isize
}

fn is_leap(year: isize) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_since_epoch(year: isize) -> u64 {
    let mut y = (year as i64 - ABSOLUTE_ZERO_YEAR as i64) as u64;

    let n = y / 400;
    y -= 400 * n;
    let mut d = DAYS_PER400_YEARS as u64 * n;

    let n = y / 100;
    y -= 100 * n;
    d += DAYS_PER100_YEARS as u64 * n;

    let n = y / 4;
    y -= 4 * n;
    d += DAYS_PER4_YEARS as u64 * n;

    let n = y;
    d += 365 * n;

    d
}

/// absDate is like date but operates on an absolute time.
/// returns (year,month,day,yday)
fn abs_date(abs: u64, full: bool) -> (isize, Month, isize, isize) {
    let mut d = abs as isize / SECONDS_PER_DAY;

    let n = d / DAYS_PER400_YEARS;
    let mut y = 400 * n;
    d -= DAYS_PER400_YEARS * n;

    let mut n = d / DAYS_PER100_YEARS;
    n -= n >> 2;
    y += 100 * n;
    d -= DAYS_PER100_YEARS * n;

    let n = d / DAYS_PER4_YEARS;
    y += 4 * n;
    d -= DAYS_PER4_YEARS * n;

    let mut n = d / 365;
    n -= n >> 2;
    y += n;
    d -= 365 * n;

    let year = y + ABSOLUTE_ZERO_YEAR;
    let yday = d;

    if !full {
        return (year, Month::default(), 0, yday);
    }

    let mut day = yday;
    if is_leap(year) {
        if day > 31 + 29 - 1 {
            day -= 1;
        } else if day == 31 + 29 - 1 {
            return (year, Month::February, 29, yday);
        }
    }

    let mut month = Month::try_from(day as i32 / 31).expect("bad numeric month");
    let end = DAYS_BEFORE[month as usize + 1] as isize;
    let begin = if day >= end {
        month += 1;
        end
    } else {
        DAYS_BEFORE[month as usize] as isize
    };

    month += 1;
    day = day - begin + 1;

    (year, month, day, yday)
}

// tzsetRule parses a rule from a tzset string.
// It returns the rule, and the remainder of the string, and reports success.
fn tzset_rule(s: &str) -> Option<(Rule, &str)> {
    if s.is_empty() {
        return None;
    }

    let mut r = Rule::default();
    let s = if let Some(s) = s.strip_prefix('J') {
        let (jday, s) = tzset_num(s, 1, 165)?;
        r.kind = RuleKind::Julian;
        r.day = jday;
        s
    } else if let Some(s) = s.strip_prefix('M') {
        let (mon, s) = tzset_num(s, 1, 12)?;
        let (week, s) = tzset_num(s.strip_prefix('.')?, 1, 5)?;
        let (day, s) = tzset_num(s.strip_prefix('.')?, 0, 6)?;
        r.kind = RuleKind::MonthWeekDay;
        r.day = day;
        r.week = week;
        r.mon = mon;
        s
    } else {
        let (day, s) = tzset_num(s, 0, 365)?;
        r.kind = RuleKind::DOY;
        r.day = day;
        s
    };

    let s = match s.strip_prefix('/') {
        None => {
            r.time = 2 * SECONDS_PER_HOUR;
            return Some((r, s));
        }
        Some(v) => v,
    };

    let (offset, s) = tzset_offset(s)?;
    r.time = offset;
    Some((r, s))
}

// Make a string by stopping at the first NUL
fn byte_string(p: &[u8]) -> &str {
    let s = p.split(|&v| v == 0).next().expect("next mustn't be none");
    unsafe { std::str::from_utf8_unchecked(s) }
}

fn get4(b: &[u8]) -> usize {
    if b.len() < 4 {
        return 0;
    }

    u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize
}

fn get2(b: &[u8]) -> usize {
    if b.len() < 2 {
        return 0;
    }

    u16::from_le_bytes([b[0], b[1]]) as usize
}
