#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::ledc::Ledc;

use rc_car::config::hardware::PinMappings;
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
    // generator version: 1.4.0
    // generator parameters: -o esp32

    // 1. OPTIMIZED CLOCK CONFIGURATION:
    // Use 240MHz for maximum performance on ESP32
    // For power-sensitive applications, could use CpuClock::Configured for 160MHz or 80MHz
    let config = esp_hal::Config::default()
        .with_cpu_clock(CpuClock::max())  // 240MHz on ESP32
        .with_wait_state(esp_hal::Config::WAIT_STATE_2);  // Optimal for 240MHz

    let peripherals = esp_hal::init(config);

    // 2. INITIALIZE CONTROLLER:
    // Get LEDC peripheral for PWM motor control
    let ledc = Ledc::new(peripherals.LEDC, esp_hal::ledc::config::Config::default())
        .unwrap();

    // Get pin mappings
    let pin_mappings = PinMappings::new();

    // Create and initialize the RC car controller
    let mut controller = RCCarController::new(ledc, pin_mappings, &peripherals)
        .expect("Failed to create RC car controller");

    controller.init()
        .expect("Failed to initialize RC car controller");

    controller.enable_motors()
        .expect("Failed to enable motors");

    // 3. MAIN CONTROL LOOP:
    // Status LED will blink to indicate system is running
    let mut loop_start = Instant::now();

    loop {
        let now = Instant::now();

        // Update controller (handles LED blinking, motor control, etc.)
        controller.update(now)
            .expect("Controller update failed");

        // Small delay to prevent tight loop (can be adjusted for performance)
        // This gives ~50Hz control loop rate
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(20) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}