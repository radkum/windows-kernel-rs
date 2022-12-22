use core::ptr::null_mut;
use winapi::km::ndis::PMDL;
use winapi::km::wdm::{KPROCESSOR_MODE, PEPROCESS};
use winapi::shared::ntdef::{NTSTATUS, ULONG};

pub type PVOID = *mut winapi::ctypes::c_void;
pub type HANDLE = PVOID;
use crate::constants::*;

extern "system" {
    pub fn MmIsAddressValid(virtual_address: PVOID) -> bool;

    pub fn PsLookupProcessByProcessId(process_id: HANDLE, process: *mut PEPROCESS) -> NTSTATUS;

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
        kernel_print::kernel_println!("MdlAddress is null");
        return null_mut();
    }

    unsafe {
        kernel_print::kernel_println!("MmGetSystemAddressForMdlSafe start ");
        if ((*mdl).mdl_flags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL)) != 0 {
            (*mdl).mapped_system_va
        } else {
            MmMapLockedPagesSpecifyCache(
                mdl as PVOID,
                KPROCESSOR_MODE::KernelMode,
                1,
                null_mut(),
                0,
                priority,
            )
        }
    }
}
