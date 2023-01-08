#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod process_info_class;
pub use process_info_class::*;

pub use crate::constants::*;
use crate::wmd::CLIENT_ID;
use kernel_string::UNICODE_STRING;
use winapi::{
    km::wdm::{PEPROCESS, PFILE_OBJECT},
    shared::{
        basetsd::SIZE_T,
        ntdef::{BOOLEAN, HANDLE, NTSTATUS, PVOID, ULONG},
    },
};

pub const REG_NT_POST_SET_VALUE_KEY: u32 = 16;

extern "system" {
    pub fn MmIsAddressValid(VirtualAddress: PVOID) -> bool;

    pub fn PsGetCurrentProcessId() -> HANDLE;

    pub fn PsGetCurrentThreadId() -> HANDLE;

    pub fn PsSetCreateProcessNotifyRoutineEx(
        NotifyRoutine: PCREATE_PROCESS_NOTIFY_ROUTINE_EX,
        Remove: BOOLEAN,
    ) -> NTSTATUS;

    pub fn PsSetCreateThreadNotifyRoutine(NotifyRoutine: PCREATE_THREAD_NOTIFY_ROUTINE)
        -> NTSTATUS;

    pub fn PsRemoveCreateThreadNotifyRoutine(
        NotifyRoutine: PCREATE_THREAD_NOTIFY_ROUTINE,
    ) -> NTSTATUS;

    pub fn PsSetLoadImageNotifyRoutine(NotifyRoutine: PLOAD_IMAGE_NOTIFY_ROUTINE) -> NTSTATUS;

    pub fn PsRemoveLoadImageNotifyRoutine(NotifyRoutine: PLOAD_IMAGE_NOTIFY_ROUTINE) -> NTSTATUS;
}

#[repr(C)]
pub struct PS_CREATE_NOTIFY_INFO {
    pub Size: SIZE_T,
    pub Flags: ULONG,
    pub ParentProcessId: HANDLE,
    pub CreatingThreadId: CLIENT_ID,
    pub FileObject: PFILE_OBJECT,
    pub ImageFileName: *mut UNICODE_STRING,
    pub CommandLine: *mut UNICODE_STRING,
    pub CreationStatus: NTSTATUS,
}

pub type PPS_CREATE_NOTIFY_INFO = *mut PS_CREATE_NOTIFY_INFO;

pub type PCREATE_PROCESS_NOTIFY_ROUTINE_EX =
    extern "system" fn(Process: PEPROCESS, ProcessId: HANDLE, CreateInfo: PPS_CREATE_NOTIFY_INFO);

pub type PCREATE_THREAD_NOTIFY_ROUTINE =
    extern "system" fn(ProcessId: HANDLE, ThreadId: HANDLE, Create: BOOLEAN);

#[repr(C)]
pub struct IMAGE_INFO {
    pub Properties: ULONG,
    pub ImageBase: PVOID,
    pub ImageSelector: ULONG,
    pub ImageSize: SIZE_T,
    pub ImageSectionNumber: ULONG,
}

pub type PIMAGE_INFO = *mut IMAGE_INFO;

pub type PLOAD_IMAGE_NOTIFY_ROUTINE = extern "system" fn(
    FullImageName: *mut UNICODE_STRING,
    ProcessId: HANDLE,
    ImageInfo: PIMAGE_INFO,
);

#[repr(C)]
pub struct FILE_DISPOSITION_INFORMATION {
    pub DeleteFile: BOOLEAN,
}

pub type PFILE_DISPOSITION_INFORMATION = *mut FILE_DISPOSITION_INFORMATION;
