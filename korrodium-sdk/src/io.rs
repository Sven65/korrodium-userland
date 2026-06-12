mod sys {
    #[link(wasm_import_module = "os::io")]
    unsafe extern "C" {
        pub fn print(ptr: *const u8, len: usize);
        pub fn read_line(ptr: *mut u8, max_len: usize) -> i32;
    }
}

pub fn print_str(s: &str) {
    unsafe { sys::print(s.as_ptr(), s.len()) };
}

pub fn read_line(buf: &mut [u8]) -> Option<&str> {
    let n = unsafe { sys::read_line(buf.as_mut_ptr(), buf.len()) };
    if n < 0 { return None; }
    core::str::from_utf8(&buf[..n as usize]).ok()
}