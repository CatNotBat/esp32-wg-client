# ESP32 Blink Project - Detailed Explanation

This document explains every aspect of this bare-metal Rust project for ESP32.

---

## Project Overview

This is a **bare-metal** (no operating system) Rust application that blinks the built-in LED on an ESP32 microcontroller. It demonstrates:
- Hardware Abstraction Layer (HAL) usage
- GPIO control
- UART logging
- The Rust `no_std` embedded ecosystem

**Target Hardware:** ESP32-D0WD-V3 (Xtensa LX6 dual-core)
**Built-in LED:** GPIO 2

---

## File Structure

```
esp32/
├── .cargo/
│   └── config.toml    # Cargo build configuration
├── src/
│   └── main.rs        # Application code
├── Cargo.toml         # Dependencies and project metadata
├── CLAUDE.md          # Project instructions for AI assistant
└── EXPLANATION.md     # This file
```

---

## src/main.rs Explained

### Crate-Level Attributes

```rust
#![no_std]
#![no_main]
```

**`#![no_std]`** - Tells the Rust compiler NOT to link the standard library (`std`).
- Why? The standard library assumes an operating system (file I/O, threads, heap allocation via OS).
- On bare-metal ESP32, there's no OS - we run directly on the hardware.
- Instead, we use `core` (language primitives) and `alloc` (optional heap allocation).

**`#![no_main]`** - Tells the compiler we define our own entry point.
- Normal Rust programs start at `fn main()` after the C runtime initializes.
- On bare-metal, there's no C runtime. The `#[esp_hal::main]` macro handles startup.

### Imports

```rust
use esp_backtrace as _;
```
- Imports the panic handler but doesn't use it directly (hence `as _`).
- When your code panics, this crate prints a stack trace over UART.
- The `as _` syntax means "import for side effects only" (it registers the panic handler).

```rust
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    time::Duration,
};
```
- **`Delay`** - Provides blocking delays (busy-wait loops).
- **`Level`** - Enum with `High` (3.3V) and `Low` (0V) for GPIO state.
- **`Output`** - Configures a GPIO pin as an output.
- **`OutputConfig`** - Configuration options for output pins (push-pull, open-drain, etc.).
- **`Duration`** - Time duration type for delays.

```rust
use log::info;
```
- The `log` crate provides logging macros (`info!`, `debug!`, `error!`, etc.).
- `esp_println` implements the actual printing over UART.

### Entry Point

```rust
#[esp_hal::main]
fn main() -> ! {
```

**`#[esp_hal::main]`** - This procedural macro:
1. Sets up the interrupt vector table
2. Initializes the stack pointer
3. Configures CPU clocks
4. Calls your `main()` function

**`-> !`** - The "never" return type. This function runs forever and never returns.
- Bare-metal code must never return (there's nowhere to return to).
- The `loop {}` at the end ensures this.

### Hardware Initialization

```rust
let peripherals = esp_hal::init(esp_hal::Config::default());
```

- Initializes all ESP32 hardware peripherals.
- Returns a `Peripherals` struct containing every peripheral (GPIO, UART, SPI, I2C, etc.).
- **Ownership:** In Rust, this struct "owns" all hardware. You can't access a peripheral without taking it from this struct, preventing conflicts.

```rust
esp_println::logger::init_logger(log::LevelFilter::Info);
```

- Initializes the UART-based logger.
- `LevelFilter::Info` means only `info!`, `warn!`, and `error!` messages are printed.
- Use `LevelFilter::Debug` or `LevelFilter::Trace` for more verbose output.

### GPIO Configuration

```rust
let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
```

**`Output::new()`** takes 3 arguments:
1. **`peripherals.GPIO2`** - Takes ownership of GPIO pin 2 from the peripherals struct.
2. **`Level::Low`** - Initial state: 0V (LED off).
3. **`OutputConfig::default()`** - Default configuration (push-pull output mode).

**`let mut`** - The `mut` keyword allows us to modify `led` (call `set_high()`/`set_low()`).

**Push-Pull vs Open-Drain:**
- **Push-pull** (default): Pin actively drives both high (3.3V) and low (0V).
- **Open-drain**: Pin can only pull low; needs external pull-up for high.

### Delay Provider

```rust
let delay = Delay::new();
```

- Creates a delay provider that uses busy-waiting.
- Busy-waiting means the CPU spins in a loop counting cycles.
- For power-sensitive applications, you'd use sleep modes instead.

### Main Loop

```rust
loop {
    info!("LED ON");
    led.set_high();
    delay.delay(Duration::from_millis(500));

    info!("LED OFF");
    led.set_low();
    delay.delay(Duration::from_millis(500));
}
```

- **`loop {}`** - Infinite loop (required for bare-metal).
- **`info!("LED ON")`** - Prints "INFO - LED ON" over UART at 115200 baud.
- **`led.set_high()`** - Sets GPIO2 to 3.3V, turning the LED on.
- **`delay.delay(Duration::from_millis(500))`** - Waits 500 milliseconds.
- **`led.set_low()`** - Sets GPIO2 to 0V, turning the LED off.

---

## Cargo.toml Explained

### Package Metadata

```toml
[package]
name = "esp32-blink"
version = "0.1.0"
edition = "2021"
```

- **`name`** - Crate name (used for the output binary).
- **`version`** - Semantic version of your project.
- **`edition`** - Rust edition (2021 is current; affects language features).

### Dependencies

```toml
[dependencies]
esp-hal = { version = "1.0.0", features = ["esp32", "unstable"] }
```

**esp-hal** - Hardware Abstraction Layer for ESP chips.
- **`esp32` feature** - Enables ESP32-specific code (vs ESP32-S3, ESP32-C3, etc.).
- **`unstable` feature** - Enables APIs that may change (like `Delay`).

```toml
esp-backtrace = { version = "0.15.0", features = ["esp32", "panic-handler", "println"] }
```

**esp-backtrace** - Panic handler that prints stack traces.
- **`panic-handler`** - Registers as the panic handler.
- **`println`** - Uses `esp-println` for output.

```toml
esp-println = { version = "0.13.1", features = ["esp32", "log"] }
```

**esp-println** - UART printing support.
- **`log` feature** - Implements the `log` crate's logging facade.

```toml
log = "0.4"
```

**log** - Rust's standard logging facade.
- Provides `info!`, `debug!`, `warn!`, `error!` macros.
- The actual output is handled by `esp-println`.

### Release Profile

```toml
[profile.release]
opt-level = "s"
lto = "fat"
codegen-units = 1
```

- **`opt-level = "s"`** - Optimize for size (important on microcontrollers with limited flash).
- **`lto = "fat"`** - Link-Time Optimization across all crates (smaller binary, slower compile).
- **`codegen-units = 1`** - Compile as single unit (better optimization, slower compile).

### esp-println Configuration

```toml
[package.metadata.esp-println]
uart_baudrate = "115_200"
```

- Sets UART baud rate to 115200 for serial output.
- Your terminal must use the same baud rate to read the output.

---

## .cargo/config.toml Explained

### Build Target

```toml
[build]
target = "xtensa-esp32-none-elf"
```

- **`xtensa`** - CPU architecture (ESP32's Tensilica Xtensa LX6).
- **`esp32`** - Specific chip variant.
- **`none`** - No operating system (bare-metal).
- **`elf`** - Output format (ELF executable).

### Target-Specific Configuration

```toml
[target.xtensa-esp32-none-elf]
runner = "espflash flash --monitor --ignore-app-descriptor"
```

- **`runner`** - Command to run when you execute `cargo run`.
- **`espflash flash`** - Flashes the binary to the ESP32.
- **`--monitor`** - Opens serial monitor after flashing.
- **`--ignore-app-descriptor`** - Skips ESP-IDF app descriptor check (needed for bare-metal).

```toml
rustflags = [
  "-C", "link-arg=-Tlinkall.x",
]
```

- **`-C link-arg=-Tlinkall.x`** - Passes linker script to the linker.
- **`linkall.x`** - Provided by `esp-hal`; defines memory layout (where code/data goes in flash/RAM).

### Unstable Features

```toml
[unstable]
build-std = ["core", "alloc"]
```

- **`build-std`** - Recompiles Rust's core libraries for our target.
- **`core`** - Language primitives (Option, Result, slices, etc.).
- **`alloc`** - Heap allocation (Vec, Box, String) - optional, not used in this project.
- Why needed? Pre-compiled `core` doesn't exist for Xtensa; we must build it ourselves.

---

## Build Commands

### Build Only
```bash
cargo build --release
```
- Compiles the project without flashing.
- Output: `target/xtensa-esp32-none-elf/release/esp32-blink`

### Build and Flash
```bash
cargo espflash flash --release --monitor --ignore-app-descriptor
```
- Builds, flashes to ESP32, and opens serial monitor.
- Requires ESP32 connected via USB.

### Monitor Only
```bash
screen /dev/ttyUSB0 115200
```
- Opens serial monitor at 115200 baud.
- Exit with `CTRL+A` then `K` then `Y`.

---

## Key Rust Concepts Used

### Ownership
```rust
let peripherals = esp_hal::init(...);
let mut led = Output::new(peripherals.GPIO2, ...);
```
- `peripherals` owns all hardware.
- `Output::new()` takes ownership of `GPIO2`, moving it out of `peripherals`.
- You can't accidentally use `GPIO2` elsewhere - the compiler prevents it.

### Mutability
```rust
let mut led = ...;
led.set_high();
```
- `mut` is required to modify the LED state.
- Without `mut`, calling `set_high()` would be a compile error.

### The Never Type (`!`)
```rust
fn main() -> ! {
    loop { ... }
}
```
- `!` means this function never returns.
- Required for bare-metal entry points.

### Zero-Cost Abstractions
- `Output`, `Delay`, etc. compile down to direct register writes.
- No runtime overhead compared to writing raw register manipulation in C.

---

## Troubleshooting

### Linker not found
```
error: linker `xtensa-esp32-elf-gcc` not found
```
Solution: Source the ESP toolchain environment:
```bash
source ~/export-esp.sh
```

### Permission denied on /dev/ttyUSB0
Solution: Add yourself to the `uucp` group (Arch) or `dialout` group (Ubuntu):
```bash
sudo usermod -aG uucp $USER
# Log out and back in
```

### Garbled serial output
- Ensure terminal baud rate matches (115200).
- Try: `screen /dev/ttyUSB0 115200`
