mod config;
mod cpu;
mod error;
mod opcodes;

pub use config::QuirkConfig;
pub use cpu::Chip8;
pub use error::Chip8Error;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const TIMER_HZ: usize = 60;
