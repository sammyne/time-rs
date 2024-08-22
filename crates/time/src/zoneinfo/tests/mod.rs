mod unix;

mod export;

use export::Testing;

use crate::Location;

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
