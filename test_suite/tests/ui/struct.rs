use peach_profiler::{time_function, time_main};

#[time_function]
struct Foo {
    bar: u8,
}

#[time_main]
fn main() {
    let a = test_function();

    assert_eq!(a, usize::MAX);
}

#[time_function]
fn test_function() -> usize {
    usize::MAX
}
