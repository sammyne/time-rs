use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed time zone information")]
    BadData,
    #[error("corrupt zip file {0}")]
    CorruptedZip(String),
    #[error("file {0} too large")]
    FileSize(String),
    #[error("invalid location name")]
    InvalidLocationName,
    #[error("io: {0}")]
    Io(io::Error),
    #[error("not found")]
    NotFound,
    #[error("unknown time zone {0}")]
    UnknownTimeZone(String),
    #[error("tzdata: {0}")]
    Tzdata(tzdata::Error),
}

impl From<tzdata::Error> for Error {
    fn from(value: tzdata::Error) -> Self {
        Self::Tzdata(value)
    }
}

impl Error {
    pub(crate) fn corrupted_zip<T: Into<String>>(name: T) -> Self {
        Self::CorruptedZip(name.into())
    }

    pub(crate) fn unknown_time_zone<T: Into<String>>(name: T) -> Self {
        Self::UnknownTimeZone(name.into())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
