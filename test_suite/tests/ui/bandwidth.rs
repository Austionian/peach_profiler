use peach_profiler::{time_block, time_function, time_main};

#[time_main]
fn main() {
    let a = {
        time_block!("test", 1024);

        test_function()
    };

    assert_eq!(a, usize::MAX);
}

#[time_function]
fn test_function() -> usize {
    usize::MAX
}
