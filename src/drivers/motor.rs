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
    /// PWM timer for the motor (declared first, dropped last)
    timer: timer::Timer<'d, LowSpeed>,
    /// PWM channel for speed control (declared second, dropped first)
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
    pub timer: timer::Number,
    /// PWM channel to use
    pub channel: channel::Number,
    /// PWM frequency in Hz
    pub frequency: u32,
    /// PWM duty resolution (bits)
    pub duty_resolution: timer::config::Duty,
    /// Invert motor direction
    pub inverted: bool,
}

impl Default for MotorConfig {
    fn default() -> Self {
        Self {
            timer: timer::Number::Timer0,
            channel: channel::Number::Channel0,
            frequency: 20_000, // 20 kHz
            duty_resolution: timer::config::Duty::Duty8Bit,
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

/// Helper function to configure PWM channel with the correct lifetime
/// This function is generic over the lifetime to avoid the invariance issue

impl<'d> MotorController<'d> {
    /// Creates a new motor controller with explicit PWM and direction pins
    ///
    /// This function creates the timer and channel but does NOT configure the channel
    /// with the timer reference. You MUST call `init()` after construction to complete
    /// the configuration. This is necessary because the channel stores a reference to
    /// the timer, which must point to the timer's final location in the struct.
    ///
    /// # Arguments
    /// * `ledc` - LEDC peripheral
    /// * `pwm_pin` - GPIO pin for PWM speed control (must implement OutputPin)
    /// * `dir1_pin` - GPIO pin for direction 1 (forward) (must implement OutputPin)
    /// * `dir2_pin` - GPIO pin for direction 2 (reverse) (must implement OutputPin)
    /// * `config` - Motor configuration (timer, channel, frequency, duty_resolution, inverted)
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
        let mut lstimer = ledc.timer::<LowSpeed>(config.timer);
        lstimer.configure(timer::config::Config {
            duty: config.duty_resolution,
            clock_source: timer::LSClockSource::APBClk,
            frequency: esp_hal::time::Rate::from_hz(config.frequency),
        })?;

        // Create PWM channel (unconfigured - timer reference will be set in init())
        let pwm_channel = ledc.channel(config.channel, pwm_pin);

        // Configure direction pins
        let dir1 = Output::new(dir1_pin, esp_hal::gpio::Level::Low, OutputConfig::default().with_drive_mode(DriveMode::PushPull));
        let dir2 = Output::new(dir2_pin, esp_hal::gpio::Level::Low, OutputConfig::default().with_drive_mode(DriveMode::PushPull));

        // Create the struct with timer and unconfigured pwm_channel
        // timer is declared first so it's dropped last (outlives pwm_channel)
        Ok(Self {
            timer: lstimer,
            pwm_channel,
            dir1,
            dir2,
            current_speed: 0,
            enabled: false,
            initialized: false,
        })
    }

    /// Initializes the motor controller
    ///
    /// This MUST be called after `new_with_pins()` and before any other methods.
    /// It configures the PWM channel with a reference to the timer field in this struct.
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
    pub fn set_speed(&mut self, speed: i8) -> Result<(), MotorError> {
        if !self.initialized {
            return Err(MotorError::ChannelConfigError);
        }

        if speed < -100 || speed > 100 {
            return Err(MotorError::InvalidSpeed);
        }

        self.current_speed = speed;

        if !self.enabled {
            return Ok(());
        }

        let direction = if speed > 0 {
            MotorDirection::Forward
        } else if speed < 0 {
            MotorDirection::Reverse
        } else {
            MotorDirection::Stop
        };

        let duty_pct = speed.abs() as u8;

        // Set direction
        self.set_direction(direction)?;

        // Set PWM duty cycle
        self.pwm_channel.set_duty(duty_pct)?;

        Ok(())
    }

    /// Stops the motor immediately
    pub fn stop(&mut self) -> Result<(), MotorError> {
        self.set_speed(0)
    }

    /// Enables the motor controller
    pub fn enable(&mut self) -> Result<(), MotorError> {
        if !self.initialized {
            return Err(MotorError::ChannelConfigError);
        }
        self.enabled = true;
        self.set_speed(self.current_speed)
    }

    /// Disables the motor controller (coasts)
    pub fn disable(&mut self) -> Result<(), MotorError> {
        if !self.initialized {
            return Err(MotorError::ChannelConfigError);
        }
        self.enabled = false;
        self.set_direction(MotorDirection::Stop)?;
        self.pwm_channel.set_duty(0)?;
        Ok(())
    }

    /// Sets the motor direction
    fn set_direction(&mut self, direction: MotorDirection) -> Result<(), MotorError> {
        match direction {
            MotorDirection::Stop => {
                self.dir1.set_low();
                self.dir2.set_low();
            }
            MotorDirection::Forward => {
                self.dir1.set_high();
                self.dir2.set_low();
            }
            MotorDirection::Reverse => {
                self.dir1.set_low();
                self.dir2.set_high();
            }
        }
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

/// Motor controller errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorError {
    /// Invalid speed value (must be -100 to 100)
    InvalidSpeed,
    /// PWM configuration error
    PwmConfigError,
    /// Timer configuration error
    TimerConfigError,
    /// Channel configuration error
    ChannelConfigError,
}

impl From<esp_hal::ledc::timer::Error> for MotorError {
    fn from(_: esp_hal::ledc::timer::Error) -> Self {
        MotorError::TimerConfigError
    }
}

impl From<esp_hal::ledc::channel::Error> for MotorError {
    fn from(_: esp_hal::ledc::channel::Error) -> Self {
        MotorError::ChannelConfigError
    }
}