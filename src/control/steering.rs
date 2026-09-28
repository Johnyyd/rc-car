/// Differential steering controller for RC car with 2 independent motor drivers

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SteeringConfig {
    pub max_speed: i8,
    pub curve_factor: f32,
}

impl Default for SteeringConfig {
    fn default() -> Self {
        Self {
            max_speed: 100,
            curve_factor: 1.0,
        }
    }
}

/// Calculate motor speeds for differential steering: left = throttle - steer, right = throttle + steer
pub fn calculate_motor_speeds(steer: i8, throttle: i8, _config: SteeringConfig) -> (i8, i8) {
    let left = throttle.saturating_sub(steer);
    let right = throttle.saturating_add(steer);
    let left_clamped = left.max(-100).min(100);
    let right_clamped = right.max(-100).min(100);
    (left_clamped, right_clamped)
}

/// Default version with standard config
pub fn calculate_motor_speeds_default(steer: i8, throttle: i8) -> (i8, i8) {
    calculate_motor_speeds(steer, throttle, SteeringConfig::default())
}
use crate::drivers::motor::MotorDirection;

/// Steering controller for differential drive RC car
pub struct SteeringController {
    /// Maximum motor speed (absolute value)
    max_speed: i8,
    /// Steering sensitivity factor (0.0 to 1.0)
    steering_sensitivity: f32,
}

impl SteeringController {
    /// Creates a new steering controller
    pub fn new(max_speed: i8, steering_sensitivity: f32) -> Self {
        Self {
            max_speed: max_speed.clamp(0, 100),
            steering_sensitivity: steering_sensitivity.clamp(0.0, 1.0),
        }
    }

    /// Calculates motor speeds for differential steering
    ///
    /// # Arguments
    /// * `steer` - Steering input from -100 (full left) to 100 (full right)
    /// * `throttle` - Throttle input from -100 (full reverse) to 100 (full forward)
    ///
    /// # Returns
    /// Tuple of (left_motor_speed, right_motor_speed), each in range -100..100
    pub fn calculate_motor_speeds(&self, steer: i8, throttle: i8) -> (i8, i8) {
        // Clamp inputs
        let steer = steer.clamp(-100, 100);
        let throttle = throttle.clamp(-100, 100);

        // Convert to normalized values (-1.0 to 1.0)
        let steer_norm = steer as f32 / 100.0;
        let throttle_norm = throttle as f32 / 100.0;

        // Apply steering sensitivity
        let steer_norm = steer_norm * self.steering_sensitivity;

        // Differential steering algorithm
        // Left motor = throttle + steer, Right motor = throttle - steer
        let left_raw = throttle_norm + steer_norm;
        let right_raw = throttle_norm - steer_norm;

        // Clamp to [-1.0, 1.0]
        let left_clamped = left_raw.clamp(-1.0, 1.0);
        let right_clamped = right_raw.clamp(-1.0, 1.0);

        // Scale to max_speed
        let left_val = left_clamped * self.max_speed as f32;
        let right_val = right_clamped * self.max_speed as f32;
        let left_speed = if left_val >= 0.0 { (left_val + 0.5) as i8 } else { (left_val - 0.5) as i8 };
        let right_speed = if right_val >= 0.0 { (right_val + 0.5) as i8 } else { (right_val - 0.5) as i8 };

        (left_speed, right_speed)
    }

    /// Calculates motor directions from speeds
    pub fn calculate_directions(
        left_speed: i8,
        right_speed: i8,
    ) -> (MotorDirection, MotorDirection) {
        let left_dir = if left_speed > 0 {
            MotorDirection::Forward
        } else if left_speed < 0 {
            MotorDirection::Reverse
        } else {
            MotorDirection::Stop
        };

        let right_dir = if right_speed > 0 {
            MotorDirection::Forward
        } else if right_speed < 0 {
            MotorDirection::Reverse
        } else {
            MotorDirection::Stop
        };

        (left_dir, right_dir)
    }
}

impl Default for SteeringController {
    fn default() -> Self {
        Self::new(100, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steering_straight_forward() {
        let controller = SteeringController::new(100, 1.0);
        let (left, right) = controller.calculate_motor_speeds(0, 100);
        assert_eq!(left, 100);
        assert_eq!(right, 100);
    }

    #[test]
    fn test_steering_straight_reverse() {
        let controller = SteeringController::new(100, 1.0);
        let (left, right) = controller.calculate_motor_speeds(0, -100);
        assert_eq!(left, -100);
        assert_eq!(right, -100);
    }

    #[test]
    fn test_steering_turn_left() {
        let controller = SteeringController::new(100, 1.0);
        let (left, right) = controller.calculate_motor_speeds(-100, 100);
        // Full left turn: left motor slows/stops, right motor goes full speed
        assert!(left <= right);
    }

    #[test]
    fn test_steering_turn_right() {
        let controller = SteeringController::new(100, 1.0);
        let (left, right) = controller.calculate_motor_speeds(100, 100);
        // Full right turn: right motor slows/stops, left motor goes full speed
        assert!(right <= left);
    }

    #[test]
    fn test_steering_spin_left() {
        let controller = SteeringController::new(100, 1.0);
        let (left, right) = controller.calculate_motor_speeds(-100, 0);
        // Spin left: left reverse, right forward
        assert_eq!(left, -100);
        assert_eq!(right, 100);
    }

    #[test]
    fn test_steering_spin_right() {
        let controller = SteeringController::new(100, 1.0);
        let (left, right) = controller.calculate_motor_speeds(100, 0);
        // Spin right: left forward, right reverse
        assert_eq!(left, 100);
        assert_eq!(right, -100);
    }

    #[test]
    fn test_steering_stop() {
        let controller = SteeringController::new(100, 1.0);
        let (left, right) = controller.calculate_motor_speeds(0, 0);
        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_steering_max_speed_limit() {
        let controller = SteeringController::new(50, 1.0);
        let (left, right) = controller.calculate_motor_speeds(0, 100);
        assert_eq!(left, 50);
        assert_eq!(right, 50);
    }

    #[test]
    fn test_steering_sensitivity() {
        let controller = SteeringController::new(100, 0.5);
        let (left, right) = controller.calculate_motor_speeds(100, 100);
        // With 0.5 sensitivity, steer is halved
        // left = 100 + 50 = 150 -> clamped to 100
        // right = 100 - 50 = 50
        assert_eq!(left, 100);
        assert_eq!(right, 50);
    }

    #[test]
    fn test_steering_directions() {
        assert_eq!(
            SteeringController::calculate_directions(100, 100),
            (MotorDirection::Forward, MotorDirection::Forward)
        );
        assert_eq!(
            SteeringController::calculate_directions(-100, -100),
            (MotorDirection::Reverse, MotorDirection::Reverse)
        );
        assert_eq!(
            SteeringController::calculate_directions(0, 0),
            (MotorDirection::Stop, MotorDirection::Stop)
        );
        assert_eq!(
            SteeringController::calculate_directions(-50, 50),
            (MotorDirection::Reverse, MotorDirection::Forward)
        );
    }
}
