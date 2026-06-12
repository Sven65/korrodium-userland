mod sys {
    #[link(wasm_import_module = "os::proc")]
    unsafe extern "C" {
        pub fn args_len() -> i32;
        pub fn args_get(ptr: *mut u8, max_len: usize) -> i32;
    }
}

pub fn args_len() -> usize {
    let n = unsafe { sys::args_len() };
    if n < 0 { 0 } else { n as usize }
}

pub fn args(buf: &mut [u8]) -> impl Iterator<Item = &str> {
    let n = unsafe { sys::args_get(buf.as_mut_ptr(), buf.len()) };
    let n = if n < 0 { 0 } else { n as usize };
    core::str::from_utf8(&buf[..n]).unwrap_or("").split('\n').filter(|s| !s.is_empty())
}