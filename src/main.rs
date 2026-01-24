#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    time::Duration,
};
use log::info;

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_println::logger::init_logger(log::LevelFilter::Info);

    info!("ESP32 Minimal POC - Starting!");

    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let delay = Delay::new();

    loop {
        info!("LED ON");
        led.set_high();
        delay.delay(Duration::from_millis(500));

        info!("LED OFF");
        led.set_low();
        delay.delay(Duration::from_millis(500));
    }
}