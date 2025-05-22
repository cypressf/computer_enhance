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

    let mut bytes = buffer.iter();

    while let Some(byte) = bytes.next() {
        let opcode = byte >> 2;

        let line = match opcode >> 2 {
            0b1011 => {
                // immediate mov
                let operation = "mov";
                let w = (byte & 0b0000_1000) >> 3;
                let reg = byte & 0b0000_0111;
                let data: u16 = match w {
                    0 => *bytes.next().unwrap() as u16,
                    1 => {
                        let bytes = [*bytes.next().unwrap(), *bytes.next().unwrap()];
                        u16::from_le_bytes(bytes)
                    }
                    _ => panic!("w must be 0 or 1, but was {:b}", w),
                };
                let register = register_for(reg, w);
                format!("{} {}, {}", operation, register, data)
            }
            _ => match opcode {
                0b100010 => {
                    // reg/reg or reg/mem mov
                    let operation = "mov";
                    let d = (byte & 0b0000_0010) >> 1;
                    let w = byte & 0b0000_0001;

                    let byte = bytes.next().expect("no second byte for reg/mem mov");
                    let mod_ = byte >> 6;
                    let reg = (byte & 0b0011_1000) >> 3;
                    let rm = byte & 0b0000_0111;

                    let (reg, rm): (String, String) = match mod_ {
                        0b00 | 0b01 | 0b10 => {
                            let address_calculation = {
                                let expression: String = match rm {
                                    0b000 => "bx + si".into(),
                                    0b001 => "bx + di".into(),
                                    0b010 => "bp + si".into(),
                                    0b011 => "bp + di".into(),
                                    0b100 => "si".into(),
                                    0b101 => "di".into(),
                                    0b110 => {
                                        if mod_ == 0b00 {
                                            format!("{}", bytes.next().unwrap())
                                        } else {
                                            "bp".into()
                                        }
                                    }
                                    0b111 => "bx".into(),
                                    _ => panic!("invalid reg {:b} must be 3 bits", rm),
                                };
                                let displacement: u16 = match mod_ {
                                    0b00 => 0,
                                    0b01 => *bytes.next().unwrap() as u16,
                                    0b10 => u16::from_le_bytes([
                                        *bytes.next().unwrap(),
                                        *bytes.next().unwrap(),
                                    ]),
                                    _ => panic!(
                                        "invalid mod {:b} must be between 0b00 and 0b10",
                                        mod_
                                    ),
                                };

                                if displacement != 0 {
                                    format!("[{} + {}]", expression, displacement)
                                } else {
                                    format!("[{}]", expression)
                                }
                            };
                            (register_for(reg, w).into(), address_calculation)
                        }
                        0b11 => (register_for(reg, w).into(), register_for(rm, w).into()),
                        _ => panic!("mod must be 2 bits, but was {:b}", mod_),
                    };

                    let (destination, source) = match d {
                        0 => (rm, reg),
                        1 => (reg, rm),
                        _ => panic!("d must be 0 or 1, but was {:b}", d),
                    };
                    format!("{} {}, {}", operation, destination, source)
                }
                _ => todo!("unknown opcode {:b}", opcode),
            },
        };
        output_file.write_all((line + "\n").as_bytes())?;
    }
    Ok(())
}

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
            "listing_0039_more_movs",
            "0039_disassembled.asm",
            "0039_reassembled",
        )
    }

    #[test]
    fn it_disassembles_challenge_movs() -> Result<(), Box<dyn Error>> {
        it_disassembles_file(
            "listing_0040_challenge_movs",
            "0040_disassembled.asm",
            "0040_reassembled",
        )
    }
}
