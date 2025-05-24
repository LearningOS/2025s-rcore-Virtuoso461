//! The panic handler

use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
/// panic handler
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        if let Some(message) = info.message() {
            println!(
                "[kernel] Panicked at {}:{} {}",
                location.file(),
                location.line(),
                message
            );
        } else {
            println!(
                "[kernel] Panicked at {}:{}",
                location.file(),
                location.line()
            );
        }
    } else {
        if let Some(message) = info.message() {
            println!("[kernel] Panicked: {}", message);
        } else {
            println!("[kernel] Panicked");
        }
    }
    shutdown()
}
