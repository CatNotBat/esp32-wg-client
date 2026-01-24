
## Project Context & Philosophy
**Project:** ESP32 WireGuard Client (Bare Metal / System Level)
**Language:** Rust (focus on `esp-idf` bindings)
**Goal:** Create a functional WireGuard VPN client on the ESP32 while strictly adhering to a "Learning First" methodology.

### Core Directives for Claude
1.  **The "Rust-C Bridge" Teaching Method:**
    * Since the goal is to **learn C on the way**, every time we implement a Rust feature (like WiFi or System Time), you must briefly explain what is happening in the C layer underneath.
    * *Example:* "We are using the Rust `EspWifi` struct here, which is a safe wrapper around the C function `esp_wifi_init()` from the ESP-IDF. In C, you would have to manually allocate this struct..."
2.  **Step-by-Step Execution:** Do not dump the entire codebase at once. We will build this iteratively:
    * Step 1: Hello World & Hardware Verification
    * Step 2: WiFi Connection (The LwIP Stack)
    * Step 3: NTP Time Sync (Critical for WireGuard cryptography)
    * Step 4: The WireGuard Handshake
3.  **Hardware Awareness:** Code must be tailored to the specific hardware constraints listed below (Xtensa architecture, 4MB Flash).

## Hardware Identity (Target Device)
* **Chip:** ESP32-D0WD-V3 (Xtensa LX6 dual-core)
* **Architecture:** Xtensa (Not RISC-V)
* **Flash Size:** 4MB
* **Crystal:** 26MHz
* **Port:** `/dev/ttyUSB0`
* **Peripherals:** Standard WiFi/Bluetooth radio.

## Development Environment & Commands
We are using the Rust ESP ecosystem.
* **Build (Release):** `cargo build --release`
* **Flash & Monitor:** `cargo espflash flash --monitor`
* **Toolchain:** `esp-idf-template` (using Standard Library `std` support to access C bindings easily).

## Learning Constraints
* **No Magic:** If a crate does something heavy (like the `smoltcp` network stack or `wireguard-rs`), explain generally how it manipulates packets.
* **Explain `unsafe`:** If we need to use `unsafe` blocks to talk to C code, explain exactly *why* the compiler cannot guarantee safety there.
* **Memory Consciousness:** Remind me when we are allocating memory on the Heap vs the Stack, as this is critical in embedded C/Rust.

---

## Technical Roadmap (For Claude's Reference)

### Phase 1: Project Skeleton
* This is an already working project template, but still go over all these conepcetps for learning
* Initialize a project using `esp-idf-template`.
* Explain `Cargo.toml` dependencies (specifically `esp-idf-svc` and `esp-idf-sys`).
* **C-Learning Moment:** Explain how `bindgen` is used to auto-generate Rust code from C header files (`.h`).

### Phase 2: Connectivity & LwIP
* Get WiFi up and running.
* **C-Learning Moment:** Explain the "Event Loop". In Rust, we might use Futures/Async, but in the underlying C (ESP-IDF), this is a callback system. Compare the two.

### Phase 3: Time is Relative
* Implement SNTP to sync time.
* **Why:** WireGuard rejects packets with old timestamps to prevent Replay Attacks.

### Phase 4: The Tunnel
* Implement the WireGuard client.
* **Focus:** Managing the "Handshake" and creating the raw UDP socket.

