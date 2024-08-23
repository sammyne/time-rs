mod unix;

mod export;
mod internal;

use export::{Defer, Testing};

use crate::zoneinfo::{Rule, RuleKind};
use crate::{zoneinfo, Error, Location, LOCAL, UTC};

#[test]
fn bad_location_err_msg() {
    let loc: &str = "Asia/SomethingNotExist";

    match Location::load(loc) {
        Ok(_) => panic!("unexpected ok"),
        Err(Error::UnknownTimeZone(got)) => assert_eq!(loc, got, "unexpected err msg"),
        Err(err) => panic!("unexpected error: {err}"),
    }
}

#[test]
#[ignore = "todo"]
fn early_location() {}

#[test]
fn env_var_usage() {
    const TEST_ZONEINFO: &str = "foo.zip";
    const ENV: &str = "ZONEINFO";

    let mut t = Testing::default();
    t.setenv(ENV, TEST_ZONEINFO);

    export::reset_zoneinfo();

    let _ = Location::load("Asia/Jerusalem");
    t.clean_up(export::reset_zoneinfo);

    let zoneinfo = export::zoneinfo();
    assert_eq!(zoneinfo, TEST_ZONEINFO, "zoneinfo does not match env variable");
}

#[test]
#[ignore = "todo"]
fn first_zone() {}

#[test]
fn load_location_validates_names() {
    const ENV: &str = "ZONEINFO";

    let mut t = Testing::default();
    t.setenv(ENV, "");

    let bad = ["/usr/foo/Foo", "\\UNC\u{000c}oo", "..", "a.."];

    for v in bad {
        match Location::load(v) {
            Ok(_) => panic!("unexpected ok for {v}"),
            Err(Error::InvalidLocationName) => {}
            Err(err) => panic!("unexpected err for {v}: {err}"),
        }
    }
}

#[test]
fn location_names() {
    assert_eq!(LOCAL.to_string(), "Local", "invalid Local location name");
    assert_eq!(UTC.to_string(), "UTC", "invalid UTC location name");
}

#[test]
#[ignore]
fn load_location_from_tz_data() {}

#[test]
#[ignore = "todo"]
fn load_location_from_tz_data_slim() {}

#[test]
fn malformed_tz_data() {
    // The goal here is just that malformed tzdata results in an error, not a panic.
    // ref: https://github.com/golang/go/issues/29437。
    let issue29437 = b"TZif\x00000000000000000\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0000";
    let _err = Location::load_from_tzdata("abc", issue29437).unwrap_err();
}

#[test]
fn tzset() {
    struct Case {
        in_str: &'static str,
        in_end: i64,
        in_sec: i64,
        expect: Option<Expect>,
    }

    #[derive(Debug, PartialEq)]
    struct Expect {
        name: &'static str,
        off: isize,
        start: i64,
        end: i64,
        is_dst: bool,
    }

    impl Case {
        fn expect_none(in_str: &'static str, in_end: i64, in_sec: i64) -> Self {
            Self {
                in_str,
                in_end,
                in_sec,
                expect: None,
            }
        }

        fn expect_some(
            in_str: &'static str,
            in_end: i64,
            in_sec: i64,
            name: &'static str,
            off: isize,
            start: i64,
            end: i64,
            is_dst: bool,
        ) -> Self {
            let expect = Expect {
                name,
                off,
                start,
                end,
                is_dst,
            };
            Self {
                in_str,
                in_end,
                in_sec,
                expect: Some(expect),
            }
        }
    }

    impl From<(&'static str, isize, i64, i64, bool)> for Expect {
        fn from((name, off, start, end, is_dst): (&'static str, isize, i64, i64, bool)) -> Self {
            Self {
                name,
                off,
                start,
                end,
                is_dst,
            }
        }
    }

    let test_vector = vec![
        Case::expect_none("", 0, 0),
        Case::expect_some(
            "PST8PDT,M3.2.0,M11.1.0",
            0,
            2159200800,
            "PDT",
            -7 * 60 * 60,
            2152173600,
            2172733200,
            true,
        ),
        Case::expect_some(
            "PST8PDT,M3.2.0,M11.1.0",
            0,
            2152173599,
            "PST",
            -8 * 60 * 60,
            2145916800,
            2152173600,
            false,
        ),
        Case::expect_some(
            "PST8PDT,M3.2.0,M11.1.0",
            0,
            2152173600,
            "PDT",
            -7 * 60 * 60,
            2152173600,
            2172733200,
            true,
        ),
        Case::expect_some(
            "PST8PDT,M3.2.0,M11.1.0",
            0,
            2152173601,
            "PDT",
            -7 * 60 * 60,
            2152173600,
            2172733200,
            true,
        ),
        Case::expect_some(
            "PST8PDT,M3.2.0,M11.1.0",
            0,
            2172733199,
            "PDT",
            -7 * 60 * 60,
            2152173600,
            2172733200,
            true,
        ),
        Case::expect_some(
            "PST8PDT,M3.2.0,M11.1.0",
            0,
            2172733200,
            "PST",
            -8 * 60 * 60,
            2172733200,
            2177452800,
            false,
        ),
        Case::expect_some(
            "PST8PDT,M3.2.0,M11.1.0",
            0,
            2172733201,
            "PST",
            -8 * 60 * 60,
            2172733200,
            2177452800,
            false,
        ),
        Case::expect_some(
            "KST-9",
            592333200,
            1677246697,
            "KST",
            9 * 60 * 60,
            592333200,
            i64::MAX,
            false,
        ),
    ];

    for (i, c) in test_vector.into_iter().enumerate() {
        let got: Option<Expect> = zoneinfo::tzset(c.in_str, c.in_end, c.in_sec).map(|v| v.into());
        assert_eq!(c.expect, got, "#{i}");
    }
}

#[test]
fn tzset_name() {
    struct Case {
        input: &'static str,
        expect: Option<(&'static str, &'static str)>,
    }

    impl Case {
        fn none(input: &'static str) -> Self {
            Self { input, expect: None }
        }
        fn some(input: &'static str, name: &'static str, out: &'static str) -> Self {
            Self {
                input,
                expect: Some((name, out)),
            }
        }
    }

    let test_vector = vec![
        Case::none(""),
        Case::none("X"),
        Case::some("PST", "PST", ""),
        Case::some("PST8PDT", "PST", "8PDT"),
        Case::some("PST-08", "PST", "-08"),
        Case::some("<A+B>+08", "A+B", "+08"),
    ];

    for (i, c) in test_vector.into_iter().enumerate() {
        let got = zoneinfo::tzset_name(&c.input);
        assert_eq!(c.expect, got, "#{i}");
    }
}

#[test]
fn tzset_offset() {
    struct Case {
        input: &'static str,
        expect: Option<(isize, &'static str)>,
    }

    impl Case {
        fn none(input: &'static str) -> Self {
            Self { input, expect: None }
        }
        fn some(input: &'static str, offset: isize, out: &'static str) -> Self {
            Self {
                input,
                expect: Some((offset, out)),
            }
        }
    }

    let test_vector = vec![
        Case::none(""),
        Case::none("X"),
        Case::none("+"),
        Case::some("+08", 8 * 60 * 60, ""),
        Case::some("-01:02:03", -1 * 60 * 60 - 2 * 60 - 3, ""),
        Case::some("01", 1 * 60 * 60, ""),
        Case::some("100", 100 * 60 * 60, ""),
        Case::none("1000"),
        Case::some("8PDT", 8 * 60 * 60, "PDT"),
    ];

    for (i, c) in test_vector.into_iter().enumerate().skip(7) {
        let got = zoneinfo::tzset_offset(&c.input);
        assert_eq!(c.expect, got, "#{i} {}", c.input);
    }
}

#[test]
fn tzset_rule() {
    struct Case {
        input: &'static str,
        expect: Option<(Rule, &'static str)>,
    }

    impl Case {
        fn none(input: &'static str) -> Self {
            Self { input, expect: None }
        }
        fn some(input: &'static str, rule: Rule, out: &'static str) -> Self {
            Self {
                input,
                expect: Some((rule, out)),
            }
        }
    }

    let test_vector = vec![
        Case::none(""),
        Case::none("X"),
        Case::some(
            "J10",
            Rule {
                kind: RuleKind::Julian,
                day: 10,
                time: 2 * 60 * 60,
                ..Default::default()
            },
            "",
        ),
        Case::some(
            "20",
            Rule {
                kind: RuleKind::DOY,
                day: 20,
                time: 2 * 60 * 60,
                ..Default::default()
            },
            "",
        ),
        Case::some(
            "M1.2.3",
            Rule {
                kind: RuleKind::MonthWeekDay,
                mon: 1,
                week: 2,
                day: 3,
                time: 2 * 60 * 60,
            },
            "",
        ),
        Case::some(
            "30/03:00:00",
            Rule {
                kind: RuleKind::DOY,
                day: 30,
                time: 3 * 60 * 60,
                ..Default::default()
            },
            "",
        ),
        Case::some(
            "M4.5.6/03:00:00",
            Rule {
                kind: RuleKind::MonthWeekDay,
                mon: 4,
                week: 5,
                day: 6,
                time: 3 * 60 * 60,
            },
            "",
        ),
        Case::none("M4.5.7/03:00:00"),
        Case::some(
            "M4.5.6/-04",
            Rule {
                kind: RuleKind::MonthWeekDay,
                mon: 4,
                week: 5,
                day: 6,
                time: -4 * 60 * 60,
            },
            "",
        ),
    ];

    for (i, c) in test_vector.into_iter().enumerate() {
        let got = zoneinfo::tzset_rule(c.input);
        assert_eq!(c.expect, got, "#{i} {}", c.input);
    }
}

#[test]
fn version3() {
    let _d: Defer<_> = internal::disable_platform_sources().into();
    match Location::load("Asia/Jerusalem") {
        Ok(_) => {}
        Err(err) => panic!("unexpected error: {err}"),
    }
}
