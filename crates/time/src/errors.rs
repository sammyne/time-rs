/// Errors for parsing durations.
#[derive(thiserror::Error, Debug)]
pub enum DurationParseError {
    #[error("invalid duration")]
    Invalid,
    #[error("missing unit in duration")]
    MissUnit,
    #[error("unknown unit {}", crate::quote(.unit))]
    UnknownUnit { unit: String },
}
