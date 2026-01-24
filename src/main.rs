#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    time::Duration,
};
use log::info;

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_println::logger::init_logger(log::LevelFilter::Info);

    info!("ESP32 Button Toggle - Starting!");

    // Configure GPIO2 as output for the built-in LED
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let delay = Delay::new();

    // Configure GPIO4 as input for the button.
    // `Pull::Up` enables the internal pull-up resistor (~45kΩ inside the chip).
    // This means:
    //   - When button is NOT pressed: pin reads HIGH (pulled up to 3.3V)
    //   - When button IS pressed: pin reads LOW (connected to GND through button)
    let button = Input::new(
        peripherals.GPIO4,
        InputConfig::default().with_pull(Pull::Up),
    );

    // State tracking variables:
    // - `led_on`: Current LED state (true = lit, false = off)
    // - `button_was_pressed`: Button state from the previous loop iteration
    //   This is needed for "edge detection" - we only want to toggle when
    //   the button transitions from released to pressed, not while it's held.
    let mut led_on = false;
    let mut button_was_pressed = false;

    info!("Ready! Connect GPIO4 to GND to toggle LED");

    loop {
        // Read current button state: is_low() returns true when pressed
        // (because pressing connects the pin to GND)
        let button_pressed = button.is_low();

        // Edge detection: Only toggle on the "falling edge"
        // This means: button is pressed NOW, but was NOT pressed before.
        // This prevents the LED from rapidly toggling while holding the button.
        if button_pressed && !button_was_pressed {
            // Toggle the LED state using the NOT operator
            led_on = !led_on;

            // Apply the new state to the actual hardware
            if led_on {
                led.set_high();
                info!("LED ON");
            } else {
                led.set_low();
                info!("LED OFF");
            }
        }

        // Remember current button state for the next iteration
        button_was_pressed = button_pressed;

        // Small delay serves two purposes:
        // 1. Debouncing: Mechanical buttons "bounce" (rapid on/off) for ~5-20ms
        // 2. CPU efficiency: Tight loops waste power. 20ms is imperceptible to humans.
        //    By only checking every 20ms, we skip over the bouncing period.
        delay.delay(Duration::from_millis(20));
    }
}