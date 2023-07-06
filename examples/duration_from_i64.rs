fn main() {
    let seconds = 10;
    assert_eq!("10s", (time::Duration(seconds) * time::SECOND).to_string());
}
