#![allow(unused)]

use winapi::shared::ntdef::PVOID;

#[repr(C)]
pub enum PoolType {
    NonPagedPool,
    NonPagedPoolExecute,
    PagedPool,
}

type PoolFlags = u64;

pub const POOL_FLAG_NON_PAGED: PoolFlags = 0x0000000000000040u64; // Non paged pool NX
pub const POOL_FLAG_NON_PAGED_EXECUTE: PoolFlags = 0x0000000000000080u64; // Non paged pool executable
pub const POOL_FLAG_PAGED: PoolFlags = 0x0000000000000100u64; // Paged pool

extern "system" {
    pub fn ExAllocatePool(PoolType: PoolType, NumberOfBytes: usize) -> *mut u64;
    pub fn ExAllocatePoolWithTag(PoolType: PoolType, NumberOfBytes: usize, Tag: u32) -> *mut u64;
    pub fn ExAllocatePool2(PoolType: PoolFlags, NumberOfBytes: usize, Tag: u32) -> *mut u64;

    pub fn ExFreePool(Pool: PVOID);
    pub fn ExFreePoolWithTag(Pool: PVOID, Tag: u32);
}
