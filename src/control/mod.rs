//! Control module for RC car
//!
//! This module contains control algorithms including steering, smoothing,
//! and motion control for the RC car.

pub mod steering;
pub mod smoothing;

pub use steering::SteeringController;
pub use smoothing::{MovingAverageFilter, SmoothingController};