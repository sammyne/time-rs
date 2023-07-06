fn main() {
    let d = time::HOUR * 1 + time::MINUTE * 2 + time::MILLISECOND * 300;
    assert_eq!("1h2m0.3s", d.to_string());

    let d = time::MILLISECOND * 300;
    assert_eq!("300ms", d.to_string());
}
