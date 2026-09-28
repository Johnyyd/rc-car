#![cfg_attr(not(test), no_std)]

pub mod config;
pub mod control;
pub mod drivers;
pub mod main;

#[cfg(test)]
mod tests {
    // Test module for unit tests
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
