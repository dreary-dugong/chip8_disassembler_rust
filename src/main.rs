use ch8disasm::{self, Config};
use std::process;

/// A minimal main function to run the CLI app, print any errors encountered, and exit the process
/// accordingly
fn main() {
    if let Err(run_error) = ch8disasm::run(Config::make()) {
        eprintln!("{}", run_error.msg);
        process::exit(1);
    }
    process::exit(0);
}
