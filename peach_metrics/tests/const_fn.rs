#[cfg(target_family = "unix")]
const _FOO: u64 = peach_metrics::get_os_time_freq();
//^ if os_time_freq is not a const, this will fail to compile
