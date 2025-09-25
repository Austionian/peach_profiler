pub fn get_os_time_freq() -> u64 {
    1_000_000
}

pub fn read_os_timer() -> u64 {
    let mut value = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };

    unsafe {
        libc::gettimeofday(&raw mut value, std::ptr::null_mut());
    }

    get_os_time_freq() * value.tv_sec as u64 + value.tv_usec as u64
}
