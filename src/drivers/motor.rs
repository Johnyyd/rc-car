//! Motor Controller Driver
//!
//! This module provides a high-level interface for controlling DC motors.

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
    speed.unsigned_abs()
}

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
    /// Not initialized error
    NotInitialized,
}

/// Motor controller for a single DC motor
pub struct MotorController {
    current_speed: i8,
    enabled: bool,
    initialized: bool,
    config: MotorConfig,
}

impl MotorController {
    /// Creates a new motor controller
    pub fn new_with_pins<_PwmPin, _Dir1Pin, _Dir2Pin>(
        _ledc: (),
        _pwm_pin: _PwmPin,
        _dir1_pin: _Dir1Pin,
        _dir2_pin: _Dir2Pin,
        config: MotorConfig,
    ) -> Result<Self, MotorError> {
        Ok(Self {
            current_speed: 0,
            enabled: false,
            initialized: true,
            config,
        })
    }

    /// Checks if the motor controller is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Initializes the motor controller
    pub fn init(&mut self) -> Result<(), MotorError> {
        self.initialized = true;
        Ok(())
    }

    /// Sets the motor speed (-100 to 100)
    pub fn set_speed(&mut self, speed: i8) -> Result<(), MotorError> {
        if !self.initialized {
            return Err(MotorError::NotInitialized);
        }

        if !(-100..=100).contains(&speed) {
            return Err(MotorError::InvalidSpeed);
        }

        self.current_speed = speed;
        Ok(())
    }

    /// Stops the motor immediately
    pub fn stop(&mut self) -> Result<(), MotorError> {
        self.set_speed(0)
    }

    /// Enables the motor controller
    pub fn enable(&mut self) -> Result<(), MotorError> {
        if !self.initialized {
            return Err(MotorError::NotInitialized);
        }
        self.enabled = true;
        self.set_speed(self.current_speed)
    }

    /// Disables the motor controller (coasts)
    pub fn disable(&mut self) -> Result<(), MotorError> {
        if !self.initialized {
            return Err(MotorError::NotInitialized);
        }
        self.enabled = false;
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

    /// Gets the motor config
    pub fn get_config(&self) -> &MotorConfig {
        &self.config
    }
}
