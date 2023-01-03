#![no_std]
#![allow(non_snake_case)]

#[macro_export]
macro_rules! NT_SUCCESS {
    ($status:expr) => {
        $status as NTSTATUS >= 0
    };
}

#[macro_export]
macro_rules! CTL_CODE {
    ($DeviceType:expr, $Function:expr, $Method:expr, $Access:expr) => {
        ($DeviceType << 16) | ($Access << 14) | ($Function << 2) | $Method
    };
}

#[macro_export]
macro_rules! HandleToU32 {
    ($Handle:expr) => {
        ($Handle as u32)
    };
}

#[macro_export]
macro_rules! PAGED_CODE {
    () => {
        // if KeGetCurrentIrql() > 1/*APC_LEVEL*/
        // {
        //     return STATUS_UNSUCCESSFUL;
        // }
    };
}
