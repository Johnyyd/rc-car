//! Motion Smoothing Module
//!
//! This module provides signal smoothing using moving average filters
//! to reduce noise and provide smooth control inputs.

/// Moving average filter for smoothing signals
pub struct MovingAverageFilter {
    /// Window size for averaging
    window_size: usize,
    /// Buffer of recent samples
    buffer: Vec<i8>,
    /// Current index in buffer
    current_index: usize,
    /// Whether buffer is full
    buffer_full: bool,
}

impl MovingAverageFilter {
    /// Creates a new moving average filter
    pub fn new(window_size: usize) -> Self {
        if window_size == 0 {
            panic!("Window size must be greater than 0");
        }
        Self {
            window_size,
            buffer: vec![0; window_size],
            current_index: 0,
            buffer_full: false,
        }
    }

    /// Processes a new sample and returns the smoothed value
    pub fn process(&mut self, sample: i8) -> i8 {
        // Store sample in buffer
        self.buffer[self.current_index] = sample;

        // Update index
        self.current_index = (self.current_index + 1) % self.window_size;

        // Check if buffer is full
        if self.current_index == 0 {
            self.buffer_full = true;
        }

        // Calculate average
        self.get_average()
    }

    /// Gets the current average of buffered samples
    pub fn get_average(&self) -> i8 {
        let sum: i32 = if self.buffer_full {
            self.buffer.iter().map(|&v| v as i32).sum()
        } else {
            // Only average over filled portion
            let fill_count = self.current_index.min(self.window_size);
            self.buffer[..fill_count].iter().map(|&v| v as i32).sum()
        };

        let count = if self.buffer_full {
            self.window_size
        } else {
            self.current_index.min(self.window_size)
        };

        if count == 0 {
            return 0;
        }

        (sum / count as i32) as i8
    }

    /// Resets the filter
    pub fn reset(&mut self) {
        self.buffer.fill(0);
        self.current_index = 0;
        self.buffer_full = false;
    }

    /// Gets the window size
    pub fn get_window_size(&self) -> usize {
        self.window_size
    }

    /// Checks if buffer is full
    pub fn is_buffer_full(&self) -> bool {
        self.buffer_full
    }
}

impl Default for MovingAverageFilter {
    fn default() -> Self {
        Self::new(5)
    }
}

/// Smoothing controller for RC inputs
pub struct SmoothingController {
    /// Filter for steering input
    steer_filter: MovingAverageFilter,
    /// Filter for throttle input
    throttle_filter: MovingAverageFilter,
}

impl SmoothingController {
    /// Creates a new smoothing controller
    pub fn new(window_size: usize) -> Self {
        Self {
            steer_filter: MovingAverageFilter::new(window_size),
            throttle_filter: MovingAverageFilter::new(window_size),
        }
    }

    /// Processes steering and throttle inputs
    pub fn process(&mut self, steer: i8, throttle: i8) -> (i8, i8) {
        let smoothed_steer = self.steer_filter.process(steer);
        let smoothed_throttle = self.throttle_filter.process(throttle);
        (smoothed_steer, smoothed_throttle)
    }

    /// Resets both filters
    pub fn reset(&mut self) {
        self.steer_filter.reset();
        self.throttle_filter.reset();
    }
}

impl Default for SmoothingController {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average_filter_new() {
        let filter = MovingAverageFilter::new(5);
        assert_eq!(filter.get_window_size(), 5);
        assert!(!filter.is_buffer_full());
    }

    #[test]
    fn test_moving_average_filter_single_sample() {
        let mut filter = MovingAverageFilter::new(5);
        let result = filter.process(100);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_moving_average_filter_multiple_samples() {
        let mut filter = MovingAverageFilter::new(5);
        // Process samples: 10, 20, 30, 40, 50
        // Average = (10+20+30+40+50)/5 = 30
        filter.process(10);
        filter.process(20);
        filter.process(30);
        filter.process(40);
        let result = filter.process(50);
        assert_eq!(result, 30);
        assert!(filter.is_buffer_full());
    }

    #[test]
    fn test_moving_average_filter_wrap_around() {
        let mut filter = MovingAverageFilter::new(3);
        filter.process(10);
        filter.process(20);
        filter.process(30);
        // Buffer: [10, 20, 30]
        assert_eq!(filter.get_average(), 20);
        // Next sample overwrites first
        filter.process(40);
        // Buffer: [40, 20, 30]
        assert_eq!(filter.get_average(), 30);
    }

    #[test]
    fn test_moving_average_filter_reset() {
        let mut filter = MovingAverageFilter::new(5);
        filter.process(100);
        filter.process(200);
        assert!(!filter.is_buffer_full());
        filter.reset();
        assert!(!filter.is_buffer_full());
        assert_eq!(filter.get_average(), 0);
    }

    #[test]
    fn test_moving_average_filter_constant_input() {
        let mut filter = MovingAverageFilter::new(5);
        for _ in 0..10 {
            let result = filter.process(50);
            assert_eq!(result, 50);
        }
    }

    #[test]
    fn test_smoothing_controller_new() {
        let controller = SmoothingController::new(5);
        assert_eq!(controller.steer_filter.get_window_size(), 5);
        assert_eq!(controller.throttle_filter.get_window_size(), 5);
    }

    #[test]
    fn test_smoothing_controller_process() {
        let mut controller = SmoothingController::new(3);
        let (steer, throttle) = controller.process(10, 20);
        assert_eq!(steer, 10);
        assert_eq!(throttle, 20);
    }

    #[test]
    fn test_smoothing_controller_smoothing() {
        let mut controller = SmoothingController::new(3);
        // Sudden change
        controller.process(0, 0);
        controller.process(0, 0);
        let (steer, throttle) = controller.process(100, 100);
        // With window of 3: (0+0+100)/3 = 33
        assert_eq!(steer, 33);
        assert_eq!(throttle, 33);
    }

    #[test]
    fn test_smoothing_controller_reset() {
        let mut controller = SmoothingController::new(5);
        controller.process(100, 100);
        controller.reset();
        assert!(!controller.steer_filter.is_buffer_full());
        assert!(!controller.throttle_filter.is_buffer_full());
    }

    #[test]
    fn test_smoothing_controller_stability() {
        let mut controller = SmoothingController::new(5);
        // Constant input should remain constant
        for _ in 0..10 {
            let (steer, throttle) = controller.process(50, 75);
            assert_eq!(steer, 50);
            assert_eq!(throttle, 75);
        }
    }
}