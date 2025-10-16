use peach_profiler::{time_block, time_function, time_main};

struct Foo {
    bar: usize,
}

#[time_main]
fn main() {
    let foo = {
        // use an expression that evaluates to a usize
        time_block!("foo", std::mem::size_of::<Foo>());

        test_function()
    };

    let _bar = {
        // use a number literal
        time_block!("_bar", 1024);

        test_function()
    };

    assert_eq!(foo, usize::MAX);
}

#[time_function]
fn test_function() -> usize {
    usize::MAX
}
