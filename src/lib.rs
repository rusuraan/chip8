use thiserror::Error;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const TIMER_HZ: usize = 60;

const MEMORY_BYTES: usize = 4096;
const REGISTER_COUNT: usize = 16;

const PROGRAM_COUNTER_START_ADDRESS: usize = 0x200;

const FONTSET_START_ADDRESS: usize = 0x50; // Conventional start point of font data
const FONTSET: [u8; 80] = [
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

type Result<T> = std::result::Result<T, Chip8Error>;

#[derive(Debug, Error)]
pub enum Chip8Error {
    #[error("ROM too large: {rom_size} bytes exceeds available memory of {available} bytes")]
    RomTooLarge { rom_size: usize, available: usize },

    #[error("unknown opcode: {0:#06X}")]
    UnknownOpcode(u16),
}

pub struct Chip8 {
    memory: [u8; MEMORY_BYTES],
    framebuffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    program_counter: u16,
    index_register: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; REGISTER_COUNT],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_BYTES];
        memory[FONTSET_START_ADDRESS..][..FONTSET.len()].copy_from_slice(&FONTSET);

        Self {
            memory,
            framebuffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            program_counter: PROGRAM_COUNTER_START_ADDRESS as u16,
            index_register: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; REGISTER_COUNT],
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        let available_memory = MEMORY_BYTES - PROGRAM_COUNTER_START_ADDRESS;
        if rom.len() > available_memory {
            return Err(Chip8Error::RomTooLarge {
                rom_size: rom.len(),
                available: available_memory,
            });
        }

        self.memory[PROGRAM_COUNTER_START_ADDRESS..][..rom.len()].copy_from_slice(rom);

        Ok(())
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn step(&mut self) -> Result<()> {
        let opcode = self.fetch();
        self.program_counter += 2;
        self.execute(opcode)
    }

    fn fetch(&self) -> u16 {
        let hi = self.memory[self.program_counter as usize] as u16;
        let lo = self.memory[(self.program_counter + 1) as usize] as u16;
        hi << 8 | lo
    }

    fn execute(&mut self, opcode: u16) -> Result<()> {
        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F),
        );

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00E0(),
            _ => return Err(Chip8Error::UnknownOpcode(opcode)),
        }
    }

    fn op_00E0(&mut self) -> Result<()> {
        self.framebuffer.fill(false);
        Ok(())
    }
}
