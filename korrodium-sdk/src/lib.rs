#![no_std]

extern crate alloc;

mod alloc_impl;
pub mod fs;
pub mod io;
pub mod proc;
pub mod screen;
pub mod time;

// Bekvämlighets-re-exports för de vanligaste.
// `time` was previously private with nothing re-exported, so `now()`/
// `DateTime` were unreachable from outside this crate — fixed alongside the
// `get_time` arity bug in `time.rs`.
pub use io::{print_str, read_key, read_line, yield_now};
pub use proc::{args, args_len, exit};
pub use time::now;