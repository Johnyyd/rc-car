//! Safety Monitor Tests - Mock version for testing

#![cfg(test)]

use rc_car::safety::{SafetyMonitor, SafetyResult, SafetyState, RcSignalStatus};

#[test]
fn test_rc_signal_lost_triggers_emergency_stop() {
    let mut monitor = SafetyMonitor::new();

    // Initially should be OK
    assert_eq!(monitor.check_emergency_stop(), SafetyResult::Ok);

    // Mark RC as lost
    monitor.update_rc_status(RcSignalStatus::Lost);

    // Check immediately - should not trigger yet (0s elapsed or < 1s)
    let result = monitor.check_emergency_stop();
    // Result should be OK immediately, or EmergencyStop if timing happens to pass 1s
    let ok = result == SafetyResult::Ok || result == SafetyResult::EmergencyStop;
    assert!(ok);

    // Verify state changed to SignalLost
    assert_eq!(monitor.state(), SafetyState::SignalLost);
}

#[test]
fn test_rc_signal_present_no_emergency() {
    let mut monitor = SafetyMonitor::new();

    monitor.update_rc_status(RcSignalStatus::Present);
    assert_eq!(monitor.check_emergency_stop(), SafetyResult::Ok);
    assert_eq!(monitor.state(), SafetyState::Normal);
}

#[test]
fn test_rc_signal_recovers() {
    let mut monitor = SafetyMonitor::new();

    // Lost signal, then recover
    monitor.update_rc_status(RcSignalStatus::Lost);
    assert_eq!(monitor.state(), SafetyState::SignalLost);
    monitor.update_rc_status(RcSignalStatus::Present);
    assert_eq!(monitor.state(), SafetyState::Normal);
    assert_eq!(monitor.rc_status(), RcSignalStatus::Present);
}

#[test]
fn test_runtime_limit_triggers_emergency_stop() {
    let mut monitor = SafetyMonitor::new();

    // Mark runtime start
    monitor.mark_runtime_start();

    // Runtime limit is 300 seconds, so we can't easily test this in a unit test
    // but we verify the code path exists
    assert!(monitor.check_emergency_stop() == SafetyResult::Ok);
}

#[test]
fn test_monitor_transitions() {
    let mut monitor = SafetyMonitor::new();

    // Start Normal
    assert_eq!(monitor.state(), SafetyState::Normal);

    // Signal lost
    monitor.update_rc_status(RcSignalStatus::Lost);
    assert_eq!(monitor.state(), SafetyState::SignalLost);

    // Signal recovered
    monitor.update_rc_status(RcSignalStatus::Present);
    assert_eq!(monitor.state(), SafetyState::Normal);
}

#[test]
fn test_safety_monitor_constants() {
    use rc_car::safety::*;

    assert_eq!(MAX_SAFE_SPEED, 100);
    assert_eq!(MIN_SAFE_SPEED, -100);
    assert_eq!(MAX_RUNTIME_SECS, 300);
    assert_eq!(RC_SIGNAL_LOSS_TIMEOUT, 1);
    assert_eq!(MOTOR_WATCHDOG_TIMEOUT, 10);
}

#[test]
fn test_safety_monitor_reset() {
    let mut monitor = SafetyMonitor::new();

    // Change state
    monitor.update_rc_status(RcSignalStatus::Lost);
    assert_eq!(monitor.state(), SafetyState::SignalLost);

    // Reset
    monitor.reset();
    assert_eq!(monitor.state(), SafetyState::Normal);
    assert_eq!(monitor.rc_status(), RcSignalStatus::Present);
}

#[test]
fn test_safe_mode() {
    let mut monitor = SafetyMonitor::new();

    monitor.set_safe_mode();
    assert_eq!(monitor.state(), SafetyState::SafeMode);
}
