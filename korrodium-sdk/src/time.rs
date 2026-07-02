mod sys {
    #[link(wasm_import_module = "os::time")]
    unsafe extern "C" {
        /// Writes an 8-byte record (year: u16 LE, month, day, hour, minute,
        /// second, pad: u8 each) to `ptr`; `max_len` must be at least 8.
        /// Returns the number of bytes written, or a negative code on
        /// failure (e.g. a bad guest pointer).
        pub fn get_time(ptr: *mut u8, max_len: usize) -> i32;
    }
}

/// Wall-clock date/time as reported by the kernel's RTC.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

/// Reads the current date/time. Returns `None` if the host couldn't fill
/// the full 8-byte record (e.g. a bad buffer).
pub fn now() -> Option<DateTime> {
    let mut buf = [0u8; 8];
    let n = unsafe { sys::get_time(buf.as_mut_ptr(), buf.len()) };
    if n < 8 { return None; }
    Some(DateTime {
        year: u16::from_le_bytes([buf[0], buf[1]]),
        month: buf[2],
        day: buf[3],
        hour: buf[4],
        minute: buf[5],
        second: buf[6],
    })
}