# windows-kernel-rs

Several crates to help work with the Windows kernel:
- kernel-alloc - declares a global kernel allocator based on ExAllocatePoolWithTag
- kernel-init - gathers in one place all the necessary elements needed to run rust code in kernel
- kernel-macros - some useful macros for kernel mode
- kernel-string - ANSI_STRING and UNICODE_STRING with necessary functions

One crate with ported functions necessary to work with drivers and minifilters.
<br><br>

Some ideas taken from:  [Writing a kernel driver with Rust.](https://not-matthias.github.io/kernel-driver-with-rust/)
