pub fn read_cpu_timer() -> u64 {
    unsafe { std::arch::x86_64::_rdtsc() }
}
