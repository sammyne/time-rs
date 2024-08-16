use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed time zone information")]
    BadData,
    #[error("file {0} too large")]
    FileSize(String),
    #[error("invalid location name")]
    InvalidLocationName,
    #[error("io: {0}")]
    Io(io::Error),
    #[error("unknown time zone {0}")]
    UnknownTimeZone(String),
}

impl Error {
    pub fn unknown_time_zone<T: Into<String>>(name: T) -> Self {
        Self::UnknownTimeZone(name.into())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
