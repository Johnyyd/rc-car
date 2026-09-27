//! Steering Controller Tests

#![cfg(test)]

use rc_car::control::steering::{calculate_motor_speeds, calculate_motor_speeds_default, SteeringConfig};

#[test]
fn test_straight_ahead() {
    // Going straight: steer=0, throttle=100
    let (left, right) = calculate_motor_speeds(0, 100, SteeringConfig::default());
    assert_eq!(left, 100, "Left motor should be 100 going straight");
    assert_eq!(right, 100, "Right motor should be 100 going straight");
}

#[test]
fn test_straight_reverse() {
    // Going reverse: steer=0, throttle=-100
    let (left, right) = calculate_motor_speeds(0, -100, SteeringConfig::default());
    assert_eq!(left, -100, "Left motor should be -100 going reverse");
    assert_eq!(right, -100, "Right motor should be -100 going reverse");
}

#[test]
fn test_turn_left() {
    // Turning left: steer=-100, throttle=100
    let (left, right) = calculate_motor_speeds(-100, 100, SteeringConfig::default());
    // Left wheel should be slower (or reverse), right wheel should be faster
    assert!(left < right, "Left speed ({}) should be less than right speed ({}) when turning left", left, right);
    // Both should have positive throttle component (forward)
    assert!(left >= 0, "Left speed should be non-negative with positive throttle and negative steer");
    assert!(right <= 100, "Right speed should not exceed max");
}

#[test]
fn test_turn_right() {
    // Turning right: steer=100, throttle=100
    let (left, right) = calculate_motor_speeds(100, 100, SteeringConfig::default());
    // Right wheel should be slower (or reverse), left wheel should be faster
    assert!(right < left, "Right speed ({}) should be less than left speed ({}) when turning right", right, left);
    // Both should have positive throttle component (forward)
    assert!(right >= 0, "Right speed should be non-negative with positive throttle and positive steer");
    assert!(left <= 100, "Left speed should not exceed max");
}

#[test]
fn test_turn_left_full() {
    // Full left turn with full throttle
    let (left, right) = calculate_motor_speeds(-100, 100, SteeringConfig::default());
    // At hard left with full forward, left may go to 0 or negative (reverse pivot)
    // and right goes to 100 (forward)
    assert!(right >= left, "Right speed should be >= left speed when turning left fully");
}

#[test]
fn test_turn_right_full() {
    // Full right turn with full throttle
    let (left, right) = calculate_motor_speeds(100, 100, SteeringConfig::default());
    // At hard right with full forward, left goes to 100, right may go to 0 or negative
    assert!(left >= right, "Left speed should be >= right speed when turning right fully");
}

#[test]
fn test_clamp_overflow() {
    // Extreme values should be clamped to [-100, 100]
    let (left, right) = calculate_motor_speeds(100, 100, SteeringConfig::default());
    assert!(left >= -100 && left <= 100, "Left speed {} should be in range [-100, 100]", left);
    assert!(right >= -100 && right <= 100, "Right speed {} should be in range [-100, 100]", right);
}

#[test]
fn test_backward_turn_left() {
    // Reversing while turning left
    let (left, right) = calculate_motor_speeds(-50, -100, SteeringConfig::default());
    // Both negative (reverse), left should be more negative (turning left while reverse)
    assert!(left < right, "Left speed ({}) should be less than right speed ({}) when backing up left", left, right);
}

#[test]
fn test_backward_turn_right() {
    // Reversing while turning right
    let (left, right) = calculate_motor_speeds(50, -100, SteeringConfig::default());
    // Both negative (reverse), right should be more negative (turning right while reverse)
    assert!(right < left, "Right speed ({}) should be less than left speed ({}) when backing up right", right, left);
}

#[test]
fn test_default_config() {
    let (left, right) = calculate_motor_speeds_default(0, 50);
    assert_eq!(left, 50);
    assert_eq!(right, 50);
}

#[test]
fn test_curve_factor_aggressive() {
    // With curve_factor > 1, steering has more effect
    let (left, right) = calculate_motor_speeds(50, 100, SteeringConfig { curve_factor: 2.0, ..Default::default() });
    // With aggressive curve, the steer effect is doubled
    assert!(left < right, "With aggressive curve, left ({}) should be less than right ({}) when turning left", left, right);
}

#[test]
fn test_curve_factor_mild() {
    // With curve_factor < 1, steering is less aggressive
    let (left, right) = calculate_motor_speeds(50, 100, SteeringConfig { curve_factor: 0.5, ..Default::default() });
    // With mild curve, the difference between left/right should be smaller
    let (left2, right2) = calculate_motor_speeds(50, 100, SteeringConfig { curve_factor: 1.0, ..Default::default() });
    let diff_curve = (left2.abs() - right2.abs()).abs();
    let diff_mild = (left.abs() - right.abs()).abs();
    // Mild curve should have smaller differential than linear
    assert!(diff_mild <= diff_curve || left == right, "Mild curve should reduce turning differential");
}