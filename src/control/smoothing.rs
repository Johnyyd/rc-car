/// Motion smoothing module for RC car control signals.
///
/// Provides a moving average filter to smooth noisy RC receiver signals,
/// reducing jitter while maintaining acceptable latency.

/// Configuration for the smoothing filter.
#[derive(Debug, Clone, Copy)]
pub struct SmootherConfig {
    /// Window size in samples (must be odd for symmetric filtering)
    pub window_size: usize,
    /// Minimum number of samples before outputting a value
    pub min_samples: usize,
}

impl Default for SmootherConfig {
    fn default() -> Self {
        Self {
            window_size: 5,
            min_samples: 5,
        }
    }
}

/// A moving average filter for smoothing RC control signals.
///
/// This filter maintains a sliding window of recent samples and returns
/// the average. It's useful for smoothing noisy joystick or sensor data
/// from RC receivers.
pub struct MovingAverageSmoother {
    /// Buffer storing recent samples (oldest to newest)
    buffer: [f32; 32],
    /// Current head position in circular buffer
    head: usize,
    /// Number of samples in buffer
    len: usize,
    /// Current sum of buffer values for O(1) averaging
    sum: f32,
    /// Configuration
    config: SmootherConfig,
    /// Number of samples received so far
    samples_received: usize,
}

impl MovingAverageSmoother {
    /// Creates a new moving average smoother with the given configuration.
    pub fn new(config: SmootherConfig) -> Self {
        let window_size = config.window_size.max(1).min(32);
        let min_samples = config.min_samples.max(1);

        Self {
            buffer: [0.0; 32],
            head: 0,
            len: 0,
            sum: 0.0,
            config: SmootherConfig {
                window_size,
                min_samples,
            },
            samples_received: 0,
        }
    }

    /// Add a new sample and return the smoothed value.
    ///
    /// # Arguments
    /// * `value` - The new input sample
    ///
    /// # Returns
    /// The smoothed output value. Returns the input value if fewer
    /// than `min_samples` have been received.
    pub fn add_sample(&mut self, value: f32) -> f32 {
        self.samples_received += 1;

        // Remove oldest value from sum if buffer is full
        if self.len >= self.config.window_size {
            let oldest_idx = (self.head + self.len - self.config.window_size) % 32;
            let oldest = self.buffer[oldest_idx];
            self.sum -= oldest;
        } else {
            self.len += 1;
        }

        // Add new value to buffer and sum
        self.buffer[self.head] = value;
        self.sum += value;

        // Move head
        self.head = (self.head + 1) % 32;

        // Return input value until we have enough samples
        if self.samples_received < self.config.min_samples {
            return value;
        }

        // Return the average
        self.sum / self.len as f32
    }

    /// Reset the smoother to its initial state.
    pub fn reset(&mut self) {
        self.buffer = [0.0; 32];
        self.head = 0;
        self.len = 0;
        self.sum = 0.0;
        self.samples_received = 0;
    }

    /// Get the current window size.
    pub fn window_size(&self) -> usize {
        self.config.window_size
    }

    /// Get the minimum samples threshold.
    pub fn min_samples(&self) -> usize {
        self.config.min_samples
    }

    /// Get the number of samples currently in the buffer.
    pub fn buffer_len(&self) -> usize {
        self.len
    }
}
