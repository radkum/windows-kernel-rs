#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

#[allow(unused_imports)]
use alloc::alloc::handle_alloc_error;
use core::alloc::{GlobalAlloc, Layout};
use km_api_sys::ntoskrnl::{ExAllocatePool2, ExFreePool, POOL_FLAG_NON_PAGED, POOL_FLAG_PAGED};

pub const POOL_TAG: u32 = u32::from_ne_bytes(*b"TSUR");

pub struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        #[allow(unused)]
        let pool_flags = POOL_FLAG_NON_PAGED;

        #[cfg(feature = "paged_pool")]
        let pool_flags = POOL_FLAG_PAGED;

        let pool = ExAllocatePool2(pool_flags, layout.size(), POOL_TAG);

        #[cfg(feature = "alloc_panic")]
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
