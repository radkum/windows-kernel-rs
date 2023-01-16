#![no_std]
#![allow(non_snake_case)]
extern crate alloc;

use alloc::{collections::VecDeque, string::String};
use core::ptr::null_mut;
use kernel_fast_mutex::{auto_lock::AutoLock, fast_mutex::FastMutex};
use kernel_fast_mutex::locker::Locker;

/// kernel-init deliver a few elements (eg. panic implementation) necessary to run code in kernel
#[allow(unused_imports)]
use kernel_init;
use kernel_macros::{NT_SUCCESS, PAGED_CODE};

use kernel_init::kernel_alloc::POOL_TAG;
use kernel_string::{PUNICODE_STRING, UNICODE_STRING};
use km_api_sys::{
    flt_kernel::*,
    ntddk::{PFILE_DISPOSITION_INFORMATION, PROCESSINFOCLASS},
    ntifs::{ObOpenObjectByPointer, PsGetThreadProcess},
    ntoskrnl::{ExAllocatePool2, ExFreePoolWithTag, POOL_FLAG_PAGED},
    wmd::{NtCurrentProcess, ZwClose, ZwQueryInformationProcess, FILE_DELETE_ON_CLOSE},
};
use winapi::{
    km::wdm::{DEVICE_TYPE, DRIVER_OBJECT, KPROCESSOR_MODE},
    shared::{
        ntdef::{HANDLE, NTSTATUS, OBJ_KERNEL_HANDLE, PVOID, ULONG, USHORT},
        ntstatus::{STATUS_ACCESS_DENIED, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS},
    },
};

const MAX_ITEM_COUNT: usize = 32;

const DEVICE_NAME: &str = "\\Device\\DelProtect";
const SYM_LINK_NAME: &str = "\\??\\DelProtect";

static mut G_PROCESS_NAMES: Option<VecDeque<String>> = None;
static mut G_MUTEX: FastMutex = FastMutex::new();
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
    Size: ::core::mem::size_of::<FLT_REGISTRATION>() as USHORT, /*sizeof*/
    Version: FLT_REGISTRATION_VERSION,
    Flags: 0,
    ContextRegistration: null_mut(),
    OperationRegistration: CALLBACKS.as_ptr(),
    FilterUnloadCallback: DelProtectUnload,
    InstanceSetupCallback: DelProtectInstanceSetup,
    InstanceQueryTeardownCallback: DelProtectInstanceQueryTeardown,
    InstanceTeardownStartCallback: DelProtectInstanceTeardownStart,
    InstanceTeardownCompleteCallback: DelProtectInstanceTeardownComplete,
    GenerateFileNameCallback: null_mut(),
    NormalizeNameComponentCallback: null_mut(),
    NormalizeContextCleanupCallback: null_mut(),
    TransactionNotificationCallback: null_mut(),
    NormalizeNameComponentExCallback: null_mut(),
    SectionNotificationCallback: null_mut(),
};

#[link_section = "INIT"]
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT,
    _path: *const UNICODE_STRING,
) -> NTSTATUS {
    kernel_print::kernel_println!("START DelProtect");

    let hello_world = UNICODE_STRING::create("Hello World!");
    kernel_print::kernel_println!("{}", hello_world.as_rust_string());

    //init mutex
    G_MUTEX.Init();

    //init processes vector
    let mut events = VecDeque::new();
    if let Err(e) = events.try_reserve_exact(MAX_ITEM_COUNT) {
        kernel_print::kernel_println!(
            "fail to reserve a {} bytes of memory. Err: {:?}",
            ::core::mem::size_of::<String>() * MAX_ITEM_COUNT,
            e
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    G_PROCESS_NAMES = Some(events);

    //temporary for cmd
    push_item_thread_safe("\\cmd.exe");

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

    PAGED_CODE!();
    unsafe {
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
    PAGED_CODE!();
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn DelProtectInstanceQueryTeardown(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceQueryTeardown");

    PAGED_CODE!();
    unsafe {
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

    PAGED_CODE!();
    kernel_print::kernel_println!("DelProtectInstanceTeardownStart SUCCESS");
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn DelProtectInstanceTeardownComplete(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS {
    kernel_print::kernel_println!("DelProtectInstanceTeardownComplete");

    PAGED_CODE!();
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
    if let KPROCESSOR_MODE::KernelMode = data.RequestorMode {
        return status;
    }
    //kernel_print::kernel_println!("DelProtectPreCreate not in kernel");

    unsafe {
        let params = &(*data.Iopb).Parameters.Create;

        if (params.Options & FILE_DELETE_ON_CLOSE) > 0 {
            kernel_print::kernel_println!("Delete on close");
            if !IsDeleteAllowed(NtCurrentProcess()) {
                *data.IoStatus.__bindgen_anon_1.Status_mut() = STATUS_ACCESS_DENIED;
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

    let params = unsafe { &(*data.Iopb).Parameters.SetFileInformation };

    match params.FileInformationClass {
        FILE_INFORMATION_CLASS::FileDispositionInformation
        | FILE_INFORMATION_CLASS::FileDispositionInformationEx => {},
        _ => return status,
    }

    let info = params.InfoBuffer as PFILE_DISPOSITION_INFORMATION;
    unsafe {
        if (*info).DeleteFile == 0 {
            return status;
        }

        let process = PsGetThreadProcess(data.Thread);
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
            *data.IoStatus.__bindgen_anon_1.Status_mut() = STATUS_ACCESS_DENIED;
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
        ExAllocatePool2(POOL_FLAG_PAGED, process_name_size, POOL_TAG) as PUNICODE_STRING;

    if process_name.is_null() {
        kernel_print::kernel_println!("fail to reserve a {} bytes of memory", process_name_size);
        return true;
    }

    let mut delete_allowed = true;
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

        if process_name.Length != 0 {
            let _locker = AutoLock::new(&mut G_MUTEX);
            if let Some(process_names) = &G_PROCESS_NAMES {
                for name in process_names {
                    if rust_process_name.contains(name) {
                        delete_allowed = false;
                        break;
                    }
                }
            }
        }
    }

    ExFreePoolWithTag(process_name as PVOID, POOL_TAG);

    delete_allowed
}

unsafe fn push_item_thread_safe(process_name: &str) {
    let mut p_name = String::new();
    if let Err(e) = p_name.try_reserve_exact(process_name.len()) {
        kernel_print::kernel_println!(
            "fail to reserve a {} bytes of memory. Err: {:?}",
            process_name.len(),
            e
        );
        return;
    }
    p_name.push_str(process_name);
    let _locker = AutoLock::new(&mut G_MUTEX);
    if let Some(process_names) = &mut G_PROCESS_NAMES {
        if process_names.len() >= MAX_ITEM_COUNT {
            process_names.pop_front();
        }
        process_names.push_back(p_name);
    }
}
