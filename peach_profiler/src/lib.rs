//! # Peach Profiler 🍑
//!
//! Peach Profiler is a high performance instrumentation-based profiler. Made for low-overhead
//! profiling when you want it and zero-cost when you don't.
//!
//! ## Design
//!
//! Relies on platform specific assembly instructions to measure clock cycles for compatibility
//! and relative exactness.
//!
//! ## Example
//!
//! ```rust
//! use peach_profiler::{time_block, time_function, time_main};
//!
//! #[time_main]
//! fn main() {
//!     let answer = {
//!         time_block!("answer_block");
//!
//!         fibonacci(22)
//!     };
//!
//!     assert_eq!(answer, 28657);
//!
//!     // inside baseball (PROFILER isn't meant to be read directly) - shows the fib
//!     // function was timed as a single function and was executed 25 times and the
//!     // block that contained was named with the input and executed only once.
//!     #[cfg(feature = "profile")]
//!     unsafe {
//!         assert_eq!(
//!             PROFILER
//!                 .into_iter()
//!                 .find(|&profile| profile.label[0..9] == *"fibonacci".as_bytes())
//!                 .unwrap()
//!                 .hit_count,
//!             57313
//!         );
//!         assert_eq!(
//!             PROFILER
//!                 .into_iter()
//!                 .find(|&profile| profile.label[0..12] == *"answer_block".as_bytes())
//!                 .unwrap()
//!                 .hit_count,
//!             1
//!         );
//!     }
//!
//!     #[cfg(not(feature = "profile"))]
//!     panic!("Profile feature must be enabled.");
//!}
//!
//!
//! #[time_function]
//! fn fibonacci(x: usize) -> usize {
//!     if x == 0 || x == 1 {
//!         return 1;
//!     }
//!
//!     fibonacci(x - 1) + fibonacci(x - 2)
//! }
//! ```
//!
//! ---------- Outputs ----------
//! ``` console
//! 28657
//!
//! ______________________________________________________
//! Total time: 1.7120ms (CPU freq 4300627921)
//!     answer_block[1]: 7396, (0.10%, 99.71% w/children)
//!     fibonacci[57313]: 7334252, (99.61%)
//! ```
extern crate peach_metrics;
extern crate peach_pit;

pub use peach_metrics::{estimate_cpu_freq, get_os_time_freq, read_cpu_timer, read_os_timer};
pub use peach_pit::{time_block, time_function, time_main};

#[doc(hidden)]
#[cfg(feature = "profile")]
#[derive(Clone)]
pub struct Timer {
    pub start: u64,
    pub index: usize,
    pub parent_anchor: usize,
    pub old_elapsed_inclusive: u64,
}

#[doc(hidden)]
#[cfg(feature = "profile")]
impl Timer {
    pub fn new(name: &str, index: usize) -> Self {
        assert!(index < 4096);

        // SAFETY: Assumes single threaded runtime! We've already asserted that the index
        // is within the PROFILER's range, and those values are already initialized. The
        // GLOBAL_PROFILER_PARENT is already initialized and is only updated as a different Timer
        // is dropped.
        let timer = unsafe {
            Self {
                start: read_cpu_timer(),
                index,
                parent_anchor: GLOBAL_PROFILER_PARENT,
                old_elapsed_inclusive: PROFILER[index].elapsed_inclusive,
            }
        };

        let label = name.as_bytes();
        let len = label.len().min(LABEL_LENGTH);

        // SAFETY: Assumes single threaded runtime! Label is an reserved 16 bytes.
        // Converting the name to a [u8] slice and then filling the reserved space
        // shouldn't fail. Updating the GLOBAL_PROFILER_PARENT with an asserted value.
        unsafe {
            // write the name to the anchor
            PROFILER[index].label[..len].copy_from_slice(&label[..len]);

            GLOBAL_PROFILER_PARENT = index;
        }

        timer
    }
}

#[doc(hidden)]
#[cfg(feature = "profile")]
impl Drop for Timer {
    fn drop(&mut self) {
        assert!(self.index < 4096);
        assert!(self.parent_anchor < 4096);

        let elapsed = read_cpu_timer() - self.start;

        // SAFETY: Assumes signle threaded runtime! Indexes self.index and self.parent_anchor have
        // already been asserted to be within the bounds of the PROFILER array.
        unsafe {
            // set the global parent back to the popped anchor's parent
            GLOBAL_PROFILER_PARENT = self.parent_anchor;

            PROFILER[self.parent_anchor].elapsed_exclusive = PROFILER[self.parent_anchor]
                .elapsed_exclusive
                .wrapping_sub(elapsed);
            PROFILER[self.index].elapsed_exclusive =
                PROFILER[self.index].elapsed_exclusive.wrapping_add(elapsed);
            PROFILER[self.index].elapsed_inclusive = self.old_elapsed_inclusive + elapsed;
            PROFILER[self.index].hit_count += 1;
        }
    }
}

// Helper function used to hash a timer to reference the anchors
#[doc(hidden)]
#[cfg(feature = "profile")]
pub const fn compile_time_hash(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut hash = 5381u32; // DJB2 hash initial value
    let mut i = 0;
    while i < bytes.len() {
        hash = hash.wrapping_mul(33).wrapping_add(bytes[i] as u32);
        i += 1;
    }
    hash
}

#[doc(hidden)]
#[cfg(feature = "profile")]
const LABEL_LENGTH: usize = 16;

#[doc(hidden)]
#[cfg(feature = "profile")]
#[derive(Copy, Clone)]
pub struct ProfileAnchor {
    pub elapsed_exclusive: u64, // cycles not including children
    pub elapsed_inclusive: u64, // cycles including children
    pub hit_count: u64,
    pub label: [u8; LABEL_LENGTH],
}

#[doc(hidden)]
#[cfg(feature = "profile")]
impl ProfileAnchor {
    pub const fn new() -> Self {
        Self {
            elapsed_exclusive: 0,
            elapsed_inclusive: 0,
            hit_count: 0,
            label: [0; LABEL_LENGTH],
        }
    }
}

#[doc(hidden)]
#[cfg(feature = "profile")]
impl Default for ProfileAnchor {
    fn default() -> Self {
        Self::new()
    }
}

// initialize the global variables
#[doc(hidden)]
#[cfg(feature = "profile")]
pub static mut PROFILER: [ProfileAnchor; 4096] = [ProfileAnchor::new(); 4096];
#[doc(hidden)]
#[cfg(feature = "profile")]
pub static mut GLOBAL_PROFILER_PARENT: usize = 0;
