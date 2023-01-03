#![no_std]
#![allow(non_snake_case)]
extern crate alloc;

/// kernel-init deliver a few elements (eg. panic implementation) necessary to run code in kernel
#[allow(unused_imports)]
use kernel_init;

use kernel_string::UnicodeString;
use winapi::km::wdm::DRIVER_OBJECT;
use winapi::shared::ntdef::{NTSTATUS, UNICODE_STRING};
use winapi::shared::ntstatus::STATUS_SUCCESS;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT,
    _path: *const UNICODE_STRING,
) -> NTSTATUS {
    kernel_print::kernel_println!("START");

    driver.DriverUnload = Some(DriverUnload);

    #[allow(unused_assignments)]
    let status = STATUS_SUCCESS;

    let hello_world = UnicodeString::create("Hello World!");
    kernel_print::kernel_println!("{}", hello_world.as_rust_string());

    status
}

extern "system" fn DriverUnload(driver: &mut DRIVER_OBJECT) {
    kernel_print::kernel_println!("rust_unload");
}
