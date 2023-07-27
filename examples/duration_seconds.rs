use time::Duration;

fn main() {
    let m: Duration = "1m30s".parse().unwrap();

    let got = format!("Take off in t-{:.0} seconds.", m.seconds());

    const EXPECT: &str = "Take off in t-90 seconds.";

    assert_eq!(EXPECT, got);
}
