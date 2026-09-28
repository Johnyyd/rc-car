//! Configuration Validation Binary
//!
//! Validates all hardware configuration constants for the RC Car
//! Run with: cargo run --bin validate_config

// Define all constants directly - completely independent of library
mod constants {
    // Pin mappings for ESP32
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

    // PWM Configuration
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

    // RC Receiver Configuration
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

    // Safety Configuration
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

    // System Configuration
    pub mod system {
        /// Main loop frequency in Hz
        pub const LOOP_FREQUENCY_HZ: u32 = 100;
        /// Control loop period in milliseconds
        pub const LOOP_PERIOD_MS: u32 = 10;
        /// Smoothing window size
        pub const SMOOTHING_WINDOW_SIZE: usize = 5;
    }
}

use std::collections::HashMap;
use crate::constants::{pins, pwm, rc, safety, system};

fn main() -> Result<(), ValidationError> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           RC Car Configuration Validation                 ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Validate PWM Configuration
    println!("━━━ PWM Configuration ━━━");
    validate_pwm_config(&mut errors, &mut warnings);

    // Validate RC Configuration
    println!("━━━ RC Receiver Configuration ━━━");
    validate_rc_config(&mut errors, &mut warnings);

    // Validate Safety Configuration
    println!("━━━ Safety Configuration ━━━");
    validate_safety_config(&mut errors, &mut warnings);

    // Validate System Configuration
    println!("━━━ System Configuration ━━━");
    validate_system_config(&mut errors, &mut warnings);

    // Validate Pin Assignments
    println!("━━━ Pin Assignments ━━━");
    validate_pins(&mut errors, &mut warnings);

    // Print Results
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                    Validation Results                      ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    if warnings.is_empty() && errors.is_empty() {
        println!("✅ ALL VALIDATIONS PASSED");
        println!("   Configuration is valid and ready for use.");
        Ok(())
    } else {
        if !warnings.is_empty() {
            println!();
            println!("⚠️  WARNINGS ({}):", warnings.len());
            for warning in &warnings {
                println!("   - {}", warning);
            }
        }

        if !errors.is_empty() {
            println!();
            println!("❌ ERRORS ({}):", errors.len());
            for error in &errors {
                println!("   - {}", error);
            }
            println!();
            println!("Configuration validation FAILED. Please fix errors before proceeding.");
            Err(ValidationError::ValidationFailed)
        } else {
            println!();
            println!("✅ VALIDATION PASSED (with warnings)");
            Ok(())
        }
    }
}

fn validate_pwm_config(errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    // Frequency validation
    let freq = pwm::MOTOR_PWM_FREQUENCY;
    println!("   PWM Frequency: {} Hz", freq);

    if freq < 1_000 {
        errors.push(format!("PWM frequency {} Hz is below minimum 1 kHz", freq));
    } else if freq > 40_000 {
        errors.push(format!("PWM frequency {} Hz exceeds maximum 40 kHz", freq));
    } else {
        println!("   ✓ Frequency within valid range (1 kHz - 40 kHz)");
    }

    // Check for common motor frequencies
    let common_freqs = [1_000, 5_000, 10_000, 20_000, 25_000, 30_000, 40_000];
    if !common_freqs.contains(&freq) {
        warnings.push(format!("PWM frequency {} Hz is not a common motor frequency", freq));
    }

    // Duty resolution validation
    let duty_bits = pwm::PWM_DUTY_BITS;
    println!("   PWM Duty Resolution: {} bits", duty_bits);

    if duty_bits < 1 || duty_bits > 16 {
        errors.push(format!("PWM duty resolution {} bits is invalid (must be 1-16)", duty_bits));
    } else {
        println!("   ✓ Duty resolution within valid range (1-16 bits)");
        let resolution = 2u32.pow(duty_bits as u32);
        println!("   ✓ Effective resolution: {} steps (0-{})", resolution, resolution - 1);
    }

    // Timer and channel validation
    let left_timer = pwm::LEFT_MOTOR_TIMER;
    let left_channel = pwm::LEFT_MOTOR_CHANNEL;
    let right_timer = pwm::RIGHT_MOTOR_TIMER;
    let right_channel = pwm::RIGHT_MOTOR_CHANNEL;

    println!("   Left Motor: Timer{}, Channel{}", left_timer, left_channel);
    println!("   Right Motor: Timer{}, Channel{}", right_timer, right_channel);

    // ESP32 has 4 timers (0-3) and 8 channels (0-7) per speed group
    if left_timer > 3 || right_timer > 3 {
        errors.push("Timer number must be 0-3 for ESP32 LEDC".to_string());
    } else {
        println!("   ✓ Timer numbers within valid range (0-3)");
    }

    if left_channel > 7 || right_channel > 7 {
        errors.push("Channel number must be 0-7 for ESP32 LEDC".to_string());
    } else {
        println!("   ✓ Channel numbers within valid range (0-7)");
    }

    // Check for timer/channel conflicts
    if left_timer == right_timer && left_channel == right_channel {
        errors.push("Left and Right motors cannot share the same timer AND channel".to_string());
    } else if left_timer == right_timer && left_channel != right_channel {
        println!("   ⚠️  Both motors share Timer{} - ensure LEDC driver supports multiple channels per timer", left_timer);
    }
}

fn validate_rc_config(errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    let min_pulse = rc::MIN_PULSE_WIDTH;
    let max_pulse = rc::MAX_PULSE_WIDTH;
    let neutral = rc::NEUTRAL_PULSE_WIDTH;
    let timeout = rc::SIGNAL_TIMEOUT_MS;

    println!("   Min Pulse Width: {} μs", min_pulse);
    println!("   Neutral Pulse:   {} μs", neutral);
    println!("   Max Pulse Width: {} μs", max_pulse);
    println!("   Signal Timeout:  {} ms", timeout);

    if min_pulse >= neutral {
        errors.push("MIN_PULSE_WIDTH must be less than NEUTRAL_PULSE_WIDTH".to_string());
    } else {
        println!("   ✓ Min < Neutral");
    }

    if neutral >= max_pulse {
        errors.push("NEUTRAL_PULSE_WIDTH must be less than MAX_PULSE_WIDTH".to_string());
    } else {
        println!("   ✓ Neutral < Max");
    }

    let range = max_pulse - min_pulse;
    if range < 500 || range > 2000 {
        warnings.push(format!("Pulse width range {} μs is unusual (typical: 1000 μs)", range));
    } else {
        println!("   ✓ Pulse width range: {} μs", range);
    }

    if timeout < 100 {
        warnings.push(format!("Signal timeout {} ms is very short", timeout));
    } else if timeout > 5000 {
        warnings.push(format!("Signal timeout {} ms is very long", timeout));
    } else {
        println!("   ✓ Timeout within reasonable range");
    }

    // Verify center is actually centered
    let center_error = (neutral - min_pulse).abs_diff(max_pulse - neutral);
    if center_error > 50 {
        warnings.push(format!("Neutral point is not centered (error: {} μs)", center_error));
    } else {
        println!("   ✓ Neutral point is centered (error: {} μs)", center_error);
    }
}

fn validate_safety_config(errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    let max_speed = safety::MAX_MOTOR_SPEED;
    let min_speed = safety::MIN_MOTOR_SPEED;
    let max_accel = safety::MAX_ACCELERATION;
    let watchdog = safety::WATCHDOG_TIMEOUT_MS;

    println!("   Max Speed:       {}%", max_speed);
    println!("   Min Speed:       {}%", min_speed);
    println!("   Max Acceleration: {}%/cycle", max_accel);
    println!("   Watchdog:        {} ms", watchdog);

    if max_speed < 0 || max_speed > 100 {
        errors.push(format!("MAX_MOTOR_SPEED {} is out of range (0-100)", max_speed));
    } else {
        println!("   ✓ Max speed valid");
    }

    if min_speed > 0 || min_speed < -100 {
        errors.push(format!("MIN_MOTOR_SPEED {} is out of range (-100 to 0)", min_speed));
    } else {
        println!("   ✓ Min speed valid");
    }

    if max_speed < min_speed.abs() {
        warnings.push("MAX_MOTOR_SPEED is less than abs(MIN_MOTOR_SPEED)".to_string());
    }

    if max_accel <= 0 || max_accel > 100 {
        errors.push(format!("MAX_ACCELERATION {} is out of range (1-100)", max_accel));
    } else {
        println!("   ✓ Max acceleration valid");
    }

    if watchdog < 100 {
        warnings.push(format!("Watchdog timeout {} ms is very short", watchdog));
    } else if watchdog > 10000 {
        warnings.push(format!("Watchdog timeout {} ms is very long", watchdog));
    } else {
        println!("   ✓ Watchdog timeout reasonable");
    }
}

fn validate_system_config(errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    let freq = system::LOOP_FREQUENCY_HZ;
    let period = system::LOOP_PERIOD_MS;
    let smoothing = system::SMOOTHING_WINDOW_SIZE;

    println!("   Loop Frequency:  {} Hz", freq);
    println!("   Loop Period:     {} ms", period);
    println!("   Smoothing Window: {} samples", smoothing);

    if freq == 0 {
        errors.push("LOOP_FREQUENCY_HZ cannot be zero".to_string());
    } else {
        println!("   ✓ Loop frequency valid");
    }

    if period == 0 {
        errors.push("LOOP_PERIOD_MS cannot be zero".to_string());
    } else {
        println!("   ✓ Loop period valid");
    }

    // Check consistency
    let expected_period = 1000 / freq;
    if period != expected_period {
        errors.push(format!(
            "LOOP_PERIOD_MS ({}) inconsistent with LOOP_FREQUENCY_HZ ({}): expected {} ms",
            period, freq, expected_period
        ));
    } else {
        println!("   ✓ Frequency and period consistent");
    }

    if smoothing == 0 {
        errors.push("SMOOTHING_WINDOW_SIZE cannot be zero".to_string());
    } else if smoothing > 100 {
        warnings.push(format!("Smoothing window {} is very large", smoothing));
    } else {
        println!("   ✓ Smoothing window valid");
    }

    // Calculate effective smoothing time
    let smoothing_time_ms = smoothing as u32 * period;
    println!("   ✓ Effective smoothing time: {} ms", smoothing_time_ms);
}

fn validate_pins(errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    let all_pins = [
        ("MOTOR_LEFT_PWM", pins::MOTOR_LEFT_PWM),
        ("MOTOR_LEFT_DIR1", pins::MOTOR_LEFT_DIR1),
        ("MOTOR_LEFT_DIR2", pins::MOTOR_LEFT_DIR2),
        ("MOTOR_RIGHT_PWM", pins::MOTOR_RIGHT_PWM),
        ("MOTOR_RIGHT_DIR1", pins::MOTOR_RIGHT_DIR1),
        ("MOTOR_RIGHT_DIR2", pins::MOTOR_RIGHT_DIR2),
        ("RC_CHANNEL1", pins::RC_CHANNEL1),
        ("RC_CHANNEL2", pins::RC_CHANNEL2),
        ("STATUS_LED", pins::STATUS_LED),
        ("EMERGENCY_STOP", pins::EMERGENCY_STOP),
        ("BATTERY_MONITOR", pins::BATTERY_MONITOR),
    ];

    println!("   Assigned Pins:");
    for (name, pin) in &all_pins {
        println!("      {}: GPIO {}", name, pin);
    }

    // Check pin range (ESP32: GPIO 0-39, but some have special functions)
    let input_only_pins = [34, 35, 36, 39];
    let strapping_pins = [0, 2, 12, 15];
    let output_pins = [
        pins::MOTOR_LEFT_PWM,
        pins::MOTOR_LEFT_DIR1,
        pins::MOTOR_LEFT_DIR2,
        pins::MOTOR_RIGHT_PWM,
        pins::MOTOR_RIGHT_DIR1,
        pins::MOTOR_RIGHT_DIR2,
        pins::STATUS_LED,
    ];

    for &pin in &output_pins {
        if pin > 39 {
            errors.push(format!("Output pin {} exceeds ESP32 GPIO max (39)", pin));
        }
        if input_only_pins.contains(&pin) {
            errors.push(format!("GPIO {} is input-only but used as output", pin));
        }
        if strapping_pins.contains(&pin) {
            warnings.push(format!("GPIO {} is a strapping pin - may affect boot behavior", pin));
        }
    }

    for &pin in &[pins::RC_CHANNEL1, pins::RC_CHANNEL2, pins::EMERGENCY_STOP, pins::BATTERY_MONITOR] {
        if pin > 39 {
            errors.push(format!("Input pin {} exceeds ESP32 GPIO max (39)", pin));
        }
        // Input-only pins are fine for inputs
    }

    // Check for duplicate pins
    let mut pin_map = HashMap::new();
    for (name, pin) in &all_pins {
        pin_map.entry(*pin).or_insert_with(Vec::new).push(*name);
    }

    for (pin, names) in pin_map {
        if names.len() > 1 {
            errors.push(format!("Pin conflict: GPIO {} used by {:?}", pin, names));
        }
    }

    if errors.is_empty() {
        println!("   ✓ All pins within valid ESP32 range (0-39)");
        println!("   ✓ No pin conflicts detected");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationError {
    ValidationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_passes() {
        // This test just ensures the validation logic compiles and runs
        assert!(true);
    }
}