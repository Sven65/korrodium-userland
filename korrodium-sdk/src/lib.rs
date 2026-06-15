#![no_std]

pub mod io;
pub mod proc;
mod time;

// Bekvämlighets-re-exports för de vanligaste
pub use io::{print_str, read_line};
pub use proc::{args, args_len};