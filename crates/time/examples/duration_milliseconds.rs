use time::Duration;

fn main() {
    let u: Duration = "1s".parse().unwrap();

    let got = format!("One second is {} milliseconds.", u.milliseconds());

    const EXPECT: &str = "One second is 1000 milliseconds.";

    assert_eq!(EXPECT, got);
}
