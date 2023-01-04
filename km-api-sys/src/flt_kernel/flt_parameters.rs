use winapi::km::ndis::PMDL;
use winapi::km::wdm::PFILE_OBJECT;
use winapi::shared::ntdef::{BOOLEAN, HANDLE, LARGE_INTEGER, PVOID, ULONG, USHORT};

#[repr(C)]
pub union FLT_PARAMETERS {
    pub Create: ::core::mem::ManuallyDrop<FLT_PARAMETERS_CREATE>,
    //create_pipe: FTL_PARAMETERS_CREATE_PIPE,
    //create_mail_slot: FTL_PARAMETERS_CREATE_MAIL_SLOT,
    pub Read: ::core::mem::ManuallyDrop<FLT_PARAMETERS_READ>,
    //pub Write: ::core::mem::ManuallyDrop<FLT_PARAMETERS_WRITE>,
    //pub QueryFileInformation: ::core::mem::ManuallyDrop<FLT_PARAMETERS_QUERY_FILE_INFORMATION>,
    pub SetFileInformation: ::core::mem::ManuallyDrop<FLT_PARAMETERS_SET_FILE_INFORMATION>,
}

pub type PIO_SECURITY_CONTEXT = PVOID;

#[repr(C)]
pub struct FLT_PARAMETERS_CREATE {
    pub security_context: PIO_SECURITY_CONTEXT,
    pub options: ULONG,
    #[cfg(target_arch = "x86_64")]
    padding_1: u32,
    pub file_attributes: USHORT,
    pub share_access: USHORT,
    #[cfg(target_arch = "x86_64")]
    padding_2: u32,
    pub ea_length: ULONG,
    pub ea_buffer: PVOID,
    pub allocation_size: LARGE_INTEGER,
}

#[repr(C)]
pub struct FLT_PARAMETERS_READ {
    length: ULONG,
    key: ULONG,
    byte_offset: LARGE_INTEGER,
    write_buffer: PVOID,
    mdl_address: PMDL,
}

#[repr(C)]
pub struct FLT_PARAMETERS_SET_FILE_INFORMATION_UNION_STRUCT {
    replace_if_exists: BOOLEAN,
    advance_only: BOOLEAN,
}

#[repr(C)]
pub union FLT_PARAMETERS_SET_FILE_INFORMATION_UNION {
    struct_union: ::core::mem::ManuallyDrop<FLT_PARAMETERS_SET_FILE_INFORMATION_UNION_STRUCT>,
    cluster_count: ULONG,
    delete_handle: HANDLE,
}

#[repr(C)]
pub struct FLT_PARAMETERS_SET_FILE_INFORMATION {
    pub length: ULONG,
    #[cfg(target_arch = "x86_64")]
    padding_1: u32,
    pub file_information_class: FILE_INFORMATION_CLASS,
    pub parent_of_target: PFILE_OBJECT,
    pub union: FLT_PARAMETERS_SET_FILE_INFORMATION_UNION,
    pub info_buffer: PVOID,
}

#[repr(C)]
pub enum FILE_INFORMATION_CLASS {
    FileDirectoryInformation = 1,
    FileFullDirectoryInformation,            // 2
    FileBothDirectoryInformation,            // 3
    FileBasicInformation,                    // 4
    FileStandardInformation,                 // 5
    FileInternalInformation,                 // 6
    FileEaInformation,                       // 7
    FileAccessInformation,                   // 8
    FileNameInformation,                     // 9
    FileRenameInformation,                   // 10
    FileLinkInformation,                     // 11
    FileNamesInformation,                    // 12
    FileDispositionInformation,              // 13
    FilePositionInformation,                 // 14
    FileFullEaInformation,                   // 15
    FileModeInformation,                     // 16
    FileAlignmentInformation,                // 17
    FileAllInformation,                      // 18
    FileAllocationInformation,               // 19
    FileEndOfFileInformation,                // 20
    FileAlternateNameInformation,            // 21
    FileStreamInformation,                   // 22
    FilePipeInformation,                     // 23
    FilePipeLocalInformation,                // 24
    FilePipeRemoteInformation,               // 25
    FileMailslotQueryInformation,            // 26
    FileMailslotSetInformation,              // 27
    FileCompressionInformation,              // 28
    FileObjectIdInformation,                 // 29
    FileCompletionInformation,               // 30
    FileMoveClusterInformation,              // 31
    FileQuotaInformation,                    // 32
    FileReparsePointInformation,             // 33
    FileNetworkOpenInformation,              // 34
    FileAttributeTagInformation,             // 35
    FileTrackingInformation,                 // 36
    FileIdBothDirectoryInformation,          // 37
    FileIdFullDirectoryInformation,          // 38
    FileValidDataLengthInformation,          // 39
    FileShortNameInformation,                // 40
    FileIoCompletionNotificationInformation, // 41
    FileIoStatusBlockRangeInformation,       // 42
    FileIoPriorityHintInformation,           // 43
    FileSfioReserveInformation,              // 44
    FileSfioVolumeInformation,               // 45
    FileHardLinkInformation,                 // 46
    FileProcessIdsUsingFileInformation,      // 47
    FileNormalizedNameInformation,           // 48
    FileNetworkPhysicalNameInformation,      // 49
    FileIdGlobalTxDirectoryInformation,      // 50
    FileIsRemoteDeviceInformation,           // 51
    FileUnusedInformation,                   // 52
    FileNumaNodeInformation,                 // 53
    FileStandardLinkInformation,             // 54
    FileRemoteProtocolInformation,           // 55

    //
    //  These are special versions of these operations (defined earlier)
    //  which can be used by kernel mode drivers only to bypass security
    //  access checks for Rename and HardLink operations.  These operations
    //  are only recognized by the IOManager, a file system should never
    //  receive these.
    //
    FileRenameInformationBypassAccessCheck, // 56
    FileLinkInformationBypassAccessCheck,   // 57

    //
    // End of special information classes reserved for IOManager.
    //
    FileVolumeNameInformation,                    // 58
    FileIdInformation,                            // 59
    FileIdExtdDirectoryInformation,               // 60
    FileReplaceCompletionInformation,             // 61
    FileHardLinkFullIdInformation,                // 62
    FileIdExtdBothDirectoryInformation,           // 63
    FileDispositionInformationEx,                 // 64
    FileRenameInformationEx,                      // 65
    FileRenameInformationExBypassAccessCheck,     // 66
    FileDesiredStorageClassInformation,           // 67
    FileStatInformation,                          // 68
    FileMemoryPartitionInformation,               // 69
    FileStatLxInformation,                        // 70
    FileCaseSensitiveInformation,                 // 71
    FileLinkInformationEx,                        // 72
    FileLinkInformationExBypassAccessCheck,       // 73
    FileStorageReserveIdInformation,              // 74
    FileCaseSensitiveInformationForceAccessCheck, // 75

    FileMaximumInformation,
}
