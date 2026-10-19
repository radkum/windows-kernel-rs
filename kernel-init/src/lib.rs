//source idea: https://os.phil-opp.com/minimal-rust-kernel/
#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::{ffi::c_void, panic::PanicInfo};

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

#[cfg(not(test))]
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[cfg(not(test))]
#[no_mangle]
extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 {
    unimplemented!()
}

/// Base code
const BUGCHECK_CODE: u32 = 0xDEAD0000;

#[cfg(not(test))]
extern "system" {
    fn KeBugCheckEx(
        BugCheckCode: u32,
        BugCheckParameter1: u32,
        BugCheckParameter2: u32,
        BugCheckParameter3: u32,
        BugCheckParameter4: u32,
    ) -> c_void;
}

#[cfg(not(test))]
/// An unrecoverable error will cause the kernel to crash.
fn unrecoverable_error(info: &PanicInfo) {
    // no allocation here: panics also come from OOM; param 1 is the panic line
    let line = info.location().map_or(0, |l| l.line());
    unsafe {
        KeBugCheckEx(BUGCHECK_CODE, line, 0, 0, 0);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unrecoverable_error(info);
    loop {}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
