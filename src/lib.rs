#![no_std]
#![feature(default_alloc_error_handler)]
#![allow(non_snake_case)]
#![feature(lang_items)]
extern crate alloc;

use core::ptr::null_mut;
use winapi::{
    km::wdm::{IoCreateDevice, DEVICE_TYPE, DRIVER_OBJECT},
    shared::{
        ntdef::{NTSTATUS},
    },
};
use winapi::km::wdm::{IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, PDEVICE_OBJECT};

pub mod include;
mod kernel_init;
pub mod log;
pub mod string;

use crate::string::{UnicodeString};
use winapi::shared::ntdef::UNICODE_STRING;
use winapi::shared::ntdef::FALSE;
use winapi::shared::ntstatus::STATUS_SUCCESS;

// Helper for converting b"string" to UNICODE_STRING
// fn ansiToUnicode(s: &[u8]) -> UNICODE_STRING {
//     let a = AnsiString::from(s);
//     let mut u = UnicodeString::default();
//     unsafe { RtlAnsiStringToUnicodeString(&mut u, &a, true) };
//     return u;
// }
macro_rules! NT_SUCCESS {
    ($status: expr) => {
        $status as NTSTATUS >= 0
    };
}

struct Cleaner {
    device_object: Option<PDEVICE_OBJECT>,
    sym_link: Option<*const UnicodeString>,
}

impl Cleaner {
    fn new() -> Self {
        Self {
            device_object: None,
            sym_link: None,
        }
    }

    fn init_device(&mut self, device: PDEVICE_OBJECT) {
        self.device_object = Some(device);
    }

    fn init_symlink(&mut self, sym_link: &UnicodeString) {
        self.sym_link = Some(sym_link as *const UnicodeString);
    }

    fn clean(&mut self) {
        unsafe {
            if let Some(device) = self.device_object {
                IoDeleteDevice(device);
            }

            if let Some(sym_link) = self.sym_link {
                IoDeleteSymbolicLink(&(*sym_link).native());
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT, _path: *const UNICODE_STRING,
) -> NTSTATUS {
    kernel_print::kernel_println!("START");

    #[allow(unused_assignments)]
    let mut status = STATUS_SUCCESS;

    let str_2 = UnicodeString::create("Hello World!");
    log!("String: %ws", str_2.ptr);

    driver.DriverUnload = Some(DriverUnload);

    let dev_name = UnicodeString::from("\\Device\\Zero");
    let sym_link = UnicodeString::from("\\??\\Zero");

    let mut cleaner = Cleaner::new();
    let mut device_object : PDEVICE_OBJECT = null_mut();

    loop {
        status = IoCreateDevice(driver,
                                    0,
                                    dev_name.native_ptr(),
                                    DEVICE_TYPE::FILE_DEVICE_UNKNOWN,
                                    0,
                                    FALSE,
                                    &mut device_object);

        if NT_SUCCESS!(status) {
            cleaner.init_device(device_object);
        } else {
            kernel_print::kernel_println!("failed to create device 0x{:08x}", status);
            break;
        }

        status = IoCreateSymbolicLink(&sym_link.native(), &dev_name.native());

        if NT_SUCCESS!(status) {
            cleaner.init_symlink(&sym_link);
        } else {
            kernel_print::kernel_println!("failed to create sym_link 0x{:08x}", status);
            break;
        }

        break;
    }

    if NT_SUCCESS!(status) {
        kernel_print::kernel_println!("SUCCESS");
    } else {
        cleaner.clean();
    }


    status
}

extern "system" fn DriverUnload(driver: &mut DRIVER_OBJECT) {
    kernel_print::kernel_println!("rust_unload");
    unsafe {
        IoDeleteDevice(driver.DeviceObject);

        let sym_link = UnicodeString::create("\\??\\Zero");
        IoDeleteSymbolicLink(&sym_link.native());
    }
}
