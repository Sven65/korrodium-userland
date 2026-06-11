#![no_std]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    korrodium_sdk::print_str("hello from rust userland\n");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }