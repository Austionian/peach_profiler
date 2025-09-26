/// Reads in the Read Time-Stamp Counter (rdtsc) and returns it.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn read_cpu_timer() -> u64 {
    // SAFETY: This code should only be compiled in x86_64 contexts
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }

    // SAFETY: This code should only be compiled in x86 contexts
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::x86::_rdtsc()
    }
}

/// Reads in the value from the Counter-timer Virtual Counter (cntvct_el0) register and returns it
#[cfg(target_arch = "aarch64")]
pub fn read_cpu_timer() -> u64 {
    let x: u64;
    // SAFETY: This function should only be compiled into aarch64 contexts.
    unsafe { core::arch::asm!("mrs {}, cntvct_el0", out(reg) x) }
    x
}
