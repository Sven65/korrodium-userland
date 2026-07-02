//! Filesystem access via the `os::fs` host module.
//!
//! Paths are resolved by the kernel the same way shell commands resolve
//! them: absolute paths (`/foo/bar`) are used as-is, relative paths are
//! joined onto the kernel's current working directory.

mod sys {
    #[link(wasm_import_module = "os::fs")]
    unsafe extern "C" {
        pub fn read_file(path_ptr: *const u8, path_len: usize, buf_ptr: *mut u8, buf_max_len: usize) -> i32;
        pub fn write_file(path_ptr: *const u8, path_len: usize, data_ptr: *const u8, data_len: usize) -> i32;
        pub fn append_file(path_ptr: *const u8, path_len: usize, data_ptr: *const u8, data_len: usize) -> i32;
        pub fn delete_file(path_ptr: *const u8, path_len: usize) -> i32;
        pub fn create_dir(path_ptr: *const u8, path_len: usize) -> i32;
        pub fn delete_dir(path_ptr: *const u8, path_len: usize) -> i32;
    }
}

/// Reads `path` into `buf`, returning the slice of `buf` that was filled.
/// Returns `None` if the file doesn't exist or `buf` is too small to hold
/// it — check the file's size beforehand if that distinction matters.
pub fn read_file<'a>(path: &str, buf: &'a mut [u8]) -> Option<&'a [u8]> {
    let n = unsafe { sys::read_file(path.as_ptr(), path.len(), buf.as_mut_ptr(), buf.len()) };
    if n < 0 { None } else { Some(&buf[..n as usize]) }
}

/// Creates or truncates `path` and writes `data` to it.
pub fn write_file(path: &str, data: &[u8]) -> bool {
    unsafe { sys::write_file(path.as_ptr(), path.len(), data.as_ptr(), data.len()) == 0 }
}

/// Appends `data` to `path`, creating it first if it doesn't exist.
pub fn append_file(path: &str, data: &[u8]) -> bool {
    unsafe { sys::append_file(path.as_ptr(), path.len(), data.as_ptr(), data.len()) == 0 }
}

/// Deletes the file at `path`.
pub fn delete_file(path: &str) -> bool {
    unsafe { sys::delete_file(path.as_ptr(), path.len()) == 0 }
}

/// Creates a directory at `path`. Its parent must already exist.
pub fn create_dir(path: &str) -> bool {
    unsafe { sys::create_dir(path.as_ptr(), path.len()) == 0 }
}

/// Deletes the (empty) directory at `path`.
pub fn delete_dir(path: &str) -> bool {
    unsafe { sys::delete_dir(path.as_ptr(), path.len()) == 0 }
}
