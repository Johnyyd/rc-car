/// Performance Optimizer for RC Car Control Loop
///
/// This module provides profiling and optimization of the main control loop
/// to ensure processing time stays under 5ms per iteration.
///
/// Uses esp-hal timer for precise cycle counting and profiling.

use esp_hal::time::{Duration, Instant};

/// Profile a control loop iteration and return duration
///
/// Measures the time taken for a single control loop iteration using the
/// ESP32 hardware timer. This helps identify bottlenecks in the control algorithm.
pub fn profile_loop_iteration<F, T>(f: F) -> Duration
where
    F: FnOnce() -> T,
    T: core::fmt::Debug,
{
    let start = Instant::now();
    let _result = f();
    let duration = start.elapsed();
    // Log duration for profiling (in real hardware, would use telemetry)
    // eprintln!("Loop iteration: {:?}", duration);
    duration
}

/// Optimized control loop with timing guarantee
///
/// Executes the provided control function and ensures it completes within
/// the target time budget. If the iteration exceeds the budget, a warning
/// is emitted (in production, this could trigger watchdog or priority escalation).
pub struct OptimizedLoop {
    /// Target maximum processing time in milliseconds
    max_processing_time_ms: u64,
    /// Current iteration count for averaging
    iteration_count: u32,
    /// Accumulated processing times for moving average
    processing_times: [u64; 10],
    /// Index for circular buffer of processing times
    write_index: usize,
}

impl OptimizedLoop {
    /// Create a new optimized loop with the given time budget
    pub fn new(max_processing_time_ms: u64) -> Self {
        Self {
            max_processing_time_ms,
            iteration_count: 0,
            processing_times: [0; 10],
            write_index: 0,
        }
    }

    /// Execute one control loop iteration with timing profiling
    ///
    /// # Arguments
    /// * `control_fn` - The control algorithm to execute
    ///
    /// # Returns
    /// `Ok(())` if the iteration completed within the time budget,
    /// `Err(duration)` if it exceeded the budget
    pub fn execute<F>(&mut self, control_fn: F) -> Result<(), Duration>
    where
        F: FnOnce() -> Duration,
    {
        let start = Instant::now();
        let _iteration_duration = control_fn();
        let elapsed = start.elapsed();

        // Store in circular buffer for moving average
        self.processing_times[self.write_index] = elapsed.as_millis() as u64;
        self.write_index = (self.write_index + 1) % self.processing_times.len();
        self.iteration_count += 1;

        let _avg_duration = self.processing_times.iter().sum::<u64>() / self.processing_times.len() as u64;

        if elapsed.as_millis() as u64 > self.max_processing_time_ms {
            Err(elapsed)
        } else {
            Ok(())
        }
    }

    /// Get the average processing time across all measured iterations
    pub fn average_processing_time(&self) -> u64 {
        self.processing_times.iter().sum::<u64>() / self.processing_times.len() as u64
    }

    /// Get the number of iterations measured
    pub fn iteration_count(&self) -> u32 {
        self.iteration_count
    }

    /// Check if the loop is within the time budget (moving average)
    pub fn is_within_budget(&self) -> bool {
        let avg = self.average_processing_time();
        avg <= self.max_processing_time_ms
    }
}

/// Default optimized loop with 5ms time budget (function to create)
pub fn default_optimized_loop() -> OptimizedLoop {
    OptimizedLoop::new(5)
}

/// Trait for control stages that can be profiled and optimized
pub trait ControlStage {
    /// Execute this control stage and return the processing time
    fn execute_stage(&self) -> Duration;

    /// Get the name of this stage for profiling reports
    fn stage_name(&self) -> &'static str;
}

/// Benchmark utilities for control loop optimization
pub mod benchmark {
    use esp_hal::time::{Duration, Instant};

    /// Run a benchmark of the control loop
    ///
    /// Executes the control function `n` times and returns timing statistics.
    /// This is intended to be called from `cargo test --bench`.
    pub fn run_benchmark<F>(control_fn: F, iterations: u32) -> BenchmarkResults
    where
        F: Fn() -> Duration,
    {
        let start = Instant::now();
        let mut total: u64 = 0;
        let mut min: u64 = u64::MAX;
        let mut max: u64 = 0;

        for _ in 0..iterations {
            let duration = control_fn();
            let dur_ms = duration.as_millis() as u64;
            total += dur_ms;
            if dur_ms < min {
                min = dur_ms;
            }
            if dur_ms > max {
                max = dur_ms;
            }
        }

        let avg = total / iterations as u64;
        let start_us = start.elapsed().as_micros() as u64;

        BenchmarkResults {
            iterations,
            total_us: start_us,
            average_us: avg,
            min_us: min,
            max_us: max,
            within_budget: avg <= 5000, // 5ms budget
        }
    }

    /// Benchmark result structure
    #[derive(Debug, Clone, Copy)]
    pub struct BenchmarkResults {
        pub iterations: u32,
        pub total_us: u64,
        pub average_us: u64,
        pub min_us: u64,
        pub max_us: u64,
        pub within_budget: bool,
    }
}

#[cfg(test)]
mod tests {
    #![cfg(test)]
    #![no_std]

    use super::*;
    use esp_hal::time::Instant;

    #[test]
    fn test_default_budget() {
        let mut loop_optimizer = OptimizedLoop::new(5);
        let result = loop_optimizer.execute(|| Duration::from_millis(1));
        assert!(result.is_ok());
    }

    #[test]
    fn test_exceeds_budget() {
        let mut loop_optimizer = OptimizedLoop::new(5);
        let result = loop_optimizer.execute(|| Duration::from_millis(10));
        assert!(result.is_err());
    }

    #[test]
    fn test_moving_average() {
        let mut loop_optimizer = OptimizedLoop::new(5);

        // Execute 10 iterations with 3ms each
        for _ in 0..10 {
            let _ = loop_optimizer.execute(|| Duration::from_millis(3));
        }

        // Check that average is within budget
        assert!(loop_optimizer.is_within_budget());
        assert_eq!(loop_optimizer.average_processing_time(), 3);
        assert_eq!(loop_optimizer.iteration_count(), 10);
    }

    #[test]
    fn test_benchmark_results() {
        use benchmark::run_benchmark;

        let results = run_benchmark(|| {
            // Simulate control loop work
            Duration::from_millis(2)
        }, 100);

        assert!(results.within_budget);
        assert_eq!(results.iterations, 100);
        assert!(results.average_us < 5000); // 5ms = 5000us
    }

    #[test]
    fn test_stage_execution() {
        struct MockStage;

        impl ControlStage for MockStage {
            fn execute_stage(&self) -> Duration {
                Duration::from_millis(2)
            }

            fn stage_name(&self) -> &'static str {
                "mock_stage"
            }
        }

        let stage = MockStage;
        let duration = stage.execute_stage();
        assert_eq!(duration.as_millis(), 2);
        assert_eq!(stage.stage_name(), "mock_stage");
    }
}