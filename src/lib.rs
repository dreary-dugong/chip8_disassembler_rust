pub fn read_write_asm_file(input_file: String, output_file: String) -> Result<(), &'static str> {
    use std::fs;

    let rom_bytes = match fs::read(input_file) {
        Ok(b) => b,
        Err(_) => return Err("Error reading input file. Does the file exist?"),
    };

    match fs::write(output_file, disassemble(rom_bytes)?) {
        Ok(_) => Ok(()),
        Err(_) => Err("Error writing output file. Is it being used by another process?"),
    }
}

fn disassemble(assembled_bytes: Vec<u8>) -> Result<String, &'static str> {

    // error handling
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
                unreachable!("We handle this possiblity earlier in the function and return an error")
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

fn convert_instruction(inst: u16) -> String {
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
    }
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

    String::from("ERR: ") + &format!("{:0>4X}", inst)
}
