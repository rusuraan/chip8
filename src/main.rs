use std::{fmt, fs};

const FONT_START: usize = 0x50;

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

#[derive(Debug)]
enum Chip8Error {
    RomTooLarge { rom_size: usize, available: usize },
    InvalidOpcode(u16),
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chip8Error::RomTooLarge {
                rom_size,
                available,
            } => write!(
                f,
                "ROM too large: {rom_size} bytes exceeds available memory of {available} bytes"
            ),
            Chip8Error::InvalidOpcode(op) => write!(f, "Invalid opcode: {op:#06X}"),
        }
    }
}

impl std::error::Error for Chip8Error {}

#[derive(Debug)]
struct Chip8 {
    memory: [u8; 4096],
    framebuffer: [bool; 64 * 32],
    program_counter: u16,
    index_register: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
}

impl Chip8 {
    fn new() -> Self {
        let mut memory = [0; 4096];
        memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);

        Self {
            memory,
            framebuffer: [false; 64 * 32],
            program_counter: 0x200,
            index_register: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
        }
    }

    fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        let start = 0x200;
        let available = self.memory.len() - start;
        if rom.len() > available {
            return Err(Chip8Error::RomTooLarge {
                rom_size: rom.len(),
                available,
            });
        }

        self.memory[start..start + rom.len()].copy_from_slice(rom);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let rom = fs::read("roms/IBM.ch8")?;
    let mut chip8 = Chip8::new();
    chip8.load_rom(&rom)?;
    println!("{chip8:?}");

    Ok(())
}
