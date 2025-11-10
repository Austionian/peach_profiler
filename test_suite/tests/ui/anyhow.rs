use peach_profiler::{time_block, time_function, time_main};

#[time_main]
fn main() -> anyhow::Result<()> {
    let a = {
        time_block!("test");

        test_function()
    };

    assert_eq!(a, usize::MAX);

    Ok(())
}

#[time_function]
fn test_function() -> usize {
    usize::MAX
}
