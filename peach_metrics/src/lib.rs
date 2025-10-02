#![allow(clippy::cast_sign_loss)]

mod cpu_timer;
mod os_timer;

pub use cpu_timer::read_cpu_timer;
pub use os_timer::{get_os_time_freq, read_os_timer};

/// Used to estimate the cpu's clock speed, from it can estimate cpu timings to realworld clock
/// time, i.e. how many clocks per seconds.
#[must_use]
pub fn estimate_cpu_freq() -> u64 {
    // Loop for effectively 100ms to guess the cpu's clock frequency.
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
