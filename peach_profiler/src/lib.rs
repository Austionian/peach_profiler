//! # Peach Profiler
//!
//! Peach Profiler is a high performance instrumentation based profiler. Made for low-overhead
//! and ease of use.
//!
//! ## Design
//!
//! todo!()

/// Example use of `#[time_main]`
///
/// ```
/// use peach_profiler::{time_block, time_function, time_main};
///
/// #[time_main]
/// fn main() {
///     let ans = {
///         time_block!("ans block");
///
///         fib(6)
///     };
///
///     assert_eq!(ans, 13);
///
///     // inside baseball - shows the fib function was timed as a single function
///     // and was executed 25 times and the block that contained was named with the
///     // input and executed only once.
///     unsafe {
///         assert_eq!(
///             PROFILER
///                 .into_iter()
///                 .find(|&profile| profile.label[0..3] == *"fib".as_bytes())
///                 .unwrap()
///                 .hit_count,
///             25
///         );
///         assert_eq!(
///             PROFILER
///                 .into_iter()
///                 .find(|&profile| profile.label[0..9] == *"ans block".as_bytes())
///                 .unwrap()
///                 .hit_count,
///             1
///         );
///     }
///}
///
///
/// #[time_function]
/// fn fib(x: usize) -> usize {
///     if x == 0 || x == 1 {
///         return 1;
///     }
///
///     fib(x - 1) + fib(x - 2)
/// }
extern crate peach_metrics;
extern crate peach_pit;

pub use peach_metrics::{estimate_cpu_freq, get_os_time_freq, read_cpu_timer, read_os_timer};
pub use peach_pit::{time_block, time_function, time_main};
