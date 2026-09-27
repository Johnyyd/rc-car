//! Test file to isolate the motor controller issue

use esp_hal::{ledc::channel::Number::Channel0, ledc::timer::Number::Timer0, Ledc, LowSpeed};

struct TestController<'d> {
    timer: timer::Timer<'d, LowSpeed>,
    channel: channel::Channel<'d, LowSpeed>,
}

impl<'d> TestController<'d> {
    fn new(mut ledc: Ledc<'d>) -> Self {
        // Create timer
        let mut timer = ledc.timer::<LowSpeed>(Timer0);
        timer.configure(esp_hal::ledc::timer::config::Config {
            duty: esp_hal::ledc::timer::config::Duty::Duty8Bit,
            clock_source: esp_hal::ledc::timer::LSClockSource::APBClk,
            frequency: esp_hal::time::Rate::from_hz(1000),
        }).unwrap();

        // Create channel
        let channel = ledc.channel(Channel0, esp_hal::gpio::IO::IO0);

        // Return struct with timer first (dropped last)
        Self { timer, channel }
    }
}

fn main() {
    // This won't actually run, just testing compilation
}