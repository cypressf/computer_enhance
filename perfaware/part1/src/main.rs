use std::{fs::File, io::Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("listing_0037_single_register_mov")?;
    let buffer = &mut [0; 2];

    file.read_exact(buffer)?;

    println!("binary: {:b} {:b}", buffer[0], buffer[1]);

    println!("binary: {:b} {:b}", buffer[0], buffer[1]);

    let opcode = buffer[0] >> 2;
    let d = (buffer[0] & 0b00000010) >> 1;
    let w = buffer[0] & 0b00000001;

    let mod_ = buffer[1] >> 6;
    let reg = (buffer[1] & 0b00111000) >> 3;
    let rm = buffer[1] & 0b00000111;

    let operation = match opcode {
        0b100010 => "mov",
        _ => panic!("unknown opcode {:b}", opcode),
    };
    let register = match (reg, w) {
        (0b000, 0) => "AL",
        (0b000, 1) => "AX",
        (0b001, 0) => "CL",
        _ => todo!("need to implement the register matching"),
    };

    Ok(())
}
