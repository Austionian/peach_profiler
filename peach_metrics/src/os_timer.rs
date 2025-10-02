/// We know that the frequency of the getTimeOfDay timer is in microseconds so we can hardcode the
/// system frequency to `1_000_000`.
#[cfg(target_family = "unix")]
#[must_use]
pub const fn get_os_time_freq() -> u64 {
    1_000_000
}

/// Makes getTimeOfDay sys call and returns the result computed to seconds since the epoch
#[cfg(target_family = "unix")]
#[must_use]
pub fn read_os_timer() -> u64 {
    let mut value = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };

    unsafe {
        libc::gettimeofday(&raw mut value, core::ptr::null_mut());
    }

    get_os_time_freq() * value.tv_sec as u64 + value.tv_usec as u64
}

/// Uses Window's QueryPerformanceFrequency to determine the timing frequency of the
/// QueryPerformanceCounter.
#[cfg(target_family = "windows")]
#[must_use]
pub fn get_os_time_freq() -> u64 {
    let mut freq = 0i64;
    unsafe {
        windows_sys::Win32::System::Performance::QueryPerformanceFrequency(&mut freq);
    }
    freq as u64
}

/// Uses Window's wallclock high-resolution counter - QueryPerformanceCounter to get the wallcock
/// time.
#[cfg(target_family = "windows")]
#[must_use]
pub fn read_os_timer() -> u64 {
    let mut value = 0i64;
    unsafe {
        windows_sys::Win32::System::Performance::QueryPerformanceCounter(&mut value);
    }
    value as u64
}
