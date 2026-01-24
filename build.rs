// build.rs - Runs during `cargo build` BEFORE compiling your Rust code
//
// C-LEARNING MOMENT: The ESP-IDF Build System
// ============================================
// ESP-IDF is a massive C project (~1 million lines). It uses:
//   1. CMake - Generates build files (Makefiles or Ninja files)
//   2. Ninja - Actually compiles the C code (faster than Make)
//   3. Kconfig - Configuration system (the menuconfig you might have seen)
//
// The `embuild` crate orchestrates all of this:
//   - Downloads ESP-IDF if needed
//   - Runs CMake to configure the build
//   - Compiles all the C code (FreeRTOS, LwIP, WiFi driver, etc.)
//   - Generates Rust bindings using `bindgen`
//
// For Phase 4, we add the esp_wireguard component which implements:
//   - Noise Protocol Framework handshake (Curve25519, ChaCha20, Poly1305)
//   - WireGuard packet encapsulation/decapsulation
//   - Integration with LwIP network stack
//
// The output goes to: target/xtensa-esp32-espidf/release/build/esp-idf-sys-*/out/
// You'll find there:
//   - bindings.rs  (auto-generated Rust code from C headers)
//   - libstd.a     (the compiled ESP-IDF static libraries)

fn main() {
    // Tell cargo to re-run this script if the components change
    println!("cargo:rerun-if-changed=components/esp_wireguard");

    // Output the ESP-IDF build configuration
    embuild::espidf::sysenv::output();
}
