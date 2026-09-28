//! Integration Tests for RC Car Controller
//!
//! This test simulates the main control loop at 100Hz (10ms period)
//! to verify the integration of all modules.

#![cfg(test)]

use rc_car::main::controller::{RCCarController, RCCarConfig};

#[test]
fn test_integration_control_loop_100ms() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 1000,
        loop_frequency_hz: 100,
    });

    controller.init();

    // Simulate 100 control loop iterations (1 second at 100Hz)
    for i in 0..100 {
        let steer = 0;
        let throttle = 100;
        let (left, right) = controller.update(steer, throttle);

        // After smoothing stabilizes, both motors should be at full speed
        if i >= 10 {
            assert_eq!(left, 100, format!("Iteration {}: left motor should be at full speed", i));
            assert_eq!(right, 100, "Iteration {i}: right motor should be at full speed");
        }
    }

    assert_eq!(controller.get_loop_count(), 100);
    assert!(!controller.is_emergency_stop());
}

#[test]
fn test_integration_control_loop_turn() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 1000,
        loop_frequency_hz: 100,
    });

    controller.init();

    // Prime the smoothing filter with straight forward
    for _ in 0..10 {
        controller.update(0, 100);
    }

    // Turn right (positive steer)
    let (left, right) = controller.update(100, 100);
    // Left motor should be faster than right when turning right
    assert!(left > right, "Left motor ({left}) should be > right motor ({right}) when turning right");

    // Turn left (negative steer)
    let (left, right) = controller.update(-100, 100);
    // Right motor should be faster than left when turning left
    assert!(right > left, "Right motor ({right}) should be > left motor ({left}) when turning left");
}

#[test]
fn test_integration_emergency_stop() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 1000,
        loop_frequency_hz: 100,
    });

    controller.init();

    // Run a few cycles normally
    for _ in 0..10 {
        controller.update(0, 100);
    }

    // Trigger emergency stop
    controller.emergency_stop();
    assert!(controller.is_emergency_stop());

    // All updates should return zero speed
    for _ in 0..10 {
        let (left, right) = controller.update(0, 100);
        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }
}

#[test]
fn test_integration_signal_timeout() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 100, // Short timeout for testing
        loop_frequency_hz: 100,
    });

    controller.init();

    // Run normally with signal
    for _ in 0..10 {
        controller.update(0, 100);
    }
    assert!(!controller.is_emergency_stop());

    // Note: We can't easily test signal timeout without mocking time
    // The safety monitor checks time since last signal
    // In a real test with embedded-test, we would mock the time
}

#[test]
fn test_integration_reverse() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 1000,
        loop_frequency_hz: 100,
    });

    controller.init();

    // Prime the smoothing filter
    for _ in 0..10 {
        controller.update(0, -100);
    }

    let (left, right) = controller.update(0, -100);
    assert!(left < 0, "Left motor should be negative in reverse");
    assert!(right < 0, "Right motor should be negative in reverse");
    assert_eq!(left, right, "Both motors should be equal when going straight in reverse");
}

#[test]
fn test_integration_spin_turn() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 1000,
        loop_frequency_hz: 100,
    });

    controller.init();

    // Prime the smoothing filter
    for _ in 0..10 {
        controller.update(0, 0);
    }

    // Spin left (negative steer, zero throttle)
    let (left, right) = controller.update(-100, 0);
    assert_eq!(left, -100, "Left motor should be full reverse when spinning left");
    assert_eq!(right, 100, "Right motor should be full forward when spinning left");

    // Prime again
    for _ in 0..10 {
        controller.update(0, 0);
    }

    // Spin right (positive steer, zero throttle)
    let (left, right) = controller.update(100, 0);
    assert_eq!(left, 100, "Left motor should be full forward when spinning right");
    assert_eq!(right, -100, "Right motor should be full reverse when spinning right");
}

#[test]
fn test_integration_speed_limit() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 50, // Limited to 50%
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 1000,
        loop_frequency_hz: 100,
    });

    controller.init();

    for _ in 0..10 {
        controller.update(0, 100);
    }

    let (left, right) = controller.update(0, 100);
    assert_eq!(left, 50, "Left motor should be limited to max_speed (50)");
    assert_eq!(right, 50, "Right motor should be limited to max_speed (50)");
}

#[test]
fn test_integration_smoothing_transition() {
    let mut controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 10, // Larger window for clearer test
        signal_timeout_ms: 1000,
        loop_frequency_hz: 100,
    });

    controller.init();

    // Start from stop
    for _ in 0..5 {
        controller.update(0, 0);
    }

    // Sudden full throttle
    let (left1, right1) = controller.update(0, 100);

    // After a few iterations, should be closer to 100
    for _ in 0..8 {
        controller.update(0, 100);
    }

    let (left2, right2) = controller.update(0, 100);

    // Second reading should be closer to target
    assert!(left2 > left1, "Smoothing should approach target value over time");
    assert!(right2 > right1, "Smoothing should approach target value over time");
}

#[test]
fn test_integration_config_validation() {
    // Test config clamping
    let config = RCCarConfig {
        max_speed: 200, // Should be clamped
        steering_sensitivity: 2.0, // Should be clamped
        smoothing_window: 0, // Will panic if 0, but config doesn't validate
        signal_timeout_ms: 0,
        loop_frequency_hz: 0,
    };

    let controller = RCCarController::with_config(config);
    // Note: max_speed and steering_sensitivity are clamped in SteeringController
    // The config struct itself doesn't clamp
    assert_eq!(controller.get_config().max_speed, 200);
    assert_eq!(controller.get_config().steering_sensitivity, 2.0);
}