pub fn read_os_timer() -> u64 {
    todo!()
}

pub fn read_os_timer() -> u64 {
    let mut value = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };

    unsafe {
        libc::gettimeofday(&mut value, std::ptr::null_mut());
    }

    get_os_time_freq() * value.tv_sec as u64 + value.tv_usec as u64
}
