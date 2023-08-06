use std::env;

fn main(){
    let mut args = env::args();
    args.next();

    let input_file = match args.next() {
        Some(f) => f,
        None => panic!("You must enter an input file"),
    };
    let output_file = match args.next() {
        Some(f) => f,
        None => panic!("You must enter an output file"),
    };

    if let Err(msg) = ch8disasm::read_write_asm_file(input_file, output_file) {
        panic!("{}", msg);
    }
}
