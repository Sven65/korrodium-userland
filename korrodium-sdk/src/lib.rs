#![no_std]

#[link(wasm_import_module = "os")]
unsafe extern "C" {
    fn print(ptr: *const u8, len: usize);
}

pub fn print_str(s: &str) {
    unsafe { print(s.as_ptr(), s.len()) };
}