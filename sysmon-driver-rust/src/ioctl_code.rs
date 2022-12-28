#![allow(unused)]

use kernel_macros::CTL_CODE;
use winapi::shared::ntdef::ULONG;

const IOCTL_REQUEST: ULONG = CTL_CODE!(0x8000, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS);

pub const FILE_ANY_ACCESS: u32 = 0u32;
pub const METHOD_BUFFERED: u32 = 0u32;

pub const MDL_SOURCE_IS_NONPAGED_POOL: u16 = 0x0004;
pub const MDL_MAPPED_TO_SYSTEM_VA: u16 = 0x0001;
