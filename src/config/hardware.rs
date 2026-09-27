//! Hardware Configuration for RC Car
//!
//! This module contains all hardware-specific constants including pin mappings,
//! PWM configuration, RC receiver settings, and safety parameters.

pub mod pins {
    /// Motor Left PWM Pin
    pub const MOTOR_LEFT_PWM: u8 = 18;
    /// Motor Left Direction Pin 1
    pub const MOTOR_LEFT_DIR1: u8 = 19;
    /// Motor Left Direction Pin 2
    pub const MOTOR_LEFT_DIR2: u8 = 21;
    /// Motor Right PWM Pin
    pub const MOTOR_RIGHT_PWM: u8 = 5;
    /// Motor Right Direction Pin 1
    pub const MOTOR_RIGHT_DIR1: u8 = 17;
    /// Motor Right Direction Pin 2
    pub const MOTOR_RIGHT_DIR2: u8 = 16;
    /// RC Receiver Channel 1 (Steering)
    pub const RC_CHANNEL1: u8 = 34;
    /// RC Receiver Channel 2 (Throttle)
    pub const RC_CHANNEL2: u8 = 35;
    /// Status LED Pin
    pub const STATUS_LED: u8 = 2;
    /// Emergency Stop Button
    pub const EMERGENCY_STOP: u8 = 0;
    /// Battery Voltage Monitor
    pub const BATTERY_MONITOR: u8 = 36;
}

pub mod pwm {
    /// Motor PWM Frequency in Hz
    /// Range: 1 kHz to 40 kHz for DC motors
    pub const MOTOR_PWM_FREQUENCY: u32 = 20_000;
    /// PWM Duty Resolution in bits
    /// Valid values: 1-16 bits
    pub const PWM_DUTY_BITS: u8 = 8;
    /// PWM Timer Number for Left Motor
    pub const LEFT_MOTOR_TIMER: u8 = 0;
    /// PWM Channel Number for Left Motor
    pub const LEFT_MOTOR_CHANNEL: u8 = 0;
    /// PWM Timer Number for Right Motor
    pub const RIGHT_MOTOR_TIMER: u8 = 0;
    /// PWM Channel Number for Right Motor
    pub const RIGHT_MOTOR_CHANNEL: u8 = 1;
}

pub mod rc {
    /// Minimum pulse width in microseconds
    pub const MIN_PULSE_WIDTH: u16 = 1000;
    /// Maximum pulse width in microseconds
    pub const MAX_PULSE_WIDTH: u16 = 2000;
    /// Neutral pulse width in microseconds
    pub const NEUTRAL_PULSE_WIDTH: u16 = 1500;
    /// RC signal timeout in milliseconds
    pub const SIGNAL_TIMEOUT_MS: u32 = 1000;
}

pub mod safety {
    /// Maximum motor speed percentage
    pub const MAX_MOTOR_SPEED: i8 = 100;
    /// Minimum motor speed percentage
    pub const MIN_MOTOR_SPEED: i8 = -100;
    /// Maximum acceleration per cycle
    pub const MAX_ACCELERATION: i8 = 5;
    /// Watchdog timeout in milliseconds
    pub const WATCHDOG_TIMEOUT_MS: u32 = 1000;
}

pub mod system {
    /// Main loop frequency in Hz
    pub const LOOP_FREQUENCY_HZ: u32 = 100;
    /// Control loop period in milliseconds
    pub const LOOP_PERIOD_MS: u32 = 10;
    /// Smoothing window size
    pub const SMOOTHING_WINDOW_SIZE: usize = 5;
}
