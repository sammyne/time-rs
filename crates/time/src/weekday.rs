use std::fmt::Display;

/// A Weekday specifies a day of the week.
#[repr(i32)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    /// Returns the English name of the day ("Sunday", "Monday", ...).
    #[deprecated(since = "0.1.0", note = "use `to_string` instead")]
    pub fn string(&self) -> String {
        self.to_string()
    }
}

impl AsRef<str> for Weekday {
    fn as_ref(&self) -> &str {
        match self {
            Weekday::Sunday => "Sunday",
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
        }
    }
}

impl Display for Weekday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.as_ref();
        f.pad(s)
    }
}
