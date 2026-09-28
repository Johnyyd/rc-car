//! RC Car Controller
//!
//! Main controller that integrates all modules:
//! - Motor control
//! - Steering
//! - Safety monitoring
//! - Motion smoothing

use crate::control::{steering::SteeringController, SmoothingController};
use crate::safety::SafetyMonitor;
use esp_hal::time::{Duration, Instant};

/// Configuration for RC car controller
#[derive(Debug, Clone, Copy)]
pub struct RCCarConfig {
    /// Maximum motor speed (absolute value)
    pub max_speed: i8,
    /// Steering sensitivity (0.0 to 1.0)
    pub steering_sensitivity: f32,
    /// Smoothing window size
    pub smoothing_window: usize,
    /// Signal timeout in milliseconds
    pub signal_timeout_ms: u32,
    /// Control loop frequency in Hz
    pub loop_frequency_hz: u32,
}

impl Default for RCCarConfig {
    fn default() -> Self {
        Self {
            max_speed: 100,
            steering_sensitivity: 1.0,
            smoothing_window: 5,
            signal_timeout_ms: 1000,
            loop_frequency_hz: 100,
        }
    }
}

/// Main RC car controller
pub struct RCCarController {
    /// Configuration
    config: RCCarConfig,
    /// Steering controller
    steering_controller: SteeringController,
    /// Smoothing controller
    smoothing_controller: SmoothingController,
    /// Safety monitor
    safety_monitor: SafetyMonitor,
    /// Last control loop time
    last_loop_time: Option<Instant>,
    /// Control loop statistics
    loop_count: u32,
    /// Whether controller is initialized
    initialized: bool,
}

impl RCCarController {
    /// Creates a new RC car controller with default configuration
    pub fn new() -> Self {
        Self::with_config(RCCarConfig::default())
    }

    /// Creates a new RC car controller with custom configuration
    pub fn with_config(config: RCCarConfig) -> Self {
        let steering_controller = SteeringController::new(
            config.max_speed,
            config.steering_sensitivity,
        );
        let smoothing_controller = SmoothingController::new(config.smoothing_window);
        let safety_monitor = SafetyMonitor::new(
            config.signal_timeout_ms,
            config.max_speed,
        );

        Self {
            config,
            steering_controller,
            smoothing_controller,
            safety_monitor,
            last_loop_time: None,
            loop_count: 0,
            initialized: false,
        }
    }

    /// Initializes the controller
    pub fn init(&mut self) {
        self.initialized = true;
        self.last_loop_time = Some(Instant::now());
    }

    /// Main control loop iteration
    ///
    /// # Arguments
    /// * `steer_input` - Steering input from -100 to 100
    /// * `throttle_input` - Throttle input from -100 to 100
    ///
    /// # Returns
    /// Tuple of (left_motor_speed, right_motor_speed), each in range -100..100
    pub fn update(&mut self, steer_input: i8, throttle_input: i8) -> (i8, i8) {
        if !self.initialized {
            return (0, 0);
        }

        // Update loop timing
        let now = Instant::now();
        if let Some(last) = self.last_loop_time {
            let elapsed = now.duration_since(last);
            let expected_interval = Duration::from_millis((1000 / self.config.loop_frequency_hz as u64) as u32);

            // Check if loop is running too fast/slow
            // (This is for monitoring purposes)
            let _ = elapsed;
        }
        self.last_loop_time = Some(now);
        self.loop_count += 1;
        self.safety_monitor.increment_loop();

        // Check signal timeout
        self.safety_monitor.check_signal_timeout();

        // Check emergency stop
        if self.safety_monitor.is_emergency_stop() {
            return (0, 0);
        }

        // Smooth inputs
        let (steer_smooth, throttle_smooth) = self.smoothing_controller
            .process(steer_input, throttle_input);

        // Update safety monitor with signal reception
        self.safety_monitor.signal_received();

        // Validate inputs with safety limits
        let steer_safe = self.safety_monitor.validate_speed(steer_smooth);
        let throttle_safe = self.safety_monitor.validate_speed(throttle_smooth);

        // Calculate motor speeds using steering algorithm
        let (left_speed, right_speed) = self.steering_controller
            .calculate_motor_speeds(steer_safe, throttle_safe);

        // Apply final safety validation
        let left_safe = self.safety_monitor.validate_speed(left_speed);
        let right_safe = self.safety_monitor.validate_speed(right_speed);

        (left_safe, right_safe)
    }

    /// Gets the current loop count
    pub fn get_loop_count(&self) -> u32 {
        self.loop_count
    }

    /// Checks if controller is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Checks if emergency stop is active
    pub fn is_emergency_stop(&self) -> bool {
        self.safety_monitor.is_emergency_stop()
    }

    /// Activates emergency stop
    pub fn emergency_stop(&mut self) {
        self.safety_monitor.activate_emergency_stop();
    }

    /// Resets the controller
    pub fn reset(&mut self) {
        self.safety_monitor.reset_emergency_stop();
        self.smoothing_controller.reset();
        self.loop_count = 0;
        self.last_loop_time = Some(Instant::now());
    }

    /// Gets the safety monitor
    pub fn safety_monitor(&self) -> &SafetyMonitor {
        &self.safety_monitor
    }

    /// Gets the safety monitor mutably
    pub fn safety_monitor_mut(&mut self) -> &mut SafetyMonitor {
        &mut self.safety_monitor
    }

    /// Gets the current configuration
    pub fn get_config(&self) -> &RCCarConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc_car_controller_new() {
        let controller = RCCarController::new();
        assert!(!controller.is_initialized());
        assert_eq!(controller.get_loop_count(), 0);
    }

    #[test]
    fn test_rc_car_controller_init() {
        let mut controller = RCCarController::new();
        controller.init();
        assert!(controller.is_initialized());
    }

    #[test]
    fn test_rc_car_controller_update_uninitialized() {
        let mut controller = RCCarController::new();
        let (left, right) = controller.update(50, 50);
        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_rc_car_controller_update_initialized() {
        let mut controller = RCCarController::new();
        controller.init();
        let (left, right) = controller.update(0, 100);
        assert_eq!(left, 100);
        assert_eq!(right, 100);
        assert_eq!(controller.get_loop_count(), 1);
    }

    #[test]
    fn test_rc_car_controller_update_multiple() {
        let mut controller = RCCarController::new();
        controller.init();
        controller.update(0, 100);
        controller.update(0, 100);
        controller.update(0, 100);
        assert_eq!(controller.get_loop_count(), 3);
    }

    #[test]
    fn test_rc_car_controller_straight_forward() {
        let mut controller = RCCarController::new();
        controller.init();
        // Skip smoothing by using same input multiple times
        for _ in 0..10 {
            controller.update(0, 100);
        }
        let (left, right) = controller.update(0, 100);
        // With smoothing, values may be slightly different but should be close
        assert!(left > 0);
        assert!(right > 0);
    }

    #[test]
    fn test_rc_car_controller_stop() {
        let mut controller = RCCarController::new();
        controller.init();
        for _ in 0..10 {
            controller.update(0, 0);
        }
        let (left, right) = controller.update(0, 0);
        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_rc_car_controller_emergency_stop() {
        let mut controller = RCCarController::new();
        controller.init();
        controller.emergency_stop();
        assert!(controller.is_emergency_stop());
        let (left, right) = controller.update(0, 100);
        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_rc_car_controller_reset() {
        let mut controller = RCCarController::new();
        controller.init();
        controller.emergency_stop();
        controller.reset();
        assert!(!controller.is_emergency_stop());
        assert_eq!(controller.get_loop_count(), 0);
    }

    #[test]
    fn test_rc_car_controller_with_config() {
        let config = RCCarConfig {
            max_speed: 50,
            steering_sensitivity: 0.5,
            smoothing_window: 3,
            signal_timeout_ms: 500,
            loop_frequency_hz: 50,
        };
        let controller = RCCarController::with_config(config);
        assert!(controller.get_config().max_speed == 50);
        assert!(controller.get_config().steering_sensitivity == 0.5);
        assert!(controller.get_config().smoothing_window == 3);
        assert!(controller.get_config().signal_timeout_ms == 500);
        assert!(controller.get_config().loop_frequency_hz == 50);
    }

    #[test]
    fn test_rc_car_controller_turn() {
        let mut controller = RCCarController::new();
        controller.init();
        // Prime the smoothing filter
        for _ in 0..10 {
            controller.update(0, 100);
        }
        // Turn right
        let (left, right) = controller.update(100, 100);
        assert!(left > right);
    }

    #[test]
    fn test_rc_car_controller_reverse() {
        let mut controller = RCCarController::new();
        controller.init();
        for _ in 0..10 {
            controller.update(0, -100);
        }
        let (left, right) = controller.update(0, -100);
        assert!(left < 0);
        assert!(right < 0);
/// Main RC Car Controller
///
/// This module integrates all subsystems (motor control, RC input, safety, etc.)
/// into a unified control system for the RC car.

use esp_hal::{
    gpio::{self, Level, Output, OutputConfig, OutputPin},
    ledc::{self, channel, timer, Ledc, LowSpeed},
    time::{self, Duration, Instant},
};
use core::boxed::Box;
use core::convert::Infallible;

use crate::config::hardware::{PinMappings, constants};
use crate::drivers::motor::{MotorController, MotorConfig, MotorDirection};

/// Main controller for the RC car
pub struct RCCarController<'d> {
    /// Left motor controller
    left_motor: MotorController<'d>,
    /// Right motor controller
    right_motor: MotorController<'d>,
    /// Status LED output
    status_led: Output<'d>,
    /// Last time we blinked the LED
    last_led_toggle: Instant,
    /// LED state (on/off)
    led_state: bool,
    /// Last control loop execution time
    last_control_time: Instant,
    /// Whether the system is initialized
    initialized: bool,
}

impl<'d> RCCarController<'d> {
    /// Create a new RCCarController from LEDC peripheral and pin mappings
    ///
    /// # Arguments
    /// * `ledc` - LEDC peripheral
    /// * `pins` - Hardware pin mappings
    ///
    /// # Returns
    /// Result containing the controller or an error
    pub fn new(
        mut ledc: Ledc<'d>,
        pins: PinMappings,
        peripherals: &esp_hal::Peripherals,
    ) -> Result<Self, Box<dyn core::fmt::Debug>> {
        // Configure LEDC global settings
        ledc.timer::<LowSpeed>(ledc::timer::Number::Timer0).configure(
            ledc::timer::config::Config {
                duty: ledc::timer::config::Duty::Duty8Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: esp_hal::time::Rate::from_hz(constants::PWM_FREQUENCY_HZ),
            },
        );

        // Helper to get GPIO pins
        let get_pin = |num: u8| -> esp_hal::gpio::Pin {
            match num {
                2 => peripherals.GPIO2,
                18 => peripherals.GPIO18,
                19 => peripherals.GPIO19,
                21 => peripherals.GPIO21,
                22 => peripherals.GPIO22,
                23 => peripherals.GPIO23,
                25 => peripherals.GPIO25,
                32 => peripherals.GPIO32,
                33 => peripherals.GPIO33,
                _ => panic!("Unsupported pin"),
            }
        };

        // Configure left motor
        let left_motor_config = MotorConfig {
            timer: ledc::timer::Number::Timer0,
            channel: ledc::channel::Number::Channel0,
            frequency: constants::PWM_FREQUENCY_HZ,
            duty_resolution: ledc::timer::config::Duty::Duty8Bit,
            inverted: false,
        };

        let mut left_motor = MotorController::new_with_pins(
            ledc,
            get_pin(pins.left_motor_pwm),
            get_pin(pins.left_motor_dir1),
            get_pin(pins.left_motor_dir2),
            left_motor_config,
        )?;

        // Configure right motor (using different timer/channel to avoid conflicts)
        let mut ledc2 = ledc;
        let right_motor_config = MotorConfig {
            timer: ledc::timer::Number::Timer1,
            channel: ledc::channel::Number::Channel1,
            frequency: constants::PWM_FREQUENCY_HZ,
            duty_resolution: ledc::timer::config::Duty::Duty8Bit,
            inverted: false,
        };

        let mut right_motor = MotorController::new_with_pins(
            ledc2,
            get_pin(pins.right_motor_pwm),
            get_pin(pins.right_motor_dir1),
            get_pin(pins.right_motor_dir2),
            right_motor_config,
        )?;

        // Configure status LED
        let status_led = Output::new(
            get_pin(pins.status_led),
            Level::Low,
            OutputConfig::default().with_drive_mode(gpio::DriveMode::PushPull),
        );

        Ok(Self {
            left_motor,
            right_motor,
            status_led,
            last_led_toggle: Instant::now(),
            led_state: false,
            last_control_time: Instant::now(),
            initialized: false,
        })
    }

    /// Initialize all subsystems
    ///
    /// This must be called before using the controller
    pub fn init(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        if self.initialized {
            return Ok(());
        }

        // Initialize motors
        self.left_motor.init()?;
        self.right_motor.init()?;

        self.initialized = true;
        Ok(())
    }

    /// Enable both motors
    pub fn enable_motors(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.enable()?;
        self.right_motor.enable()?;
        Ok(())
    }

    /// Disable both motors (coast to stop)
    pub fn disable_motors(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.disable()?;
        self.right_motor.disable()?;
        Ok(())
    }

    /// Set motor speeds for differential steering
    ///
    /// # Arguments
    /// * `left_speed` - Left motor speed (-100 to 100)
    /// * `right_speed` - Right motor speed (-100 to 100)
    pub fn set_motor_speeds(
        &mut self,
        left_speed: i8,
        right_speed: i8,
    ) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.set_speed(left_speed)?;
        self.right_motor.set_speed(right_speed)?;
        Ok(())
    }

    /// Stop both motors
    pub fn stop(&mut self) -> Result<(), Box<dyn core::fmt::Debug>> {
        self.left_motor.stop()?;
        self.right_motor.stop()?;
        Ok(())
    }

    /// Update the status LED based on system activity
    ///
    /// This should be called regularly in the main loop
    pub fn update_status_led(&mut self, now: Instant) {
        // Blink LED at regular intervals to show system is alive
        if now.duration_since(self.last_led_toggle) >= Duration::from_millis(constants::LED_BLINK_RATE_MS) {
            self.led_state = !self.led_state;
            let _ = self.status_led.set_level(if self.led_state { Level::High } else { Level::Low });
            self.last_led_toggle = now;
        }
    }

    /// Run one iteration of the control loop
    ///
    /// In a real implementation, this would read RC input, apply control algorithms,
    /// and set motor outputs. For now, it just maintains the status LED.
    pub fn update(&mut self, now: Instant) -> Result<(), Box<dyn core::fmt::Debug>> {
        // Update status LED to show system is active
        self.update_status_led(now);

        // In a full implementation, we would:
        // 1. Read RC receiver inputs
        // 2. Apply safety checks
        // 3. Calculate motor commands using steering/throttle mixing
        // 4. Set motor speeds accordingly
        // 5. Handle any error conditions

        self.last_control_time = now;
        Ok(())
    }
}