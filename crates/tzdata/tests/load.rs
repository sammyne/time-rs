#[test]
fn load_ok() {
    let test_vector = vec!["Asia/Shanghai", "Asia/Jerusalem", "America/Los_Angeles"];

    for v in test_vector {
        if let Err(err) = tzdata::load(v) {
            panic!("unexpected error {err} for loading {v}");
        }
    }
}
