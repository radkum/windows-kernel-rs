//it must be defined in lib.rs
//#![feature(alloc_error_handler)]

use alloc::alloc::handle_alloc_error;
use core::alloc::{GlobalAlloc, Layout};
use km_api_sys::ntoskrnl::{ExAllocatePool2, ExFreePool, POOL_FLAG_PAGED};

pub const POOL_TAG: u32 = u32::from_ne_bytes(*b"TSUR");

pub struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pool = ExAllocatePool2(POOL_FLAG_PAGED, layout.size(), POOL_TAG);

        if pool.is_null() {
            handle_alloc_error(layout);
        }

        pool as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ExFreePool(ptr as _);
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}
