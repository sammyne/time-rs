use std::{io::Write, vec};

use time::Duration;

fn main() {
    let d: Duration = "1h15m30.918273645s".parse().unwrap();

    let round = vec![
        time::NANOSECOND,
        time::MICROSECOND,
        time::MILLISECOND,
        time::SECOND,
        2 * time::SECOND,
        time::MINUTE,
        10 * time::MINUTE,
        time::HOUR,
    ];

    let mut got = vec![];
    for r in round {
        let _ = writeln!(&mut got, "d.round({:>6}) = {}", r.to_string(), d.round(r));
    }

    let got = unsafe { String::from_utf8_unchecked(got) };

    const EXPECT: &str = r#"d.round(   1ns) = 1h15m30.918273645s
d.round(   1µs) = 1h15m30.918274s
d.round(   1ms) = 1h15m30.918s
d.round(    1s) = 1h15m31s
d.round(    2s) = 1h15m30s
d.round(  1m0s) = 1h16m0s
d.round( 10m0s) = 1h20m0s
d.round(1h0m0s) = 1h0m0s
"#;

    assert_eq!(EXPECT, got);
}
