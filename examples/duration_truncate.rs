use std::io::Write;

use time::Duration;

fn main() {
    let d: Duration = "1h15m30.918273645s".parse().unwrap();

    let trunc = vec![
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
    for r in trunc {
        let _ = writeln!(&mut got, "d.truncate({:>6}) = {}", r, d.truncate(r));
    }

    let got = unsafe { String::from_utf8_unchecked(got) };

    const EXPECT: &str = r#"d.truncate(   1ns) = 1h15m30.918273645s
d.truncate(   1µs) = 1h15m30.918273s
d.truncate(   1ms) = 1h15m30.918s
d.truncate(    1s) = 1h15m30s
d.truncate(    2s) = 1h15m30s
d.truncate(  1m0s) = 1h15m0s
d.truncate( 10m0s) = 1h10m0s
d.truncate(1h0m0s) = 1h0m0s
"#;

    assert_eq!(EXPECT, got);
}
