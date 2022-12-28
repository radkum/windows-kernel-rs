use winapi::km::wdm::PEPROCESS;
use winapi::shared::ntdef::NTSTATUS;
pub type PVOID = *mut winapi::ctypes::c_void;
pub type HANDLE = PVOID;

extern "system" {
    pub fn PsLookupProcessByProcessId(process_id: HANDLE, process: *mut PEPROCESS) -> NTSTATUS;
}
