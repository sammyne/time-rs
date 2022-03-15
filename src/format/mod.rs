use crate::Duration;

pub fn parse_duration<S>(s: S) -> Result<Duration, String>
where
    S: AsRef<str>,
{
    s.as_ref().parse()
}
