//source idea: https://os.phil-opp.com/minimal-rust-kernel/

use alloc::string::ToString;
use core::panic::PanicInfo;
use winapi::{
    km::wdm::DRIVER_OBJECT,
    shared::{basetsd::ULONG_PTR, minwindef::ULONG, ntdef::VOID},
};

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