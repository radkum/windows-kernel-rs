#![no_std]
#![feature(default_alloc_error_handler)]
#![allow(non_snake_case)]
#![feature(lang_items)]
extern crate alloc;

mod cleaner;
mod constants;
pub mod include;
mod kernel_init;
pub mod log;
pub mod macros;
pub mod string;

use crate::cleaner::Cleaner;
use crate::string::UnicodeString;

use winapi::km::wdm::{
    IoCompleteRequest, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink,
    IoGetCurrentIrpStackLocation, DEVICE_OBJECT, DEVICE_TYPE, DRIVER_OBJECT, IRP, IRP_MJ,
    PDEVICE_OBJECT,
};
use winapi::shared::ntdef::{FALSE, NTSTATUS, UNICODE_STRING};
use winapi::shared::ntstatus::{
    STATUS_INSUFFICIENT_RESOURCES, STATUS_INVALID_BUFFER_SIZE, STATUS_INVALID_DEVICE_REQUEST,
    STATUS_SUCCESS,
};

use crate::include::{MmGetSystemAddressForMdlSafe, MDL};
use core::ptr::null_mut;
use winapi::km::wdm::DEVICE_FLAGS::DO_DIRECT_IO;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT,
    _path: *const UNICODE_STRING,
) -> NTSTATUS {
    kernel_print::kernel_println!("START");
    driver.DriverUnload = Some(DriverUnload);

    driver.MajorFunction[IRP_MJ::CREATE as usize] = Some(DispatchCreateClose);
    driver.MajorFunction[IRP_MJ::CLOSE as usize] = Some(DispatchCreateClose);
    driver.MajorFunction[IRP_MJ::DEVICE_CONTROL as usize] = Some(DispatchDeviceControl);
    driver.MajorFunction[IRP_MJ::READ as usize] = Some(DispatchRead);

    #[allow(unused_assignments)]
    let mut status = STATUS_SUCCESS;

    let str_2 = UnicodeString::create("Hello World!");
    log!("String: %ws", str_2.ptr);

    let dev_name = UnicodeString::from("\\Device\\Zero");
    let sym_link = UnicodeString::from("\\??\\Zero");

    let mut cleaner = Cleaner::new();
    let mut device_object: PDEVICE_OBJECT = null_mut();

    loop {
        status = IoCreateDevice(
            driver,
            0,
            dev_name.native_ptr(),
            DEVICE_TYPE::FILE_DEVICE_UNKNOWN,
            0,
            FALSE,
            &mut device_object,
        );

        if NT_SUCCESS!(status) {
            cleaner.init_device(device_object);
        } else {
            kernel_print::kernel_println!("failed to create device 0x{:08x}", status);
            break;
        }

        (*device_object).Flags |= DO_DIRECT_IO as u32;

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

extern "system" fn DispatchCreateClose(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    complete_irp_success(irp)
}

extern "system" fn DispatchDeviceControl(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    unsafe {
        let stack = IoGetCurrentIrpStackLocation(irp);
        let device_io = (*stack).Parameters.DeviceIoControl();

        match device_io.IoControlCode {
            IOCTL_REQUEST => kernel_print::kernel_println!("device control success"),
            _ => {
                return complete_irp_with_status(irp, STATUS_INVALID_DEVICE_REQUEST);
            },
        }
    }

    complete_irp_success(irp)
}

struct Parameters_Read {
    len: u32
}
extern "system" fn DispatchRead(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    kernel_print::kernel_println!("DispatchRead begin");

    unsafe {
        let stack = IoGetCurrentIrpStackLocation(irp);
        //stack->Parameters.Read.Length;
        let len = (*stack).Parameters.DeviceIoControl().OutputBufferLength;

        kernel_print::kernel_println!("read len: {}", len);

        if len == 0 {
            kernel_print::kernel_println!("len is zero");
            return complete_irp_with_status(irp, STATUS_INVALID_BUFFER_SIZE);
        }

        let buffer = MmGetSystemAddressForMdlSafe(
            irp.MdlAddress as *mut MDL,
            16, /*NormalPagePriority*/
        );
        if buffer.is_null() {
            kernel_print::kernel_println!("buffer is null");
            return complete_irp_with_status(irp, STATUS_INSUFFICIENT_RESOURCES);
        }

        let buffer = buffer as *mut u8;
        for i in 0..len {
            *buffer.offset(i as isize) = 5;
        }
        kernel_print::kernel_println!("DispatchRead success");

        complete_irp(irp, STATUS_SUCCESS, len as usize)
    }
}

fn complete_irp(irp: &mut IRP, status: NTSTATUS, info: usize) -> NTSTATUS {
    unsafe {
        let s = irp.IoStatus.__bindgen_anon_1.Status_mut();
        *s = status;
        irp.IoStatus.Information = info;
        IoCompleteRequest(irp, 0);
    }

    status
}

fn complete_irp_with_status(irp: &mut IRP, status: NTSTATUS) -> NTSTATUS {
    complete_irp(irp, status, 0)
}

fn complete_irp_success(irp: &mut IRP) -> NTSTATUS {
    complete_irp_with_status(irp, STATUS_SUCCESS)
}
