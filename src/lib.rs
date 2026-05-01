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

    #[error("stack underflow")]
    StackUnderflow,
}

#[derive(Default)]
pub struct QuirkConfig {
    shift: bool,
    load_store: bool,
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
    quirk_config: QuirkConfig,
    draw_flag: bool,
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
            quirk_config: Default::default(),
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
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn draw_flag(&self) -> bool {
        self.draw_flag
    }

    pub fn clear_draw_flag(&mut self) {
        self.draw_flag = false;
    }

    pub fn get_framebuffer(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.framebuffer
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

        println!("Executing opcode: {opcode:#06X}");

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
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
            (0xD, _, _, _) => self.op_dxyn(x, y, n),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            _ => Err(Chip8Error::UnknownOpcode(opcode)),
        }
    }

    fn op_00e0(&mut self) -> Result<()> {
        self.framebuffer.fill(false);
        self.draw_flag = true;
        Ok(())
    }

    fn op_00ee(&mut self) -> Result<()> {
        self.program_counter = self.stack.pop().ok_or(Chip8Error::StackUnderflow)?;
        Ok(())
    }

    fn op_1nnn(&mut self, nnn: u16) -> Result<()> {
        self.program_counter = nnn;
        Ok(())
    }

    fn op_2nnn(&mut self, nnn: u16) -> Result<()> {
        self.stack.push(self.program_counter);
        self.program_counter = nnn;
        Ok(())
    }

    fn op_3xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        if self.registers[x] == nn {
            self.program_counter += 2;
        }
        Ok(())
    }

    fn op_4xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        if self.registers[x] != nn {
            self.program_counter += 2;
        }
        Ok(())
    }

    fn op_5xy0(&mut self, x: usize, y: usize) -> Result<()> {
        if self.registers[x] == self.registers[y] {
            self.program_counter += 2;
        }
        Ok(())
    }

    fn op_6xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        self.registers[x] = nn;
        Ok(())
    }

    fn op_7xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        self.registers[x] = self.registers[x].wrapping_add(nn);
        Ok(())
    }

    fn op_8xy0(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] = self.registers[y];
        Ok(())
    }

    fn op_8xy1(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] |= self.registers[y];
        Ok(())
    }

    fn op_8xy2(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] &= self.registers[y];
        Ok(())
    }

    fn op_8xy3(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] ^= self.registers[y];
        Ok(())
    }

    fn op_8xy4(&mut self, x: usize, y: usize) -> Result<()> {
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = overflow as u8;
        Ok(())
    }

    fn op_8xy5(&mut self, x: usize, y: usize) -> Result<()> {
        let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = !overflow as u8;
        Ok(())
    }

    fn op_8xy6(&mut self, x: usize, y: usize) -> Result<()> {
        if !self.quirk_config.shift {
            self.registers[x] = self.registers[y];
        }

        self.registers[0xF] = self.registers[x] & 0x1;
        self.registers[x] >>= 1;
        Ok(())
    }

    fn op_8xy7(&mut self, x: usize, y: usize) -> Result<()> {
        let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[x] = result;
        self.registers[0xF] = !overflow as u8;
        Ok(())
    }

    fn op_8xye(&mut self, x: usize, y: usize) -> Result<()> {
        if !self.quirk_config.shift {
            self.registers[x] = self.registers[y];
        }
        self.registers[0xF] = (self.registers[x] >> 7) & 0x1;
        self.registers[x] <<= 1;
        Ok(())
    }

    fn op_9xy0(&mut self, x: usize, y: usize) -> Result<()> {
        if self.registers[x] != self.registers[y] {
            self.program_counter += 2;
        }
        Ok(())
    }

    fn op_annn(&mut self, nnn: u16) -> Result<()> {
        self.index_register = nnn;
        Ok(())
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> Result<()> {
        let x_coord = self.registers[x] as usize % SCREEN_WIDTH;
        let y_coord = self.registers[y] as usize % SCREEN_HEIGHT;
        self.registers[0xF] = 0;

        for row in 0..n {
            let y_pos = y_coord + row;
            if y_pos >= SCREEN_HEIGHT {
                break;
            }

            let sprite_row = self.memory[(self.index_register as usize) + row];

            for col in 0..8 {
                let x_pos = x_coord + col;
                if x_pos >= SCREEN_WIDTH {
                    break;
                }

                if sprite_row & (0x80 >> col) != 0 {
                    let idx = y_pos * SCREEN_WIDTH + x_pos;
                    if self.framebuffer[idx] {
                        self.registers[0xF] = 1;
                    }

                    self.framebuffer[idx] ^= true;
                }
            }
        }

        self.draw_flag = true;
        Ok(())
    }

    fn op_fx29(&mut self, x: usize) -> Result<()> {
        self.index_register = FONTSET_START_ADDRESS as u16 + 5 * self.registers[x] as u16;
        Ok(())
    }

    fn op_fx55(&mut self, x: usize) -> Result<()> {
        for i in 0..x {
            self.registers[i] = self.memory[self.index_register as usize + i];
            if !self.quirk_config.load_store {
                self.index_register += x as u16 + 1;
            }
        }
        Ok(())
    }
}
