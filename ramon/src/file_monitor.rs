// trait ActivityMonitor {
//
// }

use crate::{G_MUTEX, G_PROCESS_NAMES, POOL_TAG};
use alloc::collections::BTreeSet;
use core::ptr::{addr_of_mut, null_mut};
use kernel_fast_mutex::{auto_lock::AutoLock, fast_mutex::FastMutex};
use kernel_macros::NT_SUCCESS;
use kernel_string::PUNICODE_STRING;
use km_api_sys::{
    flt_kernel::{FLT_CALLBACK_DATA, FLT_PREOP_CALLBACK_STATUS, PFLT_RELATED_OBJECTS},
    ntddk::PROCESSINFOCLASS,
    ntifs::{ObOpenObjectByPointer, PsGetThreadProcess},
    ntoskrnl::{ExAllocatePool2, ExFreePoolWithTag, POOL_FLAG_PAGED},
    wmd::{NtCurrentProcess, ZwClose, ZwQueryInformationProcess},
};
use winapi::km::wdm::KPROCESSOR_MODE;

use winapi::shared::ntdef::{HANDLE, NTSTATUS, OBJ_KERNEL_HANDLE, PVOID, ULONG};
use winapi::shared::ntstatus::STATUS_INFO_LENGTH_MISMATCH;

pub(crate) struct FileMonitor {}
impl FileMonitor {
    /*************************************************************************
    MiniFilter callback routines.
    *************************************************************************/
    pub(crate) extern "system" fn RamonPreCreate(
        data: &mut FLT_CALLBACK_DATA,
        _flt_objects: PFLT_RELATED_OBJECTS,
        _reserved: *mut PVOID,
    ) -> FLT_PREOP_CALLBACK_STATUS {
        //log::info!("RamonPreCreate");
        let status = FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_SUCCESS_NO_CALLBACK;

        if let KPROCESSOR_MODE::KernelMode = data.RequestorMode {
            //log::info!("RamonPreCreate kernel request")
        }

        FileMonitor::ProcessFileEvent(NtCurrentProcess());

        status
    }

    pub(crate) extern "system" fn RamonPreSetInformation(
        data: &mut FLT_CALLBACK_DATA,
        _flt_objects: PFLT_RELATED_OBJECTS,
        _reserved: *mut PVOID,
    ) -> FLT_PREOP_CALLBACK_STATUS {
        //log::info!("RamonPreSetInformation");
        let status = FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_SUCCESS_NO_CALLBACK;

        unsafe {
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

            FileMonitor::ProcessFileEvent(h_process);
            ZwClose(h_process);
        }
        status
    }
}

impl FileMonitor {
    fn ProcessFileEvent(h_process: HANDLE) {
        let process_name_size = 300;
        let process_name = unsafe {
            ExAllocatePool2(POOL_FLAG_PAGED, process_name_size, POOL_TAG) as PUNICODE_STRING
        };

        if process_name.is_null() {
            log::info!("fail to reserve a {} bytes of memory", process_name_size);
            return;
        }

        let mut return_length: ULONG = 0;
        let status = unsafe {
            ZwQueryInformationProcess(
                h_process,
                PROCESSINFOCLASS::ProcessImageFileName,
                process_name as PVOID,
                (process_name_size - 2) as u32,
                &mut return_length,
            )
        };

        if status == STATUS_INFO_LENGTH_MISMATCH {
            //too small buffer
            unsafe { ExFreePoolWithTag(process_name as PVOID, POOL_TAG) };
            return;
        }

        //prevent spam
        unsafe {
            //log::info!("Before lock. Len: {}", return_length);
            let _locker = AutoLock::new(&mut G_MUTEX);
            if let Some(process_names) = &mut G_PROCESS_NAMES {
                if process_names.contains(&return_length) {
                    return;
                }
                process_names.push_back(return_length);
            }

        }

        log::info!(
            "ZwQueryInformationProcess - status: {}, returnLength: {}",
            status,
            return_length
        );

        if NT_SUCCESS!(status) {
            let unicode_process_name = unsafe { &*process_name };
            let rust_process_name = unicode_process_name.as_rust_string();
            log::info!("Name: {}", rust_process_name);

            // unsafe {
            // let event =
            //     {
            //     let size_to_alloc = size_of::<FileEvent>() + rust_process_name.len();
            //     let event = ExAllocatePool2(POOL_FLAG_PAGED, size_to_alloc, POOL_TAG) as *mut FileEvent;
            //     RtlZeroMemory( event as PVOID, size_to_alloc);
            //     (*event).path_len = rust_process_name.len() as u32;
            //     let mut path_ptr = &mut (*event).path as *mut u8;
            //     for b in rust_process_name.as_bytes() {
            //         *path_ptr = *b;
            //         path_ptr+=1;
            //     }
            //     *path_ptr = 0;
            //     event
            // };

            unsafe { ExFreePoolWithTag(process_name as PVOID, POOL_TAG) };
        }
    }
}
