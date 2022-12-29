#![no_std]

extern crate alloc;

use alloc::string::String;
use core::fmt::{Display, Formatter};
use core::slice;
use winapi::shared::ntdef::{BOOLEAN, NTSTATUS, UNICODE_STRING};

#[repr(C)]
pub struct AnsiString {
    pub len: u16,
    pub max_len: u16,
    pub ptr: *const u8,
}

impl AnsiString {
    fn create(buffer: &[u8]) -> Self {
        AnsiString::from(buffer)
    }
}

impl<'a> From<&'a [u8]> for AnsiString {
    fn from(buffer: &'a [u8]) -> Self {
        let mut str = AnsiString::default();

        let mut buffer = buffer.to_vec();
        if *buffer.last().unwrap() != 0 {
            //let mut buffer = buffer.to_vec();
            buffer.push(0);
        }

        unsafe {
            RtlInitAnsiString(&mut str, buffer.as_ptr());
        }

        ::core::mem::forget(buffer);
        str
    }
}

impl<'a> From<&str> for AnsiString {
    fn from(buffer: &str) -> Self {
        AnsiString::from(buffer.as_bytes())
    }
}

impl Default for AnsiString {
    fn default() -> Self {
        Self {
            len: 0,
            max_len: 0 as u16,
            ptr: ::core::ptr::null(),
        }
    }
}

#[repr(C)]
pub struct UnicodeString {
    pub len: u16,
    pub max_len: u16,
    pub ptr: *const u16,
}
pub type PUnicodeString = *mut UnicodeString;

impl UnicodeString {
    pub fn create(buffer: &str) -> Self {
        UnicodeString::from(buffer.as_bytes())
    }

    pub fn as_unicode_string(&self) -> UNICODE_STRING {
        UNICODE_STRING {
            Length: self.len,
            MaximumLength: self.max_len,
            Buffer: self.ptr as *mut u16,
        }
    }

    pub fn as_rust_string(&self) -> String {
        unsafe {
            let ar = slice::from_raw_parts(self.ptr, self.len as usize / 2);
            if let Ok(s) = String::from_utf16(ar) {
                s
            } else {
                String::new()
            }
        }
    }

    pub fn as_ptr(&self) -> *const UNICODE_STRING {
        self as *const Self as *const UNICODE_STRING
    }
}

impl From<UNICODE_STRING> for UnicodeString {
    fn from(unicode: UNICODE_STRING) -> Self {
        UnicodeString {
            len: unicode.Length,
            max_len: unicode.MaximumLength,
            ptr: unicode.Buffer,
        }
    }
}

impl<'a> From<&'a [u8]> for UnicodeString {
    fn from(buffer: &'a [u8]) -> Self {
        UnicodeString::from(&AnsiString::create(buffer))
    }
}

impl<'a> From<&str> for UnicodeString {
    fn from(buffer: &str) -> Self {
        UnicodeString::from(buffer.as_bytes())
    }
}

impl<'a> From<&'a [u16]> for UnicodeString {
    fn from(buffer: &'a [u16]) -> Self {
        let mut str = UnicodeString::default();

        let mut buffer = buffer.to_vec();
        if *buffer.last().unwrap() == 0 {
            buffer.push(0);
        }

        unsafe {
            RtlCreateUnicodeString(&mut str, buffer.as_ptr());
        }
        str
    }
}

impl<'a> From<&AnsiString> for UnicodeString {
    fn from(source: &AnsiString) -> Self {
        let mut u = UnicodeString::default();
        unsafe {
            RtlAnsiStringToUnicodeString(&mut u, source, true);
        }
        u
    }
}

impl Display for UnicodeString {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_rust_string())
    }
}

impl Default for UnicodeString {
    fn default() -> Self {
        Self {
            len: 0,
            max_len: 0 as u16,
            ptr: ::core::ptr::null(),
        }
    }
}

impl Drop for UnicodeString {
    fn drop(&mut self) {
        unsafe { RtlFreeUnicodeString(self) }
    }
}

extern "system" {
    pub fn RtlInitAnsiString(dest: &mut AnsiString, str: *const u8);

    pub fn RtlCreateUnicodeString(dest: &mut UnicodeString, src: *const u16) -> BOOLEAN;

    pub fn RtlFreeUnicodeString(src: &mut UnicodeString);

    pub fn RtlIntegerToUnicodeString(Value: u32, Base: u32, String: &mut UnicodeString)
                                     -> NTSTATUS;
    // pub fn RtlInt64ToUnicodeString(Value: u64, Base: u32, String: &mut
    // UNICODE_STRING) -> NTSTATUS; pub fn RtlUnicodeStringToInteger(String:
    // &CONST_UNICODE_STRING, Base: u32, Value: &mut u32) -> NTSTATUS;
    //
    // pub fn RtlUnicodeStringToAnsiString(DestinationString: &mut ANSI_STRING,
    // SourceString: &CONST_UNICODE_STRING, AllocateDestination: bool) ->
    // NTSTATUS; pub fn RtlUnicodeStringToAnsiSize(SourceString:
    // &CONST_UNICODE_STRING) -> u32;
    //
    pub fn RtlAnsiStringToUnicodeString(
        DestinationString: &mut UnicodeString,
        SourceString: &AnsiString,
        AllocateDestination: bool,
    ) -> NTSTATUS;
    // pub fn RtlAnsiStringToUnicodeSize(SourceString: &CONST_ANSI_STRING) ->
    // u32;
    //
    // pub fn RtlCompareUnicodeString (String1: &CONST_UNICODE_STRING, String2:
    // &CONST_UNICODE_STRING, CaseInSensitive: bool) -> i32;
    // pub fn RtlCompareString (String1: &CONST_ANSI_STRING, String2:
    // &CONST_ANSI_STRING, CaseInSensitive: bool) -> i32;
    //
    // pub fn RtlEqualUnicodeString(String1: &CONST_UNICODE_STRING, String2:
    // &CONST_UNICODE_STRING) -> bool; pub fn RtlEqualString(String1:
    // &CONST_ANSI_STRING, String2: &CONST_ANSI_STRING) -> bool;
    //
    // pub fn RtlFreeAnsiString(UnicodeString: &mut ANSI_STRING);
    // pub fn RtlFreeUnicodeString(UnicodeString: &mut UNICODE_STRING);
}
