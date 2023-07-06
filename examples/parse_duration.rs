fn main() {
    let hours = time::parse_duration("10h").unwrap();
    let complex = time::parse_duration("1h10m10s").unwrap();
    // let micro = time::parse_duration("1µs").unwrap();
    // The crate also accepts the incorrect but common prefix u for micro.
    // let micro2 = time::parse_duration("1us").unwrap();

    assert_eq!("10h0m0s", hours.to_string());
    assert_eq!("1h10m10s", complex.to_string());
    //assert_eq!("There are 4210 seconds in 1h10m10s.", format!("There are {} seconds in {}", complex.seconds(), complex));
}
