#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

use rc_car::main::controller::RCCarController;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // 1. OPTIMIZED CLOCK CONFIGURATION:
    // Use 240MHz for maximum performance on ESP32
    let config = esp_hal::Config::default()
        .with_cpu_clock(CpuClock::max());  // 240MHz on ESP32

    let peripherals = esp_hal::init(config);

    // 2. STATUS LED INDICATOR:
    let mut status_led = Output::new(
        peripherals.GPIO2,
        Level::Low,
        OutputConfig::default(),
    );

    // 3. INITIALIZE CONTROLLER:
    let mut controller = RCCarController::new();
    controller.init();

    // 4. MAIN CONTROL LOOP (~50Hz):
    let mut last_led_toggle = Instant::now();
    let mut led_on = false;

    loop {
        // Blink LED every 500ms to show system is running
        if last_led_toggle.elapsed() >= Duration::from_millis(500) {
            led_on = !led_on;
            status_led.set_level(if led_on { Level::High } else { Level::Low });
            last_led_toggle = Instant::now();
        }

        // Run control loop iteration
        let _speeds = controller.update(0, 0);

        // Control loop delay (20ms -> 50Hz)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(20) {}
    }
}