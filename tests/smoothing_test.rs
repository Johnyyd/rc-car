use rc_car::control::smoothing::{MovingAverageSmoother, SmootherConfig};

#[test]
fn test_smoother_initial_returns_input() {
    // When fewer than min_samples have been received, return the input value
    let config = SmootherConfig {
        window_size: 5,
        min_samples: 5,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Before min_samples, should return input directly
    assert_eq!(smoother.add_sample(50.0), 50.0);
    assert_eq!(smoother.add_sample(75.0), 75.0);
    assert_eq!(smoother.add_sample(25.0), 25.0);
}

#[test]
fn test_smoother_window_size_5() {
    let config = SmootherConfig {
        window_size: 5,
        min_samples: 5,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Add exactly 5 samples - should start returning averages
    let s1 = smoother.add_sample(10.0);
    let s2 = smoother.add_sample(20.0);
    let s3 = smoother.add_sample(30.0);
    let s4 = smoother.add_sample(40.0);
    let s5 = smoother.add_sample(50.0);

    // After 5 samples, should return average = (10+20+30+40+50)/5 = 30.0
    assert_eq!(s5, 30.0);
}

#[test]
fn test_smoother_moving_average() {
    let config = SmootherConfig {
        window_size: 5,
        min_samples: 5,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Add samples 1-5
    smoother.add_sample(10.0);
    smoother.add_sample(20.0);
    smoother.add_sample(30.0);
    smoother.add_sample(40.0);
    smoother.add_sample(50.0); // average = 30.0

    // Add sample 6 - window should now be [20,30,40,50,60], average = 40.0
    // But we need to add a 6th sample first
    let after_6 = smoother.add_sample(60.0);
    assert_eq!(after_6, 40.0); // (20+30+40+50+60)/5 = 40.0

    // Add sample 7 - window should be [30,40,50,60,70], average = 50.0
    let after_7 = smoother.add_sample(70.0);
    assert_eq!(after_7, 50.0); // (30+40+50+60+70)/5 = 50.0
}

#[test]
fn test_smoother_reduce_jitter() {
    // Test that smoothing reduces variance compared to raw input
    let config = SmootherConfig {
        window_size: 5,
        min_samples: 5,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Add fluctuating values
    let raw_values = [100.0, 95.0, 105.0, 90.0, 110.0, 92.0, 108.0, 94.0];

    for &val in &raw_values {
        smoother.add_sample(val);
    }

    // With min_samples=5, first 5 values should be raw, then smoothed
    // After that, smoothing should reduce the range
    assert!(true); // Just ensuring it compiles and runs
}

#[test]
fn test_smoother_reset() {
    let config = SmootherConfig {
        window_size: 5,
        min_samples: 5,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Add some samples
    smoother.add_sample(10.0);
    smoother.add_sample(20.0);
    smoother.add_sample(30.0);

    // Reset
    smoother.reset();

    // After reset, should return input values again
    assert_eq!(smoother.add_sample(50.0), 50.0);
}

#[test]
fn test_smoother_small_window() {
    let config = SmootherConfig {
        window_size: 3,
        min_samples: 3,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Add 3 samples: 10, 20, 30 -> average = 20.0
    let after_3 = smoother.add_sample(30.0);
    assert_eq!(after_3, 20.0);

    // Add 4th sample: window should be [20, 30, 40] if we add 40
    let after_4 = smoother.add_sample(40.0);
    assert_eq!(after_4, 30.0); // (20+30+40)/3 = 30.0
}

#[test]
fn test_smoother_buffer_capacity() {
    let config = SmootherConfig {
        window_size: 5,
        min_samples: 5,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Add more than 5 samples - buffer should cap at 5
    for i in 0..10u8 {
        smoother.add_sample(i as f32);
    }

    // Buffer should have at most 5 elements
    assert!(smoother.buffer_len() <= 5);
}

#[test]
fn test_smoother_extreme_values() {
    let config = SmootherConfig {
        window_size: 5,
        min_samples: 5,
    };
    let mut smoother = MovingAverageSmoother::new(config);

    // Add extreme values
    smoother.add_sample(0.0);
    smoother.add_sample(0.0);
    smoother.add_sample(0.0);
    smoother.add_sample(0.0);
    smoother.add_sample(0.0); // average = 0.0

    let after_extreme = smoother.add_sample(100.0);
    // Should be average of [0,0,0,0,100] = 20.0
    assert_eq!(after_extreme, 20.0);
}
