use std::{error::Error, fs::File, io::Read, io::Write};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        panic!("Usage: disassemble input_file output_file")
    }

    let mut input_file = File::open(&args[1])?;
    let mut output_file = File::create(&args[2])?;

    writeln!(output_file, "bits 16")?;
    writeln!(output_file)?;

    let mut buffer: Vec<u8> = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    fn register_for(reg: u8, w: u8) -> &'static str {
        match (reg, w) {
            (0b000, 0) => "al",
            (0b001, 0) => "cl",
            (0b010, 0) => "dl",
            (0b011, 0) => "bl",

            (0b100, 0) => "ah",
            (0b101, 0) => "ch",
            (0b110, 0) => "dh",
            (0b111, 0) => "bh",

            (0b000, 1) => "ax",
            (0b001, 1) => "cx",
            (0b010, 1) => "dx",
            (0b011, 1) => "bx",

            (0b100, 1) => "sp",
            (0b101, 1) => "bp",
            (0b110, 1) => "si",
            (0b111, 1) => "di",

            _ => panic!("Not valid binary"),
        }
    }

    for i in (0..buffer.len()).step_by(2) {
        let byte1 = buffer[i];
        let opcode = byte1 >> 2;
        let d = (byte1 & 0b00000010) >> 1;
        let w = byte1 & 0b00000001;

        let byte2 = buffer[i + 1];
        let mod_ = byte2 >> 6;
        let reg = (byte2 & 0b00111000) >> 3;
        let rm = byte2 & 0b00000111;

        if mod_ != 0b11 {
            panic!("Not a register to register mov");
        }

        let operation = match opcode {
            0b100010 => "mov",
            _ => todo!("unknown opcode {:b}", opcode),
        };

        let reg = register_for(reg, w);
        let rm = register_for(rm, w);

        let (destination, source) = match d {
            0 => (rm, reg),
            1 => (reg, rm),
            _ => panic!("Not valid binary"),
        };
        writeln!(output_file, "{} {}, {}", operation, destination, source)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use std::error::Error;

    const NASM: &str = "nasm";
    const DISASSEMBLE: &str = "disassemble";
    const CMP: &str = "cmp";

    fn it_disassembles_file(
        input: &str,
        disassembled: &str,
        reassembled: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin(DISASSEMBLE)?
            .arg(input)
            .arg(disassembled)
            .ok()?;
        Command::new(NASM)
            .arg(disassembled)
            .arg("-o")
            .arg(reassembled)
            .ok()?;
        Command::new(CMP)
            .arg(input)
            .arg(reassembled)
            .assert()
            .success()
            .stdout("");
        Ok(())
    }

    #[test]
    fn it_disassembles_single_register_mov() -> Result<(), Box<dyn Error>> {
        it_disassembles_file(
            "listing_0037_single_register_mov",
            "0037_disassembled.asm",
            "0037_reassembled",
        )
    }

    #[test]
    fn it_disassembles_many_register_mov() -> Result<(), Box<dyn Error>> {
        it_disassembles_file(
            "listing_0038_many_register_mov",
            "0038_disassembled.asm",
            "0038_reassembled",
        )
    }

    #[test]
    fn it_disassembles_more_movs() -> Result<(), Box<dyn Error>> {
        it_disassembles_file(
            "listing_0039_more_movs.asm",
            "0039_disassembled",
            "0039_reassembled.asm",
        )
    }

    #[test]
    fn it_disassembles_challenge_movs() -> Result<(), Box<dyn Error>> {
        it_disassembles_file(
            "listing_0040_challenge_movs.asm",
            "0040_disassembled",
            "0040_reassembled.asm",
        )
    }
}
