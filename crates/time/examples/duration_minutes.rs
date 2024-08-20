use time::Duration;

fn main() {
    let m: Duration = "1h30m".parse().unwrap();

    let got = format!("The movie is {:.0} minutes long.", m.minutes());

    const EXPECT: &str = "The movie is 90 minutes long.";
    assert_eq!(EXPECT, got);
}
