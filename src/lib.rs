#![no_std]
#![feature(default_alloc_error_handler)]
#![allow(non_snake_case)]
#![feature(lang_items)]
extern crate alloc;

use winapi::{
    km::wdm::DRIVER_OBJECT,
    shared::{basetsd::ULONG_PTR, minwindef::ULONG, ntdef::VOID},
};
use winapi::km::wdm::{DEVICE_TYPE, IoCreateDevice};
use winapi::shared::ntdef::NTSTATUS;


pub mod include;
pub mod log;
pub mod string;
mod kernel_init;

use crate::{include::MmIsAddressValid, string::create_unicode_string};

use winapi::shared::ntdef::UNICODE_STRING;
use crate::string::{AnsiString, RtlAnsiStringToUnicodeString, UnicodeString};


// Helper for converting b"string" to UNICODE_STRING
// fn ansiToUnicode(s: &[u8]) -> UNICODE_STRING {
//     let a = AnsiString::from(s);
//     let mut u = UnicodeString::default();
//     unsafe { RtlAnsiStringToUnicodeString(&mut u, &a, true) };
//     return u;
// }

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(driver: &mut DRIVER_OBJECT, _path: *const UNICODE_STRING) -> NTSTATUS {
    kernel_print::kernel_println!("START");

    driver.DriverUnload = Some(DriverUnload);

    //IoCreateDevice(driver, 0, (), DEVICE_TYPE::FILE_DEVICE_BEEP, 0, (), ())
    // MmIsAddressValid
    //
    let is_valid = unsafe { MmIsAddressValid(0 as _) };
    log!("MmIsAddressValid(0) returned %i", is_valid as u64);

    // String
    let string = create_unicode_string(&['H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'l' as u16, '\0' as u16]);
    log!("String: %ws", string.Buffer);

    /* STATUS_SUCCESS */

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
