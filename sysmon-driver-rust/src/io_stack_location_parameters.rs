use winapi::km::wdm::IO_STACK_LOCATION_s1_Parameters;
use winapi::shared::ntdef::{LONGLONG, ULONG};

#[derive(Debug)]
#[repr(C)]
pub struct ParametersRead {
    // offsets for x64
    pub length: ULONG, // 0x00
    #[cfg(target_pointer_width = "64")]
    pub __bindgen_padding_0: u32, //0x04

    pub key: ULONG, // 0x08
    #[cfg(target_pointer_width = "64")]
    pub flags: ULONG, // 0x0C

    pub byte_offset: LONGLONG, // 0x10
} //sizeof 0x18

pub type ParametersWrite = ParametersRead;

pub struct Parameters();

impl Parameters {
    pub fn Read(param: &mut IO_STACK_LOCATION_s1_Parameters) -> *mut ParametersRead {
        param as *mut IO_STACK_LOCATION_s1_Parameters as *mut ParametersRead
    }

    pub fn Write(param: &mut IO_STACK_LOCATION_s1_Parameters) -> *mut ParametersWrite {
        param as *mut IO_STACK_LOCATION_s1_Parameters as *mut ParametersWrite
    }
}
