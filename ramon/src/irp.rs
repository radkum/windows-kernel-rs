/*************************************************************************
                    IRP functions
*************************************************************************/
use winapi::{
    km::wdm::{IoCompleteRequest, IRP},
    shared::{ntdef::NTSTATUS, ntstatus::STATUS_SUCCESS},
};

fn complete_irp_with_status(irp: &mut IRP, status: NTSTATUS) -> NTSTATUS {
    complete_irp(irp, status, 0)
}

pub(crate) fn complete_irp_success(irp: &mut IRP) -> NTSTATUS {
    complete_irp_with_status(irp, STATUS_SUCCESS)
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
