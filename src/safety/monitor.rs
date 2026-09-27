//! Safety Monitor Module
//!
//! This module provides safety features including watchdog timeout,
//! signal loss detection, and emergency stop functionality.

use esp_hal::time::{Duration, Instant};

/// Safety monitor for RC car
pub struct SafetyMonitor {
    /// Maximum allowed time without RC signal (milliseconds)
    signal_timeout_ms: u32,
    /// Last time a valid RC signal was received
    last_signal_time: Option<Instant>,
    /// Whether emergency stop is active
    emergency_stop: bool,
    /// Maximum allowed motor speed (absolute value)
    max_speed: i8,
    /// Current loop iteration count
    loop_count: u32,
}

impl SafetyMonitor {
    /// Creates a new safety monitor
    pub fn new(signal_timeout_ms: u32, max_speed: i8) -> Self {
        Self {
            signal_timeout_ms,
            last_signal_time: None,
            emergency_stop: false,
            max_speed: max_speed.clamp(0, 100),
            loop_count: 0,
        }
    }

    /// Updates the last signal time to now
    pub fn signal_received(&mut self) {
        self.last_signal_time = Some(Instant::now());
    }

    /// Checks if signal timeout has occurred
    pub fn check_signal_timeout(&mut self) -> bool {
        if let Some(last_time) = self.last_signal_time {
            let elapsed = Instant::now().duration_since_epoch().as_millis() as i64
                - last_time.duration_since_epoch().as_millis() as i64;
            if elapsed >= self.signal_timeout_ms as i64 {
                self.emergency_stop = true;
                return true;
            }
        }
        false
    }

    /// Checks if emergency stop is active
    pub fn is_emergency_stop(&self) -> bool {
        self.emergency_stop
    }

    /// Activates emergency stop manually
    pub fn activate_emergency_stop(&mut self) {
        self.emergency_stop = true;
    }

    /// Resets emergency stop (requires valid signal)
    pub fn reset_emergency_stop(&mut self) {
        if self.last_signal_time.is_some() {
            self.emergency_stop = false;
        }
    }

    /// Validates and clamps motor speed to safe limits
    pub fn validate_speed(&self, speed: i8) -> i8 {
        if self.emergency_stop {
            return 0;
        }
        speed.clamp(-self.max_speed, self.max_speed)
    }

    /// Gets the maximum allowed speed
    pub fn get_max_speed(&self) -> i8 {
        self.max_speed
    }

    /// Sets the maximum allowed speed
    pub fn set_max_speed(&mut self, max_speed: i8) {
        self.max_speed = max_speed.clamp(0, 100);
    }

    /// Gets the signal timeout in milliseconds
    pub fn get_signal_timeout_ms(&self) -> u32 {
        self.signal_timeout_ms
    }

    /// Sets the signal timeout in milliseconds
    pub fn set_signal_timeout_ms(&mut self, timeout_ms: u32) {
        self.signal_timeout_ms = timeout_ms;
    }

    /// Increments loop counter
    pub fn increment_loop(&mut self) {
        self.loop_count = self.loop_count.wrapping_add(1);
    }

    /// Gets current loop count
    pub fn get_loop_count(&self) -> u32 {
        self.loop_count
    }

    /// Gets time since last signal in milliseconds
    pub fn get_time_since_signal_ms(&self) -> Option<u32> {
        self.last_signal_time.map(|last_time| {
            let elapsed = Instant::now().duration_since_epoch().as_millis() as i64
                - last_time.duration_since_epoch().as_millis() as i64;
            elapsed.max(0) as u32
        })
    }
}

impl Default for SafetyMonitor {
    fn default() -> Self {
        Self::new(1000, 100) // 1 second timeout, full speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use esp_hal::time::Duration;

    #[test]
    fn test_safety_monitor_default() {
        let monitor = SafetyMonitor::default();
        assert_eq!(monitor.get_signal_timeout_ms(), 1000);
        assert_eq!(monitor.get_max_speed(), 100);
        assert!(!monitor.is_emergency_stop());
    }

    #[test]
    fn test_safety_monitor_new() {
        let monitor = SafetyMonitor::new(500, 50);
        assert_eq!(monitor.get_signal_timeout_ms(), 500);
        assert_eq!(monitor.get_max_speed(), 50);
    }

    #[test]
    fn test_signal_received() {
        let mut monitor = SafetyMonitor::new(1000, 100);
        assert!(monitor.get_time_since_signal_ms().is_none());
        monitor.signal_received();
        assert!(monitor.get_time_since_signal_ms().is_some());
    }

    #[test]
    fn test_validate_speed_no_emergency() {
        let monitor = SafetyMonitor::new(1000, 50);
        assert_eq!(monitor.validate_speed(100), 50);
        assert_eq!(monitor.validate_speed(-100), -50);
        assert_eq!(monitor.validate_speed(25), 25);
        assert_eq!(monitor.validate_speed(0), 0);
    }

    #[test]
    fn test_validate_speed_emergency() {
        let mut monitor = SafetyMonitor::new(1000, 100);
        monitor.activate_emergency_stop();
        assert_eq!(monitor.validate_speed(100), 0);
        assert_eq!(monitor.validate_speed(-50), 0);
        assert_eq!(monitor.validate_speed(0), 0);
    }

    #[test]
    fn test_emergency_stop_activation() {
        let mut monitor = SafetyMonitor::new(1000, 100);
        assert!(!monitor.is_emergency_stop());
        monitor.activate_emergency_stop();
        assert!(monitor.is_emergency_stop());
    }

    #[test]
    fn test_emergency_stop_reset() {
        let mut monitor = SafetyMonitor::new(1000, 100);
        monitor.activate_emergency_stop();
        assert!(monitor.is_emergency_stop());
        monitor.signal_received();
        monitor.reset_emergency_stop();
        assert!(!monitor.is_emergency_stop());
    }

    #[test]
    fn test_emergency_stop_reset_without_signal() {
        let mut monitor = SafetyMonitor::new(1000, 100);
        monitor.activate_emergency_stop();
        // Without signal, reset should not work
        monitor.reset_emergency_stop();
        assert!(monitor.is_emergency_stop());
    }

    #[test]
    fn test_loop_counter() {
        let mut monitor = SafetyMonitor::new(1000, 100);
        assert_eq!(monitor.get_loop_count(), 0);
        monitor.increment_loop();
        assert_eq!(monitor.get_loop_count(), 1);
        monitor.increment_loop();
        assert_eq!(monitor.get_loop_count(), 2);
    }

    #[test]
    fn test_loop_counter_wraps() {
        let mut monitor = SafetyMonitor::new(1000, 100);
        monitor.loop_count = u32::MAX;
        monitor.increment_loop();
        assert_eq!(monitor.get_loop_count(), 0);
    }
}