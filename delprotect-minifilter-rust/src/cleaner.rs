use kernel_string::{PCUNICODE_STRING, UNICODE_STRING};
use km_api_sys::flt_kernel::{FltUnregisterFilter, PFLT_FILTER};
use winapi::km::wdm::{IoDeleteDevice, IoDeleteSymbolicLink, PDEVICE_OBJECT};

pub struct Cleaner {
    device_object: Option<PDEVICE_OBJECT>,
    sym_link: Option<PCUNICODE_STRING>,
    filter_handle: Option<PFLT_FILTER>,
}

impl Cleaner {
    pub fn new() -> Self {
        Self {
            device_object: None,
            sym_link: None,
            filter_handle: None,
        }
    }

    pub fn init_device(&mut self, device: PDEVICE_OBJECT) {
        self.device_object = Some(device);
    }

    pub fn init_symlink(&mut self, sym_link: &UNICODE_STRING) {
        self.sym_link = Some(sym_link as PCUNICODE_STRING);
    }

    pub fn init_filter_handle(&mut self, callback: PFLT_FILTER) {
        self.filter_handle = Some(callback);
    }

    pub fn clean(&mut self) {
        unsafe {
            if let Some(device) = self.device_object {
                IoDeleteDevice(device);
            }

            if let Some(sym_link) = self.sym_link {
                IoDeleteSymbolicLink(&(*sym_link).as_ntdef_unicode());
            }

            if let Some(filter_handle) = self.filter_handle {
                FltUnregisterFilter(filter_handle);
            }
        }
    }
}
