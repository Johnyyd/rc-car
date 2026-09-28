//! Safety Monitor Module for RC Car
//!
//! This module provides safety monitoring, signal loss detection,
//! and emergency stop functionality for the RC car system.

use esp_hal::time::Instant;

/// Maximum safe speed percentage (-100 to 100)
pub const MAX_SAFE_SPEED: i8 = 100;

/// Minimum safe speed percentage (-100 to 100)
pub const MIN_SAFE_SPEED: i8 = -100;

/// Maximum runtime in seconds before automatic shutoff
pub const MAX_RUNTIME_SECS: u64 = 300; // 5 minutes

/// RC signal loss timeout in seconds before emergency stop
pub const RC_SIGNAL_LOSS_TIMEOUT: u64 = 1; // 1 second

/// Watchdog timeout for motor operations in seconds
pub const MOTOR_WATCHDOG_TIMEOUT: u64 = 10;

/// Safety state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyState {
    /// System operating normally
    Normal,
    /// RC signal lost, emergency stop triggered
    SignalLost,
    /// System in safe mode, reduced operation
    SafeMode,
    /// Critical error, immediate stop required
    Critical,
}

/// RC signal status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RcSignalStatus {
    /// RC signal present and valid
    Present,
    /// RC signal lost
    Lost,
    /// RC signal invalid (noise, interference)
    Invalid,
}

/// Safety monitoring result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyResult {
    /// No action needed
    Ok,
    /// Emergency stop triggered
    EmergencyStop,
    /// System entering safe mode
    SafeMode,
    /// Critical error detected
    Critical,
}

/// Safety Monitor - watches RC signal and system state for safety violations
pub struct SafetyMonitor {
    state: SafetyState,
    rc_status: RcSignalStatus,
    last_rc_change: Option<Instant>,
    last_signal_time: Option<Instant>,
    runtime_start: Option<Instant>,
    emergency_stop: bool,
    signal_timeout_ms: u32,
    max_speed: i8,
    loop_count: u32,
}

impl Default for SafetyMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyMonitor {
    /// Creates a new safety monitor with default parameters (1s timeout, 100 max speed)
    pub fn new() -> Self {
        Self::with_params(1000, 100)
    }

    /// Creates a new safety monitor with custom timeout and max speed
    pub fn with_params(signal_timeout_ms: u32, max_speed: i8) -> Self {
        Self {
            state: SafetyState::Normal,
            rc_status: RcSignalStatus::Present,
            last_rc_change: None,
            last_signal_time: None,
            runtime_start: None,
            emergency_stop: false,
            signal_timeout_ms,
            max_speed: max_speed.clamp(0, 100),
            loop_count: 0,
        }
    }

    /// Update RC signal status
    pub fn update_rc_status(&mut self, status: RcSignalStatus) {
        self.rc_status = status;

        match status {
            RcSignalStatus::Present => {
                self.last_rc_change = None;
                self.last_signal_time = Some(Instant::now());
                if self.state == SafetyState::SignalLost {
                    self.state = SafetyState::Normal;
                    self.emergency_stop = false;
                }
            }
            RcSignalStatus::Lost => {
                self.last_rc_change = Some(Instant::now());
                if self.state == SafetyState::Normal {
                    self.state = SafetyState::SignalLost;
                }
            }
            RcSignalStatus::Invalid => {
                self.rc_status = RcSignalStatus::Present;
            }
        }
    }

    /// Check if emergency stop should be triggered
    pub fn check_emergency_stop(&mut self) -> SafetyResult {
        let result = match self.rc_status {
            RcSignalStatus::Present => SafetyResult::Ok,
            RcSignalStatus::Lost => {
                if let Some(start) = self.last_rc_change {
                    let elapsed = start.elapsed().as_secs();
                    if elapsed >= RC_SIGNAL_LOSS_TIMEOUT {
                        self.emergency_stop = true;
                        return SafetyResult::EmergencyStop;
                    }
                }
                SafetyResult::Ok
            }
            RcSignalStatus::Invalid => SafetyResult::Critical,
        };

        if let Some(start) = self.runtime_start {
            let elapsed = start.elapsed().as_secs();
            if elapsed >= MAX_RUNTIME_SECS {
                self.emergency_stop = true;
                return SafetyResult::EmergencyStop;
            }
        }

        result
    }

    /// Updates the last signal time to now
    pub fn signal_received(&mut self) {
        self.last_signal_time = Some(Instant::now());
        self.update_rc_status(RcSignalStatus::Present);
    }

    /// Checks if signal timeout has occurred based on milliseconds
    pub fn check_signal_timeout(&mut self) -> bool {
        if let Some(last_time) = self.last_signal_time {
            let elapsed_ms = last_time.elapsed().as_millis() as u64;
            if elapsed_ms >= self.signal_timeout_ms as u64 {
                self.emergency_stop = true;
                self.state = SafetyState::SignalLost;
                return true;
            }
        }
        false
    }

    /// Checks if emergency stop is active
    pub fn is_emergency_stop(&self) -> bool {
        self.emergency_stop || self.state == SafetyState::Critical
    }

    /// Activates emergency stop manually
    pub fn activate_emergency_stop(&mut self) {
        self.emergency_stop = true;
        self.state = SafetyState::Critical;
    }

    /// Resets emergency stop (requires valid signal)
    pub fn reset_emergency_stop(&mut self) {
        if self.last_signal_time.is_some() || self.rc_status == RcSignalStatus::Present {
            self.emergency_stop = false;
            self.state = SafetyState::Normal;
        }
    }

    /// Validates and clamps motor speed to safe limits
    pub fn validate_speed(&self, speed: i8) -> i8 {
        if self.is_emergency_stop() {
            return 0;
        }
        speed.clamp(-self.max_speed, self.max_speed)
    }

    /// Mark runtime start (when system starts operating)
    pub fn mark_runtime_start(&mut self) {
        self.runtime_start = Some(Instant::now());
    }

    /// Get current safety state
    pub fn state(&self) -> SafetyState {
        self.state
    }

    /// Get current RC signal status
    pub fn rc_status(&self) -> RcSignalStatus {
        self.rc_status
    }

    /// Set the system to safe mode
    pub fn set_safe_mode(&mut self) {
        self.state = SafetyState::SafeMode;
    }

    /// Reset the monitor to normal operation
    pub fn reset(&mut self) {
        self.state = SafetyState::Normal;
        self.rc_status = RcSignalStatus::Present;
        self.last_rc_change = None;
        self.last_signal_time = None;
        self.runtime_start = None;
        self.emergency_stop = false;
        self.loop_count = 0;
    }

    /// Increments loop counter
    pub fn increment_loop(&mut self) {
        self.loop_count = self.loop_count.wrapping_add(1);
    }

    /// Gets current loop count
    pub fn get_loop_count(&self) -> u32 {
        self.loop_count
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

    /// Gets time since last signal in milliseconds
    pub fn get_time_since_signal_ms(&self) -> Option<u32> {
        self.last_signal_time.map(|last_time| last_time.elapsed().as_millis() as u32)
    }
}

pub mod monitor;
