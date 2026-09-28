#![no_std]
#![allow(special_module_name)]

pub mod config;
pub mod control;
pub mod drivers;
pub mod main;
pub mod safety;

#[cfg(test)]
mod tests {
    // Test module for unit tests
}
