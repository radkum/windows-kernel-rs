use winapi::km::wdm::{PEPROCESS, PETHREAD};
use winapi::shared::ntdef::{NTSTATUS, ULONG};
use winapi::um::winnt::ACCESS_MASK;

use crate::wmd::IoGetCurrentProcess;
use winapi::km::wdm::KPROCESSOR_MODE;

pub type PVOID = *mut winapi::ctypes::c_void;
pub type HANDLE = PVOID;

extern "system" {
    pub fn PsLookupProcessByProcessId(process_id: HANDLE, process: *mut PEPROCESS) -> NTSTATUS;

    pub fn PsGetThreadProcess(thread: PETHREAD) -> PEPROCESS;

    pub fn ObOpenObjectByPointer(
        object: PVOID,
        handle_attributes: ULONG,
        passed_access_state: PVOID,
        desired_access: ACCESS_MASK,
        object_type: PVOID,
        access_mode: KPROCESSOR_MODE,
        handle: *mut HANDLE,
    ) -> NTSTATUS;
}

pub const PsGetCurrentProcess: unsafe extern "system" fn() -> PEPROCESS = IoGetCurrentProcess;
