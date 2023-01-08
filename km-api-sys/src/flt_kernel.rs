#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod flt_parameters;
pub use flt_parameters::*;

use core::ptr::null_mut;
use winapi::{
    km::wdm::{
        DEVICE_TYPE, IO_STATUS_BLOCK, KPROCESSOR_MODE, PDRIVER_OBJECT, PETHREAD, PFILE_OBJECT,
    },
    shared::ntdef::{LIST_ENTRY, NTSTATUS, PVOID, UCHAR, ULONG, USHORT},
};

#[link(name = "fltMgr")]
extern "system" {
    pub fn FltRegisterFilter(
        Driver: PDRIVER_OBJECT,
        Registration: &FLT_REGISTRATION,
        RetFilter: &mut PFLT_FILTER,
    ) -> NTSTATUS;

    pub fn FltUnregisterFilter(Filter: PFLT_FILTER) -> NTSTATUS;

    pub fn FltStartFiltering(Filter: PFLT_FILTER) -> NTSTATUS;
}

pub type FLT_REGISTRATION_FLAGS = ULONG;
pub type FLT_POST_OPERATION_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_INSTANCE_TEARDOWN_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_INSTANCE_QUERY_TEARDOWN_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_INSTANCE_SETUP_FLAGS = FLT_REGISTRATION_FLAGS;
pub type FLT_FILTER_UNLOAD_FLAGS = FLT_REGISTRATION_FLAGS;

#[repr(C)]
pub struct FLT_FILTER {
    Filler: [u8; 0x120],
}
pub type PFLT_FILTER = *mut FLT_FILTER;

pub const FLT_REGISTRATION_VERSION: u16 = 0x0203;

#[repr(C)]
pub struct FLT_REGISTRATION {
    pub Size: USHORT,
    pub Version: USHORT,
    pub Flags: FLT_REGISTRATION_FLAGS,
    pub ContextRegistration: PVOID, /*PFLT_CONTEXT_REGISTRATION*/
    pub OperationRegistration: PFLT_OPERATION_REGISTRATION,
    pub FilterUnloadCallback: PFLT_FILTER_UNLOAD_CALLBACK,
    pub InstanceSetupCallback: PFLT_INSTANCE_SETUP_CALLBACK,
    pub InstanceQueryTeardownCallback: PFLT_INSTANCE_QUERY_TEARDOWN_CALLBACK,
    pub InstanceTeardownStartCallback: PFLT_INSTANCE_TEARDOWN_CALLBACK,
    pub InstanceTeardownCompleteCallback: PFLT_INSTANCE_TEARDOWN_CALLBACK,
    pub GenerateFileNameCallback: PVOID,
    pub NormalizeNameComponentCallback: PVOID,
    pub NormalizeContextCleanupCallback: PVOID,
    pub TransactionNotificationCallback: PVOID,
    pub NormalizeNameComponentExCallback: PVOID,
    pub SectionNotificationCallback: PVOID,
}
pub type PFLT_REGISTRATION = *mut FLT_REGISTRATION;

#[repr(C)]
pub enum FLT_PREOP_CALLBACK_STATUS {
    FLT_PREOP_SUCCESS_WITH_CALLBACK,
    FLT_PREOP_SUCCESS_NO_CALLBACK,
    FLT_PREOP_PENDING,
    FLT_PREOP_DISALLOW_FASTIO,
    FLT_PREOP_COMPLETE,
    FLT_PREOP_SYNCHRONIZE,
    FLT_PREOP_DISALLOW_FSFILTER_IO,
}

#[repr(C)]
pub enum FLT_POSTOP_CALLBACK_STATUS {
    FLT_POSTOP_FINISHED_PROCESSING,
    FLT_POSTOP_MORE_PROCESSING_REQUIRED,
    FLT_POSTOP_DISALLOW_FSFILTER_IO,
}

pub type PFLT_PRE_OPERATION_CALLBACK = extern "system" fn(
    Data: &mut FLT_CALLBACK_DATA,
    FltObjects: PFLT_RELATED_OBJECTS,
    CompletionContext: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS;

pub type PFLT_POST_OPERATION_CALLBACK = extern "system" fn(
    Data: &mut FLT_CALLBACK_DATA,
    FltObjects: PFLT_RELATED_OBJECTS,
    CompletionContext: PVOID,
    Flags: FLT_POST_OPERATION_FLAGS,
) -> FLT_POSTOP_CALLBACK_STATUS;

#[repr(C)]
#[derive(Copy, Clone)]
pub union PFLT_OPERATION_CALLBACK_UNION {
    nullptr: PVOID,
    preop_fn_ptr: PFLT_PRE_OPERATION_CALLBACK,
    postop_fn_ptr: PFLT_POST_OPERATION_CALLBACK,
}

#[repr(C)]
pub struct FLT_OPERATION_REGISTRATION {
    MajorFunction: UCHAR,
    Flags: FLT_REGISTRATION_FLAGS,
    PreOperation: PFLT_OPERATION_CALLBACK_UNION,
    PostOperation: PFLT_OPERATION_CALLBACK_UNION,
    Reserved: PVOID,
}
pub type PFLT_OPERATION_REGISTRATION = *const FLT_OPERATION_REGISTRATION;

impl FLT_OPERATION_REGISTRATION {
    pub const IRP_MJ_CREATE: UCHAR = 0x00;
    pub const IRP_MJ_SET_INFORMATION: UCHAR = 0x06;
    pub const IRP_MJ_OPERATION_END: UCHAR = 0x80;

    pub const fn new() -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: 0,
            Flags: 0,
            PreOperation: PFLT_OPERATION_CALLBACK_UNION {
                nullptr: null_mut(),
            },
            PostOperation: PFLT_OPERATION_CALLBACK_UNION {
                nullptr: null_mut(),
            },
            Reserved: null_mut(),
        }
    }

    pub const fn set_major_function(&self, major_function: UCHAR) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: major_function,
            Flags: self.Flags,
            PreOperation: self.PreOperation,
            PostOperation: self.PostOperation,
            Reserved: null_mut(),
        }
    }

    pub const fn set_flags(&self, flags: FLT_REGISTRATION_FLAGS) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.MajorFunction,
            Flags: flags,
            PreOperation: self.PreOperation,
            PostOperation: self.PostOperation,
            Reserved: null_mut(),
        }
    }

    pub const fn set_preop(&self, preop: PFLT_PRE_OPERATION_CALLBACK) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.MajorFunction,
            Flags: self.Flags,
            PreOperation: PFLT_OPERATION_CALLBACK_UNION {
                preop_fn_ptr: preop,
            },
            PostOperation: self.PostOperation,
            Reserved: null_mut(),
        }
    }

    pub const fn set_postop(&self, postop: PFLT_POST_OPERATION_CALLBACK) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.MajorFunction,
            Flags: self.Flags,
            PreOperation: self.PreOperation,
            PostOperation: PFLT_OPERATION_CALLBACK_UNION {
                postop_fn_ptr: postop,
            },
            Reserved: null_mut(),
        }
    }
}

pub type FLT_CALLBACK_DATA_FLAGS = ULONG;

#[repr(C)]
pub struct PFLT_CALLBACK_DATA_UNION_STRUCT {
    QueueLinks: LIST_ENTRY,
    QueueContext: [PVOID; 2],
}

#[repr(C)]
pub union PFLT_CALLBACK_DATA_UNION {
    F1: ::core::mem::ManuallyDrop<PFLT_CALLBACK_DATA_UNION_STRUCT>,
    F2: [PVOID; 4],
}
const _SIZE_CHECKER: [u8; 32] = [0; ::core::mem::size_of::<PFLT_CALLBACK_DATA_UNION>()];

pub type PFLT_INSTANCE = PVOID;

#[repr(C)]
pub struct FLT_IO_PARAMETER_BLOCK {
    pub IrpFlags: ULONG,
    pub MajorFunction: UCHAR,
    pub MinorFunction: UCHAR,
    pub OperationFlags: UCHAR,
    Reserved: UCHAR,
    pub TargetFileObject: PFILE_OBJECT,
    pub TargetInstance: PFLT_INSTANCE,
    pub Parameters: FLT_PARAMETERS,
}
pub type PFLT_IO_PARAMETER_BLOCK = *mut FLT_IO_PARAMETER_BLOCK;

#[repr(C)]
pub struct FLT_CALLBACK_DATA {
    pub Flags: FLT_CALLBACK_DATA_FLAGS,
    pub Thread: PETHREAD,
    pub Iopb: PFLT_IO_PARAMETER_BLOCK,
    pub IoStatus: IO_STATUS_BLOCK,
    pub RagData: PVOID, /* PFLT_TAG_DATA_BUFFER */
    pub FieldUnion: PFLT_CALLBACK_DATA_UNION,
    pub RequestorMode: KPROCESSOR_MODE,
}
pub type PFLT_CALLBACK_DATA = *mut FLT_CALLBACK_DATA;

const _SIZE_CHECKER2: [u8; 88] = [0; ::core::mem::size_of::<FLT_CALLBACK_DATA>()];

pub type PFLT_RELATED_OBJECTS = PVOID;

pub type PFLT_FILTER_UNLOAD_CALLBACK =
    extern "system" fn(Flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS;

pub type FLT_FILESYSTEM_TYPE = ULONG;
pub type PFLT_INSTANCE_SETUP_CALLBACK = extern "system" fn(
    FltObjects: PFLT_RELATED_OBJECTS,
    Flags: FLT_INSTANCE_SETUP_FLAGS,
    VolumeDeviceType: DEVICE_TYPE,
    VolumeFilesystemType: FLT_FILESYSTEM_TYPE,
) -> NTSTATUS;

pub type PFLT_INSTANCE_QUERY_TEARDOWN_CALLBACK = extern "system" fn(
    FltObjects: PFLT_RELATED_OBJECTS,
    Reason: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
) -> NTSTATUS;

pub type PFLT_INSTANCE_TEARDOWN_CALLBACK = extern "system" fn(
    FltObjects: PFLT_RELATED_OBJECTS,
    Reason: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS;
