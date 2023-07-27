use time::Duration;

fn main() {
    let h: Duration = "4h30m".parse().unwrap();

    let got = format!("I've got {:.1} hours of work left.", h.hours());

    const EXPECT: &str = "I've got 4.5 hours of work left.";
    assert_eq!(EXPECT, got);
}
