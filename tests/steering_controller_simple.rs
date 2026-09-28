//! Steering Controller Simple Tests

#![cfg_attr(test, no_std)]
#![cfg_attr(not(test), no_std)]

use rc_car::control::steering::{calculate_motor_speeds, calculate_motor_speeds_default, SteeringConfig};

#[test]
fn test_straight_ahead() {
    let (left, right) = calculate_motor_speeds(0, 100, SteeringConfig::default());
    assert_eq!(left, 100);
    assert_eq!(right, 100);
}

#[test]
fn test_straight_reverse() {
    let (left, right) = calculate_motor_speeds(0, -100, SteeringConfig::default());
    assert_eq!(left, -100);
    assert_eq!(right, -100);
}

#[test]
fn test_turn_left() {
    let (left, right) = calculate_motor_speeds(-100, 100, SteeringConfig::default());
    assert!(left < right);
    assert!(left >= 0);
    assert!(right <= 100);
}

#[test]
fn test_turn_right() {
    let (left, right) = calculate_motor_speeds(100, 100, SteeringConfig::default());
    assert!(right < left);
    assert!(right >= 0);
    assert!(left <= 100);
}

#[test]
fn test_clamp_overflow() {
    let (left, right) = calculate_motor_speeds(100, 100, SteeringConfig::default());
    assert!(left >= -100 && left <= 100);
    assert!(right >= -100 && right <= 100);
}

#[test]
fn test_stop() {
    let (left, right) = calculate_motor_speeds(0, 0, SteeringConfig::default());
    assert_eq!(left, 0);
    assert_eq!(right, 0);
}