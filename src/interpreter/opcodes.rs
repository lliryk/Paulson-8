// These OPs will be capitalized, RFC be aware!
#[allow(clippy::upper_case_acronyms)]
#[derive(Eq, PartialEq, Debug)]
pub enum OP {
    CLS,
    RET,
    CALL { addr: u16 },
    SE { vx: u8, byte: u8 },
    SNE { vx: u8, byte: u8 },
    // SE vx
    SER { vx: u8, vy: u8 },
    LD { vx: u8, byte: u8 },
    JP { addr: u16 },
    ADD { vx: u8, byte: u8 },
    LDR { vx: u8, vy: u8 },
    OR { vx: u8, vy: u8 },
    AND { vx: u8, vy: u8 },
    XOR { vx: u8, vy: u8 },
    ADDR { vx: u8, vy: u8 }, // This is inconsistent with the other names
    SUB { vx: u8, vy: u8 },
    SHR { vx: u8 },
    SUBN { vx: u8, vy: u8 },
    SHL { vx: u8 },
    SNER { vx: u8, vy: u8 },
    LDI { addr: u16 },
    JPR { addr: u16 },
    RND { vx: u8, byte: u8 },
    DRW { vx: u8, vy: u8, height: u8 },
    SKP { vx: u8 },
    SKNP { vx: u8 },
    LDDT { vx: u8 },
    LDK { vx: u8 },
    LDT { vx: u8 },
    LDST { vx: u8 },
    ADDI { vx: u8 },
    LDF { vx: u8 },
    LDB { vx: u8 },
    LDIA { vx: u8 },
    LDRA { vx: u8 },
    INV { opcode: u16 }, // Invalid opcode
}

impl From<u16> for OP {
    fn from(v: u16) -> Self {
        match v {
            0x00E0 => OP::CLS,
            0x00EE => OP::RET,
            0x1000..=0x1FFF => OP::JP { addr: v & 0x0FFF },
            0x2000..=0x2FFF => OP::CALL { addr: v & 0x0FFF },
            0x3000..=0x3FFF => OP::SE {
                vx: ((v & 0x0F00) >> 8) as u8,
                byte: (v & 0x00FF) as u8,
            },
            0x4000..=0x4FFF => OP::SNE {
                vx: ((v & 0x0F00) >> 8) as u8,
                byte: (v & 0x00FF) as u8,
            },
            0x5000..=0x5FFF if v & 0x000F == 0 => OP::SER {
                vx: ((v & 0x0F00) >> 8) as u8,
                vy: ((v & 0x00F0) >> 4) as u8,
            },
            0x6000..=0x6FFF => OP::LD {
                vx: ((v & 0x0F00) >> 8) as u8,
                byte: (v & 0x00FF) as u8,
            },
            0x7000..=0x7FFF => OP::ADD {
                vx: ((v & 0x0F00) >> 8) as u8,
                byte: (v & 0x00FF) as u8,
            },
            0x8000..=0x8FFF => {
                let vx = ((v & 0x0F00) >> 8) as u8;
                let vy = ((v & 0x00F0) >> 4) as u8;

                match v & 0x000F {
                    0x00 => OP::LDR { vx, vy },
                    0x01 => OP::OR { vx, vy },
                    0x02 => OP::AND { vx, vy },
                    0x03 => OP::XOR { vx, vy },
                    0x04 => OP::ADDR { vx, vy },
                    0x05 => OP::SUB { vx, vy },
                    0x06 => OP::SHR { vx }, // Potentially move vy here also? Used in orignal implemenation
                    0x07 => OP::SUBN { vx, vy },
                    0x0E => OP::SHL { vx },
                    _ => OP::INV { opcode: v },
                }
            }
            0x9000..=0x9FFF if v & 0x000F == 0 => OP::SNER {
                vx: ((v & 0x0F00) >> 8) as u8,
                vy: ((v & 0x00F0) >> 4) as u8,
            },
            0xA000..=0xAFFF => OP::LDI { addr: v & 0x0FFF },
            0xB000..=0xBFFF => OP::JPR { addr: v & 0x0FFF },
            0xC000..=0xCFFF => OP::RND {
                vx: ((v & 0x0F00) >> 8) as u8,
                byte: (v & 0x00FF) as u8,
            },
            0xD000..=0xDFFF => OP::DRW {
                vx: ((v & 0x0F00) >> 8) as u8,
                vy: ((v & 0x00F0) >> 4) as u8,
                height: (v & 0x0F) as u8,
            },
            0xE000..=0xEFFF if v & 0x00FF == 0xE9 => OP::SKP {
                vx: ((v & 0x0F00) >> 8) as u8,
            },
            0xE000..=0xEFFF if v & 0x00FF == 0xA1 => OP::SKNP {
                vx: ((v & 0x0F00) >> 8) as u8,
            },
            0xF000..=0xFFFF => {
                let vx = ((v & 0x0F00) >> 8) as u8;
                match v & 0x00FF {
                    0x07 => OP::LDDT { vx },
                    0x0A => OP::LDK { vx },
                    0x15 => OP::LDT { vx },
                    0x18 => OP::LDST { vx },
                    0x1E => OP::ADDI { vx },
                    0x29 => OP::LDF { vx },
                    0x33 => OP::LDB { vx },
                    0x55 => OP::LDIA { vx },
                    0x65 => OP::LDRA { vx },
                    _ => OP::INV { opcode: v },
                }
            }
            _ => OP::INV { opcode: v },
        }
    }
}

impl std::fmt::Display for OP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            OP::CLS => "CLS",
            OP::RET => "RET",
            OP::JP { .. } => "JMP",
            OP::CALL { .. } => "CALL",
            OP::SE { .. } => "SE",
            OP::SNE { .. } => "SNE",
            OP::SER { .. } => "SER", // Should this be "SE"? SER is only used internally
            OP::LD { .. } => "LD",
            OP::ADD { .. } => "ADD",
            OP::LDR { .. } => "LDR", // Should this be "LD"?
            OP::OR { .. } => "OR",
            OP::AND { .. } => "AND",
            OP::XOR { .. } => "XOR",
            OP::ADDR { .. } => "ADDR", // Should this be "ADD"?
            OP::SUB { .. } => "SUB",
            OP::SHR { .. } => "SHR",
            OP::SUBN { .. } => "SUBN",
            OP::SHL { .. } => "SHL",
            OP::SNER { .. } => "SNER",
            OP::LDI { .. } => "LDI", // Should this be "LD"?
            OP::JPR { .. } => "JPR", // Should this be "JP"?
            OP::RND { .. } => "RND",
            OP::DRW { .. } => "DRW",
            OP::SKP { .. } => "SKP",
            OP::SKNP { .. } => "SKNP",
            OP::LDDT { .. } => "LDDT", // Should this be "LD"?
            OP::LDK { .. } => "LDK",
            OP::LDT { .. } => "LDT", // This is ld delay timer register, but LDDTR is too long
            OP::LDST { .. } => "LDST",
            OP::ADDI { .. } => "ADDI",
            OP::LDF { .. } => "LDF",
            OP::LDB { .. } => "LDB",
            OP::LDIA { .. } => "LDIA", // Load index array, is there a better name?
            OP::LDRA { .. } => "LDRA", // Load register array
            OP::INV { .. } => "INV",
        })
    }
}

#[cfg(test)]
mod test {
    use super::OP;

    #[test]
    fn parse_clear() {
        assert_eq!(OP::from(0x00E0), OP::CLS);
    }

    #[test]
    fn parse_jump() {
        assert_eq!(OP::from(0x1000), OP::JP { addr: 0x0000 });
        assert_eq!(OP::from(0x1FFF), OP::JP { addr: 0x0FFF });
        assert_eq!(OP::from(0x1456), OP::JP { addr: 0x0456 });
    }

    #[test]
    fn parse_call() {
        assert_eq!(OP::from(0x2000), OP::CALL { addr: 0x0000 });
        assert_eq!(OP::from(0x2FFF), OP::CALL { addr: 0x0FFF });
        assert_eq!(OP::from(0x2456), OP::CALL { addr: 0x0456 });
    }

    #[test]
    fn parse_skip_next_instruction_equal() {
        assert_eq!(
            OP::from(0x3000),
            OP::SE {
                vx: 0x00,
                byte: 0x00
            }
        );
        assert_eq!(
            OP::from(0x3FFF),
            OP::SE {
                vx: 0x0F,
                byte: 0xFF
            }
        );
        assert_eq!(
            OP::from(0x3456),
            OP::SE {
                vx: 0x04,
                byte: 0x56
            }
        );
    }

    #[test]
    fn parse_skip_next_instruction_not_equal() {
        assert_eq!(
            OP::from(0x4000),
            OP::SNE {
                vx: 0x00,
                byte: 0x00
            }
        );
        assert_eq!(
            OP::from(0x4FFF),
            OP::SNE {
                vx: 0x0F,
                byte: 0xFF
            }
        );
        assert_eq!(
            OP::from(0x4456),
            OP::SNE {
                vx: 0x04,
                byte: 0x56
            }
        );
    }

    #[test]
    fn parse_skip_next_instruction_register() {
        assert_eq!(OP::from(0x5000), OP::SER { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x5FF0), OP::SER { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x5450), OP::SER { vx: 0x04, vy: 0x05 });

        // The last bit of the instruction must be 0
        assert_eq!(OP::from(0x5001), OP::INV { opcode: 0x5001 });
        assert_eq!(OP::from(0x5256), OP::INV { opcode: 0x5256 });
    }

    #[test]
    fn parse_load() {
        assert_eq!(
            OP::from(0x6000),
            OP::LD {
                vx: 0x00,
                byte: 0x00
            }
        );
        assert_eq!(
            OP::from(0x6FFF),
            OP::LD {
                vx: 0x0F,
                byte: 0xFF
            }
        );
        assert_eq!(
            OP::from(0x6456),
            OP::LD {
                vx: 0x04,
                byte: 0x56
            }
        );
    }

    #[test]
    fn parse_add() {
        assert_eq!(
            OP::from(0x7000),
            OP::ADD {
                vx: 0x00,
                byte: 0x00
            }
        );
        assert_eq!(
            OP::from(0x7FFF),
            OP::ADD {
                vx: 0x0F,
                byte: 0xFF
            }
        );
        assert_eq!(
            OP::from(0x7456),
            OP::ADD {
                vx: 0x04,
                byte: 0x56
            }
        );
    }

    #[test]
    fn parse_load_vx() {
        assert_eq!(OP::from(0x8000), OP::LDR { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x8FF0), OP::LDR { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x8450), OP::LDR { vx: 0x04, vy: 0x05 });
    }

    #[test]
    fn parse_or() {
        assert_eq!(OP::from(0x8001), OP::OR { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x8FF1), OP::OR { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x8451), OP::OR { vx: 0x04, vy: 0x05 });
    }

    #[test]
    fn parse_and() {
        assert_eq!(OP::from(0x8002), OP::AND { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x8FF2), OP::AND { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x8452), OP::AND { vx: 0x04, vy: 0x05 });
    }

    #[test]
    fn parse_xor() {
        assert_eq!(OP::from(0x8003), OP::XOR { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x8FF3), OP::XOR { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x8453), OP::XOR { vx: 0x04, vy: 0x05 });
    }

    #[test]
    fn parse_addr() {
        assert_eq!(OP::from(0x8004), OP::ADDR { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x8FF4), OP::ADDR { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x8454), OP::ADDR { vx: 0x04, vy: 0x05 });
    }

    #[test]
    fn parse_sub() {
        assert_eq!(OP::from(0x8005), OP::SUB { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x8FF5), OP::SUB { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x8455), OP::SUB { vx: 0x04, vy: 0x05 });
    }

    #[test]
    fn parse_shift_right() {
        assert_eq!(OP::from(0x8006), OP::SHR { vx: 0x00 });
        assert_eq!(OP::from(0x8456), OP::SHR { vx: 0x04 });
        assert_eq!(OP::from(0x8FF6), OP::SHR { vx: 0x0F });
    }

    #[test]
    fn parse_subn() {
        assert_eq!(OP::from(0x8007), OP::SUBN { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x8FF7), OP::SUBN { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x8457), OP::SUBN { vx: 0x04, vy: 0x05 });
    }

    #[test]
    fn parse_shift_left() {
        assert_eq!(OP::from(0x800E), OP::SHL { vx: 0x00 });
        assert_eq!(OP::from(0x845E), OP::SHL { vx: 0x04 });
        assert_eq!(OP::from(0x8FFE), OP::SHL { vx: 0x0F });
    }

    #[test]
    fn parse_skip_next_instruction_not_equal_register() {
        assert_eq!(OP::from(0x9000), OP::SNER { vx: 0x00, vy: 0x00 });
        assert_eq!(OP::from(0x9FF0), OP::SNER { vx: 0x0F, vy: 0x0F });
        assert_eq!(OP::from(0x9450), OP::SNER { vx: 0x04, vy: 0x05 });

        // Last bit 0x---0 must be zero
        assert_eq!(OP::from(0x9001), OP::INV { opcode: 0x9001 });
    }

    #[test]
    fn parse_load_index() {
        assert_eq!(OP::from(0xA000), OP::LDI { addr: 0x0000 });
        assert_eq!(OP::from(0xA456), OP::LDI { addr: 0x0456 });
        assert_eq!(OP::from(0xAFFE), OP::LDI { addr: 0x0FFE });
    }

    #[test]
    fn parse_jump_register() {
        assert_eq!(OP::from(0xB000), OP::JPR { addr: 0x0000 });
        assert_eq!(OP::from(0xB456), OP::JPR { addr: 0x0456 });
        assert_eq!(OP::from(0xBFFE), OP::JPR { addr: 0x0FFE });
    }

    #[test]
    fn parse_random() {
        assert_eq!(
            OP::from(0xC000),
            OP::RND {
                vx: 0x00,
                byte: 0x00
            }
        );
        assert_eq!(
            OP::from(0xCFFF),
            OP::RND {
                vx: 0x0F,
                byte: 0xFF
            }
        );
        assert_eq!(
            OP::from(0xC456),
            OP::RND {
                vx: 0x04,
                byte: 0x56
            }
        );
    }

    #[test]
    fn parse_draw() {
        assert_eq!(
            OP::from(0xD000),
            OP::DRW {
                vx: 0x00,
                vy: 0x00,
                height: 0x00
            }
        );
        assert_eq!(
            OP::from(0xDFF4),
            OP::DRW {
                vx: 0x0F,
                vy: 0x0F,
                height: 0x04
            }
        );
        assert_eq!(
            OP::from(0xD45E),
            OP::DRW {
                vx: 0x04,
                vy: 0x05,
                height: 0x0E
            }
        );
    }

    #[test]
    fn parse_skip() {
        assert_eq!(OP::from(0xE0E9), OP::SKP { vx: 0x00 });
        assert_eq!(OP::from(0xEFE9), OP::SKP { vx: 0x0F });
        assert_eq!(OP::from(0xE000), OP::INV { opcode: 0xE000 });
    }

    #[test]
    fn parse_skip_not_equal() {
        assert_eq!(OP::from(0xE0A1), OP::SKNP { vx: 0x00 });
        assert_eq!(OP::from(0xEFA1), OP::SKNP { vx: 0x0F });
        assert_eq!(OP::from(0xE000), OP::INV { opcode: 0xE000 });
    }

    #[test]
    fn parse_load_delay_timer() {
        assert_eq!(OP::from(0xF007), OP::LDDT { vx: 0x00 });
        assert_eq!(OP::from(0xFF07), OP::LDDT { vx: 0x0F });
    }

    #[test]
    fn parse_load_key() {
        assert_eq!(OP::from(0xF00A), OP::LDK { vx: 0x00 });
        assert_eq!(OP::from(0xFF0A), OP::LDK { vx: 0x0F });
    }

    #[test]
    fn parse_set_delay_timer() {
        assert_eq!(OP::from(0xF015), OP::LDT { vx: 0x00 });
        assert_eq!(OP::from(0xFF15), OP::LDT { vx: 0x0F });
    }

    #[test]
    fn parse_set_sound_timer() {
        assert_eq!(OP::from(0xF018), OP::LDST { vx: 0x00 });
        assert_eq!(OP::from(0xFF18), OP::LDST { vx: 0x0F });
    }

    #[test]
    fn parse_add_index() {
        assert_eq!(OP::from(0xF01E), OP::ADDI { vx: 0x00 });
        assert_eq!(OP::from(0xFF1E), OP::ADDI { vx: 0x0F });
    }

    #[test]
    fn parse_load_font() {
        assert_eq!(OP::from(0xF029), OP::LDF { vx: 0x00 });
        assert_eq!(OP::from(0xFF29), OP::LDF { vx: 0x0F });
    }

    #[test]
    fn parse_store_bcd() {
        assert_eq!(OP::from(0xF033), OP::LDB { vx: 0x00 });
        assert_eq!(OP::from(0xFF33), OP::LDB { vx: 0x0F });
    }

    #[test]
    fn parse_load_index_array() {
        assert_eq!(OP::from(0xF055), OP::LDIA { vx: 0x00 });
        assert_eq!(OP::from(0xFF55), OP::LDIA { vx: 0x0F });
    }

    #[test]
    fn parse_load_register_array() {
        assert_eq!(OP::from(0xF065), OP::LDRA { vx: 0x00 });
        assert_eq!(OP::from(0xFF65), OP::LDRA { vx: 0x0F });
    }
}
