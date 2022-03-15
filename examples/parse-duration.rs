fn main() {
    let hours = time::parse_duration("10h").unwrap();

    println!("{}", hours);
}
