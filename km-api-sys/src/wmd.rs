#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub const FILE_DELETE_ON_CLOSE: u32 = 0x00001000;

use crate::{intrinsics::ReadCR8, ntddk::PROCESSINFOCLASS};
use core::{mem, ptr::null_mut};
use winapi::{
    km::{
        ndis::PMDL,
        wdm::{
            SynchronizationEvent, DISPATCHER_HEADER, DRIVER_OBJECT, EVENT_TYPE, KEVENT,
            KPROCESSOR_MODE, PEPROCESS, PKEVENT,
        },
    },
    shared::{
        basetsd::PULONG_PTR,
        ntdef::{
            BOOLEAN, FALSE, HANDLE, LIST_ENTRY, LONG, LONGLONG, NTSTATUS, PULONG, PVOID, UCHAR,
            ULONG,
        },
    },
};

#[repr(C)]
#[derive(Copy, Clone)]
struct LARGE_INTEGER_PARTS {
    LowPart: ULONG,
    HighPart: LONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LARGE_INTEGER {
    u: ::core::mem::ManuallyDrop<LARGE_INTEGER_PARTS>,
    QuadPart: LONGLONG,
}

impl LARGE_INTEGER {
    pub const fn new() -> Self {
        Self { QuadPart: 0 }
    }

    pub const fn new_from_i64(time: i64) -> Self {
        Self { QuadPart: time }
    }
}

pub type PLARGE_INTEGER = *mut LARGE_INTEGER;

pub use crate::constants::*;
use kernel_string::{PUNICODE_STRING, UNICODE_STRING};

extern "system" {
    pub fn ObfDereferenceObject(Object: PVOID);

    //couldn't find such a function in library. Is it a macro???
    //pub fn MmGetSystemAddressForMdlSafe(MemoryDescriptorList: PMDL, Priority: ULONG) -> PVOID;

    pub fn MmMapLockedPagesSpecifyCache(
        MemoryDescriptorList: PMDL,
        AccessMode: KPROCESSOR_MODE,
        CacheType: ULONG,
        RequestedAddress: PVOID,
        BugCheckOnFailure: ULONG,
        Priority: ULONG,
    ) -> PVOID;

    pub fn CmRegisterCallbackEx(
        Function: PVOID,
        Altitude: &UNICODE_STRING,
        Driver: &DRIVER_OBJECT,
        Context: PVOID,
        Cookie: &LARGE_INTEGER,
        Reserved: PVOID,
    ) -> NTSTATUS;

    pub fn CmUnRegisterCallback(Cookie: LARGE_INTEGER) -> NTSTATUS;

    //another macro
    //pub fn ExInitializeFastMutex(mutex: PFAST_MUTEX);

    pub fn ExAcquireFastMutex(Mutex: PFAST_MUTEX);

    pub fn ExReleaseFastMutex(Mutex: PFAST_MUTEX);

    pub fn KeInitializeEvent(Mutex: PKEVENT, Type: EVENT_TYPE, State: BOOLEAN);

    pub fn CmCallbackGetKeyObjectIDEx(
        Cookie: &LARGE_INTEGER,
        Object: PVOID,
        ObjectId: PULONG_PTR,
        Object_name: &mut PUNICODE_STRING,
        Flags: ULONG,
    ) -> NTSTATUS;

    pub fn CmCallbackReleaseKeyObjectIDEx(ObjectName: PUNICODE_STRING);

    pub fn IoGetCurrentProcess() -> PEPROCESS;

    pub fn ZwQueryInformationProcess(
        ProcessHandle: HANDLE,
        ProcessInformationClass: PROCESSINFOCLASS,
        ProcessInformation: PVOID,
        ProcessInformationLength: ULONG,
        ReturnLength: PULONG,
    ) -> NTSTATUS;

    pub fn ZwClose(Handle: HANDLE);
}

pub type KIRQL = UCHAR;
pub unsafe fn KeGetCurrentIrql() -> KIRQL {
    ReadCR8() as KIRQL
}

pub unsafe fn ExInitializeFastMutex(Mutex: &mut FAST_MUTEX) {
    Mutex.Count = 1;
    Mutex.Owner = null_mut();
    Mutex.Contention = 0;

    KeInitializeEvent(&mut Mutex.Event as PKEVENT, SynchronizationEvent, FALSE);
}

//structures are defined for x64. I'm not sure it will works for x86. So fail to compile on x86
const _SIZE_CHECKER: [u8; 8] = [0; mem::size_of::<usize>()];

#[repr(C)]
pub struct FAST_MUTEX {
    pub(crate) Count: LONG,
    pub(crate) Owner: PVOID,
    pub(crate) Contention: ULONG,
    pub(crate) Event: KEVENT,
    pub(crate) OldIrql: ULONG,
}

impl FAST_MUTEX {
    pub const fn new() -> Self {
        Self {
            Count: 0,
            Owner: null_mut(),
            Contention: 9,
            Event: KEVENT {
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
            OldIrql: 0,
        }
    }
}
type PFAST_MUTEX = *mut FAST_MUTEX;

#[repr(C)]
pub struct CLIENT_ID {
    pub UniqueProcess: HANDLE,
    pub UniqueThread: HANDLE,
}

#[repr(C)]
pub struct MDL {
    pub Next: *mut MDL,
    pub Size: u16,
    pub MdlFlags: u16,
    pub Process: PVOID,
    pub MappedSystemVa: PVOID,
    pub StartVa: PVOID,
    pub ByteCount: ULONG,
    pub ByteOffset: ULONG,
}

pub const MDL_SOURCE_IS_NONPAGED_POOL: u16 = 0x0004;
pub const MDL_MAPPED_TO_SYSTEM_VA: u16 = 0x0001;

pub fn MmGetSystemAddressForMdlSafe(Mdl: *mut MDL, Priority: ULONG) -> PVOID {
    if Mdl.is_null() {
        return null_mut();
    }

    unsafe {
        if ((*Mdl).MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL)) != 0 {
            (*Mdl).MappedSystemVa
        } else {
            MmMapLockedPagesSpecifyCache(
                Mdl as PVOID,
                KPROCESSOR_MODE::KernelMode,
                1, /*MmCached*/
                null_mut(),
                0,
                Priority,
            )
        }
    }
}

#[repr(C)]
pub struct REG_POST_OPERATION_INFORMATION {
    pub Object: PVOID,
    pub Status: NTSTATUS,
    pub PreInformation: PVOID,
    pub ReturnStatus: NTSTATUS,
    pub CallContext: PVOID,
    pub ObjectContext: PVOID,
    pub Reserved: PVOID,
}
pub type PREG_POST_OPERATION_INFORMATION = *mut REG_POST_OPERATION_INFORMATION;

#[repr(C)]
pub struct REG_SET_VALUE_KEY_INFORMATION {
    pub Object: PVOID,
    pub ValueName: PUNICODE_STRING,
    pub TitleIndex: ULONG,
    pub DataType: ULONG,
    pub Data: PVOID,
    pub DataSize: ULONG,
    pub CallContext: PVOID,
    pub ObjectContext: PVOID,
    pub Reserved: PVOID,
}
pub type PREG_SET_VALUE_KEY_INFORMATION = *mut REG_SET_VALUE_KEY_INFORMATION;

pub const fn NtCurrentProcess() -> HANDLE {
    usize::MAX as HANDLE
}
