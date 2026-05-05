mod opcodes;

use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    config::QuirkConfig,
    error::{Chip8Error, Result},
};

pub const KEY_COUNT: usize = 16;

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

pub struct Chip8 {
    memory: [u8; MEMORY_BYTES],
    framebuffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    program_counter: u16,
    index_register: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; REGISTER_COUNT],
    quirk_config: QuirkConfig,
    keypad: [bool; KEY_COUNT],
    last_keypad: [bool; KEY_COUNT],
    waiting_vblank: bool,
    draw_flag: bool,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    pub fn new() -> Self {
        Self::with_config(QuirkConfig::default())
    }

    pub fn with_config(quirk_config: QuirkConfig) -> Self {
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
            quirk_config,
            keypad: [false; KEY_COUNT],
            last_keypad: [false; KEY_COUNT],
            waiting_vblank: false,
            draw_flag: false,
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
        self.waiting_vblank = false;
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn take_draw_flag(&mut self) -> bool {
        let flag = self.draw_flag;
        self.draw_flag = false;
        flag
    }

    pub fn should_beep(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn framebuffer(&self) -> &[bool] {
        &self.framebuffer
    }

    pub fn set_keys(&mut self, keys: &[bool; KEY_COUNT]) {
        self.last_keypad = self.keypad;
        self.keypad = *keys;
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

        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee()?,
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2, _, _, _) => self.op_2nnn(nnn),
            (0x3, _, _, _) => self.op_3xnn(x, nn),
            (0x4, _, _, _) => self.op_4xnn(x, nn),
            (0x5, _, _, 0x0) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),
            (0x8, _, _, 0x6) => self.op_8xy6(x, y),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(x, y),
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xC, _, _, _) => self.op_cxnn(x, nn),
            (0xD, _, _, _) => self.op_dxyn(x, y, n)?,
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x)?,
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),
            _ => return Err(Chip8Error::UnknownOpcode(opcode)),
        }
        Ok(())
    }
}
