fn main() {
    let mut a = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Hello {}", a);
        a += 1;
    }
}
