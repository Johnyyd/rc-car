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