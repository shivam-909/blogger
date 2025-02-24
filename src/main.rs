use blogger::cli;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("\x1b[93m{}\x1b[0m", err); // Use {} to invoke Display formatting
        std::process::exit(1);
    }
}
