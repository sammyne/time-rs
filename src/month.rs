use std::{fmt::Display, ops::AddAssign};

/// A Month specifies a month of the year (January = 1, ...).
#[repr(i32)]
#[derive(Clone, Copy, Default)]
pub enum Month {
    #[default]
    January = 1,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    /// Returns the English name of the month ("January", "February", ...).
    #[deprecated(since = "0.1.0", note = "use `to_string` instead")]
    pub fn string(&self) -> String {
        self.to_string()
    }
}

impl AddAssign<i32> for Month {
    fn add_assign(&mut self, rhs: i32) {
        if rhs % 12 == 0 {
            return;
        }

        let v = (*self as i32 + rhs + 1) % 12;
        *self = match Self::try_from(v) {
            Ok(v) => v,
            Err(_) => unreachable!(),
        };
    }
}

impl AsRef<str> for Month {
    fn as_ref(&self) -> &str {
        match self {
            Month::January => "January",
            Month::February => "February",
            Month::March => "March",
            Month::April => "April",
            Month::May => "May",
            Month::June => "June",
            Month::July => "July",
            Month::August => "August",
            Month::September => "September",
            Month::October => "October",
            Month::November => "November",
            Month::December => "December",
        }
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.as_ref();
        f.pad(s)
    }
}

impl TryFrom<i32> for Month {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let out = match value {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            _ => return Err(()),
        };
        Ok(out)
    }
}
