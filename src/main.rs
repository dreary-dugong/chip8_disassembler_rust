use ch8disasm::{self, Config};
use std::process;

fn main() {
    if let Err(run_error) = ch8disasm::run(Config::make()) {
        eprintln!("{}", run_error.msg);
        process::exit(1);
    }
    process::exit(0);
}
