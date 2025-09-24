#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_os = "linux")]
pub use linux::{get_os_time_freq, read_os_timer};
#[cfg(target_os = "macos")]
pub use macos::{get_os_time_freq, read_os_timer};
#[cfg(target_os = "windows")]
pub use windows::{get_os_time_freq, read_os_timer};

#[cfg(target_arch = "x86_64")]
pub use x86_64::read_cpu_timer;
#[cfg(target_arch = "aarch64")]
pub use x86_64::read_cpu_timer;

pub fn estimate_cpu_freq() -> u64 {
    let ms_to_wait = 100u64;

    let os_freq = get_os_time_freq();

    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();
    let mut os_end;
    let mut os_elasped = 0;
    let os_wait_time = os_freq * ms_to_wait / 1_000;

    while os_elasped < os_wait_time {
        os_end = read_os_timer();
        os_elasped = os_end - os_start;
    }

    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    let mut cpu_freq = 0;

    if os_elasped != 0 {
        cpu_freq = os_freq * cpu_elapsed / os_elasped;
    }

    cpu_freq
}
