use chip8::{self, Chip8};
use minifb::{Key, Scale, Window, WindowOptions};
use rodio::source::{SineWave, Source};
use std::{
    fs, process,
    time::{Duration, Instant},
};

const WINDOW_NAME: &str = "CHIP-8";
const OPCODE_HZ: usize = 700;
const BEEP_HZ: f32 = 440.0;
const REFRESH_RATE: usize = 60;
const WINDOW_SCALE: Scale = Scale::X16;
const VOLUME: f32 = 0.2;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rom>", args[0]);
        process::exit(1);
    }

    if let Err(e) = run(&args[1]) {
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

fn map_key(key: Key) -> Option<u8> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xC),
        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),
        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}

fn run(rom_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut chip8 = Chip8::new();
    let rom = fs::read(rom_path)?;
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

    let sink_handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = rodio::Player::connect_new(sink_handle.mixer());
    let source = SineWave::new(BEEP_HZ).amplify(VOLUME);
    player.append(source);
    player.pause();

    let cpu_dt = Duration::from_secs_f64(1.0 / OPCODE_HZ as f64);
    let timer_dt = Duration::from_secs_f64(1.0 / chip8::TIMER_HZ as f64);

    let mut cpu_acc = Duration::ZERO;
    let mut timer_acc = Duration::ZERO;

    let mut last = Instant::now();

    let mut buffer = framebuffer_to_u32(chip8.framebuffer());
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now - last;
        last = now;

        let mut key_state = [false; chip8::KEY_COUNT];
        for key in window.get_keys() {
            if let Some(chip8_key) = map_key(key) {
                key_state[chip8_key as usize] = true;
            }
        }
        chip8.set_keys(&key_state);

        cpu_acc += dt;
        timer_acc += dt;

        if chip8.should_beep() {
            player.play();
        } else {
            player.pause();
        }

        while cpu_acc >= cpu_dt {
            chip8.step()?;
            cpu_acc -= cpu_dt;
        }

        while timer_acc >= timer_dt {
            chip8.tick_timers();
            timer_acc -= timer_dt;
        }

        if chip8.take_draw_flag() {
            buffer = framebuffer_to_u32(chip8.framebuffer());
        }

        window.update_with_buffer(&buffer, chip8::SCREEN_WIDTH, chip8::SCREEN_HEIGHT)?;
    }

    Ok(())
}
