fn main() {
    kavionics_cli::run().unwrap_or_else(|_| {
        std::process::exit(1);
    });
}
