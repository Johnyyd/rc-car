//! Hardware Configuration for RC Car
//!
//! This module defines pin mappings and constants for ESP32 GPIO connections
//! for the RC car control system.

/// Pin mappings for ESP32 GPIO
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinMappings {
    /// Motor 1 (Left) pins
    pub motor1_pwm: u8,
    pub motor1_dir1: u8,
    pub motor1_dir2: u8,

    /// Motor 2 (Right) pins
    pub motor2_pwm: u8,
    pub motor2_dir1: u8,
    pub motor2_dir2: u8,

    /// RC Receiver PWM input pins
    pub receiver_channel1: u8, // Throttle
    pub receiver_channel2: u8, // Steering

    /// LED indicator pins
    pub led_status: u8,
    pub led_error: u8,

    /// Safety/emergency pins
    pub emergency_stop: u8,
    pub buzzer: u8,
}

impl PinMappings {
    /// Create default pin mappings for RC car
    pub fn default() -> Self {
        Self {
            // Motor 1 (Left) - GPIO 18, 19, 21
            motor1_pwm: 18,
            motor1_dir1: 19,
            motor1_dir2: 21,

            // Motor 2 (Right) - GPIO 5, 17, 16
            motor2_pwm: 5,
            motor2_dir1: 17,
            motor2_dir2: 16,

            // RC Receiver - GPIO 32, 33
            receiver_channel1: 32, // Throttle
            receiver_channel2: 33, // Steering

            // LED indicators - GPIO 2, 4
            led_status: 2,
            led_error: 4,

            // Safety - GPIO 15, 23
            emergency_stop: 15,
            buzzer: 23,
        }
    }

    /// Check for pin conflicts within the mapping
    pub fn has_conflicts(&self) -> bool {
        let pins = [
            self.motor1_pwm,
            self.motor1_dir1,
            self.motor1_dir2,
            self.motor2_pwm,
            self.motor2_dir1,
            self.motor2_dir2,
            self.receiver_channel1,
            self.receiver_channel2,
            self.led_status,
            self.led_error,
            self.emergency_stop,
            self.buzzer,
        ];

        for i in 0..pins.len() {
            for j in (i + 1)..pins.len() {
                if pins[i] == pins[j] {
                    return true;
                }
            }
        }
        false
    }

    /// Validate that all pins are within ESP32 valid GPIO range
    pub fn is_valid(&self) -> bool {
        let pins = [
            self.motor1_pwm,
            self.motor1_dir1,
            self.motor1_dir2,
            self.motor2_pwm,
            self.motor2_dir1,
            self.motor2_dir2,
            self.receiver_channel1,
            self.receiver_channel2,
            self.led_status,
            self.led_error,
            self.emergency_stop,
            self.buzzer,
        ];

        // ESP32 GPIOs 0-39 are generally valid (excluding strapping pins in some cases)
        pins.iter().all(|&pin| pin <= 39 && pin >= 0)
    }
}

// Motor pin constants
pub mod motor_pins {
    pub const MOTOR1_PWM: u8 = 18;
    pub const MOTOR1_DIR1: u8 = 19;
    pub const MOTOR1_DIR2: u8 = 21;

    pub const MOTOR2_PWM: u8 = 5;
    pub const MOTOR2_DIR1: u8 = 17;
    pub const MOTOR2_DIR2: u8 = 16;
}

// Receiver pin constants
pub mod receiver_pins {
    pub const CHANNEL1_THROTTLE: u8 = 32;
    pub const CHANNEL2_STEERING: u8 = 33;
}

// LED pin constants
pub mod led_pins {
    pub const STATUS_LED: u8 = 2;
    pub const ERROR_LED: u8 = 4;
}

// Safety pin constants
pub mod safety_pins {
    pub const EMERGENCY_STOP: u8 = 15;
    pub const BUZZER: u8 = 23;
}

// PWM Configuration constants
pub mod pwm_config {
    pub const FREQUENCY_HZ: u32 = 1000;
    pub const DUTY_RESOLUTION_BITS: u8 = 10;
    pub const MIN_DUTY: u16 = 0;
    pub const MAX_DUTY: u16 = 1023;
}

// Timing constants
pub mod timing {
    pub const RC_CONTROL_LOOP_MS: u64 = 100;
    pub const SAFETY_TIMEOUT_MS: u64 = 1000;
    pub const PWM_UPDATE_INTERVAL_MS: u64 = 10;
}

// Safety thresholds
pub mod safety {
    pub const MAX_SPEED_PERCENT: i8 = 100;
    pub const MIN_SPEED_PERCENT: i8 = -100;
    pub const EMERGENCY_STOP_THRESHOLD_MS: u64 = 1000;
}

// Test constants
#[cfg(test)]
pub mod test_config {
    use super::PinMappings;

    pub fn get_test_mappings() -> PinMappings {
        PinMappings::default()
    }
}

#[cfg(test)]
mod tests {
    use super::PinMappings;

    #[test]
    fn test_default_pin_mappings_no_conflicts() {
        let mappings = PinMappings::default();
        assert!(!mappings.has_conflicts(), "Default pin mappings should have no conflicts");
    }

    #[test]
    fn test_default_pin_mappings_valid() {
        let mappings = PinMappings::default();
        assert!(mappings.is_valid(), "Default pin mappings should be valid");
    }

    #[test]
    fn test_pin_conflict_detection() {
        let mut mappings = PinMappings::default();
        mappings.motor1_pwm = 18;
        mappings.motor1_dir1 = 18;
        assert!(mappings.has_conflicts(), "Should detect pin conflict");
    }

    #[test]
    fn test_invalid_pin_out_of_range() {
        let mut mappings = PinMappings::default();
        mappings.motor1_pwm = 99;
        assert!(!mappings.is_valid(), "Should reject invalid pin numbers");
    }

    #[test]
    fn test_all_motor_pins_unique() {
        let mappings = PinMappings::default();
        let motor_pins = [
            mappings.motor1_pwm,
            mappings.motor1_dir1,
            mappings.motor1_dir2,
            mappings.motor2_pwm,
            mappings.motor2_dir1,
            mappings.motor2_dir2,
        ];

        for i in 0..motor_pins.len() {
            for j in (i + 1)..motor_pins.len() {
                assert_ne!(motor_pins[i], motor_pins[j], "Motor pins should be unique");
            }
        }
    }
}
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
