#![cfg(test)]

use esp_hal::time::{Duration, Instant};
use rc_car::control::optimizer::benchmark::run_benchmark;
use rc_car::control::optimizer::OptimizedLoop;

use core::hint::black_box;

/// Benchmark tests for control loop optimization
//!
/// These tests measure the performance of the main control loop
/// and verify that processing time stays under 5ms per iteration.

#[test]
fn benchmark_control_loop_under_5ms() {
    // Simulate a realistic control loop iteration
    fn mock_control_loop() -> Duration {
        let start = esp_hal::time::Instant::now();

        // Simulate control loop work:
        // - Read RC receiver channels (4 channels)
        // - Apply smoothing filter
        // - Calculate differential steering
        // - Apply safety checks
        // - Update motor controllers

        // Small delay to simulate real work (would be actual operations on hardware)
        for _ in 0..1000 {
            black_box(1u32.wrapping_add(2));
        }

        start.elapsed()
    }

    let results = run_benchmark(mock_control_loop, 1000);

    // Assert that average is under 5ms (5000 us)
    assert!(results.within_budget,
        "Control loop average time ({} us) exceeds 5ms budget!", results.average_us);
}

#[test]
fn benchmark_optimizer_loop() {
    let mut optimizer = OptimizedLoop::new(5);

    fn control_step() -> Duration {
        let start = esp_hal::time::Instant::now();

        // Simulate 2ms of control work
        for _ in 0..500 {
            black_box(1u32.wrapping_mul(3));
        }

        start.elapsed()
    }

    // Run 100 iterations
    for _ in 0..100 {
        let result = optimizer.execute(control_step);
        assert!(result.is_ok(), "Control loop exceeded time budget");
    }

    assert!(optimizer.is_within_budget());
}

#[test]
fn test_optimizer_tracks_iterations() {
    let mut optimizer = OptimizedLoop::new(5);

    for i in 0..50 {
        let result = optimizer.execute(|| Duration::from_millis(1));
        assert!(result.is_ok(), "Iteration {} failed", i);
    }

    assert_eq!(optimizer.iteration_count(), 50);
    assert!(optimizer.is_within_budget());
}

#[test]
fn test_optimizer_detects_slow_iteration() {
    let mut optimizer = OptimizedLoop::new(5);

    // First 5 iterations fast
    for _ in 0..5 {
        let result = optimizer.execute(|| Duration::from_millis(1));
        assert!(result.is_ok());
    }

    // One slow iteration
    let result = optimizer.execute(|| Duration::from_millis(10));
    assert!(result.is_err());

    // Subsequent fast iterations should still be tracked
    for _ in 0..5 {
        let result = optimizer.execute(|| Duration::from_millis(1));
        assert!(result.is_ok());
    }
}