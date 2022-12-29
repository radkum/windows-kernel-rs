use crate::locker::Locker;
use km_api_sys::wmd::{
    ExAcquireFastMutex, ExInitializeFastMutex, ExReleaseFastMutex, FAST_MUTEX,
};

pub struct FastMutex {
    mutex: FAST_MUTEX,
}

impl FastMutex {
    pub const fn new() -> Self {
        Self {
            mutex: FAST_MUTEX::new(),
        }
    }
}

impl Locker for FastMutex {
    fn init(&mut self) {
        unsafe { ExInitializeFastMutex(&mut self.mutex) }
    }

    fn lock(&mut self) {
        unsafe { ExAcquireFastMutex(&mut self.mutex as *mut FAST_MUTEX) }
    }

    fn unlock(&mut self) {
        unsafe { ExReleaseFastMutex(&mut self.mutex as *mut FAST_MUTEX) }
    }
}
