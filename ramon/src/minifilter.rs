use crate::{file_monitor::FileMonitor, POOL_TAG};
use core::{mem, mem::size_of, ptr::null_mut};
use kernel_macros::{NT_SUCCESS, PAGED_CODE};
use kernel_string::UNICODE_STRING;
use km_api_sys::{
    flt_kernel::*,
    ntoskrnl::{ExAllocatePoolWithTag, ExFreePoolWithTag, PoolType},
};
use winapi::{
    km::wdm::{DEVICE_TYPE, PDRIVER_OBJECT},
    shared::{
        ntdef::{
            InitializeObjectAttributes, NTSTATUS, OBJ_CASE_INSENSITIVE,
            OBJ_KERNEL_HANDLE, PUCHAR, PULONG, PVOID, ULONG, USHORT, OBJECT_ATTRIBUTES
        },
        ntstatus::{STATUS_INVALID_PARAMETER, STATUS_SUCCESS},
    },
};
use windows_sys::Wdk::Storage::FileSystem::Minifilters::{
    FltBuildDefaultSecurityDescriptor, FltCloseClientPort, FltCloseCommunicationPort,
    FltFreeSecurityDescriptor, FltCreateCommunicationPort, PFLT_PORT,
};

use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES as WDK_OBJECT_ATTRIBUTES;
//type CONST_PVOID =*const PVOID;

type FFI_PVOID =*mut core::ffi::c_void;
type CONST_FFI_PVOID =*const core::ffi::c_void;
type PSECURITY_DESCRIPTOR = *mut core::ffi::c_void;
type PPSECURITY_DESCRIPTOR = *mut PSECURITY_DESCRIPTOR;

use winapi::um::winnt::RtlZeroMemory;

const COMM_PORT_NAME: &str = "\\RAMON.KM2UM.Port";

const FLT_PORT_ALL_ACCESS: u32 = 0x001F0001 as u32;
pub(crate) type PMINIFILTER = *mut Minifilter;
static mut S_MINIFILTER: PMINIFILTER = null_mut();

pub(crate) struct Minifilter {
    filter_handle: PFLT_FILTER,
    server_port: PFLT_PORT,
    client_port: PFLT_PORT,
    //on_command: FN,
    //context: PVOID
}

impl Minifilter {
    pub(crate) fn factory(driver: PDRIVER_OBJECT) -> NTSTATUS {
        unsafe {
            let minifilter =
                ExAllocatePoolWithTag(PoolType::NonPagedPool, size_of::<Minifilter>(), POOL_TAG)
                    as PMINIFILTER;

            RtlZeroMemory(minifilter as PVOID, size_of::<Minifilter>());

            (*minifilter).init(driver);
        }
        STATUS_SUCCESS
    }

    pub(crate) unsafe fn init(&mut self, driver: PDRIVER_OBJECT) -> NTSTATUS {
        #[allow(unused_assignments)]
        let mut status = STATUS_SUCCESS;
        let mut filter_registered = false;
        let mut comm_initialized = false;
        loop {
            //--------------------FILTER_HANDLE-----------------------
            S_MINIFILTER = self;
            status = FltRegisterFilter(driver, &FILTER_REGISTRATION, &mut self.filter_handle);

            if NT_SUCCESS!(status) {
                filter_registered = true;
            } else {
                log::info!("failed to register filter 0x{:08x}", status);
                break;
            }

            status = self.init_comm_channel();
            if NT_SUCCESS!(status) {
                comm_initialized = true;
            } else {
                log::info!("failed to initialize comm channel 0x{:08x}", status);
                break;
            }

            status = FltStartFiltering(self.filter_handle);
            if !NT_SUCCESS!(status) {
                log::info!("failed to start filtering 0x{:08x}", status);
                break;
            }
            return status;
        }

        if filter_registered && !self.filter_handle.is_null() {
            FltUnregisterFilter(self.filter_handle);
            self.filter_handle = null_mut();
            log::info!("FltUnregisterFilter() => m_FltFilter(0x{:08p}).", self.filter_handle );
        }

        if comm_initialized {
            self.close_comm();
            self.filter_handle = null_mut();
            log::info!("FltUnregisterFilter() => m_FltFilter(0x{:08p}).", self.filter_handle );
        }

        //clean
        status
    }

    pub(crate) fn deinit(&mut self) {
        unsafe {
            if !self.filter_handle.is_null() {
                FltUnregisterFilter(self.filter_handle);

                self.filter_handle = null_mut();
            }
        }
    }

    //comm functions
    pub(crate) fn is_comm_active(&self) -> bool {
        self.server_port != 0 && self.client_port != 0
    }

    //comm callbacks
    pub(crate) unsafe extern "system" fn on_connect(
        client_port: PFLT_PORT,
        server_port_cookie: CONST_FFI_PVOID,
        _connection_context: CONST_FFI_PVOID,
        _size_of_context: u32,
        connection_port_cookie: *mut FFI_PVOID,
    ) -> NTSTATUS {
        if server_port_cookie.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

            let minifilter = server_port_cookie as PMINIFILTER;
            (*minifilter).client_port = client_port;

            *connection_port_cookie = server_port_cookie as FFI_PVOID;


        STATUS_SUCCESS
    }

    pub(crate) unsafe extern "system" fn on_disconnect(connection_cookie: CONST_FFI_PVOID) {
        if connection_cookie.is_null() {
            return;
        }

            let l_this = connection_cookie as PMINIFILTER;

            FltCloseClientPort((*l_this).filter_handle as isize, &mut (*l_this).client_port);
            (*l_this).client_port = 0;

    }

    pub(crate) unsafe extern "system" fn on_command(
        port_cookie: CONST_FFI_PVOID,
        p_in: CONST_FFI_PVOID,
        in_size: ULONG,
        p_out: FFI_PVOID,
        out_size: ULONG,
        out_return: PULONG,
    ) -> NTSTATUS {
        *out_return = 0;


        if !port_cookie.is_null() && in_size as usize >= size_of::<ULONG>() {
            let minifilter = port_cookie as PMINIFILTER;

            //if !minifilter.on_command.is_null() {
            unsafe {
                // we should use method with object to store info, but for know we use static fn
                //return (*minifilter).on_command((*minifilter).context, p_in, in_size, p_out, out_size, out_return);
                return Minifilter::mock_on_command(p_in as PUCHAR, in_size, p_out as PUCHAR, out_size, out_return);
            }
            //}
        }

        STATUS_SUCCESS
    }

    unsafe fn init_comm_channel(&self /*context: PVOID, CB_on_command */) -> NTSTATUS {
        let mut port_security: PSECURITY_DESCRIPTOR = null_mut();
        let mut port_name = UNICODE_STRING::create(COMM_PORT_NAME);
        let mut oa: OBJECT_ATTRIBUTES = unsafe { mem::zeroed() };

        let mut status = STATUS_SUCCESS;
        loop {
            status = FltBuildDefaultSecurityDescriptor(
                &mut port_security as PPSECURITY_DESCRIPTOR,
                FLT_PORT_ALL_ACCESS,
            );
            if NT_SUCCESS!(status) {
                log::warn!("failed to build security descriptor. Status: {}", status);
                break;
            }

            InitializeObjectAttributes(
                &mut oa as *mut OBJECT_ATTRIBUTES ,
                port_name.as_mut_ptr(),
                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                null_mut(),
                port_security as PVOID,
            );

            status = FltCreateCommunicationPort(
                self.filter_handle as isize,
                &self.server_port  as *const isize as *mut isize,
                &oa as *const OBJECT_ATTRIBUTES as *mut WDK_OBJECT_ATTRIBUTES,
                self as *const Minifilter as CONST_FFI_PVOID,
                Some(Minifilter::on_connect),
                Some(Minifilter::on_disconnect),
                Some(Minifilter::on_command),
                1,
            );
            if NT_SUCCESS!(status) {
                log::warn!("failed to create comm port. Status: {}", status);
                break;
            }

            return status;
        }
        //cleanup
        if !port_security.is_null() {
            FltFreeSecurityDescriptor(port_security);
        }
        status
    }

    unsafe fn close_comm(&mut self) {
        if self.server_port != 0 {
            FltCloseCommunicationPort(self.server_port);
            self.server_port = 0;
        }
    }

    fn mock_on_command(
        _p_in: PUCHAR,
        _in_size: ULONG,
        _p_out: PUCHAR,
        _out_size: ULONG,
        _out_return: PULONG,
    ) -> NTSTATUS {
        STATUS_SUCCESS
    }
}

impl Drop for Minifilter {
    fn drop(&mut self) {
        self.deinit();
    }
}

/*************************************************************************
    MiniFilter initialization and unload routines.
*************************************************************************/

const CALLBACKS: &'static [FLT_OPERATION_REGISTRATION] = {
    &[
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_CREATE)
            .set_preop(FileMonitor::RamonPreCreate),
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_SET_INFORMATION)
            .set_preop(FileMonitor::RamonPreSetInformation),
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_OPERATION_END),
    ]
};

const FILTER_REGISTRATION: FLT_REGISTRATION = FLT_REGISTRATION {
    Size: ::core::mem::size_of::<FLT_REGISTRATION>() as USHORT, /*sizeof*/
    Version: FLT_REGISTRATION_VERSION,
    Flags: 0,
    ContextRegistration: null_mut(),
    OperationRegistration: CALLBACKS.as_ptr(),
    FilterUnloadCallback: RamonUnload,
    InstanceSetupCallback: RamonInstanceSetup,
    InstanceQueryTeardownCallback: RamonInstanceQueryTeardown,
    InstanceTeardownStartCallback: RamonInstanceTeardownStart,
    InstanceTeardownCompleteCallback: RamonInstanceTeardownComplete,
    GenerateFileNameCallback: null_mut(),
    NormalizeNameComponentCallback: null_mut(),
    NormalizeContextCleanupCallback: null_mut(),
    TransactionNotificationCallback: null_mut(),
    NormalizeNameComponentExCallback: null_mut(),
    SectionNotificationCallback: null_mut(),
};

extern "system" fn RamonUnload(_flags: FLT_REGISTRATION_FLAGS) -> NTSTATUS {
    log::info!("ramon_unload");

    unsafe {
        if !S_MINIFILTER.is_null() {
            (*S_MINIFILTER).deinit();
        }

        let mem = S_MINIFILTER as PVOID;
        RtlZeroMemory(mem, size_of::<Minifilter>());
        ExFreePoolWithTag(mem, POOL_TAG);

        S_MINIFILTER = null_mut();
    }

    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn RamonInstanceSetup(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_SETUP_FLAGS,
    _volume_device_type: DEVICE_TYPE,
    _volume_filesystem_type: FLT_FILESYSTEM_TYPE,
) -> NTSTATUS {
    //log::info!("RamonInstanceSetup");
    PAGED_CODE!();
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn RamonInstanceQueryTeardown(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
) -> NTSTATUS {
    //log::info!("RamonInstanceQueryTeardown");

    PAGED_CODE!();

    //fileMon FltInstanceQueryTeardown

    //log::info!("RamonInstanceQueryTeardown SUCCESS");
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn RamonInstanceTeardownStart(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS {
    //log::info!("RamonInstanceTeardownStart");

    PAGED_CODE!();
    //log::info!("RamonInstanceTeardownStart SUCCESS");
    STATUS_SUCCESS
}

#[link_section = "PAGE"]
extern "system" fn RamonInstanceTeardownComplete(
    _flt_objects: PFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) -> NTSTATUS {
    //log::info!("RamonInstanceTeardownComplete");

    PAGED_CODE!();
    //log::info!("RamonInstanceTeardownComplete SUCCESS");
    STATUS_SUCCESS
}
