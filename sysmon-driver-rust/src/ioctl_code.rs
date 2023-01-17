#![allow(unused)]
use kernel_macros::CTL_CODE;
use km_api_sys::wmd::{FILE_ANY_ACCESS, METHOD_BUFFERED};
use winapi::shared::ntdef::ULONG;

pub(super) const IOCTL_REQUEST: ULONG = CTL_CODE!(0x8000, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS);
