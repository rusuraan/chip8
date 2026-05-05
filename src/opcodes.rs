use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    cpu::Chip8,
    cpu::{FONTSET_START_ADDRESS, KEY_COUNT},
    error::{Chip8Error, Result},
};

impl Chip8 {
    pub(crate) fn op_00e0(&mut self) -> Result<()> {
        self.framebuffer.fill(false);
        self.draw_flag = true;
        Ok(())
    }

    pub(crate) fn op_00ee(&mut self) -> Result<()> {
        self.program_counter = self.stack.pop().ok_or(Chip8Error::StackUnderflow)?;
        Ok(())
    }

    pub(crate) fn op_1nnn(&mut self, nnn: u16) -> Result<()> {
        self.program_counter = nnn;
        Ok(())
    }

    pub(crate) fn op_2nnn(&mut self, nnn: u16) -> Result<()> {
        self.stack.push(self.program_counter);
        self.program_counter = nnn;
        Ok(())
    }

    pub(crate) fn op_3xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        if self.registers[x] == nn {
            self.program_counter += 2;
        }
        Ok(())
    }

    pub(crate) fn op_4xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        if self.registers[x] != nn {
            self.program_counter += 2;
        }
        Ok(())
    }

    pub(crate) fn op_5xy0(&mut self, x: usize, y: usize) -> Result<()> {
        if self.registers[x] == self.registers[y] {
            self.program_counter += 2;
        }
        Ok(())
    }

    pub(crate) fn op_6xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        self.registers[x] = nn;
        Ok(())
    }

    pub(crate) fn op_7xnn(&mut self, x: usize, nn: u8) -> Result<()> {
        self.registers[x] = self.registers[x].wrapping_add(nn);
        Ok(())
    }

    pub(crate) fn op_8xy0(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] = self.registers[y];
        Ok(())
    }

    pub(crate) fn op_8xy1(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] |= self.registers[y];
        self.registers[0xF] = 0;
        Ok(())
    }

    pub(crate) fn op_8xy2(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] &= self.registers[y];
        self.registers[0xF] = 0;
        Ok(())
    }

    pub(crate) fn op_8xy3(&mut self, x: usize, y: usize) -> Result<()> {
        self.registers[x] ^= self.registers[y];
        self.registers[0xF] = 0;
        Ok(())
    }

    pub(crate) fn op_8xy4(&mut self, x: usize, y: usize) -> Result<()> {
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = overflow as u8;
        Ok(())
    }

    pub(crate) fn op_8xy5(&mut self, x: usize, y: usize) -> Result<()> {
        let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = !overflow as u8;
        Ok(())
    }

    pub(crate) fn op_8xy6(&mut self, x: usize, y: usize) -> Result<()> {
        if !self.quirk_config.shift {
            self.registers[x] = self.registers[y];
        }

        let shifted_bit = self.registers[x] & 0x1;
        self.registers[x] >>= 1;
        self.registers[0xF] = shifted_bit;
        Ok(())
    }

    pub(crate) fn op_8xy7(&mut self, x: usize, y: usize) -> Result<()> {
        let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[x] = result;
        self.registers[0xF] = !overflow as u8;
        Ok(())
    }

    pub(crate) fn op_8xye(&mut self, x: usize, y: usize) -> Result<()> {
        if !self.quirk_config.shift {
            self.registers[x] = self.registers[y];
        }

        let shifted_bit = (self.registers[x] >> 7) & 0x1;
        self.registers[x] <<= 1;
        self.registers[0xF] = shifted_bit;
        Ok(())
    }

    pub(crate) fn op_9xy0(&mut self, x: usize, y: usize) -> Result<()> {
        if self.registers[x] != self.registers[y] {
            self.program_counter += 2;
        }
        Ok(())
    }

    pub(crate) fn op_annn(&mut self, nnn: u16) -> Result<()> {
        self.index_register = nnn;
        Ok(())
    }

    pub(crate) fn op_bnnn(&mut self, nnn: u16) -> Result<()> {
        let base = if self.quirk_config.jumping {
            let x = (nnn as usize & 0xF00) >> 8;
            self.registers[x] as u16
        } else {
            self.registers[0] as u16
        };
        self.program_counter = nnn + base;
        Ok(())
    }

    pub(crate) fn op_cxnn(&mut self, x: usize, nn: u8) -> Result<()> {
        let random_byte: u8 = rand::random();
        self.registers[x] = random_byte & nn;
        Ok(())
    }

    pub(crate) fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> Result<()> {
        if self.waiting_vblank {
            self.program_counter -= 2;
            return Ok(());
        }
        self.waiting_vblank = true;

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

    pub(crate) fn op_ex9e(&mut self, x: usize) -> Result<()> {
        if self.keypad[self.registers[x] as usize] {
            self.program_counter += 2;
        }
        Ok(())
    }

    pub(crate) fn op_exa1(&mut self, x: usize) -> Result<()> {
        if !self.keypad[self.registers[x] as usize] {
            self.program_counter += 2;
        }
        Ok(())
    }

    pub(crate) fn op_fx0a(&mut self, x: usize) -> Result<()> {
        for i in 0..KEY_COUNT {
            if self.last_keypad[i] && !self.keypad[i] {
                self.registers[x] = i as u8;
                return Ok(());
            }
        }

        self.program_counter -= 2;
        Ok(())
    }

    pub(crate) fn op_fx07(&mut self, x: usize) -> Result<()> {
        self.registers[x] = self.delay_timer;
        Ok(())
    }

    pub(crate) fn op_fx15(&mut self, x: usize) -> Result<()> {
        self.delay_timer = self.registers[x];
        Ok(())
    }

    pub(crate) fn op_fx18(&mut self, x: usize) -> Result<()> {
        self.sound_timer = self.registers[x];
        Ok(())
    }

    pub(crate) fn op_fx1e(&mut self, x: usize) -> Result<()> {
        self.index_register += self.registers[x] as u16;
        Ok(())
    }

    pub(crate) fn op_fx29(&mut self, x: usize) -> Result<()> {
        self.index_register = FONTSET_START_ADDRESS as u16 + 5 * self.registers[x] as u16;
        Ok(())
    }

    pub(crate) fn op_fx33(&mut self, x: usize) -> Result<()> {
        let addr = self.index_register as usize;
        self.memory[addr] = self.registers[x] / 100;
        self.memory[addr + 1] = (self.registers[x] / 10) % 10;
        self.memory[addr + 2] = self.registers[x] % 10;
        Ok(())
    }

    pub(crate) fn op_fx55(&mut self, x: usize) -> Result<()> {
        for i in 0..=x {
            self.memory[self.index_register as usize + i] = self.registers[i];
        }
        if !self.quirk_config.load_store {
            self.index_register += x as u16 + 1;
        }
        Ok(())
    }

    pub(crate) fn op_fx65(&mut self, x: usize) -> Result<()> {
        for i in 0..=x {
            self.registers[i] = self.memory[self.index_register as usize + i];
        }
        if !self.quirk_config.load_store {
            self.index_register += x as u16 + 1;
        }

        Ok(())
    }
}
