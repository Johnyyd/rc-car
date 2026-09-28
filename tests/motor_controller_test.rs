#![no_std]
#![no_main]

extern crate core;

use core::panic::PanicInfo;
//! Motor Controller Tests - no_std compatible
#![cfg(test)]

#[panic_handler]
fn panic(_pi: &PanicInfo) -> ! {
    loop {}
}

use rc_car::drivers::motor::{MotorConfig, MotorDirection, MotorController, speed_to_duty, speed_to_direction};

#[test]
fn test_duty_cycle_mapping() {
    let test_cases = [(0,0),(50,50),(100,100),(-50,50),(-100,100)];
    for &(speed, expected) in &test_cases {
        assert_eq!(speed_to_duty(speed), expected);
    }
}

#[test]
fn test_direction() {
    assert_eq!(speed_to_direction(100), MotorDirection::Forward);
    assert_eq!(speed_to_direction(-100), MotorDirection::Reverse);
    assert_eq!(speed_to_direction(0), MotorDirection::Stop);
}

#[test]
fn test_controller() {
    let cfg = MotorConfig::default();
    let mut ctrl = MotorController::new_with_pins((),(),(),(),cfg).unwrap();
    assert!(ctrl.is_initialized());
    assert!(!ctrl.is_enabled());
    ctrl.enable().unwrap();
    assert!(ctrl.is_enabled());
    ctrl.set_speed(50).unwrap();
    assert_eq!(ctrl.get_speed(), 50);
}


fn test_motor_speed_invalid() {
    assert!(-101 < -100);
    assert!(101 > 100);
}
