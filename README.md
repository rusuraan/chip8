# chip8
A CHIP-8 interpreter written in Rust.

## Overview
This project is organized as a Cargo workspace with two crates:
- **`chip8-core`** - platform-agnostic emulator logic (CPU, memory, display, timers, opcode decoding)
- **`chip8-desktop`** - desktop frontend using **`minifb`** (windowing/input) and **`rodio`** (audio)

## Structure
```
chip8/
├── Cargo.toml
├── chip8-core/
│   └── src/
└── chip8-desktop/
    └── src/
```

## Building
Targets modern Linux (tested on Fedora 44). Requires ALSA development headers.
```sh
# Clone the repo
git clone https://github.com/rusuraan/chip8.git
cd chip8

# Build everything
cargo build --release

# Run a ROM
cargo run --release -p chip8-desktop -- path/to/rom.ch8
```
## ROMs
**`roms/`** includes a test suite and Pong.

## License
Licensed under the [GNU Affero General Public License v3.0](LICENSE).
