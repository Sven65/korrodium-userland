use crate::time;

#[link(wasm_import_module = "os::time")]
unsafe extern "C" {
    pub fn get_time(ptr: *mut u8) -> i32;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

pub fn now() -> Option<DateTime> {
    let mut buf = [0u8; 8];
    let n = unsafe { time::get_time(buf.as_mut_ptr()) };
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