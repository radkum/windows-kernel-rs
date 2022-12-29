#![allow(unused)]

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
    pub fn ExAllocatePool(pool_type: PoolType, number_of_bytes: usize) -> *mut u64;
    pub fn ExAllocatePoolWithTag(pool_type: PoolType, number_of_bytes: usize, tag: u32)
        -> *mut u64;
    pub fn ExAllocatePool2(pool_type: PoolFlags, number_of_bytes: usize, tag: u32) -> *mut u64;

    pub fn ExFreePool(pool: u64);
    pub fn ExFreePoolWithTag(pool: u64, tag: u32);
}
