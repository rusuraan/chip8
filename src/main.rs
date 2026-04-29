use chip8::{self, Chip8};
use minifb::{Key, Window, WindowOptions};
use std::{
    fs, process,
    time::{Duration, Instant},
};

const WINDOW_NAME: &str = "CHIP-8";
const OPCODE_HZ: usize = 600;
const REFRESH_RATE: usize = 60;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut chip8 = Chip8::new();
    let rom = fs::read("roms/IBM.ch8")?;
    chip8.load_rom(&rom)?;

    let mut window = Window::new(
        WINDOW_NAME,
        chip8::SCREEN_WIDTH,
        chip8::SCREEN_HEIGHT,
        WindowOptions::default(),
    )?;

    window.set_target_fps(REFRESH_RATE);

    let cpu_dt = Duration::from_secs_f64(1.0 / OPCODE_HZ as f64);
    let timer_dt = Duration::from_secs_f64(1.0 / chip8::TIMER_HZ as f64);

    let mut cpu_acc = Duration::ZERO;
    let mut timer_acc = Duration::ZERO;

    let mut last = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now - last;
        last = now;

        cpu_acc += dt;
        timer_acc += dt;

        while cpu_acc >= cpu_dt {
            chip8.step()?;
            cpu_acc -= cpu_dt;
        }

        while timer_acc >= timer_dt {
            chip8.tick_timers();
            timer_acc -= timer_dt;
        }

        window.update();
    }

    Ok(())
}
