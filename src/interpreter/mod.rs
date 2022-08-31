// Specifications

// Memory: 4k ram
// Display 64 x 32 pixels, monochrome
// Program Counter (PC)
// One 16-bit register "I"
// Stack for 16-bit address, used for functions
// 8-bit delay timer, decremented at 60hz till 0
// 8-bit sound timer, like delay but beeps when not at 0
// 16-bit (one byte) general purpose variable registers labled 0-F hex, ie V0-VF
// VF is also commonly used as the flag register

pub mod opcodes;

use log::{error, info, trace, warn};
use opcodes::OP;
use rand;

pub struct Chip8 {
    registers: [u8; 16],
    memory: Memory,
    index: u16,
    program_counter: u16,
    stack: Stack,
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: KeyPad,
    video: VideoBuffer,
    opcode: u16,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    // Shut
    #![allow(dead_code)]

    const START_ADDRESS: u16 = 0x200;
    const FONT_ADDRESS: u16 = 0x50;
    const FONT: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];
    pub const VIDEO_WIDTH: u8 = 64;
    pub const VIDEO_HEIGHT: u8 = 32;
    pub fn new() -> Self {
        // Load font into memory
        let mut memory = Memory::new();
        memory.0[Chip8::FONT_ADDRESS as usize..Chip8::FONT_ADDRESS as usize + Chip8::FONT.len()]
            .copy_from_slice(&Chip8::FONT);

        Chip8 {
            registers: [0; 16],
            memory,
            index: 0,
            program_counter: Chip8::START_ADDRESS,
            stack: Stack::new(),
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: KeyPad::new(),
            video: VideoBuffer::new(),
            opcode: 0,
        }
    }

    pub fn load(&mut self, _filepath: &std::path::Path) -> std::io::Result<()> {
        // let file = match std::fs::read(filepath) {
        //     Ok(file) => file,
        //     Err(e) => {
        //         error!("Could not open file: {}", e);
        //         return Err(e);
        //     }
        // };

        // let file = include_bytes!("../../res/chip8-test-suite.ch8");
        let file = include_bytes!("../../res/IBM Logo.ch8");

        // Would be nice to have start address be usize...
        self.memory.0[Chip8::START_ADDRESS as usize..Chip8::START_ADDRESS as usize + file.len()]
            .copy_from_slice(file);

        Ok(())
    }

    fn execute(&mut self, op: OP) {
        match op {
            OP::CLS => {
                // Clear video buffer by creating new one
                self.video = VideoBuffer::new();
            }
            OP::RET => {
                self.stack_pointer = match self.stack_pointer.checked_sub(1) {
                    Some(v) => v,
                    None => {
                        error!("Stack underflow! Instruction {} underflowed the stack.", op);
                        todo!("Implement returning an error");
                    }
                };
                self.program_counter = self.stack.0[self.stack_pointer as usize];
            }
            OP::JP { addr } => {
                self.program_counter = addr;
            }
            OP::CALL { addr } => {
                self.stack.0[self.stack_pointer as usize] = self.program_counter;

                // Check for overflow
                if self.stack_pointer >= 15 {
                    error!("Stack overflow! Instruction {} overflowed the stack.", op);
                    todo!("Implement returning an error");
                } else {
                    self.stack_pointer += 1;
                }

                self.program_counter = addr;
            }
            OP::SE { vx, byte } => {
                if self.registers[vx as usize] == byte {
                    self.program_counter += 2;
                }
            }
            OP::SNE { vx, byte } => {
                if self.registers[vx as usize] != byte {
                    self.program_counter += 2;
                }
            }
            OP::SER { vx, vy } => {
                if self.registers[vx as usize] == self.registers[vy as usize] {
                    self.program_counter += 2;
                }
            }
            OP::LD { vx, byte } => self.registers[vx as usize] = byte,
            OP::ADD { vx, byte } => self.registers[vx as usize] += byte,
            OP::LDR { vx, vy } => self.registers[vx as usize] = self.registers[vy as usize],
            OP::OR { vx, vy } => self.registers[vx as usize] |= self.registers[vy as usize],
            OP::AND { vx, vy } => self.registers[vx as usize] &= self.registers[vy as usize],
            OP::XOR { vx, vy } => self.registers[vx as usize] ^= self.registers[vy as usize],
            OP::ADDR { vx, vy } => {
                match self.registers[vx as usize].checked_add(self.registers[vy as usize]) {
                    Some(v) => {
                        self.registers[vx as usize] = v;
                        self.registers[0x0F] = 0;
                    }
                    None => {
                        self.registers[vx as usize] = 255;
                        self.registers[0x0F] = 1;
                    }
                }
            }
            OP::SUB { vx, vy } => {
                if self.registers[vx as usize] > self.registers[vy as usize] {
                    self.registers[0x0F] = 1;
                } else {
                    self.registers[0x0F] = 0;
                }
                self.registers[vx as usize] =
                    self.registers[vx as usize].wrapping_sub(self.registers[vy as usize]);
            }
            OP::SHR { vx } => {
                self.registers[0x0F] = self.registers[vx as usize] & 0x01;
                self.registers[vx as usize] >>= 1;
            }
            OP::SUBN { vx, vy } => {
                if self.registers[vy as usize] > self.registers[vx as usize] {
                    self.registers[0x0F] = 1;
                } else {
                    self.registers[0x0F] = 0;
                }
                self.registers[vx as usize] =
                    self.registers[vy as usize].wrapping_sub(self.registers[vx as usize]);
            }
            OP::SHL { vx } => {
                self.registers[0x0F] = (self.registers[vx as usize] & 0x80) >> 7;
                self.registers[vx as usize] <<= 1;
            }
            OP::SNER { vx, vy } => {
                if self.registers[vx as usize] != self.registers[vy as usize] {
                    self.program_counter += 2;
                }
            }
            OP::LDI { addr } => self.index = addr,
            OP::JPR { addr } => self.program_counter = self.registers[0] as u16 + addr,
            // This should be able to be seeded
            OP::RND { vx, byte } => self.registers[vx as usize] = rand::random::<u8>() & byte,
            OP::DRW { vx, vy, height } => {
                // Wrap if values are beyond boundries
                let x_pos = (self.registers[vx as usize] % Chip8::VIDEO_WIDTH) as usize;
                let y_pos = (self.registers[vy as usize] % Chip8::VIDEO_HEIGHT) as usize;

                // Clear VF flag for collisions
                self.registers[0x0F] = 0;

                for row in 0..height as usize {
                    if y_pos + row >= Chip8::VIDEO_HEIGHT as usize {
                        break;
                    }

                    let sprite_byte = self.memory.0[self.index as usize + row];

                    // We know that sprites have a width of 8
                    for col in 0..8 {
                        if x_pos + col >= Chip8::VIDEO_WIDTH as usize {
                            break;
                        }
                        let sprite_pixel = sprite_byte & (0x80 >> col);
                        if sprite_pixel != 0 {
                            let x = (x_pos + col) % Chip8::VIDEO_WIDTH as usize;
                            let y = (y_pos + row) % Chip8::VIDEO_HEIGHT as usize;

                            let idx = x + Chip8::VIDEO_WIDTH as usize * y;
                            let screen_pixel = &mut self.video.0[idx];

                            if *screen_pixel == 0xFF {
                                //  Collision
                                self.registers[0x0F] = 1;
                            }

                            *screen_pixel ^= 0xFF;
                        }
                    }
                }
            }
            OP::SKP { vx } => {
                if self.keypad.0[vx as usize] == 0xFF {
                    self.program_counter += 2;
                }
            }
            OP::SKNP { vx } => {
                if self.keypad.0[vx as usize] != 0xFF {
                    self.program_counter += 2;
                }
            }
            OP::LDDT { vx } => self.registers[vx as usize] = self.delay_timer,
            OP::LDK { vx } => {
                if let Some(index) = self.keypad.0.iter().position(|x| *x == 0xFF) {
                    self.registers[vx as usize] = index as u8;
                } else {
                    // No key was pressed
                    self.program_counter -= 2;
                }
            }
            OP::LDT { vx } => self.delay_timer = self.registers[vx as usize],
            OP::LDST { vx } => self.sound_timer = self.registers[vx as usize],
            OP::ADDI { vx } => self.index += self.registers[vx as usize] as u16,
            OP::LDF { vx } => {
                self.index = Chip8::FONT_ADDRESS + (5 * self.registers[vx as usize] as u16)
            }
            OP::LDB { vx } => {
                let mut value = self.registers[vx as usize];

                // One digit
                self.memory.0[self.index as usize + 2] = value % 10;
                value /= 10;

                // Tens digit
                self.memory.0[self.index as usize + 1] = value % 10;
                value /= 10;

                // Hundreds digit
                self.memory.0[self.index as usize] = value % 10;
            }
            OP::LDIA { vx } => {
                for i in 0..vx as usize {
                    self.memory.0[self.index as usize + i] = self.registers[i];
                }
            }
            OP::LDRA { vx } => {
                for i in 0..vx as usize {
                    self.registers[i] = self.memory.0[self.index as usize + i];
                }
            }
            OP::INV { opcode } => warn!("Attempted to execute invalid opcode: 0x{:04x}", opcode),
        }
    }

    pub fn cycle(&mut self) {
        let first_byte = self.memory.0[self.program_counter as usize] as u16;
        let second_byte = self.memory.0[self.program_counter as usize + 1];
        let op = OP::from(first_byte << 8 | second_byte as u16);

        trace!("PC: {}, OP: {}", self.program_counter, op);
        self.program_counter += 2;

        self.execute(op);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn get_video_buffer(
        &self,
    ) -> [u8; Chip8::VIDEO_WIDTH as usize * Chip8::VIDEO_HEIGHT as usize] {
        self.video.0
    }
}

struct Memory([u8; 4096]);

impl Memory {
    pub fn new() -> Self {
        Memory([0; 4096])
    }
}

// Do I actually want to implement these like this
struct Stack([u16; 16]);

impl Stack {
    pub fn new() -> Self {
        Stack([0; 16])
    }
}

struct KeyPad([u8; 16]);

impl KeyPad {
    pub fn new() -> Self {
        KeyPad([0; 16])
    }
}
struct VideoBuffer([u8; 64 * 32]);

impl VideoBuffer {
    const SIZE: usize = 64 * 32;
    pub fn new() -> Self {
        VideoBuffer([0; VideoBuffer::SIZE])
    }
}
