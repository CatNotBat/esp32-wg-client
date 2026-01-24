# Project: ESP32 Firmware

## Context
- **Target:** ESP32 wroom xx5r69
- **Language:** Rust (no_std)
- **Source of Truth:** `datasheet.txt`

## Commands
- **Build:** `cargo build --release`
- **Flash:** `cargo espflash flash --monitor`

## Hardware Identity (Verified via esptool)
- **Chip:** ESP32-D0WD-V3 (Xtensa LX6)
- **Flash Size:** 4MB
- **Crystal:** 26MHz
- **Port:** `/dev/ttyUSB0`

## Critical Pinout Rules (The "Clone" Warning)
- **WARNING:** This board (Keyestudio XX5R69) has misleading silkscreen labels.
- **Rule:** Do NOT trust board labels like "D0", "D1".
- **Rule:** Always map pins to the physical GPIO number in code.
- **Built-in LED:** GPIO 2 (Verify this first).

## Rules
1. **Source of Truth:** `datasheet.txt` exists but is **too large to read**.
2. **How to Consult:** Do NOT read the whole file. You MUST use `rg` (ripgrep) to search for specific terms.
   - Example: `rg -C 5 "GPIO 2" datasheet.txt` (Finds "GPIO 2" plus 5 lines of context).
   - Example: `rg "Register 0x" datasheet.txt` (Finds register addresses).
2. **No Guessing:** If the pin isn't in `datasheet.txt`, ask me.
3. **Logging:** Use `log::info!`, never `println!`.

## Learning Mode Instructions
While I have a background in C, I am learning Rust. When you answer:
1. **Explain First:** Explain the concept (e.g., Ownership, MMIO, Registers) before showing code.
2. **Comment Heavily:** Add comments explaining *why* a specific Rust feature (like `Option` or `&mut`) is used.
3. **No Magic:** Do not use complex macros without explaining what they expand to.