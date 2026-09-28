#![no_std]

pub mod config;
pub mod drivers;

#[cfg(test)]
mod tests {
    // Test module for unit tests
}

pub mod control;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
