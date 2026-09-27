/// Main RC Car Controller
///
/// This module integrates all subsystems (motor control, RC input, safety, etc.)
/// into a unified control system for the RC car.

use esp_hal::{
    gpio::{self, Level, Output, OutputConfig, OutputPin},
    ledc::{self, channel, timer, Ledc, LowSpeed},
    time::{self, Duration, Instant},
};
use core::boxed::Box;
use core::convert::Infallible;

use crate::config::hardware::{PinMappings, constants};
use crate::drivers::motor::{MotorController, MotorConfig, MotorDirection};

/// Main controller for the RC car
pub struct RCCarController<'d> {
    /// Left motor controller
    left_motor: MotorController<'d>,
    /// Right motor controller
    right_motor: MotorController<'d>,
    /// Status LED output
    status_led: Output<'d>,
    /// Last time we blinked the LED
    last_led_toggle: Instant,
    /// LED state (on/off)
    led_state: bool,
    /// Last control loop execution time
    last_control_time: Instant,
    /// Whether the system is initialized
    initialized: bool,
}

impl<'d> RCCarController<'d> {
    /// Create a new RCCarController from LEDC peripheral and pin mappings
    ///
    /// # Arguments
    /// * `ledc` - LEDC peripheral
    /// * `pins` - Hardware pin mappings
    ///
    /// # Returns
    /// Result containing the controller or an error
    pub fn new(
        mut ledc: Ledc<'d>,
        pins: PinMappings,
        peripherals: &esp_hal::Peripherals,
    ) -> Result<Self, Box<dyn core::fmt::Debug>> {
        // Configure LEDC global settings
        ledc.timer::<LowSpeed>(ledc::timer::Number::Timer0).configure(
            ledc::timer::config::Config {
                duty: ledc::timer::config::Duty::Duty8Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: esp_hal::time::Rate::from_hz(constants::PWM_FREQUENCY_HZ),
            },
        );

        // Helper to get GPIO pins
        let get_pin = |num: u8| -> esp_hal::gpio::Pin {
            match num {
                2 => peripherals.GPIO2,
                18 => peripherals.GPIO18,
                19 => peripherals.GPIO19,
                21 => peripherals.GPIO21,
                22 => peripherals.GPIO22,
                23 => peripherals.GPIO23,
                25 => peripherals.GPIO25,
                32 => peripherals.GPIO32,
                33 => peripherals.GPIO33,
                _ => panic!("Unsupported pin"),
            }
        };

        // Configure left motor
        let left_motor_config = MotorConfig {
            timer: ledc::timer::Number::Timer0,
            channel: ledc::channel::Number::Channel0,
            frequency: constants::PWM_FREQUENCY_HZ,
            duty_resolution: ledc::timer::config::Duty::Duty8Bit,
            inverted: false,
        };

        let mut left_motor = MotorController::new_with_pins(
            ledc,
            get_pin(pins.left_motor_pwm),
            get_pin(pins.left_motor_dir1),
            get_pin(pins.left_motor_dir2),
            left_motor_config,
        )?;

        // Configure right motor (using different timer/channel to avoid conflicts)
        let mut ledc2 = ledc;
        let right_motor_config = MotorConfig {
            timer: ledc::timer::Number::Timer1,
            channel: ledc::channel::Number::Channel1,
            frequency: constants::PWM_FREQUENCY_HZ,
            duty_resolution: ledc::timer::config::Duty::Duty8Bit,
            inverted: false,
        };

        let mut right_motor = MotorController::new_with_pins(
            ledc2,
            get_pin(pins.right_motor_pwm),
            get_pin(pins.right_motor_dir1),
            get_pin(pins.right_motor_dir2),
            right_motor_config,
        )?;

        // Configure status LED
        let status_led = Output::new(
            get_pin(pins.status_led),
            Level::Low,
            OutputConfig::default().with_drive_mode(gpio::DriveMode::PushPull),
        );

        Ok(Self {
            left_motor,
            right_motor,
            status_led,
            last_led_toggle: Instant::now(),
            led_state: false,
            last_control_time: Instant::now(),
            initialized: false,
        })
    }

    /// Initialize all subsystems
    ///
    /// This must be called before using the controller
    pub fn init(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        if self.initialized {
            return Ok(());
        }

        // Initialize motors
        self.left_motor.init()?;
        self.right_motor.init()?;

        self.initialized = true;
        Ok(())
    }

    /// Enable both motors
    pub fn enable_motors(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.enable()?;
        self.right_motor.enable()?;
        Ok(())
    }

    /// Disable both motors (coast to stop)
    pub fn disable_motors(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.disable()?;
        self.right_motor.disable()?;
        Ok(())
    }

    /// Set motor speeds for differential steering
    ///
    /// # Arguments
    /// * `left_speed` - Left motor speed (-100 to 100)
    /// * `right_speed` - Right motor speed (-100 to 100)
    pub fn set_motor_speeds(
        &mut self,
        left_speed: i8,
        right_speed: i8,
    ) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.set_speed(left_speed)?;
        self.right_motor.set_speed(right_speed)?;
        Ok(())
    }

    /// Stop both motors
    pub fn stop(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.stop()?;
        self.right_motor.stop()?;
        Ok(())
    }

    /// Update the status LED based on system activity
    ///
    /// This should be called regularly in the main loop
    pub fn update_status_led(&mut self, now: Instant) {
        // Blink LED at regular intervals to show system is alive
        if now.duration_since(self.last_led_toggle) >= Duration::from_millis(constants::LED_BLINK_RATE_MS) {
            self.led_state = !self.led_state;
            let _ = self.status_led.set_level(if self.led_state { Level::High } else { Level::Low });
            self.last_led_toggle = now;
        }
    }

    /// Run one iteration of the control loop
    ///
    /// In a real implementation, this would read RC input, apply control algorithms,
    /// and set motor outputs. For now, it just maintains the status LED.
    pub fn update(&mut self, now: Instant) -> Result<(), Box<dyn core::fmt::Debug>> {
        // Update status LED to show system is active
        self.update_status_led(now);

        // In a full implementation, we would:
        // 1. Read RC receiver inputs
        // 2. Apply safety checks
        // 3. Calculate motor commands using steering/throttle mixing
        // 4. Set motor speeds accordingly
        // 5. Handle any error conditions

        self.last_control_time = now;
        Ok(())
    }
}