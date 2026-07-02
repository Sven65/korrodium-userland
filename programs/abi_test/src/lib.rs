#![no_std]

use core::panic::PanicInfo;
use korrodium_sdk::fs;
use korrodium_sdk::screen::{self, Color};

const FS_PATH: &str = "/wasmi_test.txt";
const FS_CONTENT: &[u8] = b"hello from wasm\n";
const FS_APPEND_CONTENT: &[u8] = b"more data\n";
const FS_DIR_PATH: &str = "/wasmi_test_dir";

fn report(fails: &mut u32, label: &str, ok: bool) {
    korrodium_sdk::print_str(label);
    korrodium_sdk::print_str(if ok { " PASS\n" } else { " FAIL\n" });
    if !ok {
        *fails += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    korrodium_sdk::print_str("wasmi ABI smoke test\n");
    let mut fails = 0u32;

    // Screen checks run first and stash their results in locals: `print_str`
    // and `os::screen` share the same VGA buffer, so later prints could
    // scroll row 24 away before we get a chance to read it back.
    let color = screen::color(Color::White, Color::Black);

    screen::write_at(24, 0, b'X', color);
    let screen_write_ok = screen::read_at(24, 0).map(|(ascii, _)| ascii) == Some(b'X');

    let col_after = screen::write_str_at(24, 10, "WASMI", color);
    let screen_write_str_ok = col_after == 15;
    let screen_write_str_content_ok = screen::read_at(24, 10).map(|(ascii, _)| ascii) == Some(b'W');

    screen::clear_row(24, color);
    let screen_clear_row_ok = screen::read_at(24, 0).map(|(ascii, _)| ascii) == Some(b' ');

    // Proves the resumable-trap trampoline resumes correctly with no values.
    korrodium_sdk::yield_now();
    korrodium_sdk::print_str("yield_now returned\n");

    report(&mut fails, "time::now() returns Some", korrodium_sdk::now().is_some());

    let args_bytes = korrodium_sdk::args_len();
    let mut args_buf = [0u8; 256];
    let args_items = korrodium_sdk::args(&mut args_buf).count();
    report(&mut fails, "args_len/args agree on emptiness", (args_bytes == 0) == (args_items == 0));

    let mut read_buf = [0u8; 256];

    report(&mut fails, "fs write_file", fs::write_file(FS_PATH, FS_CONTENT));

    let read = fs::read_file(FS_PATH, &mut read_buf);
    report(&mut fails, "fs read_file length matches", read.map(<[u8]>::len) == Some(FS_CONTENT.len()));
    report(&mut fails, "fs read_file content matches", read == Some(FS_CONTENT));

    report(&mut fails, "fs append_file", fs::append_file(FS_PATH, FS_APPEND_CONTENT));

    let read = fs::read_file(FS_PATH, &mut read_buf);
    let expected_len = FS_CONTENT.len() + FS_APPEND_CONTENT.len();
    report(&mut fails, "fs read_file length after append", read.map(<[u8]>::len) == Some(expected_len));
    report(
        &mut fails,
        "fs append_file tail content matches",
        read.map(|d| &d[FS_CONTENT.len()..]) == Some(FS_APPEND_CONTENT),
    );

    report(&mut fails, "fs delete_file", fs::delete_file(FS_PATH));
    report(&mut fails, "fs read_file returns None after delete", fs::read_file(FS_PATH, &mut read_buf).is_none());

    report(&mut fails, "fs create_dir", fs::create_dir(FS_DIR_PATH));
    report(&mut fails, "fs delete_dir", fs::delete_dir(FS_DIR_PATH));

    report(&mut fails, "screen width == 80", screen::width() == 80);
    report(&mut fails, "screen height == 25", screen::height() == 25);
    report(&mut fails, "screen write_at/read_at roundtrip", screen_write_ok);
    report(&mut fails, "screen write_str_at return col", screen_write_str_ok);
    report(&mut fails, "screen write_str_at/read_at roundtrip", screen_write_str_content_ok);
    report(&mut fails, "screen clear_row blanks cell", screen_clear_row_ok);

    korrodium_sdk::print_str("--- results ---\n");

    korrodium_sdk::exit(fails as i32);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
