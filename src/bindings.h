/*
 * bindings.h - Header file for bindgen to generate Rust FFI bindings
 *
 * C-LEARNING MOMENT: What is bindgen?
 * ====================================
 * bindgen is a tool that reads C header files and automatically generates
 * Rust FFI (Foreign Function Interface) bindings. This saves us from
 * manually writing `extern "C"` declarations for every C function.
 *
 * When you see a Rust struct like:
 *   #[repr(C)]
 *   pub struct wireguard_config_t { ... }
 *
 * That was auto-generated from the C struct definition in esp_wireguard.h.
 *
 * The process:
 *   1. This file includes the C headers we want bindings for
 *   2. During `cargo build`, esp-idf-sys runs bindgen
 *   3. bindgen parses the C headers using libclang
 *   4. It outputs Rust code with equivalent types and function signatures
 *   5. We can then call the C functions from Rust using `unsafe` blocks
 */

/* Include the WireGuard API */
#include "esp_wireguard.h"
