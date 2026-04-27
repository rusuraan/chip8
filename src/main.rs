use chip8::Chip8;
use std::time::{Duration, Instant};
use std::{fs, thread};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom = fs::read("roms/IBM.ch8")?;
    let mut chip8 = Chip8::new();
    chip8.load_rom(&rom)?;

    let cycles_per_frame = 500 / 60;
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);

    loop {
        let frame_start = Instant::now();
        // handle_input()

        for _ in 0..cycles_per_frame {
            chip8.step().map_err(|e| e.to_string())?;
        }

        chip8.tick_timers();

        if let Some(remaining) = frame_duration.checked_sub(frame_start.elapsed()) {
            thread::sleep(remaining);
        }
    }
}
