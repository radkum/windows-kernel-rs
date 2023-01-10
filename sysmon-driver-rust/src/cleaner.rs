use crate::{PsSetCreateProcessNotifyRoutineEx, UNICODE_STRING};
use km_api_sys::{
    ntddk::{
        PsRemoveCreateThreadNotifyRoutine, PsRemoveLoadImageNotifyRoutine,
        PCREATE_PROCESS_NOTIFY_ROUTINE_EX, PCREATE_THREAD_NOTIFY_ROUTINE,
        PLOAD_IMAGE_NOTIFY_ROUTINE,
    },
    wmd::{CmUnRegisterCallback, LARGE_INTEGER},
};
use winapi::{
    km::wdm::{IoDeleteDevice, IoDeleteSymbolicLink, PDEVICE_OBJECT},
    shared::ntdef::TRUE,
};

pub struct Cleaner {
    device_object: Option<PDEVICE_OBJECT>,
    sym_link: Option<*const UNICODE_STRING>,
    create_process_callback: Option<PCREATE_PROCESS_NOTIFY_ROUTINE_EX>,
    thread_process_callback: Option<PCREATE_THREAD_NOTIFY_ROUTINE>,
    load_image_callback: Option<PLOAD_IMAGE_NOTIFY_ROUTINE>,
    registry_callback: Option<LARGE_INTEGER>,
}

impl Cleaner {
    pub fn new() -> Self {
        Self {
            device_object: None,
            sym_link: None,
            create_process_callback: None,
            thread_process_callback: None,
            load_image_callback: None,
            registry_callback: None,
        }
    }

    pub fn init_device(&mut self, device: PDEVICE_OBJECT) {
        self.device_object = Some(device);
    }

    pub fn init_symlink(&mut self, sym_link: &UNICODE_STRING) {
        self.sym_link = Some(sym_link as *const UNICODE_STRING);
    }

    pub fn init_process_create_callback(&mut self, callback: PCREATE_PROCESS_NOTIFY_ROUTINE_EX) {
        self.create_process_callback = Some(callback);
    }

    pub fn init_thread_create_callback(&mut self, callback: PCREATE_THREAD_NOTIFY_ROUTINE) {
        self.thread_process_callback = Some(callback);
    }

    pub fn init_image_load_callback(&mut self, callback: PLOAD_IMAGE_NOTIFY_ROUTINE) {
        self.load_image_callback = Some(callback);
    }

    pub fn init_registry_callback(&mut self, cookie: LARGE_INTEGER) {
        self.registry_callback = Some(cookie);
    }

    pub fn clean(&mut self) {
        unsafe {
            if let Some(device) = self.device_object {
                IoDeleteDevice(device);
            }

            if let Some(sym_link) = self.sym_link {
                IoDeleteSymbolicLink(&(*sym_link).as_ntdef_unicode());
            }

            if let Some(routine) = self.create_process_callback {
                PsSetCreateProcessNotifyRoutineEx(routine, TRUE);
            }

            if let Some(routine) = self.thread_process_callback {
                PsRemoveCreateThreadNotifyRoutine(routine);
            }

            if let Some(routine) = self.load_image_callback {
                PsRemoveLoadImageNotifyRoutine(routine);
            }

            if let Some(cookie) = self.registry_callback {
                CmUnRegisterCallback(cookie);
            }
        }
    }
}
