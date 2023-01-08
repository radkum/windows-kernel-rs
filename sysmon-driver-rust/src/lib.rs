#![no_std]
#![allow(non_snake_case)]
extern crate alloc;

mod cleaner;
mod ioctl_code;
mod item;

/// kernel-init deliver a few elements (eg. panic implementation) necessary to run code in kernel
use kernel_init;

use alloc::collections::VecDeque;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::mem::forget;

use winapi::km::wdm::{IoCompleteRequest, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoGetCurrentIrpStackLocation, DEVICE_OBJECT, DEVICE_TYPE, DRIVER_OBJECT, IRP, IRP_MJ, PDEVICE_OBJECT, PEPROCESS, IO_STACK_LOCATION, _IO_STACK_LOCATION_READ};
use winapi::shared::ntdef::{BOOLEAN, FALSE, LONGLONG, NTSTATUS, TRUE, UNICODE_STRING};

use winapi::shared::ntstatus::{
    STATUS_INSUFFICIENT_RESOURCES, STATUS_INVALID_BUFFER_SIZE, STATUS_INVALID_DEVICE_REQUEST,
    STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};

use kernel_fast_mutex::auto_lock::AutoLock;
use kernel_fast_mutex::fast_mutex::FastMutex;
use kernel_fast_mutex::locker::Locker;

use core::ptr::null_mut;
use kernel_macros::{HandleToU32, NT_SUCCESS};
use kernel_print::kernel_print;
use kernel_string::{PUnicodeString, UnicodeString};

use km_api_sys::ntddk::{
    PsGetCurrentProcessId, PsGetCurrentThreadId, PsRemoveCreateThreadNotifyRoutine,
    PsRemoveLoadImageNotifyRoutine, PsSetCreateProcessNotifyRoutineEx,
    PsSetCreateThreadNotifyRoutine, PsSetLoadImageNotifyRoutine, HANDLE, PIMAGE_INFO,
    PPS_CREATE_NOTIFY_INFO, PS_CREATE_NOTIFY_INFO, PVOID, REG_NT_POST_SET_VALUE_KEY,
};
use km_api_sys::wmd::{
    CmCallbackGetKeyObjectIDEx, CmCallbackReleaseKeyObjectIDEx, CmRegisterCallbackEx,
    CmUnRegisterCallback, MmGetSystemAddressForMdlSafe, MDL, PREG_POST_OPERATION_INFORMATION,
    PREG_SET_VALUE_KEY_INFORMATION,
};
use winapi::km::wdm::DEVICE_FLAGS::DO_DIRECT_IO;

use crate::cleaner::Cleaner;
use crate::item::ItemInfo;

use crate::ItemInfo::{
    ImageLoad, ProcessCreate, ProcessExit, RegistrySetValue, ThreadCreate, ThreadExit,
};

const DEVICE_NAME: &str = "\\Device\\Zero";
const SYM_LINK_NAME: &str = "\\??\\Zero";

const MAX_ITEM_COUNT: usize = 256;

static mut G_EVENTS: Option<VecDeque<ItemInfo>> = None;
static mut G_MUTEX: FastMutex = FastMutex::new();
static mut G_COOKIE: LONGLONG = 0;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT,
    _path: *const UNICODE_STRING,
) -> NTSTATUS {
    kernel_print::kernel_println!("START");

    G_MUTEX.Init();

    let events = VecDeque::new();

    //todo try reserve
    //events.try_reserve(MAX_ITEM_COUNT);
    G_EVENTS = Some(events);

    driver.DriverUnload = Some(DriverUnload);

    driver.MajorFunction[IRP_MJ::CREATE as usize] = Some(DispatchCreateClose);
    driver.MajorFunction[IRP_MJ::CLOSE as usize] = Some(DispatchCreateClose);
    driver.MajorFunction[IRP_MJ::DEVICE_CONTROL as usize] = Some(DispatchDeviceControl);
    driver.MajorFunction[IRP_MJ::READ as usize] = Some(DispatchRead);
    driver.MajorFunction[IRP_MJ::WRITE as usize] = Some(DispatchWrite);

    #[allow(unused_assignments)]
    let mut status = STATUS_SUCCESS;

    let hello_world = UnicodeString::create("Hello World!");
    kernel_print::kernel_println!("{}", hello_world.as_rust_string());

    let dev_name = UnicodeString::from(DEVICE_NAME);
    let sym_link = UnicodeString::from(SYM_LINK_NAME);

    let mut cleaner = Cleaner::new();
    let mut device_object: PDEVICE_OBJECT = null_mut();

    loop {
        //--------------------DEVICE-----------------------
        status = IoCreateDevice(
            driver,
            0,
            dev_name.as_ptr(),
            DEVICE_TYPE::FILE_DEVICE_UNKNOWN,
            0,
            FALSE,
            &mut device_object,
        );

        if NT_SUCCESS!(status) {
            cleaner.init_device(device_object);
        } else {
            kernel_print::kernel_println!("failed to create device 0x{:08x}", status);
            break;
        }

        (*device_object).Flags |= DO_DIRECT_IO as u32;

        //--------------------SYMLINK-----------------------
        status = IoCreateSymbolicLink(&sym_link.as_unicode_string(), &dev_name.as_unicode_string());

        if NT_SUCCESS!(status) {
            cleaner.init_symlink(&sym_link);
        } else {
            kernel_print::kernel_println!("failed to create sym_link 0x{:08x}", status);
            break;
        }

        //--------------------PROCESS NOTIFY-----------------------
        status = PsSetCreateProcessNotifyRoutineEx(OnProcessNotify, FALSE);

        if NT_SUCCESS!(status) {
            cleaner.init_process_create_callback(OnProcessNotify);
        } else {
            kernel_print::kernel_println!(
                "failed to create process nofity rountine 0x{:08x}",
                status
            );
            break;
        }

        //--------------------THREAD NOTIFY-----------------------
        status = PsSetCreateThreadNotifyRoutine(OnThreadNotify);

        if NT_SUCCESS!(status) {
            cleaner.init_thread_create_callback(OnThreadNotify);
        } else {
            kernel_print::kernel_println!(
                "failed to create thread nofity rountine 0x{:08x}",
                status
            );
            break;
        }

        //--------------------IMAGE NOTIFY-----------------------
        status = PsSetLoadImageNotifyRoutine(OnImageLoadNotify);

        if NT_SUCCESS!(status) {
            cleaner.init_image_load_callback(OnImageLoadNotify);
        } else {
            kernel_print::kernel_println!("failed to create image load routine 0x{:08x}", status);
            break;
        }

        //--------------------REGISTRY NOTIFY-----------------------
        let altitude = UnicodeString::create("7657.124");
        status = CmRegisterCallbackEx(
            OnRegistryNotify as PVOID,
            &altitude,
            driver,
            null_mut(),
            &G_COOKIE,
            null_mut(),
        );

        if NT_SUCCESS!(status) {
            cleaner.init_registry_callback(G_COOKIE);
        } else {
            kernel_print::kernel_println!("failed to create registry routine 0x{:08x}", status);
            break;
        }
        break;
    }

    if NT_SUCCESS!(status) {
        kernel_print::kernel_println!("SUCCESS");
    } else {
        cleaner.clean();
    }

    status
}

extern "system" fn DriverUnload(driver: &mut DRIVER_OBJECT) {
    kernel_print::kernel_println!("rust_unload");
    unsafe {
        IoDeleteDevice(driver.DeviceObject);

        let sym_link = UnicodeString::create(SYM_LINK_NAME);
        IoDeleteSymbolicLink(&sym_link.as_unicode_string());

        PsSetCreateProcessNotifyRoutineEx(OnProcessNotify, TRUE);

        PsRemoveCreateThreadNotifyRoutine(OnThreadNotify);

        PsRemoveLoadImageNotifyRoutine(OnImageLoadNotify);

        CmUnRegisterCallback(G_COOKIE);
    }
}

extern "system" fn DispatchCreateClose(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    complete_irp_success(irp)
}

extern "system" fn DispatchDeviceControl(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    unsafe {
        let stack = IoGetCurrentIrpStackLocation(irp);
        let device_io = (*stack).Parameters.DeviceIoControl();

        match device_io.IoControlCode {
            IOCTL_REQUEST => kernel_print::kernel_println!("device control success"),
            _ => {
                return complete_irp_with_status(irp, STATUS_INVALID_DEVICE_REQUEST);
            },
        }
    }

    complete_irp_success(irp)
}

extern "system" fn DispatchRead(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    kernel_print::kernel_println!("DispatchRead begin");

    unsafe {
        let stack = IoGetCurrentIrpStackLocation(irp);
        let parameters_read = (*stack).ParametersRead();
        let len = parameters_read.Length;

        kernel_print::kernel_println!("read len: {}", len);

        if len == 0 {
            kernel_print::kernel_println!("len is zero");
            return complete_irp_with_status(irp, STATUS_INVALID_BUFFER_SIZE);
        }

        let buffer = MmGetSystemAddressForMdlSafe(
            irp.MdlAddress as *mut MDL,
            16, /*NormalPagePriority*/
        );
        if buffer.is_null() {
            kernel_print::kernel_println!("buffer is null");
            return complete_irp_with_status(irp, STATUS_INSUFFICIENT_RESOURCES);
        }

        let buffer = buffer as *mut u8;
        for i in 0..len {
            *buffer.offset(i as isize) = 5;
        }
        kernel_print::kernel_println!("DispatchRead success");

        print_items();

        complete_irp(irp, STATUS_SUCCESS, len as usize)
    }
}

#[allow(non_camel_case_types)]
pub type _IO_STACK_LOCATION_WRITE = _IO_STACK_LOCATION_READ;
pub fn ParametersWrite(stack_loc: &mut IO_STACK_LOCATION) -> &mut _IO_STACK_LOCATION_WRITE {
    stack_loc.ParametersRead()
}

extern "system" fn DispatchWrite(_driver: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    kernel_print::kernel_println!("DispatchWrite begin");

    unsafe {
        let stack = IoGetCurrentIrpStackLocation(irp);
        let stack = &mut *stack;
        let parameters_write = ParametersWrite(stack);

        let len = parameters_write.Length;

        complete_irp(irp, STATUS_SUCCESS, len as usize)
    }
}
fn complete_irp(irp: &mut IRP, status: NTSTATUS, info: usize) -> NTSTATUS {
    unsafe {
        let s = irp.IoStatus.__bindgen_anon_1.Status_mut();
        *s = status;
        irp.IoStatus.Information = info;
        IoCompleteRequest(irp, 0);
    }

    status
}

extern "system" fn OnProcessNotify(
    _process: PEPROCESS,
    process_id: HANDLE,
    create_info: PPS_CREATE_NOTIFY_INFO,
) {
    unsafe {
        kernel_print!("process create");

        let item = if !create_info.is_null() {
            let create_info: &PS_CREATE_NOTIFY_INFO = &*create_info;
            let create_info: &PS_CREATE_NOTIFY_INFO = &*create_info;

            let image_file_name = &*create_info.image_file_name;
            ProcessCreate {
                pid: HandleToU32!(process_id),
                parent_pid: HandleToU32!(create_info.parent_process_id),
                command_line: image_file_name.as_rust_string(),
            }
        } else {
            ProcessExit {
                pid: HandleToU32!(process_id),
            }
        };

        push_item_thread_safe(item);
    }
}

extern "system" fn OnThreadNotify(process_id: HANDLE, thread_id: HANDLE, create: BOOLEAN) {
    unsafe {
        kernel_print!("thread create");

        let item = if create == TRUE {
            ThreadCreate {
                pid: HandleToU32!(process_id),
                tid: HandleToU32!(thread_id),
            }
        } else {
            ThreadExit {
                pid: HandleToU32!(process_id),
                tid: HandleToU32!(thread_id),
            }
        };

        push_item_thread_safe(item);
    }
}

extern "system" fn OnImageLoadNotify(
    full_image_name: PUnicodeString,
    process_id: HANDLE,
    image_info: PIMAGE_INFO,
) {
    if process_id.is_null() {
        // system image, ignore
        return;
    }

    unsafe {
        kernel_print!("image load");

        let image_name = if full_image_name.is_null() {
            "(unknown)".to_string()
        } else {
            (*full_image_name).as_rust_string()
        };

        let image_info = &*image_info;
        let item = ImageLoad {
            pid: HandleToU32!(process_id),
            load_address: image_info.image_base,
            image_size: image_info.image_size,
            image_file_name: image_name,
        };

        push_item_thread_safe(item);
    }
}

extern "system" fn OnRegistryNotify(_context: PVOID, arg1: PVOID, arg2: PVOID) -> NTSTATUS {
    let reg_notify = HandleToU32!(arg1);
    if reg_notify == REG_NT_POST_SET_VALUE_KEY {
        kernel_print!("RegNtPostSetValueKey");
        unsafe {
            let op_info = &*(arg2 as PREG_POST_OPERATION_INFORMATION);
            if !NT_SUCCESS!(op_info.status) {
                return STATUS_SUCCESS;
            }

            let mut name: PUnicodeString = null_mut();
            let status =
                CmCallbackGetKeyObjectIDEx(&G_COOKIE, op_info.object, null_mut(), &mut name, 0);
            if !NT_SUCCESS!(status) {
                return STATUS_SUCCESS;
            }

            if name.is_null() {
                //something wrong
                return STATUS_UNSUCCESSFUL;
            }

            loop {
                let key_name = (*name).as_rust_string();
                let registry_machine = "\\REGISTRY\\MACHINE";

                // filter out none-HKLM writes
                if key_name.contains(registry_machine) {
                    if op_info.pre_information.is_null() {
                        //something wrong
                        break;
                    }

                    let pre_info = &*(op_info.pre_information as PREG_SET_VALUE_KEY_INFORMATION);
                    let value_name = (*pre_info.value_name).as_rust_string();
                    let v = Vec::from_raw_parts(
                        pre_info.data as *mut u8,
                        pre_info.data_size as usize,
                        pre_info.data_size as usize,
                    );

                    let item = RegistrySetValue {
                        pid: HandleToU32!(PsGetCurrentProcessId()),
                        tid: HandleToU32!(PsGetCurrentThreadId()),
                        key_name,
                        value_name,
                        data_type: pre_info.data_type,
                        data: v.clone(),
                    };

                    forget(v);
                    push_item_thread_safe(item);
                }
                break;
            }
            CmCallbackReleaseKeyObjectIDEx(name);
        }
    }

    STATUS_SUCCESS
}

unsafe fn push_item_thread_safe(item: ItemInfo) {
    let _locker = AutoLock::new(&mut G_MUTEX);
    if let Some(events) = &mut G_EVENTS {
        if events.len() >= MAX_ITEM_COUNT {
            events.pop_front();
        }
        events.push_back(item);
    }
}

unsafe fn print_items() {
    let _locker = AutoLock::new(&mut G_MUTEX);
    if let Some(events) = &mut G_EVENTS {
        for elem in events {
            kernel_print!("{:?}", elem);
        }
    }
}

fn complete_irp_with_status(irp: &mut IRP, status: NTSTATUS) -> NTSTATUS {
    complete_irp(irp, status, 0)
}

fn complete_irp_success(irp: &mut IRP) -> NTSTATUS {
    complete_irp_with_status(irp, STATUS_SUCCESS)
}
