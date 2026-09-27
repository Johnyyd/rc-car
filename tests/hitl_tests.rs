//! Hardware-in-the-Loop Tests for RC Car Control Logic
//! These tests verify control logic without requiring hardware dependencies

// Test motor direction calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MotorDirection {
    Stop,
    Forward,
    Reverse,
}

// Mock motor controller for testing logic
struct MockMotorController {
    dir1_high: bool,
    dir2_high: bool,
    pwm_duty: u8,
    current_speed: i8,
    enabled: bool,
}

impl MockMotorController {
    fn new() -> Self {
        Self {
            dir1_high: false,
            dir2_high: false,
            pwm_duty: 0,
            current_speed: 0,
            enabled: false,
        }
    }

    fn set_direction(&mut self, direction: MotorDirection) {
        match direction {
            MotorDirection::Stop => {
                self.dir1_high = false;
                self.dir2_high = false;
            }
            MotorDirection::Forward => {
                self.dir1_high = true;
                self.dir2_high = false;
            }
            MotorDirection::Reverse => {
                self.dir1_high = false;
                self.dir2_high = true;
            }
        }
    }

    fn set_speed(&mut self, speed: i8) -> Result<(), &'static str> {
        if speed < -100 || speed > 100 {
            return Err("Speed must be -100 to 100");
        }

        self.current_speed = speed;

        if !self.enabled {
            return Ok(());
        }

        let direction = if speed > 0 {
            MotorDirection::Forward
        } else if speed < 0 {
            MotorDirection::Reverse
        } else {
            MotorDirection::Stop
        };

        self.set_direction(direction);
        self.pwm_duty = speed.abs() as u8;

        Ok(())
    }

    fn enable(&mut self) -> Result<(), &'static str> {
        self.enabled = true;
        self.set_speed(self.current_speed)
    }

    fn disable(&mut self) -> Result<(), &'static str> {
        self.enabled = false;
        self.set_direction(MotorDirection::Stop);
        self.pwm_duty = 0;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), &'static str> {
        self.set_speed(0)
    }

    fn get_speed(&self) -> i8 {
        self.current_speed
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// Differential steering logic (same as in actual code)
fn calculate_motor_speeds(steer: i8, throttle: i8) -> (i8, i8) {
    let steer_factor = steer as f32 / 100.0;
    let throttle_factor = throttle as f32 / 100.0;

    let left_speed = (throttle_factor * (1.0 + steer_factor)) * 100.0;
    let right_speed = (throttle_factor * (1.0 - steer_factor)) * 100.0;

    (
        left_speed.max(-100.0).min(100.0) as i8,
        right_speed.max(-100.0).min(100.0) as i8,
    )
}

// RC pulse width to speed conversion (same as in actual code)
fn pulse_to_speed(pulse_width: u16) -> i8 {
    // Convert 1000-2000us pulse to -100 to 100 speed
    // 1500us = 0 (neutral)
    // 1000us = -100 (full reverse)
    // 2000us = 100 (full forward)
    let center = 1500u16;
    let range = 500u16;

    if pulse_width < 1000 || pulse_width > 2000 {
        return 0; // Invalid pulse width, return neutral
    }

    let offset = pulse_width as i32 - center as i32;
    let speed = (offset as f32 / range as f32 * 100.0) as i8;

    speed.clamp(-100, 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motor_direction_pins() {
        let mut controller = MockMotorController::new();

        // Test forward
        controller.set_direction(MotorDirection::Forward);
        assert_eq!(controller.dir1_high, true);
        assert_eq!(controller.dir2_high, false);

        // Test reverse
        controller.set_direction(MotorDirection::Reverse);
        assert_eq!(controller.dir1_high, false);
        assert_eq!(controller.dir2_high, true);

        // Test stop
        controller.set_direction(MotorDirection::Stop);
        assert_eq!(controller.dir1_high, false);
        assert_eq!(controller.dir2_high, false);
    }

    #[test]
    fn test_motor_speed_range() {
        let mut controller = MockMotorController::new();

        // Valid speed range
        assert!(controller.set_speed(100).is_ok());
        assert!(controller.set_speed(-100).is_ok());
        assert!(controller.set_speed(0).is_ok());
        assert!(controller.set_speed(50).is_ok());
        assert!(controller.set_speed(-50).is_ok());

        // Invalid speed range
        assert!(controller.set_speed(101).is_err());
        assert!(controller.set_speed(-101).is_err());
    }

    #[test]
    fn test_motor_speed_mapping() {
        let mut controller = MockMotorController::new();
        controller.enable().unwrap();

        // Test forward speed
        controller.set_speed(75).unwrap();
        assert_eq!(controller.get_speed(), 75);
        assert_eq!(controller.pwm_duty, 75);

        // Test reverse speed
        controller.set_speed(-50).unwrap();
        assert_eq!(controller.get_speed(), -50);
        assert_eq!(controller.pwm_duty, 50);

        // Test zero speed
        controller.set_speed(0).unwrap();
        assert_eq!(controller.get_speed(), 0);
        assert_eq!(controller.pwm_duty, 0);
    }

    #[test]
    fn test_motor_enable_disable() {
        let mut controller = MockMotorController::new();

        // Initially disabled
        assert!(!controller.is_enabled());
        controller.set_speed(50).unwrap();
        assert_eq!(controller.pwm_duty, 0); // No PWM when disabled

        // Enable and set speed
        controller.enable().unwrap();
        assert!(controller.is_enabled());
        assert_eq!(controller.pwm_duty, 50); // PWM now active

        // Disable
        controller.disable().unwrap();
        assert!(!controller.is_enabled());
        assert_eq!(controller.pwm_duty, 0); // PWM stopped
    }

    #[test]
    fn test_motor_stop() {
        let mut controller = MockMotorController::new();
        controller.enable().unwrap();
        controller.set_speed(80).unwrap();

        controller.stop().unwrap();
        assert_eq!(controller.get_speed(), 0);
        assert_eq!(controller.pwm_duty, 0);
    }

    #[test]
    fn test_steering_straight() {
        let steer = 0i8;
        let throttle = 50i8;

        let (left, right) = calculate_motor_speeds(steer, throttle);

        assert_eq!(left, 50);
        assert_eq!(right, 50);
    }

    #[test]
    fn test_steering_left_turn() {
        let steer = -50i8;
        let throttle = 50i8;

        let (left, right) = calculate_motor_speeds(steer, throttle);

        // Left should be slower, right faster
        assert!(left < right);
        assert!(left < 50);
        assert!(right > 50);
    }

    #[test]
    fn test_steering_right_turn() {
        let steer = 50i8;
        let throttle = 50i8;

        let (left, right) = calculate_motor_speeds(steer, throttle);

        // Right should be slower, left faster
        assert!(left > right);
        assert!(right < 50);
        assert!(left > 50);
    }

    #[test]
    fn test_steering_stop() {
        let steer = 0i8;
        let throttle = 0i8;

        let (left, right) = calculate_motor_speeds(steer, throttle);

        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_rc_pulse_to_speed_neutral() {
        let pulse_width = 1500u16; // Neutral position
        let speed = pulse_to_speed(pulse_width);
        assert_eq!(speed, 0);
    }

    #[test]
    fn test_rc_pulse_to_speed_max_forward() {
        let pulse_width = 2000u16; // Max forward
        let speed = pulse_to_speed(pulse_width);
        assert_eq!(speed, 100);
    }

    #[test]
    fn test_rc_pulse_to_speed_max_reverse() {
        let pulse_width = 1000u16; // Max reverse
        let speed = pulse_to_speed(pulse_width);
        assert_eq!(speed, -100);
    }

    #[test]
    fn test_rc_pulse_to_speed_mid_forward() {
        let pulse_width = 1750u16; // Half forward
        let speed = pulse_to_speed(pulse_width);
        assert_eq!(speed, 50);
    }

    #[test]
    fn test_rc_pulse_to_speed_mid_reverse() {
        let pulse_width = 1250u16; // Half reverse
        let speed = pulse_to_speed(pulse_width);
        assert_eq!(speed, -50);
    }
}