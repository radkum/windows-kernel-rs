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
