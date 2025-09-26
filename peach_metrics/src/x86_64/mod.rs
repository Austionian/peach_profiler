pub fn read_cpu_timer() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}
