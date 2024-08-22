mod duration;
mod errors;
mod internal;
mod month;
mod sys;
mod weekday;
mod zoneinfo;

pub use duration::*;
pub use errors::*;
pub use month::*;
pub use weekday::*;
pub use zoneinfo::*;

const SECONDS_PER_MINUTE: isize = 60;
const SECONDS_PER_HOUR: isize = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: isize = 24 * SECONDS_PER_HOUR;
const SECONDS_PER_WEEK: isize = 7 * SECONDS_PER_DAY;
const DAYS_PER400_YEARS: isize = 365 * 400 + 97;
const DAYS_PER100_YEARS: isize = 365 * 100 + 24;
const DAYS_PER4_YEARS: isize = 365 * 4 + 1;

// The unsigned zero year for internal calculations.
// Must be 1 mod 400, and times before it will not compute correctly,
// but otherwise can be changed at will.
const ABSOLUTE_ZERO_YEAR: isize = -292277022399;

// The year of the zero Time.
// Assumed by the unixToInternal computation below.
const INTERNAL_YEAR: isize = 1;

// Offsets to convert between internal and absolute or Unix times.
const ABSOLUTE_TO_INTERNAL: i64 =
    ((ABSOLUTE_ZERO_YEAR - INTERNAL_YEAR) as f64 * 365.2425 * (SECONDS_PER_DAY as f64)) as i64;
const INTERNAL_TO_ABSOLUTE: i64 = -ABSOLUTE_TO_INTERNAL;

const UNIX_TO_INTERNAL: i64 = (1969 * 365 + 1969 / 4 - 1969 / 100 + 1969 / 400) * SECONDS_PER_DAY as i64;
const INTERNAL_TO_UNIX: i64 = -UNIX_TO_INTERNAL;

const WALL_TO_INTERNAL: i64 = (1884 * 365 + 1884 / 4 - 1884 / 100 + 1884 / 400) * SECONDS_PER_DAY as i64;
