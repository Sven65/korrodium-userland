mod sys {
    #[link(wasm_import_module = "os::io")]
    unsafe extern "C" {
        pub fn print(ptr: *const u8, len: usize);
        pub fn read_line(ptr: *mut u8, max_len: usize) -> i32;
        pub fn yield_now();
        pub fn read_key() -> i32;
    }
}

/// Writes `s` to the scrolling console.
pub fn print_str(s: &str) {
    unsafe { sys::print(s.as_ptr(), s.len()) };
}

/// Blocks (busy-spinning on the kernel side, not yielding to other tasks —
/// see the `os::io::read_line` host fn) until a full line is entered, or
/// `None` on Ctrl+C. Bypasses the shell's normal input-focus queue, so avoid
/// mixing this with [`read_key`] in the same program.
pub fn read_line(buf: &mut [u8]) -> Option<&str> {
    let n = unsafe { sys::read_line(buf.as_mut_ptr(), buf.len()) };
    if n < 0 { return None; }
    core::str::from_utf8(&buf[..n as usize]).ok()
}

/// Yields to the kernel's cooperative task executor so other tasks can run,
/// then resumes. Implemented as a resumable host trap: from the guest's
/// point of view this is just a blocking call with no side effects.
pub fn yield_now() {
    unsafe { sys::yield_now() };
}

/// A key delivered by [`read_key`], decoded per the kernel's
/// `os::io::read_key` encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// A Unicode scalar value, including control characters such as
    /// Enter (`'\n'`/`'\r'`), Tab (`'\t'`), Backspace (`0x08`), and Esc
    /// (`0x1b`).
    Unicode(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    Delete,
    PageUp,
    PageDown,
    /// A key the host recognized but doesn't have a distinct encoding for
    /// yet (F-keys, bare modifiers, etc.) — these currently collapse to the
    /// same wire value as a literal NUL byte on the host side, so treat this
    /// variant as "unknown", not as an actual null character.
    Other,
}

/// Blocks — properly yielding to the executor, unlike [`read_line`] — until
/// a key is available while this program holds input focus, then returns it
/// decoded.
pub fn read_key() -> Key {
    match unsafe { sys::read_key() } {
        -1 => Key::ArrowUp,
        -2 => Key::ArrowDown,
        -3 => Key::ArrowLeft,
        -4 => Key::ArrowRight,
        -5 => Key::Home,
        -6 => Key::End,
        -7 => Key::Delete,
        -8 => Key::PageUp,
        -9 => Key::PageDown,
        0 => Key::Other,
        n => char::from_u32(n as u32).map(Key::Unicode).unwrap_or(Key::Other),
    }
}