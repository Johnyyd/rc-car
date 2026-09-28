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

    /// Create new pin mappings (alias for default)
    pub fn new() -> Self {
        Self::default()
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

        for &pin in &pins {
            // ESP32 valid GPIOs are generally 0-39 (excluding some strapped/input-only pins)
            if pin > 39 {
                return false;
            }
        }
        true
    }
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
    pub const MAX_MOTOR_SPEED: i8 = 100;
    pub const MIN_MOTOR_SPEED: i8 = -100;
    pub const MAX_ACCELERATION: i8 = 5;
    pub const WATCHDOG_TIMEOUT_MS: u32 = 1000;
}

// Test constants
#[cfg(test)]
pub mod test_config {
    use super::PinMappings;

    pub fn get_test_mappings() -> PinMappings {
        PinMappings::default()
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
    pub const RC_PULSE_MIN_US: u32 = 1000; // Minimum pulse width (full reverse/left)
    pub const RC_PULSE_MAX_US: u32 = 2000; // Maximum pulse width (full forward/right)
    pub const RC_PULSE_NEUTRAL_US: u32 = 1500; // Neutral pulse width (center/stop)

    /// Control loop timing
    pub const CONTROL_LOOP_MS: u64 = 20; // 50 Hz control loop

    /// Status LED blink rate when active (ms)
    pub const LED_BLINK_RATE_MS: u64 = 500; // Blink every 500ms when active
}

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
    pub const MOTOR_PWM_FREQUENCY: u32 = 20_000;
    /// PWM Duty Resolution in bits
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

pub mod system {
    /// Main loop frequency in Hz
    pub const LOOP_FREQUENCY_HZ: u32 = 100;
    /// Control loop period in milliseconds
    pub const LOOP_PERIOD_MS: u32 = 10;
    /// Smoothing window size
    pub const SMOOTHING_WINDOW_SIZE: usize = 5;
}

#[cfg(test)]
mod tests {
    use super::PinMappings;

    #[test]
    fn test_default_pin_mappings_no_conflicts() {
        let mappings = PinMappings::default();
        assert!(
            !mappings.has_conflicts(),
            "Default pin mappings should have no conflicts"
        );
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
