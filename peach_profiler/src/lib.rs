//! # Peach Profiler
//!
//! Peach Profiler is a high performance instrumentation based profiler. Made for low-overhead
//! and ease of use.
//!
//! ## Design
//!
//!
//!

extern crate peach_metrics;
extern crate peach_pit;

pub use peach_metrics::{estimate_cpu_freq, get_os_time_freq, read_cpu_timer, read_os_timer};
pub use peach_pit::{time_block, time_function, time_main};
