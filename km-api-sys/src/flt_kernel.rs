#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ptr::null_mut;
use winapi::km::wdm::{DEVICE_TYPE, PDRIVER_OBJECT};
use winapi::shared::ntdef::ULONG;
use winapi::shared::ntdef::USHORT;
use winapi::shared::ntdef::UCHAR;
use winapi::shared::ntdef::NTSTATUS;

use crate::wmd::PVOID;

#[link(name = "fltMgr")]
extern "system" {
    pub fn FltRegisterFilter(driver: PDRIVER_OBJECT, registration: &FLT_REGISTRATION, ret_filter: &mut PFLT_FILTER) -> NTSTATUS;

    pub fn FltUnregisterFilter(filter: PFLT_FILTER) -> NTSTATUS;

    pub fn FltStartFiltering(filter: PFLT_FILTER) -> NTSTATUS;
}

pub type FLT_REGISTRATION_FLAGS = ULONG;
pub type FLT_POST_OPERATION_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_INSTANCE_TEARDOWN_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_INSTANCE_QUERY_TEARDOWN_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_INSTANCE_SETUP_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_FILTER_UNLOAD_FLAGS = FLT_REGISTRATION_FLAGS;

#[repr(C)]
pub struct FLT_FILTER {
    filler: [u8; 0x120],
}
pub type PFLT_FILTER = *mut FLT_FILTER;

pub const FLT_REGISTRATION_VERSION: USHORT = 0x0203;

#[repr(C)]
pub struct FLT_REGISTRATION {
    pub size: USHORT,
    pub version: USHORT,
    pub flags: FLT_REGISTRATION_FLAGS,
    pub context_registration: PVOID,
    pub operation_registration: PFLT_OPERATION_REGISTRATION,
    pub filter_unload_callback: PFLT_FILTER_UNLOAD_CALLBACK,
    pub instance_setup_callback: PFLT_INSTANCE_SETUP_CALLBACK,
    pub instance_query_teardown_callback: PFLT_INSTANCE_QUERY_TEARDOWN_CALLBACK,
    pub instance_teardown_start_callback: PFLT_INSTANCE_TEARDOWN_CALLBACK,
    pub instance_teardown_complete_callback: PFLT_INSTANCE_TEARDOWN_CALLBACK,
    pub generate_file_name_callback: PVOID,
    pub normalize_namecomponent_callback: PVOID,
    pub normalize_context_cleanup_callback: PVOID,
    pub transaction_notification_callback: PVOID,
    pub normalize_name_component_ex_callback: PVOID,
    pub section_notification_callback: PVOID,
}
pub type PFLT_REGISTRATION = *mut FLT_REGISTRATION;

pub type FLT_PREOP_CALLBACK_STATUS = i32;
pub type PFLT_PRE_OPERATION_CALLBACK =
    extern "system" fn(data: PFLT_CALLBACK_DATA, flt_objects: PFLT_RELATED_OBJECTS, completion_context: *mut PVOID) -> FLT_PREOP_CALLBACK_STATUS;

pub type FLT_POST_CALLBACK_STATUS = i32;
pub type FLT_POST_OPERATION_CALLBACK =
    extern "system" fn(data: PFLT_CALLBACK_DATA, flt_objects: PFLT_RELATED_OBJECTS, completion_context: PVOID, flags: FLT_POST_OPERATION_FLAGS) -> FLT_POST_CALLBACK_STATUS;
pub type PFLT_POST_OPERATION_CALLBACK = *mut FLT_POST_OPERATION_CALLBACK;

#[repr(C)]
pub struct FLT_OPERATION_REGISTRATION {
    major_function: UCHAR,
    flags: FLT_REGISTRATION_FLAGS,
    pre_operation: Option<PFLT_PRE_OPERATION_CALLBACK>,
    post_operation: Option<PFLT_POST_OPERATION_CALLBACK>,
    reserved: PVOID,
}
pub type PFLT_OPERATION_REGISTRATION = *const FLT_OPERATION_REGISTRATION;

impl FLT_OPERATION_REGISTRATION {
    pub const IRP_MJ_CREATE: UCHAR = 0x00;
    pub const IRP_MJ_SET_INFORMATION: UCHAR = 0x06;
    pub const IRP_MJ_OPERATION_END: UCHAR = 0x80;

    pub const fn new() -> Self {
        FLT_OPERATION_REGISTRATION {
            major_function: 0,
            flags: 0,
            pre_operation: None,
            post_operation: None,
            reserved: null_mut(),
        }
    }

    pub const fn set_major_function(&self, major_function: UCHAR) -> Self {
        FLT_OPERATION_REGISTRATION {
            major_function,
            flags: self.flags,
            pre_operation: self.pre_operation,
            post_operation: self.post_operation,
            reserved: null_mut(),
        }
    }

    pub const fn set_flags(&self, flags: FLT_REGISTRATION_FLAGS) -> Self {
        FLT_OPERATION_REGISTRATION {
            major_function: self.major_function,
            flags,
            pre_operation: self.pre_operation,
            post_operation: self.post_operation,
            reserved: null_mut(),
        }
    }

    pub const fn set_preop(&self, preop: PFLT_PRE_OPERATION_CALLBACK) -> Self {
        FLT_OPERATION_REGISTRATION {
            major_function: self.major_function,
            flags: self.flags,
            pre_operation: Some(preop),
            post_operation: self.post_operation,
            reserved: null_mut(),
        }
    }

    pub const fn set_postop(&self, postop: PFLT_POST_OPERATION_CALLBACK) -> Self {
        FLT_OPERATION_REGISTRATION {
            major_function: self.major_function,
            flags: self.flags,
            pre_operation: self.pre_operation,
            post_operation: Some(postop),
            reserved: null_mut(),
        }
    }
}

pub type PFLT_CALLBACK_DATA = PVOID;
pub type PFLT_RELATED_OBJECTS = PVOID;

pub type PFLT_FILTER_UNLOAD_CALLBACK =
    extern "system" fn(flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS;

pub type FLT_FILESYSTEM_TYPE = ULONG;
pub type PFLT_INSTANCE_SETUP_CALLBACK =
    extern "system" fn(flt_objects: PFLT_RELATED_OBJECTS, flags: FLT_INSTANCE_SETUP_FLAGS, volume_device_type: DEVICE_TYPE, volume_filesystem_type: FLT_FILESYSTEM_TYPE) -> NTSTATUS;

pub type PFLT_INSTANCE_QUERY_TEARDOWN_CALLBACK =
    extern "system" fn(flt_objects: PFLT_RELATED_OBJECTS, reason: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS) -> NTSTATUS;

pub type PFLT_INSTANCE_TEARDOWN_CALLBACK =
    extern "system" fn(flt_objects: PFLT_RELATED_OBJECTS, reason: FLT_INSTANCE_TEARDOWN_FLAGS) -> NTSTATUS;