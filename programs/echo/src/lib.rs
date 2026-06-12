#![no_std]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    korrodium_sdk::print_str("name? ");
    let mut buf = [0u8; 64];
    match korrodium_sdk::read_line(&mut buf) {
        Some(name) => {
            korrodium_sdk::print_str("hello ");
            korrodium_sdk::print_str(name);
            korrodium_sdk::print_str("!\n");
        }
        None => korrodium_sdk::print_str("aborted\n"),
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }