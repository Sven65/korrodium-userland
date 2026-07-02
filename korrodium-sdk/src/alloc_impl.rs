//! A minimal global allocator so programs can use `alloc` (`Vec`, `String`,
//! ...), backed directly by wasm's `memory.grow`.
//!
//! It only bumps a pointer forward and never frees. That's correct here
//! because: (1) wasmi calls a program's `main` synchronously and this
//! target has no threads, so there's no concurrent access to race on; and
//! (2) each program instance is a whole wasm `Store` the kernel throws away
//! when the program exits, so "leaking" for the process lifetime is fine —
//! there is no long-running reuse to accumulate garbage in.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::Cell;

unsafe extern "C" {
    /// Provided by the wasm32 linker: the address of the first byte after
    /// this module's static data. Everything from here to the end of
    /// linear memory is free for us to hand out.
    static __heap_base: u8;
}

const WASM_PAGE_SIZE: usize = 64 * 1024;

struct BumpAllocator {
    next: Cell<usize>,
    end: Cell<usize>,
}

// Single-threaded guest (see module docs) — no real concurrent access to
// the `Cell`s is possible, so this is sound despite `Cell` not being `Sync`.
unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.next.get() == 0 {
            let base = &raw const __heap_base as usize;
            self.next.set(base);
            self.end.set(base);
        }

        let align = layout.align();
        let aligned = (self.next.get() + align - 1) & !(align - 1);
        let new_next = aligned + layout.size();

        if new_next > self.end.get() {
            let needed = new_next - self.end.get();
            let pages = needed.div_ceil(WASM_PAGE_SIZE).max(1);
            if core::arch::wasm32::memory_grow(0, pages) == usize::MAX {
                return core::ptr::null_mut();
            }
            self.end.set(self.end.get() + pages * WASM_PAGE_SIZE);
        }

        self.next.set(new_next);
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Never frees — see module docs.
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator { next: Cell::new(0), end: Cell::new(0) };
