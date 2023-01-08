use winapi::{
    km::wdm::{PEPROCESS, PETHREAD},
    shared::ntdef::{HANDLE, NTSTATUS, PVOID, ULONG},
    um::winnt::ACCESS_MASK,
};

use crate::wmd::IoGetCurrentProcess;
use winapi::km::wdm::KPROCESSOR_MODE;

extern "system" {
    pub fn PsLookupProcessByProcessId(ProcessId: HANDLE, Process: *mut PEPROCESS) -> NTSTATUS;

    pub fn PsGetThreadProcess(Thread: PETHREAD) -> PEPROCESS;

    pub fn ObOpenObjectByPointer(
        Object: PVOID,
        HandleAttributes: ULONG,
        PassedAccessState: PVOID,
        DesiredAccess: ACCESS_MASK,
        ObjectType: PVOID,
        AccessMode: KPROCESSOR_MODE,
        Handle: *mut HANDLE,
    ) -> NTSTATUS;
}

#[allow(non_upper_case_globals)]
pub const PsGetCurrentProcess: unsafe extern "system" fn() -> PEPROCESS = IoGetCurrentProcess;
