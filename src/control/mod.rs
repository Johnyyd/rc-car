//! Control module for RC car
//!
//! This module contains control algorithms including steering, smoothing,
//! and motion control for the RC car.

pub mod smoothing;
pub mod steering;

pub use smoothing::{MovingAverageFilter, SmoothingController};
pub use steering::SteeringController;
pub mod optimizer;
