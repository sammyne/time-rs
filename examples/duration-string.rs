fn main() {
    println!(
        "{}",
        time::HOUR * 1 + time::MINUTE * 2 + time::MILLISECOND * 300
    );

    println!("{}", time::MILLISECOND * 300);
}
