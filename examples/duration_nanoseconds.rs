use time::Duration;

fn main() {
    let u: Duration = "1µs".parse().unwrap();

    let got = format!("One microsecond is {} nanoseconds.", u.nanoseconds());

    const EXPECT: &str = "One microsecond is 1000 nanoseconds.";

    assert_eq!(EXPECT, got);
}
