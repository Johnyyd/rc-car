#![cfg_attr(not(test), no_std)]

pub mod drivers;
pub mod control;

#[cfg(test)]
mod tests {
    // Test module for unit tests
}
