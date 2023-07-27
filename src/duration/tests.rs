use super::{Duration, MAX_DURATION, MINUTE, MIN_DURATION};

#[test]
fn abs() {
    struct Case {
        d: Duration,
        want: Duration,
    }

    let test_vector = vec![
        (0, 0),
        (1, 1),
        (-1, 1),
        (1 * MINUTE.0, 1 * MINUTE.0),
        (-1 * MINUTE.0, 1 * MINUTE.0),
        (MIN_DURATION.0, MAX_DURATION.0),
        (MIN_DURATION.0 + 1, MAX_DURATION.0),
        (MIN_DURATION.0 + 2, MAX_DURATION.0 - 1),
        (MAX_DURATION.0, MAX_DURATION.0),
        (MAX_DURATION.0 - 1, MAX_DURATION.0 - 1),
    ]
    .into_iter()
    .map(|(d, want)| (Duration(d), Duration(want)))
    .map(|(d, want)| Case { d, want });

    for (i, c) in test_vector.enumerate() {
        assert_eq!(c.d.abs(), c.want, "#{i}");
    }
}
