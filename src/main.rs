fn main() {
    if let Err(error) = loc_checker::run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
