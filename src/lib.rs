#![no_std]
#![feature(default_alloc_error_handler)]
#![allow(non_snake_case)]
#![feature(lang_items)]
extern crate alloc;

use alloc::string::ToString;
use core::panic::PanicInfo;

use winapi::{
    km::wdm::DRIVER_OBJECT,
    shared::{basetsd::ULONG_PTR, minwindef::ULONG, ntdef::VOID},
};

pub mod include;
pub mod log;
pub mod string;
use crate::{include::MmIsAddressValid, string::create_unicode_string};

use winapi::shared::ntdef::UNICODE_STRING;

//source idea: https://os.phil-opp.com/minimal-rust-kernel/
#[cfg(not(test))]
#[global_allocator]
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

#[cfg(not(test))]
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[cfg(not(test))]
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 { unimplemented!() }

/// Base code for Device Control
///
/// TODO: Extend control codes to include more than just Device Control basic.
const DEVICE_CONTROL_BUGCHECK_CODE: ULONG = 0xDC000000;

#[cfg(not(test))]
#[cfg_attr(all(target_env = "msvc", feature = "kernel"), link(name = "ntoskrnl"))]
extern "system" {
    fn KeBugCheckEx(
        BugCheckCode: ULONG, BugCheckParameter1: ULONG_PTR, BugCheckParameter2: ULONG_PTR,
        BugCheckParameter3: ULONG_PTR, BugCheckParameter4: ULONG_PTR,
    ) -> VOID;
}

#[cfg(not(test))]
/// An unrecoverable error will cause the kernel to crash.
/// This function is called when a panic occurs.
///
/// # Arguments
/// * BugCheckCode: The error code to be passed to the kernel. (0xDC000000)
/// * BugCheckParameter1: A pointer with a string describing the error.
fn unrecoverable_error(info: &PanicInfo) {
    let msg = info.to_string();
    unsafe {
        KeBugCheckEx(DEVICE_CONTROL_BUGCHECK_CODE, msg.as_ptr() as usize, 0, 0, 0);
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

#[no_mangle]
pub extern "system" fn DriverEntry(driver_object: &mut DRIVER_OBJECT, _path: *const UNICODE_STRING) -> u32 {
    kernel_print::kernel_println!("START");

    // MmIsAddressValid
    //
    let is_valid = unsafe { MmIsAddressValid(0 as _) };
    log!("MmIsAddressValid(0) returned %i", is_valid as u64);

    // String
    let string = create_unicode_string(&['H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'l' as u16, '\0' as u16]);
    log!("String: %ws", string.Buffer);

    /* STATUS_SUCCESS */
    driver_object.DriverUnload = Some(DriverUnload);
    //
    // driver_object.DriverUnload.unwrap()(driver_object);
    //
    // driver_object.DriverUnload = Some(DriverUnload);

    kernel_print::kernel_println!("FINISH");
    0
}

extern "system" fn DriverUnload(_driver: &mut DRIVER_OBJECT) {
    kernel_print::kernel_println!("rust_unload");
}
