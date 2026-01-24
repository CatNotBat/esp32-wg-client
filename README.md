# ESP32 WireGuard Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A WireGuard VPN client for the ESP32, written in Rust.

## Hardware

| Component | Specification |
|-----------|---------------|
| Chip | ESP32-D0WD-V3 (Xtensa LX6 dual-core) |
| Flash | 4MB |
| Crystal | 26MHz |
| Connection | USB via `/dev/ttyUSB0` |

## Setup

### Install Toolchain

```bash
# Install ESP Rust toolchain
cargo install espup
espup install
source ~/export-esp.sh  # Add to .bashrc/.zshrc

# Install flash tool and linker proxy
cargo install espflash ldproxy

# System dependencies (Arch Linux)
sudo pacman -S ninja libxml2
```

### Configure Credentials

```bash
cp src/config.rs.example src/config.rs
# Edit src/config.rs with your WiFi and WireGuard credentials
```

### Generate WireGuard Keys

```bash
# Generate ESP32 keypair
wg genkey | tee esp32_private.key | wg pubkey > esp32_public.key

# Add to your WireGuard server (/etc/wireguard/wg0.conf):
# [Peer]
# PublicKey = <esp32_public.key contents>
# AllowedIPs = 10.0.0.2/32

sudo wg syncconf wg0 <(wg-quick strip wg0)
```

### Build & Flash

```bash
cargo build --release
cargo run --release
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Your Rust Code (src/main.rs)              │
│   - WireGuardTunnel wrapper, WiFi, NTP, tunnel management   │
├─────────────────────────────────────────────────────────────┤
│   esp-idf-svc (Rust wrappers for ESP-IDF services)          │
│   - EspWifi, BlockingWifi, EspSntp                          │
├─────────────────────────────────────────────────────────────┤
│   esp-idf-sys (Auto-generated C bindings via bindgen)       │
│   - sys::wireguard::* from components/esp_wireguard         │
├─────────────────────────────────────────────────────────────┤
│   ESP-IDF + Components (C Framework)                        │
│   ┌──────────┬──────────┬──────────┬──────────────────┐     │
│   │ FreeRTOS │   LwIP   │  WiFi    │  esp_wireguard   │     │
│   │ (RTOS)   │ (TCP/IP) │ (Driver) │  (WG Protocol)   │     │
│   └──────────┴──────────┴──────────┴──────────────────┘     │
├─────────────────────────────────────────────────────────────┤
│              ESP32 Hardware (Xtensa LX6)                    │
└─────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Rust-C Bridge (FFI)

This project demonstrates calling C code from Rust using Foreign Function Interface (FFI):

- **bindgen**: Auto-generates Rust bindings from C header files (`.h`). The `esp-idf-sys` crate runs bindgen during build to create `sys::*` modules from ESP-IDF headers.

- **`#[repr(C)]`**: Ensures Rust structs use C memory layout (field order, padding alignment).

- **`CString`**: Rust's null-terminated string type for passing strings to C. Rust's `&str` is not null-terminated.

- **`*mut T` / `*const T`**: Raw pointers equivalent to C's `T*`. Unlike Rust references, the compiler cannot verify their validity.

- **`unsafe` blocks**: Required when calling C functions because Rust cannot verify C code's memory safety guarantees.

### ESP-IDF Event System

ESP-IDF uses a callback-based event system in C:
```c
esp_event_handler_register(WIFI_EVENT, ESP_EVENT_ANY_ID, &wifi_handler, NULL);
```

The `esp-idf-svc` Rust wrappers convert this into blocking calls using FreeRTOS semaphores. `BlockingWifi::connect()` internally registers callbacks and waits on a semaphore until the connection event fires.

### LwIP (Lightweight IP)

The ESP32's TCP/IP stack. When WiFi connects, LwIP handles:
- DHCP client (automatic IP assignment)
- DNS resolution
- TCP/UDP sockets
- Routing tables

WireGuard creates a virtual network interface (`wg0`) in LwIP and sends encrypted UDP packets through the physical WiFi interface.

### NTP and WireGuard Timestamps

WireGuard uses TAI64N timestamps in handshakes to prevent replay attacks. The server rejects packets with timestamps older than the last received. Without NTP sync, the ESP32 boots at Unix epoch 0 (Jan 1, 1970), causing all handshakes to fail.

### The WireGuard Handshake

```
┌─────────────┐                           ┌─────────────┐
│   ESP32     │                           │  VPN Server │
│ (Initiator) │                           │ (Responder) │
└──────┬──────┘                           └──────┬──────┘
       │  1. Initiation: encrypted(pubkey,       │
       │     timestamp) + MAC                    │
       │────────────────────────────────────────>│
       │                                         │
       │  2. Response: encrypted(ephemeral_key)  │
       │     + MAC                               │
       │<────────────────────────────────────────│
       │                                         │
       │  3. Session keys derived, tunnel UP     │
       │<==========encrypted UDP data===========>│
```

Cryptographic primitives:
- **Curve25519**: Elliptic curve Diffie-Hellman key exchange
- **ChaCha20-Poly1305**: Authenticated encryption (AEAD)
- **BLAKE2s**: Cryptographic hashing

### Memory Layout

| Region | Size | Usage |
|--------|------|-------|
| Flash | 4MB | Code + Read-only data |
| DRAM | ~320KB | Heap + Stack + BSS |
| IRAM | ~128KB | Interrupt handlers |

Stack allocations (local variables) are fast but limited. Heap allocations (`Box`, `Vec`, `String`) are flexible but require the allocator. In embedded, heap fragmentation can cause out-of-memory failures even with free space available.

## Resources

- [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/)
- [esp-idf-svc Documentation](https://docs.rs/esp-idf-svc)
- [WireGuard Protocol](https://www.wireguard.com/protocol/)
- [Rust on ESP Book](https://docs.esp-rs.org/book/)
