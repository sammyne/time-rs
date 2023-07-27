use time::{Duration, HOUR, MICROSECOND, MILLISECOND, MINUTE, NANOSECOND, SECOND};

#[test]
fn hours() {
    let test_vector = vec![
        (Duration(-3600000000000), -1.0),
        (Duration(-1), -1.0 / 3600e9),
        (Duration(1), 1.0 / 3600e9),
        (Duration(3600000000000), 1.0),
        (Duration(36), 1e-11),
    ];

    for (i, (c, expect)) in test_vector.into_iter().enumerate() {
        assert_eq!(expect, c.hours(), "#{i}");
    }
}

#[test]
fn nanoseconds() {
    let test_vector = vec![
        (Duration(-1000), -1000),
        (Duration(-1), -1),
        (Duration(1), 1),
        (Duration(1000), 1000),
    ];

    for (i, (c, expect)) in test_vector.into_iter().enumerate() {
        assert_eq!(expect, c.nanoseconds(), "#{i}");
    }
}

#[test]
fn parse_duration() {
    for (i, c) in PARSE_TESTS.iter().enumerate() {
        let got: Duration = c
            .input
            .parse()
            .expect(&format!("#{} parse '{}'", i, c.input));
        assert_eq!(c.want, got, "#{} parse '{}'", i, c.input);
    }
}

#[test]
fn seconds() {
    let test_vector = vec![(Duration(300000000), 0.3)];

    for (i, (c, expect)) in test_vector.into_iter().enumerate() {
        assert_eq!(expect, c.seconds(), "#{i}");
    }
}

#[test]
fn to_string() {
    let test_vector: Vec<(&str, Duration)> = vec![
        ("0s", 0),
        ("1ns", 1 * NANOSECOND),
        ("1.1µs", 1100 * NANOSECOND),
        ("2.2ms", 2200 * MICROSECOND),
        ("3.3s", 3300 * MILLISECOND),
        ("4m5s", 4 * MINUTE + 5 * SECOND),
        ("4m5.001s", 4 * MINUTE + 5001 * MILLISECOND),
        ("5h6m7.001s", 5 * HOUR + 6 * MINUTE + 7001 * MILLISECOND),
        ("8m0.000000001s", 8 * MINUTE + 1 * NANOSECOND),
        ("2562047h47m16.854775807s", i64::MAX),
        ("-2562047h47m16.854775808s", i64::MIN),
    ]
    .into_iter()
    .map(|(s, d)| (s, d.into()))
    .collect();

    for (i, (expect, d)) in test_vector.into_iter().enumerate() {
        assert_eq!(
            expect,
            d.to_string(),
            "#{} Duration({}).to_string()",
            i,
            d.0
        );

        if d.0 > 0 {
            let expect = format!("-{expect}");
            let d = Duration(-d.0);
            assert_eq!(
                expect,
                d.to_string(),
                "#{} Duration({}).to_string()",
                i,
                -d.0
            );
        }
    }
}

lazy_static::lazy_static! {
  static ref PARSE_TESTS: Vec<ParseTest> = vec![
    // simple
    ("0", 0),
    ("5s", 5 * SECOND),
    ("30s", 30 * SECOND),
    ("1478s", 1478 * SECOND),
    // sign
    ("-5s", -5 * SECOND),
    ("+5s", 5 * SECOND),
    ("-0", 0),
    ("+0", 0),
    // decimal
    ("5.0s", 5 * SECOND),
    ("5.6s", 5*SECOND + 600*MILLISECOND),
    ("5.s", 5 * SECOND),
    (".5s", 500 * MILLISECOND),
    ("1.0s", 1 * SECOND),
    ("1.00s", 1 * SECOND),
    ("1.004s", 1*SECOND + 4*MILLISECOND),
    ("1.0040s", 1*SECOND + 4*MILLISECOND),
    ("100.00100s", 100*SECOND + 1*MILLISECOND),
    // different units
    ("10ns", 10 * NANOSECOND),
    ("11us", 11 * MICROSECOND),
    ("12µs", 12 * MICROSECOND), // U+00B5
    ("12μs", 12 * MICROSECOND), // U+03BC
    ("13ms", 13 * MILLISECOND),
    ("14s", 14 * SECOND),
    ("15m", 15 * MINUTE),
    ("16h", 16 * HOUR),
    // composite durations
    ("3h30m", 3*HOUR + 30*MINUTE),
    ("10.5s4m", 4*MINUTE + 10*SECOND + 500*MILLISECOND),
    ("-2m3.4s", -(2*MINUTE + 3*SECOND + 400*MILLISECOND)),
    ("1h2m3s4ms5us6ns", 1*HOUR + 2*MINUTE + 3*SECOND + 4*MILLISECOND + 5*MICROSECOND + 6*NANOSECOND),
    ("39h9m14.425s", 39*HOUR + 9*MINUTE + 14*SECOND + 425*MILLISECOND),
    // large value
    ("52763797000ns", 52763797000 * NANOSECOND),
    // more than 9 digits after decimal point, see https://golang.org/issue/6617
    ("0.3333333333333333333h", 20 * MINUTE),
    // 9007199254740993 = 1<<53+1 cannot be stored precisely in a float64
    ("9007199254740993ns", ((1<<53) + 1) * NANOSECOND),
    // largest duration that can be represented by int64 in nanoseconds
    ("9223372036854775807ns", i64::MAX* NANOSECOND),
    ("9223372036854775.807us", i64::MAX * NANOSECOND),
    ("9223372036s854ms775us807ns", i64::MAX * NANOSECOND),
    ("-9223372036854775808ns", i64::MIN * NANOSECOND),
    ("-9223372036854775.808us", -1 << 63 * NANOSECOND),
    ("-9223372036s854ms775us808ns", -1 << 63 * NANOSECOND),
    // largest negative value
    ("-9223372036854775808ns", -1 << 63 * NANOSECOND),
    // largest negative round trip value, see https://golang.org/issue/48629
    ("-2562047h47m16.854775808s", -1 << 63 * NANOSECOND),
    // huge string; issue 15011.
    ("0.100000000000000000000h", 6 * MINUTE),
    // This value tests the first overflow check in leadingFraction.
    ("0.830103483285477580700h", 49*MINUTE + 48*SECOND + 372539827*NANOSECOND),
  ].into_iter().map(|v| ParseTest::new(v.0,v.1)).collect();


}

struct ParseTest {
    input: &'static str,
    want: Duration,
}

impl ParseTest {
    fn new<D>(input: &'static str, want: D) -> Self
    where
        D: Into<Duration>,
    {
        Self {
            input,
            want: want.into(),
        }
    }
}
