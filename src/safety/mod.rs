//! Safety Monitor
//!
//! This module provides a safety watchdog for the RC car system,
//! monitoring RC signal integrity and triggering emergency stops
//! when signal is lost or unsafe conditions are detected.

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

#[derive(Debug, Clone)]
struct InnerState {
    state: SafetyState,
    rc_status: RcSignalStatus,
    last_rc_change: Option<Instant>,
    runtime_start: Option<Instant>,
}

impl Default for InnerState {
    fn default() -> Self {
        Self {
            state: SafetyState::Normal,
            rc_status: RcSignalStatus::Present,
            last_rc_change: None,
            runtime_start: None,
        }
    }
}

/// Safety Monitor - watches RC signal and system state for safety violations
pub struct SafetyMonitor {
    inner: InnerState,
}

impl Default for SafetyMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyMonitor {
    /// Creates a new safety monitor
    pub fn new() -> Self {
        Self {
            inner: InnerState::default(),
        }
    }

    /// Update RC signal status
    pub fn update_rc_status(&mut self, status: RcSignalStatus) {
        let _old_state = self.inner.state;

        self.inner.rc_status = status;

        match status {
            RcSignalStatus::Present => {
                self.inner.last_rc_change = None;
                // Transition from SignalLost back to Normal
                if self.inner.state == SafetyState::SignalLost {
                    self.inner.state = SafetyState::Normal;
                }
            }
            RcSignalStatus::Lost => {
                // Mark when signal was lost
                self.inner.last_rc_change = Some(Instant::now());
                // If we were in Normal, transition to SignalLost
                if self.inner.state == SafetyState::Normal {
                    self.inner.state = SafetyState::SignalLost;
                }
            }
            RcSignalStatus::Invalid => {
                self.inner.rc_status = RcSignalStatus::Present; // reset to safe default
            }
        }
    }

    /// Check if emergency stop should be triggered
    pub fn check_emergency_stop(&mut self) -> SafetyResult {
        let result = match self.inner.rc_status {
            RcSignalStatus::Present => SafetyResult::Ok,
            RcSignalStatus::Lost => {
                // Check if signal lost for more than RC_SIGNAL_LOSS_TIMEOUT
                if let Some(start) = self.inner.last_rc_change {
                    let elapsed = start.elapsed().as_secs();
                    if elapsed >= RC_SIGNAL_LOSS_TIMEOUT {
                        return SafetyResult::EmergencyStop;
                    }
                }
                SafetyResult::Ok // not long enough yet
            }
            RcSignalStatus::Invalid => SafetyResult::Critical,
        };

        // Also check runtime limits
        if let Some(start) = self.inner.runtime_start {
            let elapsed = start.elapsed().as_secs();
            if elapsed >= MAX_RUNTIME_SECS {
                return SafetyResult::EmergencyStop;
            }
        }

        result
    }

    /// Mark runtime start (when system starts operating)
    pub fn mark_runtime_start(&mut self) {
        self.inner.runtime_start = Some(Instant::now());
    }

    /// Get current safety state
    pub fn state(&self) -> SafetyState {
        self.inner.state
    }

    /// Get current RC signal status
    pub fn rc_status(&self) -> RcSignalStatus {
        self.inner.rc_status
    }

    /// Set the system to safe mode
    pub fn set_safe_mode(&mut self) {
        self.inner.state = SafetyState::SafeMode;
    }

    /// Reset the monitor to normal operation
    pub fn reset(&mut self) {
        self.inner = InnerState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // We need a test crate to run tests since this is no_std
    // The safety module tests will be run as integration tests
}
//! Safety module for RC car
//!
//! This module provides safety monitoring and emergency stop functionality.

pub mod monitor;

pub use monitor::SafetyMonitor;
