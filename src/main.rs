use chip8::{self, Chip8};
use minifb::{Key, Scale, Window, WindowOptions};
use std::{
    fs, process,
    time::{Duration, Instant},
};

const WINDOW_NAME: &str = "CHIP-8";
const OPCODE_HZ: usize = 700;
const REFRESH_RATE: usize = 60;
const WINDOW_SCALE: Scale = Scale::X16;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn framebuffer_to_u32(framebuffer: &[bool]) -> Vec<u32> {
    framebuffer
        .iter()
        .map(|&on| if on { 0x00FFFFFF } else { 0x00000000 })
        .collect()
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut chip8 = Chip8::new();
    let rom = fs::read("roms/chip8-test-suite/5-quirks.ch8")?;
    chip8.load_rom(&rom)?;

    let mut window = Window::new(
        WINDOW_NAME,
        chip8::SCREEN_WIDTH,
        chip8::SCREEN_HEIGHT,
        WindowOptions {
            scale: WINDOW_SCALE,
            ..Default::default()
        },
    )?;

    window.set_target_fps(REFRESH_RATE);

    let cpu_dt = Duration::from_secs_f64(1.0 / OPCODE_HZ as f64);
    let timer_dt = Duration::from_secs_f64(1.0 / chip8::TIMER_HZ as f64);

    let mut cpu_acc = Duration::ZERO;
    let mut timer_acc = Duration::ZERO;

    let mut last = Instant::now();

    let mut buffer = framebuffer_to_u32(chip8.get_framebuffer());
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

        if chip8.draw_flag() {
            buffer = framebuffer_to_u32(chip8.get_framebuffer());
            chip8.clear_draw_flag();
        }

        window.update_with_buffer(&buffer, chip8::SCREEN_WIDTH, chip8::SCREEN_HEIGHT)?;
    }

    Ok(())
}
