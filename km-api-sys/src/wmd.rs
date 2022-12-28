#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::mem;
use core::ptr::null_mut;
use winapi::km::ndis::PMDL;
use winapi::km::wdm::{SynchronizationEvent, DRIVER_OBJECT, EVENT_TYPE, KPROCESSOR_MODE};
use winapi::shared::ntdef::{BOOLEAN, FALSE, LONG, LONGLONG, NTSTATUS, ULONG};

pub type PVOID = *mut winapi::ctypes::c_void;
pub type HANDLE = PVOID;
pub type SIZE_T = usize;

pub use crate::constants::*;
use kernel_string::UnicodeString;

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

    pub fn KeInitializeEvent(mutex: PVOID, typee: EVENT_TYPE, state: BOOLEAN);
}

pub unsafe fn ExInitializeFastMutex(mutex: &mut FAST_MUTEX) {
    mutex.count = 1;
    mutex.owner = null_mut();
    mutex.contention = 0;

    KeInitializeEvent(
        &mut mutex.byte_filler as *mut u8 as PVOID,
        SynchronizationEvent,
        FALSE,
    );
}

#[repr(C)]
pub struct FAST_MUTEX {
    pub(crate) count: LONG,
    pub(crate) owner: PVOID,
    pub(crate) contention: ULONG,
    pub(crate) byte_filler: [u8; 36],
    pub(crate) old_irql: ULONG,
}
const _SIZE_CHECKER: [u8; 8] = [0; mem::size_of::<usize>()];

impl FAST_MUTEX {
    pub const fn new() -> Self {
        Self {
            count: 0,
            owner: null_mut(),
            contention: 9,
            byte_filler: [0; 36],
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

// #[repr(C)]
// pub struct FAST_MUTEX {
//     pub(crate) count: LONG,
//     pub(crate) owner: PVOID,
//     pub(crate) contention: ULONG,
//     pub(crate) event: KEVENT,
//     pub(crate) old_irql: ULONG,
// }

// impl FAST_MUTEX {
//     pub fn new() -> Self {
//         Self {
//             count: 0,
//             owner: null_mut(),
//             contention: 0,
//             event: KEVENT {
//                 Header: DISPATCHER_HEADER {
//                     Type: 0,
//                     Absolute: 0,
//                     Size: 0,
//                     Inserted: 0,
//                     SignalState: 0,
//                     WaitListHead: LIST_ENTRY {Blink: null_mut(), Flink: null_mut()},
//                 },
//             },
//             old_irql: 0,
//         }
//     }
// }
