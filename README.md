# llc-controller
A simple LLC controller - STM32F411 Embedded Rust Project

## Prerequisites

1. **Rust toolchain with embedded support:**
   ```bash
   rustup target add thumbv7em-none-eabihf
   ```

2. **cargo-embed for flashing and debugging:**
   ```bash
   cargo install cargo-embed
   ```

3. **Optional - ARM GCC toolchain (for advanced debugging):**
   ```bash
   # On macOS with Homebrew:
   brew install --cask gcc-arm-embedded
   ```

## Hardware Setup

This project is configured for the STM32F411 microcontroller with:
- 512KB Flash memory
- 128KB RAM
- 25MHz external crystal (HSE)
- LED connected to PC13 (common on development boards)

## Building

To build the project:
```bash
cargo build --release
```

## Flashing and Running

### Using cargo-embed (Recommended)

1. **Flash the firmware:**
   ```bash
   cargo embed --release
   ```

2. **Flash and monitor RTT output:**
   ```bash
   cargo embed --release
   ```
   This will flash the firmware and open an RTT terminal for debugging output.

### Alternative Methods

#### Using cargo-flash
```bash
cargo install cargo-flash
cargo flash --chip STM32F411CEUx --release
```

#### Using OpenOCD directly
1. Start OpenOCD:
   ```bash
   openocd -f openocd.cfg
   ```
2. In another terminal:
   ```bash
   cargo run --release
   ```

## Debugging

### With cargo-embed
1. Enable GDB in `Embed.toml` by setting `enabled = true` under `[default.gdb]`
2. Run:
   ```bash
   cargo embed --release
   ```
3. Connect with GDB in another terminal:
   ```bash
   arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/llc-controller
   (gdb) target remote :1337
   ```

### Real-Time Transfer (RTT)
The project is configured for RTT debugging. Add RTT output to your code:
```rust
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, RTT!");
    // ... rest of your code
}
```

## Project Structure

- `src/main.rs` - Main application code (LED blinky example)
- `Embed.toml` - cargo-embed configuration
- `memory.x` - Memory layout for STM32F411
- `.cargo/config.toml` - Cargo configuration for embedded target
- `openocd.cfg` - OpenOCD configuration (backup method)
- `build.rs` - Build script for memory layout

## Configuration

The `Embed.toml` file configures:
- **Chip**: STM32F411CEUx (change if using different variant)
- **Protocol**: SWD for ST-Link
- **RTT**: Enabled for debugging output
- **Flashing**: Automatic with reset

## Troubleshooting

- **Probe not found**: Ensure ST-Link drivers are installed and board is connected
- **Flash errors**: Check that the correct chip variant is specified in `Embed.toml`
- **RTT not working**: Make sure `rtt-target` is added to dependencies if using RTT
- **Permission errors**: On Linux, add your user to the `dialout` group 
