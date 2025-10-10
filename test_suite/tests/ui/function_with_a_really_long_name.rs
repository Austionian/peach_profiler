use peach_profiler::{time_function, time_main};

#[time_main]
fn main() {
    assert_eq!(function_with_a_really_long_name(), usize::MAX);
}

// This example will compile, but this function's label will be function_with_a_ because labels
// are clipped to 16 bytes.
#[time_function]
fn function_with_a_really_long_name() -> usize {
    usize::MAX
}
