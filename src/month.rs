use std::fmt::Display;

/// A Month specifies a month of the year (January = 1, ...).
#[repr(i32)]
pub enum Month {
    January,
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
