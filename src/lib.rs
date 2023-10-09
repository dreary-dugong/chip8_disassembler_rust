use clap::Parser;
use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::path::PathBuf;

/// CLAP parser to handle CLI arguments and help page
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // input file. If empty, we use stdin
    #[arg(short, long)]
    input: Option<PathBuf>,
    // output file. If empty, we use stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// An enum for the app config that says whether it should read from a file or from stdin
enum InputOption {
    File(PathBuf),
    Stdin,
}

/// An enum for the app config that says whether it should write to a file or to stdout
enum OutputOption {
    File(PathBuf),
    Stdout,
}

/// A struct representing the configuration for the cli app. See enums above.
pub struct Config {
    in_opt: InputOption,
    out_opt: OutputOption,
}

impl Config {
    /// Automatically construct the app config based on the args passed to the binary
    pub fn make() -> Self {
        let args = Args::parse();

        let in_opt = match args.input {
            Some(p) => InputOption::File(p),
            None => InputOption::Stdin,
        };
        let out_opt = match args.output {
            Some(p) => OutputOption::File(p),
            None => OutputOption::Stdout,
        };

        Config { in_opt, out_opt }
    }
}

/// A struct to represent an error that occured while running the app to be communicated back to the user via a
/// message printed to stderr
pub struct RunError {
    pub msg: &'static str,
}

impl From<io::Error> for RunError {
    /// Convert io error to our RunError so we can use the ? operator
    fn from(e: io::Error) -> Self {
        let msg = match e.kind() {
            ErrorKind::NotFound => "File does not exist",
            _ => "An unanticipated error was encountered while reading or writing to file",
        };
        RunError { msg }
    }
}

impl From<&'static str> for RunError {
    /// convert a simple static string slice error to a run error so we can use the ? operator
    // Ideally, disassemble would return a custom error type instead of a static string but it really doesn't matter
    fn from(s: &'static str) -> Self {
        let msg = s;
        RunError { msg }
    }
}

/// Run the CLI app based on the config constructed and passed in by the main function
pub fn run(config: Config) -> Result<(), RunError> {
    // read rom bytes from file or stdin
    let rom_bytes = match config.in_opt {
        InputOption::File(p) => fs::read(p)?,
        InputOption::Stdin => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            buf
        }
    };
    // disassemble the bytes into a long string of instructions seperated by newlines
    let instructions = disassemble(rom_bytes)?;
    // write the disassembled instructions to a file or stdout
    match config.out_opt {
        OutputOption::File(f) => fs::write(f, instructions)?,
        OutputOption::Stdout => {
            io::stdout().write_all(instructions.as_bytes())?;
        }
    };
    Ok(())
}

/// Given a vector of bytes from a rom, return a string of disassembled instructions separated by
/// newlines
fn disassemble(assembled_bytes: Vec<u8>) -> Result<String, &'static str> {
    // error handling; We handle these here so we don't need awkward pattern matching later
    if assembled_bytes.is_empty() {
        return Err("Error parsing rom: empty rom");
    }
    if assembled_bytes.len() % 2 != 0 {
        return Err("Error parsing rom: uneven number of bytes");
    }

    let disassembled_string = assembled_bytes
        // group file bytes into pairs to parse 16-bit instructions
        .chunks(2)
        // convert iterator of u8 pairs to iterator of u16s
        .map(|chunk| {
            if let [b1, b2] = chunk {
                ((*b1 as u16) << 8) | (*b2 as u16)
            } else {
                unreachable!(
                    "We handle this possiblity earlier in the function and return an error"
                )
            }
        })
        // convert instruction code to asm string
        .map(convert_instruction)
        // convert to one long string to write to output file
        .fold(String::new(), |mut acc, inst| {
            acc.push_str(&inst);
            acc.push('\n');
            acc
        });

    Ok(disassembled_string)
}

/// Given a u16 representing an assembled chip8 instruction, return the human-readable string
/// format of that instruction
/// Instructions and format outlined at http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
fn convert_instruction(inst: u16) -> String {
    // instructions with 16-bit opcodes and no arguments
    if inst == 0x00E0 {
        return String::from("CLS");
    }
    if inst == 0x00EE {
        return String::from("RET");
    }

    // instructions with opcode for first 4 bits, single argument for bottom 12
    let upper_four = inst >> 12;
    let lower_twelve = inst & 0x0FFF;
    let addr = lower_twelve;
    if upper_four == 0 {
        return String::from("SYS 0x") + &format!("{:0>3X}", addr);
    } // note that this interprets null bytes as SYS 0x000. Realistically this instruction is
      // probably unused
    if upper_four == 1 {
        return String::from("JP 0x") + &format!("{:0>3X}", addr);
    }
    if upper_four == 2 {
        return String::from("CALL 0x") + &format!("{:0>3X}", addr);
    }
    if upper_four == 0xA {
        return String::from("LD I, 0x") + &format!("{:0>3X}", addr);
    }
    if upper_four == 0xB {
        return String::from("JP V0, 0x") + &format!("{:0>3X}", addr);
    }

    // instructions with opcode for first 4 bits, one 4-bit arg, and one 8-bit arg
    let x_arg = (inst & 0x0F00) >> 8;
    let lower_eight = inst & 0x00FF;
    let byte = lower_eight;
    if upper_four == 3 {
        return String::from("SE V") + &format!("{:X}", x_arg) + ", 0x" + &format!("{:0>2X}", byte);
    }
    if upper_four == 4 {
        return String::from("SNE V")
            + &format!("{:X}", x_arg)
            + ", 0x"
            + &format!("{:0>2X}", byte);
    }
    if upper_four == 6 {
        return String::from("LD V") + &format!("{:X}", x_arg) + ", 0x" + &format!("{:0>2X}", byte);
    }
    if upper_four == 7 {
        return String::from("ADD V")
            + &format!("{:X}", x_arg)
            + ", 0x"
            + &format!("{:0>2X}", byte);
    }
    if upper_four == 0xC {
        return String::from("RND V")
            + &format!("{:X}", x_arg)
            + ", 0x"
            + &format!("{:0>2X}", byte);
    }

    // instructions with opcode for first 4 and last 4 bits, two 4-bit args
    let y_arg = (inst & 0x00F0) >> 4;
    let lower_four = inst & 0x000F;
    if upper_four == 5 && lower_four == 0 {
        return String::from("SE V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 8 && lower_four == 0 {
        return String::from("LD V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 8 && lower_four == 1 {
        return String::from("OR V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 8 && lower_four == 2 {
        return String::from("AND V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 8 && lower_four == 3 {
        return String::from("XOR V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 8 && lower_four == 4 {
        return String::from("ADD V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 8 && lower_four == 5 {
        return String::from("SUB V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 8 && lower_four == 7 {
        return String::from("SUBN V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    if upper_four == 9 && lower_four == 0 {
        return String::from("SNE V") + &format!("{:X}", x_arg) + ", V" + &format!("{:X}", y_arg);
    }
    // the second 4-bit arg is ignored for these two
    if upper_four == 8 && lower_four == 6 {
        return String::from("SHR V") + &format!("{:X}", x_arg);
    }
    if upper_four == 8 && lower_four == 0xE {
        return String::from("SHL V") + &format!("{:X}", x_arg);
    }

    // instructions with opcde for first 4 bits, three 4-bit args
    let nibble = lower_four;
    if upper_four == 0xD {
        return String::from("DRW V")
            + &format!("{:X}", x_arg)
            + ", V"
            + &format!("{:X}", y_arg)
            + ", 0x"
            + &format!("{:X}", nibble);
    }

    // instructions with opcode for first 4 bits and last 8 bits, one 4-bit arg
    if upper_four == 0xE && lower_eight == 0x9E {
        return String::from("SKP V") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xE && lower_eight == 0xA1 {
        return String::from("SKNP V") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xF && lower_eight == 0x07 {
        return String::from("LD V") + &format!("{:X}", x_arg) + ", DT";
    }
    if upper_four == 0xF && lower_eight == 0x0A {
        return String::from("LD V") + &format!("{:X}", x_arg) + ", K";
    }
    if upper_four == 0xF && lower_eight == 0x15 {
        return String::from("LD DT, v") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xF && lower_eight == 0x18 {
        return String::from("LD ST, V") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xF && lower_eight == 0x1E {
        return String::from("ADD I, V") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xF && lower_eight == 0x29 {
        return String::from("LD F, V") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xF && lower_eight == 0x33 {
        return String::from("LD B, V") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xF && lower_eight == 0x55 {
        return String::from("LD [I], V") + &format!("{:X}", x_arg);
    }
    if upper_four == 0xF && lower_eight == 0x65 {
        return String::from("LD V") + &format!("{:X}", x_arg) + ", [I]";
    }

    // instruction not found, probably a bitmap graphic or other data
    String::from("0x") + &format!("{:0>4X}", inst)
}
