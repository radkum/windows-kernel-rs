use crate::UnicodeString;
use winapi::km::wdm::{IoDeleteDevice, IoDeleteSymbolicLink, PDEVICE_OBJECT};

pub struct Cleaner {
    device_object: Option<PDEVICE_OBJECT>,
    sym_link: Option<*const UnicodeString>,
}

impl Cleaner {
    pub fn new() -> Self {
        Self {
            device_object: None,
            sym_link: None,
        }
    }

    pub fn init_device(&mut self, device: PDEVICE_OBJECT) {
        self.device_object = Some(device);
    }

    pub fn init_symlink(&mut self, sym_link: &UnicodeString) {
        self.sym_link = Some(sym_link as *const UnicodeString);
    }

    pub fn clean(&mut self) {
        unsafe {
            if let Some(device) = self.device_object {
                IoDeleteDevice(device);
            }

            if let Some(sym_link) = self.sym_link {
                IoDeleteSymbolicLink(&(*sym_link).native());
            }
        }
    }
}
