#![no_std]
#![allow(non_snake_case)]
extern crate alloc;

use core::ptr::null_mut;
/// kernel-init deliver a few elements (eg. panic implementation) necessary to run code in kernel
#[allow(unused_imports)]
use kernel_init;
use kernel_macros::{NT_SUCCESS, PAGED_CODE};

use kernel_string::UnicodeString;
use km_api_sys::flt_kernel::{
    FltRegisterFilter, FltStartFiltering, FltUnregisterFilter, FLT_FILESYSTEM_TYPE,
    FLT_INSTANCE_QUERY_TEARDOWN_FLAGS, FLT_INSTANCE_SETUP_FLAGS, FLT_INSTANCE_TEARDOWN_FLAGS,
    FLT_OPERATION_REGISTRATION, FLT_PREOP_CALLBACK_STATUS, FLT_REGISTRATION,
    FLT_REGISTRATION_FLAGS, FLT_REGISTRATION_VERSION, PFLT_CALLBACK_DATA, PFLT_FILTER,
    PFLT_OPERATION_REGISTRATION, PFLT_POST_OPERATION_CALLBACK, PFLT_RELATED_OBJECTS,
};
use km_api_sys::wmd::{KeGetCurrentIrql, PVOID};
use winapi::km::wdm::{DEVICE_TYPE, DRIVER_OBJECT};
use winapi::shared::ntdef::{NTSTATUS, UNICODE_STRING};
use winapi::shared::ntstatus::{STATUS_SUCCESS, STATUS_UNSUCCESSFUL};

static mut G_FILTER_HANDLE: PFLT_FILTER = null_mut();

const CALLBACKS: &'static [FLT_OPERATION_REGISTRATION] = {
    &[
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_CREATE)
            .set_preop(DelProtectPreCreate),
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_SET_INFORMATION)
            .set_preop(DelProtectPreSetInformation),
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_OPERATION_END)
    ]
};

const FILTER_REGISTRATION: FLT_REGISTRATION = FLT_REGISTRATION {
    size: 112, /*sizeof*/
    version: FLT_REGISTRATION_VERSION,
    flags: 0,
    context_registration: null_mut(),
    operation_registration: CALLBACKS.as_ptr(),
    filter_unload_callback: DelProtectUnload,
    instance_setup_callback: DelProtectInstanceSetup,
    instance_query_teardown_callback: DelProtectInstanceQueryTeardown,
    instance_teardown_start_callback: DelProtectInstanceTeardownStart,
    instance_teardown_complete_callback: DelProtectInstanceTeardownComplete,
    generate_file_name_callback: null_mut(),
    normalize_namecomponent_callback: null_mut(),
    normalize_context_cleanup_callback: null_mut(),
    transaction_notification_callback: null_mut(),
    normalize_name_component_ex_callback: null_mut(),
    section_notification_callback: null_mut(),
};

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT,
    _path: *const UNICODE_STRING,
) -> NTSTATUS {
    kernel_print::kernel_println!("START");

    let hello_world = UnicodeString::create("Hello World!");
    kernel_print::kernel_println!("{}", hello_world.as_rust_string());

    #[allow(unused_assignments)]
    let mut status = STATUS_SUCCESS;
    kernel_print::kernel_println!("status: {}, G_FILTER_HANDLE: {:p}", status, G_FILTER_HANDLE);
    status = FltRegisterFilter(driver, &FILTER_REGISTRATION, &mut G_FILTER_HANDLE);
    kernel_print::kernel_println!(
        "after FltRegisterFilter: {}, G_FILTER_HANDLE: {:p}",
        status,
        G_FILTER_HANDLE
    );
    if NT_SUCCESS!(status) {
        status = FltStartFiltering(G_FILTER_HANDLE);
        kernel_print::kernel_println!("after FltStartFiltering: {}", status);
        if !NT_SUCCESS!(status) {
            FltUnregisterFilter(G_FILTER_HANDLE);
        }
    }

    kernel_print::kernel_println!("SUCCESS: {}", status);
    status
}

extern "system" fn DelProtectUnload(flags: FLT_REGISTRATION_FLAGS) -> NTSTATUS {
    kernel_print::kernel_println!("delprotect_unload");

    unsafe {
        PAGED_CODE!();
        FltUnregisterFilter(G_FILTER_HANDLE);
    }

    STATUS_SUCCESS
}

extern "system" fn DelProtectInstanceSetup(
    flt_objects: PFLT_RELATED_OBJECTS,
    flags: FLT_INSTANCE_SETUP_FLAGS,
    volume_device_type: DEVICE_TYPE,
    volume_filesystem_type: FLT_FILESYSTEM_TYPE,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceSetup");

    unsafe {
        PAGED_CODE!();
    }
    STATUS_SUCCESS
}

extern "system" fn DelProtectInstanceQueryTeardown(
    flt_objects: PFLT_RELATED_OBJECTS,
    flags: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceQueryTeardown");

    unsafe {
        PAGED_CODE!();
        FltUnregisterFilter(G_FILTER_HANDLE);
    }
    kernel_print::kernel_println!("DelProtectInstanceQueryTeardown SUCCESS");
    STATUS_SUCCESS
}

extern "system" fn DelProtectInstanceTeardownStart(
    flt_objects: PFLT_RELATED_OBJECTS,
    flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceTeardownStart");

    unsafe {
        PAGED_CODE!();
    }
    kernel_print::kernel_println!("DelProtectInstanceTeardownStart SUCCESS");
    STATUS_SUCCESS
}

extern "system" fn DelProtectInstanceTeardownComplete(
    flt_objects: PFLT_RELATED_OBJECTS,
    flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceTeardownComplete");

    unsafe {
        PAGED_CODE!();
    }
    kernel_print::kernel_println!("DelProtectInstanceTeardownComplete SUCCESS");
    STATUS_SUCCESS
}

/*************************************************************************
    MiniFilter callback routines.
*************************************************************************/
extern "system" fn DelProtectPreCreate(
    data: PFLT_CALLBACK_DATA,
    flt_objects: PFLT_RELATED_OBJECTS,
    reserved: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS {
    kernel_print::kernel_println!("DelProtectPreCreate");
    1
}

extern "system" fn DelProtectPreSetInformation(
    data: PFLT_CALLBACK_DATA,
    flt_objects: PFLT_RELATED_OBJECTS,
    reserved: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS {
    kernel_print::kernel_println!("DelProtectPreSetInformation");
    1
}