pub fn get_os_time_freq() -> u64 {
    let mut freq = 0i64;
    unsafe { windows_sys::Win32::System::Performance::QueryPerformanceFrequency(&mut freq) }
    freq.QuadPart
}

pub fn read_os_timer() -> u64 {
    let mut value = 0i64;
    unsafe { windows_sys::Win32::System::Performance::QueryPerformanceCounter(&mut value) }
    value.QuadPart
}
