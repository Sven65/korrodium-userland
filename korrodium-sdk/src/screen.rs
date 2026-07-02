//! Direct VGA text-mode access via the `os::screen` host module.
//!
//! This addresses the same 80x25 character buffer that the scrolling
//! console (`print_str`/`read_line`) writes to. Anything printed via
//! `print_str` can scroll over cells written here, and [`clear_screen`]
//! wipes anything `print_str` has written — coordinate the two carefully if
//! a program uses both (e.g. do direct-screen work first, capture what you
//! need to know from [`read_at`], *then* start printing).

mod sys {
    #[link(wasm_import_module = "os::screen")]
    unsafe extern "C" {
        pub fn width() -> i32;
        pub fn height() -> i32;
        pub fn write_at(row: i32, col: i32, byte: i32, color: i32);
        pub fn write_str_at(row: i32, col: i32, ptr: *const u8, len: usize, color: i32) -> i32;
        pub fn clear_row(row: i32, color: i32);
        pub fn clear_screen();
        pub fn move_cursor(row: i32, col: i32);
        pub fn read_at(row: i32, col: i32) -> i32;
    }
}

/// VGA text-mode colors, matching the kernel's `vga::Color` enum exactly —
/// the numeric values here are load-bearing, not just labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Packs a foreground/background pair into the single byte `write_at` and
/// friends expect. Must match the kernel's `ColorCode` bit layout exactly:
/// `(background << 4) | foreground`.
pub const fn color(fg: Color, bg: Color) -> u8 {
    ((bg as u8) << 4) | (fg as u8)
}

/// Number of text columns. Always 80 on this kernel's VGA text mode.
pub fn width() -> usize {
    unsafe { sys::width() as usize }
}

/// Number of text rows. Always 25 on this kernel's VGA text mode.
pub fn height() -> usize {
    unsafe { sys::height() as usize }
}

/// Writes a single byte at `(row, col)` without moving the scrolling
/// console's cursor. Out-of-bounds coordinates are silently ignored.
pub fn write_at(row: usize, col: usize, byte: u8, color: u8) {
    unsafe { sys::write_at(row as i32, col as i32, byte as i32, color as i32) };
}

/// Writes `s` starting at `(row, col)`, truncated at the right edge of the
/// screen. Returns the column just past the last character written.
pub fn write_str_at(row: usize, col: usize, s: &str, color: u8) -> usize {
    let col_after = unsafe { sys::write_str_at(row as i32, col as i32, s.as_ptr(), s.len(), color as i32) };
    col_after.max(0) as usize
}

/// Fills an entire row with spaces in the given color.
pub fn clear_row(row: usize, color: u8) {
    unsafe { sys::clear_row(row as i32, color as i32) };
}

/// Clears the whole screen and resets the scrolling console's cursor to
/// `(0, 0)`. See the module-level note: this wipes anything `print_str` has
/// written so far.
pub fn clear_screen() {
    unsafe { sys::clear_screen() };
}

/// Moves the blinking hardware cursor. Purely cosmetic — it does not affect
/// where the scrolling console (`print_str`) writes next.
pub fn move_cursor(row: usize, col: usize) {
    unsafe { sys::move_cursor(row as i32, col as i32) };
}

/// Reads back the character at `(row, col)` as `(ascii_byte, color)`.
/// Returns `None` if `(row, col)` is out of bounds.
pub fn read_at(row: usize, col: usize) -> Option<(u8, u8)> {
    let packed = unsafe { sys::read_at(row as i32, col as i32) };
    if packed < 0 { return None; }
    Some(((packed & 0xff) as u8, ((packed >> 8) & 0xff) as u8))
}
