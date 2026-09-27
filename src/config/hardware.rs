/// Hardware Configuration for RC Car
///
/// This module defines the pin mappings and hardware constants for the ESP32-based RC car.

/// Pin mappings for the RC car hardware
#[derive(Debug, Clone, Copy)]
pub struct PinMappings {
    /// Left motor PWM pin
    pub left_motor_pwm: u8,
    /// Left motor direction pin 1 (forward)
    pub left_motor_dir1: u8,
    /// Left motor direction pin 2 (reverse)
    pub left_motor_dir2: u8,
    /// Right motor PWM pin
    pub right_motor_pwm: u8,
    /// Right motor direction pin 1 (forward)
    pub right_motor_dir1: u8,
    /// Right motor direction pin 2 (reverse)
    pub right_motor_dir2: u8,
    /// RC receiver channel 1 (steering) input
    pub rc_ch1: u8,
    /// RC receiver channel 2 (throttle) input
    pub rc_ch2: u8,
    /// Status LED pin
    pub status_led: u8,
}

impl PinMappings {
    /// Default pin mappings for common ESP32 RC car setup
    ///
    /// These pins can be adjusted based on actual hardware wiring
    pub fn new() -> Self {
        Self {
            // Motor pins (using GPIO pins suitable for PWM)
            left_motor_pwm: 18,   // PWM for left motor
            left_motor_dir1: 19,  // Direction 1 for left motor
            left_motor_dir2: 21,  // Direction 2 for left motor
            right_motor_pwm: 22,  // PWM for right motor
            right_motor_dir1: 23, // Direction 1 for right motor
            right_motor_dir2: 25, // Direction 2 for right motor

            // RC receiver inputs (using GPIO pins that can capture PWM signals)
            rc_ch1: 32,   // Steering channel
            rc_ch2: 33,   // Throttle channel

            // Status LED (built-in LED on many ESP32 dev boards)
            status_led: 2, // GPIO2 often connected to built-in LED
        }
    }
}

/// Hardware constants for motor control
pub mod constants {
    /// PWM frequency for motor control (Hz)
    pub const PWM_FREQUENCY_HZ: u32 = 20_000; // 20 kHz - good for motor control

    /// PWM resolution (bits)
    pub const PWM_RESOLUTION_BITS: u8 = 8; // 8-bit resolution (0-255)

    /// Motor PWM duty cycle limits
    pub const MOTOR_PWM_MIN: u8 = 0;
    pub const MOTOR_PWM_MAX: u8 = 255;

    /// RC signal timing constants (microseconds)
    pub const RC_PULSE_MIN_US: u32 = 1000;   // Minimum pulse width (full reverse/left)
    pub const RC_PULSE_MAX_US: u32 = 2000;   // Maximum pulse width (full forward/right)
    pub const RC_PULSE_NEUTRAL_US: u32 = 1500; // Neutral pulse width (center/stop)

    /// Control loop timing
    pub const CONTROL_LOOP_MS: u64 = 20; // 50 Hz control loop

    /// Status LED blink rate when active (ms)
    pub const LED_BLINK_RATE_MS: u64 = 500; // Blink every 500ms when active
}