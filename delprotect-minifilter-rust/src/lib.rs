#![no_std]
#![allow(non_snake_case)]
extern crate alloc;

use core::ptr::null_mut;

/// kernel-init deliver a few elements (eg. panic implementation) necessary to run code in kernel
#[allow(unused_imports)]
use kernel_init;
use kernel_macros::{NT_SUCCESS, PAGED_CODE};

use kernel_init::kernel_alloc::POOL_TAG;
use kernel_string::{PUnicodeString, UnicodeString};
use km_api_sys::flt_kernel::*;
use km_api_sys::ntddk::{PFILE_DISPOSITION_INFORMATION, PROCESSINFOCLASS};
use km_api_sys::ntifs::{ObOpenObjectByPointer, PsGetThreadProcess};
use km_api_sys::ntoskrnl::{ExAllocatePool2, POOL_FLAG_PAGED};
use km_api_sys::wmd::{
    NtCurrentProcess, ZwClose, ZwQueryInformationProcess, FILE_DELETE_ON_CLOSE, HANDLE, PVOID,
};
use winapi::km::wdm::{DEVICE_TYPE, DRIVER_OBJECT, KPROCESSOR_MODE};
use winapi::shared::ntdef::{NTSTATUS, OBJ_KERNEL_HANDLE, ULONG, UNICODE_STRING, USHORT};
use winapi::shared::ntstatus::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};

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
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_OPERATION_END),
    ]
};

const FILTER_REGISTRATION: FLT_REGISTRATION = FLT_REGISTRATION {
    size: ::core::mem::size_of::<FLT_REGISTRATION>() as USHORT, /*sizeof*/
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

#[link_section = "INIT"]
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

extern "system" fn DelProtectUnload(_flags: FLT_REGISTRATION_FLAGS) -> NTSTATUS {
    kernel_print::kernel_println!("delprotect_unload");

    unsafe {
        PAGED_CODE!();
        FltUnregisterFilter(G_FILTER_HANDLE);
    }

    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn DelProtectInstanceSetup(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_SETUP_FLAGS,
    _volume_device_type: DEVICE_TYPE,
    _volume_filesystem_type: FLT_FILESYSTEM_TYPE,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceSetup");

    //unsafe {
    PAGED_CODE!();
    //}
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn DelProtectInstanceQueryTeardown(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceQueryTeardown");

    unsafe {
        PAGED_CODE!();
        FltUnregisterFilter(G_FILTER_HANDLE);
    }
    kernel_print::kernel_println!("DelProtectInstanceQueryTeardown SUCCESS");
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn DelProtectInstanceTeardownStart(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceTeardownStart");

    //unsafe {
    PAGED_CODE!();
    //}
    kernel_print::kernel_println!("DelProtectInstanceTeardownStart SUCCESS");
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn DelProtectInstanceTeardownComplete(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceTeardownComplete");

    //unsafe {
    PAGED_CODE!();
    //}
    kernel_print::kernel_println!("DelProtectInstanceTeardownComplete SUCCESS");
    STATUS_SUCCESS
}

/*************************************************************************
    MiniFilter callback routines.
*************************************************************************/
extern "system" fn DelProtectPreCreate(
    data: &mut FLT_CALLBACK_DATA,
    _flt_objects: PFLT_RELATED_OBJECTS,
    _reserved: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS {
    let mut status = FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_SUCCESS_NO_CALLBACK;

    //let mut data = data as &mut FLT_CALLBACK_DATA;
    if let KPROCESSOR_MODE::KernelMode = data.requestor_mode {
        return status;
    }
    //kernel_print::kernel_println!("DelProtectPreCreate not in kernel");

    unsafe {
        let params = &(*data.iopb).parameters.Create;

        if (params.options & FILE_DELETE_ON_CLOSE) > 0 {
            kernel_print::kernel_println!("Delete on close");
            if !IsDeleteAllowed(NtCurrentProcess()) {
                *data.io_status.__bindgen_anon_1.Status_mut() = STATUS_ACCESS_DENIED;
                status = FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_COMPLETE;
                kernel_print::kernel_println!("Prevent delete by cmd.exe");
            }
        }
    }

    status
}

extern "system" fn DelProtectPreSetInformation(
    data: &mut FLT_CALLBACK_DATA,
    _flt_objects: PFLT_RELATED_OBJECTS,
    _reserved: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS {
    //kernel_print::kernel_println!("DelProtectPreSetInformation");
    let mut status = FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_SUCCESS_NO_CALLBACK;

    let params = unsafe { &(*data.iopb).parameters.SetFileInformation };

    match params.file_information_class {
        FILE_INFORMATION_CLASS::FileDispositionInformation
        | FILE_INFORMATION_CLASS::FileDispositionInformationEx => {},
        _ => return status,
    }

    let info = params.info_buffer as PFILE_DISPOSITION_INFORMATION;
    unsafe {
        if (*info).delete_file == 0 {
            return status;
        }

        let process = PsGetThreadProcess(data.thread);
        if process.is_null() {
            //something is wrong
            return status;
        }

        let mut h_process: HANDLE = usize::MAX as HANDLE;
        let ret = ObOpenObjectByPointer(
            process,
            OBJ_KERNEL_HANDLE,
            null_mut(),
            0,
            null_mut(),
            KPROCESSOR_MODE::KernelMode,
            &mut h_process,
        );
        if !NT_SUCCESS!(ret) {
            return status;
        }

        if !IsDeleteAllowed(h_process) {
            *data.io_status.__bindgen_anon_1.Status_mut() = STATUS_ACCESS_DENIED;
            status = FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_COMPLETE;
            kernel_print::kernel_println!("Prevent delete by cmd.exe");
        }
        ZwClose(h_process);
    }
    status
}

unsafe fn IsDeleteAllowed(h_process: HANDLE) -> bool {
    let process_name_size = 300;
    let process_name =
        ExAllocatePool2(POOL_FLAG_PAGED, process_name_size, POOL_TAG) as PUnicodeString;

    let mut return_length: ULONG = 0;
    let status = ZwQueryInformationProcess(
        h_process,
        PROCESSINFOCLASS::ProcessImageFileName,
        process_name as PVOID,
        (process_name_size - 2) as u32,
        &mut return_length,
    );

    kernel_print::kernel_println!(
        "ZwQueryInformationProcess - status: {}, returnLength: {}",
        status,
        return_length
    );

    if NT_SUCCESS!(status) {
        let process_name = &*process_name;
        let rust_process_name = process_name.as_rust_string();
        kernel_print::kernel_println!("Delete operation from {}", rust_process_name);

        if process_name.len == 0 {
            return true;
        }

        if rust_process_name.contains("\\System32\\cmd.exe")
            || rust_process_name.contains("\\SysWOW64\\cmd.exe")
        {
            return false;
        }
    }

    true
}
