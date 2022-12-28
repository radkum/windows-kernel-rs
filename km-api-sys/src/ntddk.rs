#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use winapi::km::wdm::{PEPROCESS, PFILE_OBJECT};
use winapi::shared::ntdef::{BOOLEAN, NTSTATUS, ULONG};

pub type PVOID = *mut winapi::ctypes::c_void;
pub type HANDLE = PVOID;
pub type SIZE_T = usize;
pub use crate::constants::*;
use crate::wmd::CLIENT_ID;
use kernel_string::UnicodeString;

extern "system" {
    pub fn MmIsAddressValid(virtual_address: PVOID) -> bool;

    pub fn PsSetCreateProcessNotifyRoutineEx(
        notify_routine: PCREATE_PROCESS_NOTIFY_ROUTINE_EX,
        remove: BOOLEAN,
    ) -> NTSTATUS;

    pub fn PsSetCreateThreadNotifyRoutine(
        notify_routine: PCREATE_THREAD_NOTIFY_ROUTINE,
    ) -> NTSTATUS;

    pub fn PsRemoveCreateThreadNotifyRoutine(
        notify_routine: PCREATE_THREAD_NOTIFY_ROUTINE,
    ) -> NTSTATUS;

    pub fn PsSetLoadImageNotifyRoutine(notify_routine: PLOAD_IMAGE_NOTIFY_ROUTINE) -> NTSTATUS;

    pub fn PsRemoveLoadImageNotifyRoutine(notify_routine: PLOAD_IMAGE_NOTIFY_ROUTINE) -> NTSTATUS;
}

#[repr(C)]
pub struct PS_CREATE_NOTIFY_INFO {
    pub size: SIZE_T,
    pub flags: ULONG,
    pub parent_process_id: HANDLE,
    pub creating_thread_id: CLIENT_ID,
    pub file_object: PFILE_OBJECT,
    pub image_file_name: *mut UnicodeString,
    pub command_line: *mut UnicodeString,
    pub creation_status: NTSTATUS,
}

pub type PPS_CREATE_NOTIFY_INFO = *mut PS_CREATE_NOTIFY_INFO;

pub type PCREATE_PROCESS_NOTIFY_ROUTINE_EX =
    extern "system" fn(process: PEPROCESS, process_id: HANDLE, create_info: PPS_CREATE_NOTIFY_INFO);

pub type PCREATE_THREAD_NOTIFY_ROUTINE =
    extern "system" fn(process_id: HANDLE, thread_id: HANDLE, create: BOOLEAN);

#[repr(C)]
pub struct IMAGE_INFO {
    pub properties: ULONG,
    pub image_base: PVOID,
    pub image_selector: ULONG,
    pub image_size: SIZE_T,
    pub image_section_number: ULONG,
}

pub type PIMAGE_INFO = *mut IMAGE_INFO;

pub type PLOAD_IMAGE_NOTIFY_ROUTINE = extern "system" fn(
    full_image_name: *mut UnicodeString,
    process_id: HANDLE,
    image_info: PIMAGE_INFO,
);
