pub fn read_cpu_timer() -> u64 {
    let x: u64;

    unsafe { core::arch::asm!("mrs {}, cntvct_el0", out(reg) x) }

    x
}
