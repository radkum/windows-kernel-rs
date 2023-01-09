fn __readcr8() -> u64 {
    #[allow(unused)]
    let x: u64;
    unsafe {
        core::arch::asm!("mov {}, cr8", out(reg) x);
    }
    x
}

#[allow(non_upper_case_globals)]
pub(crate) const ReadCR8: fn() -> u64 = __readcr8;
