#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod flt_parameters;
pub use flt_parameters::*;

use core::ptr::null_mut;
use winapi::{
    km::wdm::{
        DEVICE_TYPE, IO_STATUS_BLOCK, KPROCESSOR_MODE, PDRIVER_OBJECT, PETHREAD, PFILE_OBJECT,
    },
    shared::ntdef::{
        LIST_ENTRY, LONG, NTSTATUS, PBOOLEAN, POBJECT_ATTRIBUTES, PULONG, PVOID, UCHAR, ULONG,
        USHORT,
    },
    um::winnt::{ACCESS_MASK, PSECURITY_DESCRIPTOR},
};

use kernel_string::UNICODE_STRING;
use kernel_string::PUNICODE_STRING;

pub type CONST_PVOID = *const winapi::ctypes::c_void;
pub type PPSECURITY_DESCRIPTOR = *mut PSECURITY_DESCRIPTOR;
pub type PPFLT_PORT = *mut PFLT_PORT;
pub type PPVOID = *mut PVOID;

#[link(name = "fltMgr")]
extern "system" {
    pub fn FltRegisterFilter(
        Driver: PDRIVER_OBJECT,
        Registration: &FLT_REGISTRATION,
        RetFilter: &mut PFLT_FILTER,
    ) -> NTSTATUS;

    pub fn FltUnregisterFilter(Filter: PFLT_FILTER) -> NTSTATUS;

    pub fn FltStartFiltering(Filter: PFLT_FILTER) -> NTSTATUS;

    pub fn FltCreateCommunicationPort(
        Filter: PFLT_FILTER,
        ServerPort: &mut PFLT_PORT,
        ObjectAttributes: POBJECT_ATTRIBUTES,
        ServerPortCookie: CONST_PVOID,
        ConnectNotifyCallback: Option<PFLT_CONNECT_NOTIFY>,
        DisconnectNotifyCallback: Option<PFLT_DISCONNECT_NOTIFY>,
        MessageNotifyCallback: Option<PFLT_MESSAGE_NOTIFY>,
        MaxConnections: LONG,
    ) -> NTSTATUS;

    pub fn FltCloseCommunicationPort(ServerPort: PFLT_PORT) -> NTSTATUS;

    pub fn FltCloseClientPort(Filter: PFLT_FILTER, ClientPort: &PFLT_PORT);

    pub fn FltSendMessage(
        Filter: PFLT_FILTER,
        ClientPort: &PFLT_PORT,
        SenderBuffer: CONST_PVOID,
        SenderBufferLength: ULONG,
        ReplyBuffer: PVOID,
        ReplyLength: PULONG,
        Timeout: PVOID,
    ) -> NTSTATUS;

    pub fn FltBuildDefaultSecurityDescriptor(
        SecurityDescriptor: &mut PSECURITY_DESCRIPTOR,
        DesiredAccess: ACCESS_MASK,
    ) -> NTSTATUS;

    pub fn FltFreeSecurityDescriptor(SecurityDescriptor: PSECURITY_DESCRIPTOR);

    pub fn FltIsDirectory(
        Filter: PFLT_FILTER,
        Instance: PFLT_INSTANCE,
        IsDirectory: PBOOLEAN,
    ) -> NTSTATUS;

    pub fn FltGetFileNameInformation(
        CallbackData: PFLT_CALLBACK_DATA,
        NameOptions: FLT_FILE_NAME_OPTIONS,
        FileNameInformation: &mut PFLT_FILE_NAME_INFORMATION,
    ) -> NTSTATUS;

    pub fn FltParseFileNameInformation(
        FileNameInformation: PFLT_FILE_NAME_INFORMATION,
    )-> NTSTATUS;

    pub fn FltReleaseFileNameInformation(
        FileNameInformation: PFLT_FILE_NAME_INFORMATION,
    );

    pub fn FltGetVolumeFromFileObject(
        Filter: PFLT_FILTER,
        FileObject: PFILE_OBJECT,
        RetVolume: &mut PFLT_VOLUME,
    )-> NTSTATUS;

    pub fn FltGetVolumeGuidName(
        Volume: PFLT_VOLUME,
        VolumeGuidName: PUNICODE_STRING,
        BufferSizeNeeded: &mut ULONG,
    )-> NTSTATUS;
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

pub type PFLT_PORT = PVOID; //*mut FLT_PORT;

#[repr(C)]
pub struct FILTER_MESSAGE_HEADER {
    pub ReplyLength: u32,
    pub MessageId: u64,
}

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
    FltObjects: &mut FLT_RELATED_OBJECTS,
    CompletionContext: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS;

pub type PFLT_POST_OPERATION_CALLBACK = extern "system" fn(
    Data: &mut FLT_CALLBACK_DATA,
    FltObjects: &mut FLT_RELATED_OBJECTS,
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

#[repr(C)]
pub struct FLT_RELATED_OBJECTS {
    pub Size: USHORT,
    pub TransactionContext: USHORT,
    pub Filter: PFLT_FILTER,
    pub Volume: PFLT_VOLUME,
    pub Instance: PFLT_INSTANCE,
    pub FileObject: PFILE_OBJECT,
    pub Transaction: PVOID, /*PKTRANSACTION*/
}
pub type PFLT_RELATED_OBJECTS = *mut FLT_RELATED_OBJECTS;

const _SIZE_CHECKER3: [u8; 48] = [0; ::core::mem::size_of::<FLT_RELATED_OBJECTS>()];

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

pub type PFLT_CONNECT_NOTIFY = unsafe extern "system" fn(
    ClientPort: PFLT_PORT,
    ServerPortCookie: PVOID,
    ConnectionContext: PVOID,
    SizeOfContext: u32,
    ConnectionPortCookie: PPVOID,
) -> NTSTATUS;

pub type PFLT_DISCONNECT_NOTIFY = unsafe extern "system" fn(ConnectionCookie: PVOID);

pub type PFLT_MESSAGE_NOTIFY = unsafe extern "system" fn(
    PortCookie: PVOID,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    ReturnOutputBufferLength: PULONG,
) -> NTSTATUS;

#[repr(C)]
pub struct FLT_FILE_NAME_INFORMATION {
    pub Size: USHORT,
    NamesParsed: FLT_FILE_NAME_PARSED_FLAGS,
    Format: FLT_FILE_NAME_OPTIONS,
    pub Name: UNICODE_STRING,
    pub Volume: UNICODE_STRING,
    Share: UNICODE_STRING,
    pub Extension: UNICODE_STRING,
    Stream: UNICODE_STRING,
    FinalComponent: UNICODE_STRING,
    pub ParentDir: UNICODE_STRING,
}

pub type PFLT_FILE_NAME_INFORMATION = *mut FLT_FILE_NAME_INFORMATION;

#[repr(C)]
pub struct FLT_FILE_NAME_OPTIONS(pub ULONG);

impl FLT_FILE_NAME_OPTIONS {
    pub const FLT_VALID_FILE_NAME_FORMATS: ULONG = 0x000000ff;
    pub const FLT_FILE_NAME_NORMALIZED: ULONG = 0x00000001;
    pub const FLT_FILE_NAME_OPENED: ULONG = 0x00000002;
    pub const FLT_FILE_NAME_SHORT: ULONG = 0x00000003;
    pub const FLT_VALID_FILE_NAME_QUERY_METHODS: ULONG = 0x0000ff00;
    pub const FLT_FILE_NAME_QUERY_DEFAULT: ULONG = 0x00000100;
    pub const FLT_FILE_NAME_QUERY_CACHE_ONLY: ULONG = 0x00000200;
    pub const FLT_FILE_NAME_QUERY_FILESYSTEM_ONLY: ULONG = 0x00000300;
    pub const FLT_FILE_NAME_QUERY_ALWAYS_ALLOW_CACHE_LOOKUP: ULONG = 0x00000400;
    pub const FLT_VALID_FILE_NAME_FLAGS: ULONG = 0xff000000;
    pub const FLT_FILE_NAME_REQUEST_FROM_CURRENT_PROVIDER: ULONG = 0x01000000;
    pub const FLT_FILE_NAME_DO_NOT_CACHE: ULONG = 0x02000000;
    pub const FLT_FILE_NAME_ALLOW_QUERY_ON_REPARSE: ULONG = 0x04000000;
}

#[repr(C)]
pub struct FLT_FILE_NAME_PARSED_FLAGS(pub USHORT);

impl FLT_FILE_NAME_PARSED_FLAGS {
    pub const FINAL_COMPONENT: USHORT = 0x0001;
    pub const EXTENSION: USHORT = 0x0002;
    pub const STREAM: USHORT = 0x0004;
    pub const PARENT_DIR: USHORT = 0x0008;
}

pub type PFLT_VOLUME = PVOID;