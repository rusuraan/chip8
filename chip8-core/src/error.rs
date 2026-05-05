use thiserror::Error;

pub type Result<T> = std::result::Result<T, Chip8Error>;

#[derive(Debug, Error)]
pub enum Chip8Error {
    #[error("ROM too large: {rom_size} bytes exceeds available memory of {available} bytes")]
    RomTooLarge { rom_size: usize, available: usize },

    #[error("unknown opcode: {0:#06X}")]
    UnknownOpcode(u16),

    #[error("stack underflow")]
    StackUnderflow,
}
