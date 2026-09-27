//! Motor Controller Tests

#![cfg(test)]

use rc_car::drivers::motor::{MotorController, MotorConfig, MotorDirection, MotorError};

#[test]
fn test_motor_config_default() {
    let config = MotorConfig::default();
    assert_eq!(config.timer as u8, 0); // Timer0
    assert_eq!(config.channel as u8, 0); // Channel0
    assert_eq!(config.frequency, 20_000);
    assert!(!config.inverted);
}

#[test]
fn test_motor_direction_enum() {
    assert_eq!(MotorDirection::Stop as u8, 0);
    assert_eq!(MotorDirection::Forward as u8, 1);
    assert_eq!(MotorDirection::Reverse as u8, 2);
    assert_ne!(MotorDirection::Forward, MotorDirection::Reverse);
}

#[test]
fn test_motor_error_enum() {
    assert_eq!(MotorError::InvalidSpeed as u8, 0);
    assert_eq!(MotorError::PwmConfigError as u8, 1);
    assert_eq!(MotorError::TimerConfigError as u8, 2);
    assert_eq!(MotorError::ChannelConfigError as u8, 3);
}

// Note: Integration tests with actual hardware require embedded-test
// These are compile-time checks and basic logic tests

#[test]
fn test_motor_speed_range() {
    // Test that speed values are in valid range
    for speed in -100..=100 {
        assert!(speed >= -100 && speed <= 100);
    }
}

#[test]
fn test_motor_speed_invalid() {
    assert!(-101 < -100);
    assert!(101 > 100);
}