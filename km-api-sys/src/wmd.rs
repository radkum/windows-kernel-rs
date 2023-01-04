#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod constants;
pub use constants::*;

use core::mem;
use core::ptr::null_mut;
use winapi::km::ndis::PMDL;
use winapi::km::wdm::{
    SynchronizationEvent, DISPATCHER_HEADER, DRIVER_OBJECT, EVENT_TYPE, KEVENT, KPROCESSOR_MODE,
    PEPROCESS, PKEVENT,
};
use winapi::shared::basetsd::PULONG_PTR;
use winapi::shared::ntdef::{
    BOOLEAN, FALSE, LIST_ENTRY, LONG, LONGLONG, NTSTATUS, PULONG, UCHAR, ULONG,
};

use crate::ntddk::PROCESSINFOCLASS;

pub type PVOID = *mut winapi::ctypes::c_void;
pub type HANDLE = PVOID;
pub type SIZE_T = usize;
pub type KIRQL = UCHAR;

pub use crate::constants::*;
use kernel_string::{PUnicodeString, UnicodeString};

extern "system" {
    pub fn ObfDereferenceObject(object: PVOID);

    //couldn't find such a function in library. Is it a macro???
    //pub fn MmGetSystemAddressForMdlSafe(pmdl: PMDL, priority: ULONG) -> PVOID;

    pub fn MmMapLockedPagesSpecifyCache(
        pmdl: PMDL,
        access_mode: KPROCESSOR_MODE,
        cache_type: ULONG,
        r: PVOID,
        b: ULONG,
        p: ULONG,
    ) -> PVOID;

    pub fn CmRegisterCallbackEx(
        function: PVOID,
        altitude: &UnicodeString,
        driver: &DRIVER_OBJECT,
        context: PVOID,
        cookie: &LONGLONG,
        reserved: PVOID,
    ) -> NTSTATUS;

    pub fn CmUnRegisterCallback(cookie: LONGLONG) -> NTSTATUS;

    //another macro
    //pub fn ExInitializeFastMutex(mutex: PFAST_MUTEX);

    pub fn ExAcquireFastMutex(mutex: PFAST_MUTEX);

    pub fn ExReleaseFastMutex(mutex: PFAST_MUTEX);

    pub fn KeInitializeEvent(mutex: PKEVENT, typee: EVENT_TYPE, state: BOOLEAN);

    pub fn CmCallbackGetKeyObjectIDEx(
        cookie: &LONGLONG,
        object: PVOID,
        object_id: PULONG_PTR,
        object_name: &mut PUnicodeString,
        flags: ULONG,
    ) -> NTSTATUS;

    pub fn CmCallbackReleaseKeyObjectIDEx(name: PUnicodeString);

    pub fn IoGetCurrentProcess() -> PEPROCESS;

    pub fn ZwQueryInformationProcess(
        process_handle: HANDLE,
        process_information_class: PROCESSINFOCLASS,
        process_information: PVOID,
        process_information_length: ULONG,
        return_lenght: PULONG,
    ) -> NTSTATUS;

    pub fn ZwClose(handle: HANDLE);
}

#[link(name = "hal")]
extern "system" {
    pub fn KeGetCurrentIrql() -> KIRQL;
}

pub unsafe fn ExInitializeFastMutex(mutex: &mut FAST_MUTEX) {
    mutex.count = 1;
    mutex.owner = null_mut();
    mutex.contention = 0;

    KeInitializeEvent(&mut mutex.event as PKEVENT, SynchronizationEvent, FALSE);
}

//structures are defined for x64. I'm not sure it will works for x86. So fail to compile on x86
const _SIZE_CHECKER: [u8; 8] = [0; mem::size_of::<usize>()];

#[repr(C)]
pub struct FAST_MUTEX {
    pub(crate) count: LONG,
    pub(crate) owner: PVOID,
    pub(crate) contention: ULONG,
    pub(crate) event: KEVENT,
    pub(crate) old_irql: ULONG,
}

impl FAST_MUTEX {
    pub const fn new() -> Self {
        Self {
            count: 0,
            owner: null_mut(),
            contention: 9,
            event: KEVENT {
                Header: DISPATCHER_HEADER {
                    Type: 0,
                    Absolute: 0,
                    Size: 0,
                    Inserted: 0,
                    SignalState: 0,
                    WaitListHead: LIST_ENTRY {
                        Blink: null_mut(),
                        Flink: null_mut(),
                    },
                },
            },
            old_irql: 0,
        }
    }
}
type PFAST_MUTEX = *mut FAST_MUTEX;

#[repr(C)]
pub struct CLIENT_ID {
    pub unique_process: HANDLE,
    pub unique_thread: HANDLE,
}

#[repr(C)]
pub struct MDL {
    pub next: *mut MDL,
    pub size: u16,
    pub mdl_flags: u16,
    pub process: PVOID,
    pub mapped_system_va: PVOID,
    pub start_va: PVOID,
    pub byte_count: ULONG,
    pub byte_offset: ULONG,
}

pub fn MmGetSystemAddressForMdlSafe(mdl: *mut MDL, priority: ULONG) -> PVOID {
    if mdl.is_null() {
        return null_mut();
    }

    unsafe {
        if ((*mdl).mdl_flags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL)) != 0 {
            (*mdl).mapped_system_va
        } else {
            MmMapLockedPagesSpecifyCache(
                mdl as PVOID,
                KPROCESSOR_MODE::KernelMode,
                1, /*MmCached*/
                null_mut(),
                0,
                priority,
            )
        }
    }
}

#[repr(C)]
pub struct REG_POST_OPERATION_INFORMATION {
    pub object: PVOID,
    pub status: NTSTATUS,
    pub pre_information: PVOID,
    pub return_status: NTSTATUS,
    pub call_context: PVOID,
    pub object_context: PVOID,
    pub reserved: PVOID,
}
pub type PREG_POST_OPERATION_INFORMATION = *mut REG_POST_OPERATION_INFORMATION;

#[repr(C)]
pub struct REG_SET_VALUE_KEY_INFORMATION {
    pub object: PVOID,
    pub value_name: PUnicodeString,
    pub title_index: ULONG,
    pub data_type: ULONG,
    pub data: PVOID,
    pub data_size: ULONG,
    pub call_context: PVOID,
    pub object_context: PVOID,
    pub reserved: PVOID,
}
pub type PREG_SET_VALUE_KEY_INFORMATION = *mut REG_SET_VALUE_KEY_INFORMATION;

pub const fn NtCurrentProcess() -> HANDLE {
    usize::MAX as HANDLE
}
