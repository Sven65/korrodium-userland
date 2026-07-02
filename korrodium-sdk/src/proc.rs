mod sys {
    #[link(wasm_import_module = "os::proc")]
    unsafe extern "C" {
        pub fn args_len() -> i32;
        pub fn args_get(ptr: *mut u8, max_len: usize) -> i32;
        pub fn exit(code: i32);
    }
}

/// Byte-length of the program's arguments joined by `\n` (0 if there are no
/// arguments). Size a buffer to at least this before calling [`args`].
pub fn args_len() -> usize {
    let n = unsafe { sys::args_len() };
    if n < 0 { 0 } else { n as usize }
}

/// Iterates over the program's arguments, as passed by the shell's
/// `run <file> <args...>` command. `buf` should be at least [`args_len`]
/// bytes; a too-small buffer silently truncates the last argument(s).
pub fn args(buf: &mut [u8]) -> impl Iterator<Item = &str> {
    let n = unsafe { sys::args_get(buf.as_mut_ptr(), buf.len()) };
    let n = if n < 0 { 0 } else { n as usize };
    core::str::from_utf8(&buf[..n]).unwrap_or("").split('\n').filter(|s| !s.is_empty())
}

/// Terminates the program immediately with `code` as its exit status.
///
/// The kernel implements this as a host trap that unwinds the wasm call
/// stack, so this never returns control to the caller; the `loop {}` below
/// only guards against a hypothetical future host that resumes execution
/// instead of aborting it.
pub fn exit(code: i32) -> ! {
    unsafe { sys::exit(code) };
    loop {}
}