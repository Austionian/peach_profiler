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
//!     // Inside baseball: (__BLOCKS isn't meant to be read directly) - shows the fibonacci
//!     // function was timed as a single function and was executed 25 times and the
//!     // block that contained it was named with the input and executed only once.
//!     #[cfg(feature = "profile")]
//!     unsafe {
//!         assert_eq!(
//!             peach_profiler::__BLOCKS
//!                 .into_iter()
//!                 .find(|&block| block.label[0..9] == *"fibonacci".as_bytes())
//!                 .unwrap()
//!                 .hit_count,
//!             57313
//!         );
//!         assert_eq!(
//!             peach_profiler::__BLOCKS
//!                 .into_iter()
//!                 .find(|&block| block.label[0..12] == *"answer_block".as_bytes())
//!                 .unwrap()
//!                 .hit_count,
//!             1
//!         );
//!     }
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

// Support using Peach Profiler without the standard library!
#![cfg_attr(not(feature = "std"), no_std)]

extern crate peach_metrics;
extern crate peach_pit;

pub use peach_metrics::{estimate_cpu_freq, get_os_time_freq, read_cpu_timer, read_os_timer};
pub use peach_pit::{time_function, time_main};

#[doc(hidden)]
pub use peach_pit::__time_bandwidth;

#[cfg(feature = "profile")]
const ARRAY_LEN: usize = 0xFFF;

// Re-export libc_print macros to support a no_std environment
#[doc(hidden)]
#[cfg(not(feature = "std"))]
pub use libc_print::std_name::{print, println};

// Re-export print macros to support a std environment
#[doc(hidden)]
#[cfg(feature = "std")]
pub use std::{print, println};

#[doc(hidden)]
#[cfg(feature = "profile")]
#[derive(Clone)]
pub struct __Timer {
    pub start: u64,
    pub index: usize,
    pub parent: usize,
    pub old_elapsed_inclusive: u64,
    pub bytes: u64,
}

#[doc(hidden)]
#[cfg(feature = "profile")]
impl __Timer {
    pub fn new(name: &str, bytes: u64, index: usize) -> Self {
        assert!(index <= ARRAY_LEN);

        // A block with an empty name ("") doesn't make sense
        debug_assert!(!name.is_empty());

        // SAFETY: Assumes single threaded runtime! We've already asserted that the index
        // is within the __Timer's range, and those values are already initialized. The
        // __PARENT_TIMER_INDEX is already initialized and is only updated as a different __Timer
        // is dropped.
        let timer = unsafe {
            Self {
                start: read_cpu_timer(),
                index,
                parent: __PARENT_TIMER_INDEX,
                old_elapsed_inclusive: __BLOCKS[index].elapsed_inclusive,
                bytes,
            }
        };

        let label = name.as_bytes();
        let len = label.len().min(LABEL_LENGTH);

        // SAFETY: Assumes single threaded runtime! Label is an reserved 16 bytes.
        // Converting the name to a [u8] slice and then filling the reserved space
        // shouldn't fail. Updating the __PARENT_TIMER_INDEX with an asserted value.
        unsafe {
            // write the name to the block
            __BLOCKS[index].label[..len].copy_from_slice(&label[..len]);

            __PARENT_TIMER_INDEX = index;
        }

        #[cfg(feature = "debug")]
        // Checks for a collision after the fact so that the label only needs to be copied
        // from &[u8] -> [u8; 16] once.
        //
        // SAFETY: Assumes single threaded runtime! Label is an reserved 16 bytes.
        // index has already been asserted to fit within the array of __DEBUG_PROFILER.
        unsafe {
            let label_value = u128::from_le_bytes(__BLOCKS[index].label);

            // If __DEBUG_BLOCKS[index] == 0 that means it hasn't been initialized with a block
            // yet, so there's nothing to check. (Unless a label is "" which wouldn't make sense
            // and would panic in debug builds.)
            //
            // If the `label_value` which is the label of the  block that was just added to
            // __BLOCKS, doesn't equal what's already in __DEBUG_BLOCKS, that means the hash of the
            // new block's location collided with a block with a different label and was overriden.
            // The same block can write to the same place in the __BLOCKS array multiple times, but
            // its label should never change once set.
            assert!(
                !(__DEBUG_BLOCKS[index] != 0 && __DEBUG_BLOCKS[index] != label_value),
                "Hash collisions found! {} and {} both hashed to {index}",
                core::str::from_utf8(&__DEBUG_BLOCKS[index].to_le_bytes())
                    .unwrap_or("invalid name"),
                core::str::from_utf8(label).unwrap_or("invalid name"),
            );

            // If match wasn't found add it
            __DEBUG_BLOCKS[index] = label_value;
        }

        timer
    }
}

#[doc(hidden)]
#[cfg(feature = "profile")]
impl Drop for __Timer {
    fn drop(&mut self) {
        assert!(self.index <= ARRAY_LEN);
        assert!(self.parent <= ARRAY_LEN);

        let elapsed = read_cpu_timer() - self.start;

        // SAFETY: Assumes signle threaded runtime! Indexes self.index and self.parent have
        // already been asserted to be within the bounds of the __BLOCKS array.
        unsafe {
            // Set the global parent back to the popped timer's parent.
            __PARENT_TIMER_INDEX = self.parent;

            // Update the timer values.
            __BLOCKS[self.parent].elapsed_exclusive = __BLOCKS[self.parent]
                .elapsed_exclusive
                .wrapping_sub(elapsed);
            __BLOCKS[self.index].elapsed_exclusive =
                __BLOCKS[self.index].elapsed_exclusive.wrapping_add(elapsed);
            __BLOCKS[self.index].elapsed_inclusive = self.old_elapsed_inclusive + elapsed;
            __BLOCKS[self.index].hit_count += 1;
            __BLOCKS[self.index].processed_byte_count += self.bytes;
        }
    }
}

// const djb2 hash function
#[doc(hidden)]
#[cfg(feature = "profile")]
#[must_use]
pub const fn __peach_hash(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut hash = 5381u32;
    let mut i = 0;
    while i < bytes.len() {
        hash = hash.wrapping_mul(33).wrapping_add(bytes[i] as u32);
        i += 1;
    }

    let hash: usize = (hash & 0xFFF) as usize; // Mask to 12 bits (0-4095)
    assert!(hash <= ARRAY_LEN);

    hash
}

#[cfg(feature = "profile")]
const LABEL_LENGTH: usize = 16;

#[doc(hidden)]
#[cfg(feature = "profile")]
#[derive(Copy, Clone)]
pub struct TimedBlock {
    pub elapsed_exclusive: u64,    // cycles not including children
    pub elapsed_inclusive: u64,    // cycles including children
    pub hit_count: u64,            // number of times timed block was entered
    pub processed_byte_count: u64, // number of bytes processed in block's execution
    pub label: [u8; LABEL_LENGTH],
}

#[cfg(feature = "profile")]
impl TimedBlock {
    const fn new() -> Self {
        Self {
            elapsed_exclusive: 0,
            elapsed_inclusive: 0,
            hit_count: 0,
            processed_byte_count: 0,
            label: [0; LABEL_LENGTH],
        }
    }
}

// initialize the global variables
#[doc(hidden)]
#[cfg(feature = "profile")]
pub static mut __BLOCKS: [TimedBlock; ARRAY_LEN] = [TimedBlock::new(); ARRAY_LEN];
#[doc(hidden)]
#[cfg(feature = "profile")]
pub static mut __PARENT_TIMER_INDEX: usize = 0;

#[doc(hidden)]
#[cfg(all(feature = "profile", feature = "debug"))]
// stores the name of the function/block as a u128 at the hashed index to check for collisions.
pub static mut __DEBUG_BLOCKS: [u128; ARRAY_LEN] = [0; ARRAY_LEN];

#[cfg(test)]
mod tests {
    #[cfg(feature = "profile")]
    mod profile_tests {
        #[test]
        fn peach_hash_hashes_consistently() {
            let expected_hash = crate::__peach_hash("test");
            assert_eq!(expected_hash, 2149);
        }

        #[test]
        fn timed_block_has_a_new_function() {
            let expected_timed_block = crate::TimedBlock::new();
            assert_eq!(expected_timed_block.elapsed_exclusive, 0);
            assert_eq!(expected_timed_block.elapsed_inclusive, 0);
            assert_eq!(expected_timed_block.hit_count, 0);
            assert_eq!(expected_timed_block.label, [0; 16]);
        }
    }
}

/// Macro to instrumentally time a block of code.
///
/// Provide just the block's name or the block's name and the number of bytes the block of
/// code will process to capture the block's bandwidth.
///
/// ```ignore
/// // In a block of code
/// let output = {
///     time_block!("block_name");
///
///     // ..
/// }
///
/// // Or in a closure
/// let a = || {
///     time_block!("closure_time");
///
///     // ..
/// };
///
/// // Or with a number of bytes to capture the block's bandwidth
/// let output = {
///     time_block!("block_with_bandwidth", 1024);
///
///     // ..
/// };
///
/// // Will produce something like this with the profile feature enabled:
///     block_name[57313]: 7334252, (54.61%)
///     closure_time[23]: 12323, (12.45%)
///     block_with_bandwidth[1200]: 789112, (44.85%) 1.229mb at 3.71gb/s
///
/// // ^ Output:
/// //  - name given to the block - _limited to 16 bytes_
/// //  - [hit count] - number of times this block was executed
/// //  - number of cycles spent executing this block
/// //  - (percent of time spent in this block relative to the total time of the binary's run.)
/// //  - number of mb executed and it's gb per second bandwidth.
/// ```
#[macro_export]
macro_rules! time_block {
    ($name:expr) => {{
        peach_profiler::__time_bandwidth!($name, 0);
    }};

    ($name:expr, $bytes:expr) => {{
        peach_profiler::__time_bandwidth!($name, $bytes);
    }};
}
