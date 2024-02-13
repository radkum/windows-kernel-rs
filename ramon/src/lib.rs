#![no_std]
#![allow(non_snake_case)]

mod event;
mod file_monitor;
mod ioctl_code;
mod irp;
mod minifilter;

extern crate alloc;

use alloc::collections::{BTreeSet, VecDeque};
use kernel_fast_mutex::fast_mutex::FastMutex;
use kernel_fast_mutex::locker::Locker;
/// kernel-init deliver a few elements (eg. panic implementation) necessary to run code in kernel
#[allow(unused_imports)]
use kernel_init;
use kernel_log::KernelLogger;
use kernel_macros::NT_SUCCESS;

use kernel_string::UNICODE_STRING;
use log::LevelFilter;
use winapi::{
    km::wdm::DRIVER_OBJECT,
    shared::{ntdef::NTSTATUS, ntstatus::STATUS_SUCCESS},
};
use winapi::shared::ntdef::ULONG;
use crate::{irp::complete_irp_success, minifilter::Minifilter};
use winapi::km::wdm::{DEVICE_OBJECT, IRP, IRP_MJ};
use winapi::shared::ntstatus::STATUS_INSUFFICIENT_RESOURCES;

pub(crate) const POOL_TAG: u32 = u32::from_ne_bytes(*b"RDER");
const MAX_ITEM_COUNT: usize = 32;

static mut G_MUTEX: FastMutex = FastMutex::new();
static mut G_PROCESS_NAMES: Option<BTreeSet<ULONG>> = None;

#[link_section = "INIT"]
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT,
    _path: *const UNICODE_STRING,
) -> NTSTATUS {
    KernelLogger::init(LevelFilter::Info).expect("Failed to initialize logger");

    log::info!("START Ramon");

    let hello_world = UNICODE_STRING::create("Hello World!");
    log::info!("{}", hello_world.as_rust_string());

    //--------------------GLOBALS--------------------------------
    G_MUTEX.Init();
    let mut processes: VecDeque<ULONG> = VecDeque::new();
    if let Err(e) = processes.try_reserve_exact(MAX_ITEM_COUNT) {
        log::info!(
            "fail to reserve a {} bytes of memory. Err: {:?}",
            ::core::mem::size_of::<ULONG>() * MAX_ITEM_COUNT,
            e
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let processes = processes.into_iter().collect();
    G_PROCESS_NAMES = Some(processes);

    //--------------------DISPATCH_ROUTINES-----------------------
    driver.MajorFunction[IRP_MJ::CREATE as usize] = Some(DispatchCreateClose);
    driver.MajorFunction[IRP_MJ::CLOSE as usize] = Some(DispatchCreateClose);
    driver.DriverUnload = Some(RamonUnloadDriver);

    //--------------------INIT MINIFILTER-----------------------
    #[allow(unused_assignments)]
    let mut status = STATUS_SUCCESS;

    status = Minifilter::factory(driver);

    if NT_SUCCESS!(status) {
        log::info!("SUCCESS");
    } else {
        //clean
    }

    log::info!("SUCCESS: {}", status);
    status
}

/*************************************************************************
                    Dispatch  routines.
*************************************************************************/
extern "system" fn RamonUnloadDriver(_driver: &mut DRIVER_OBJECT) {
    log::info!("rust_unload");
}

extern "system" fn DispatchCreateClose(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    complete_irp_success(irp)
}
