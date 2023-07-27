use time::Duration;

fn main() {
    let u: Duration = "1s".parse().unwrap();

    let got = format!("One second is {} microseconds.", u.microseconds());

    const EXPECT: &str = "One second is 1000000 microseconds.";

    assert_eq!(EXPECT, got);
}
