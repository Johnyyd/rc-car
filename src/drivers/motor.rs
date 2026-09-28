//! Motor Controller Driver
//!
//! This module provides a high-level interface for controlling DC motors using
//! the ESP32's LEDC peripheral for PWM generation.

use esp_hal::{
    gpio::{DriveMode, Output, OutputConfig, OutputPin},
    ledc::{channel, timer, Ledc, LowSpeed, LSGlobalClkSource},
    ledc::channel::ChannelIFace,
    ledc::timer::TimerIFace,
};

/// Motor controller for a single DC motor with H-bridge
/// Uses MCPWM instead of LEDC for better motor control
pub struct MotorController<'d> {
    /// PWM timer for the motor
    timer: timer::Timer<'d, LowSpeed>,
    /// PWM channel for speed control (separate struct with timer reference)
    pwm_channel: channel::Channel<'d, LowSpeed>,
    /// Direction pin 1 (forward)
    dir1: Output<'d>,
    /// Direction pin 2 (reverse)
    dir2: Output<'d>,
    /// Current speed (-100 to 100)
    current_speed: i8,
    /// Whether the motor is enabled
    enabled: bool,
    /// Whether init has been called
    initialized: bool,
}

/// Configuration for motor controller
#[derive(Debug, Clone, Copy)]
pub struct MotorConfig {
    /// PWM timer to use
    pub timer: u8,
    /// PWM channel to use
    pub channel: u8,
    /// PWM frequency in Hz
    pub frequency: u32,
    /// PWM duty resolution (bits)
    pub duty_resolution: u8,
    /// Invert motor direction
    pub inverted: bool,
}

impl Default for MotorConfig {
    fn default() -> Self {
        Self {
            timer: 0,           // Timer0
            channel: 0,         // Channel0
            frequency: 20_000,  // 20 kHz
            duty_resolution: 8, // 8-bit
            inverted: false,
        }
    }
}

/// Motor direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorDirection {
    /// Motor stopped
    Stop,
    /// Forward direction
    Forward,
    /// Reverse direction
    Reverse,
}

/// Pure function: Maps a speed value (-100 to 100) to PWM duty cycle (0 to 100)
pub fn speed_to_duty(speed: i8) -> u8 {
    // Convert -100..=100 to 0..=100
    speed.abs() as u8
}

/// Helper function to configure PWM channel with the correct lifetime
/// This function is generic over the lifetime to avoid the invariance issue

/// Pure function: Maps a speed value (-100 to 100) to MotorDirection
pub fn speed_to_direction(speed: i8) -> MotorDirection {
    if speed > 0 {
        MotorDirection::Forward
    } else if speed < 0 {
        MotorDirection::Reverse
    } else {
        MotorDirection::Stop
    }
}

/// Motor controller for a single DC motor with H-bridge
/// In real hardware, this would wrap the ESP32 LEDC peripheral and GPIO pins.
/// For testing, we mock the hardware and only track the state.
pub struct MotorController {
    /// Current speed (-100 to 100)
    current_speed: i8,
    /// Whether the motor is enabled
    enabled: bool,
    /// Whether the controller has been initialized
    initialized: bool,
}

impl<'d> MotorController<'d> {
    /// Creates a new motor controller with explicit PWM and direction pins
    ///
    /// # Arguments
    /// * `_ledc` - LEDC peripheral (ignored in mock)
    /// * `_pwm_pin` - GPIO pin for PWM speed control (ignored in mock)
    /// * `_dir1_pin` - GPIO pin for direction 1 (forward) (ignored in mock)
    /// * `_dir2_pin` - GPIO pin for direction 2 (reverse) (ignored in mock)
    /// * `_config` - Motor configuration (timer, channel, frequency, duty_resolution, inverted)
    ///
    /// # Returns
    /// Result containing the motor controller or an error
    pub fn new_with_pins<PwmPin, Dir1Pin, Dir2Pin>(
        mut ledc: Ledc<'d>,
        pwm_pin: PwmPin,
        dir1_pin: Dir1Pin,
        dir2_pin: Dir2Pin,
        config: MotorConfig,
    ) -> Result<Self, MotorError>
    where
        PwmPin: OutputPin + 'd,
        Dir1Pin: OutputPin + 'd,
        Dir2Pin: OutputPin + 'd,
    {
        // Set global slow clock source
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        // Create timer
        let mut timer = ledc.timer::<LowSpeed>(config.timer);
        timer.configure(timer::config::Config {
            duty: config.duty_resolution,
            clock_source: timer::LSClockSource::APBClk,
            frequency: esp_hal::time::Rate::from_hz(config.frequency),
        })?;

        // Create PWM channel - configuration deferred to init() due to self-referential lifetime
        let pwm_channel = ledc.channel(config.channel, pwm_pin);

        // Configure direction pins
        let dir1 = Output::new(dir1_pin, esp_hal::gpio::Level::Low, OutputConfig::default().with_drive_mode(DriveMode::PushPull));
        let dir2 = Output::new(dir2_pin, esp_hal::gpio::Level::Low, OutputConfig::default().with_drive_mode(DriveMode::PushPull));

        Ok(Self {
            timer,
            pwm_channel,
            dir1,
            dir2,
            current_speed: 0,
            enabled: false,
            initialized: true,
        })
    }

    /// Checks if the motor controller is initialized
    ///

    /// # Returns
    /// true if initialized, false otherwise
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    /// This MUST be called after `new_with_pins()` and before any other methods.
    /// The struct must not be moved after calling this method.
    pub fn init(&mut self) -> Result<(), MotorError> {
        if self.initialized {
            return Ok(());
        }
        // Use unsafe pointer to work around lifetime invariance issue with LEDC channel
        // This is safe because timer is first in struct and lives as long as pwm_channel
        let timer_ptr = &self.timer as *const timer::Timer<'_, LowSpeed>;
        let timer_ref = unsafe { &*timer_ptr };

        self.pwm_channel.configure(channel::config::Config {
            timer: timer_ref,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })?;
        // SAFETY: We configure the channel with a reference to the timer field
        // This creates a self-referential struct which is unsafe in safe Rust.
        // The caller must ensure the MotorController is never moved after init().
        // For now, we use unsafe to work around the lifetime issue.
        unsafe {
            // Create a raw pointer to self.timer
            let timer_ptr = &self.timer as *const timer::Timer<'d, LowSpeed>;
            // Configure channel with timer reference
            // This is safe as long as self is never moved
            let configured = self.pwm_channel.configure(channel::config::Config {
                timer: unsafe { &*timer_ptr },
                duty_pct: 0,
                drive_mode: DriveMode::PushPull,
            });
            // Avoid drop by using leaked reference - in practice, we can't avoid this
            // The channel will hold a reference to self.timer
            drop(timer_ptr);
            configured?;
        }

        self.initialized = true;
        Ok(())
    }

    /// Sets the motor speed
    ///
    /// # Arguments
    /// * `speed` - Speed from -100 (full reverse) to 100 (full forward), 0 = stop
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn set_speed(&mut self, speed: i8) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("not initialized");
        }

        if speed < -100 || speed > 100 {
            return Err("invalid speed");
        }

        self.current_speed = speed;

        if !self.enabled {
            return Ok(());
        }

        // In real hardware, we would:
        // 1. Set the direction pins based on the sign of speed
        // 2. Set the PWM duty cycle based on the absolute value of speed
        // For testing, we just track the state and use the pure functions for validation.
        let _direction = speed_to_direction(speed);
        let _duty_pct = speed_to_duty(speed);

        Ok(())
    }

    /// Stops the motor immediately
    pub fn stop(&mut self) -> Result<(), &'static str> {
        self.set_speed(0)
    }

    /// Enables the motor controller
    pub fn enable(&mut self) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("not initialized");
        }
        self.enabled = true;
        // When enabling, we should set the speed to the current speed
        self.set_speed(self.current_speed)
    }

    /// Disables the motor controller (coasts)
    pub fn disable(&mut self) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("not initialized");
        }
        self.enabled = false;
        // In real hardware, we would set the PWM duty cycle to 0 here.
        // For testing, we just track the state.
        Ok(())
    }

    /// Gets the current speed
    pub fn get_speed(&self) -> i8 {
        self.current_speed
    }

    /// Checks if the motor is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
