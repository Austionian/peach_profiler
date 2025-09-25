<!-- Peach Profiler readme rendered on crates.io -->

**Peach Profiler 🍑 is a performant, low-overhead profiler. Just peachy.**

---

## Peach Profiler in action

```rust
use peach_profiler::{time_block, time_main, time_function};

// Add the `time_main` macro to the main function
#[time_main]
fn main() {
    let answer = {
        // Time a block
        time_block!("ans block");

        fib(6)
    };

    println!("{answer}");
}

// Time a function
#[time_function]
fn fib(x: usize) -> usize {
    if x == 0 || x == 1 {
        return 1;
    }

    fib(x - 1) + fib(x - 2)
}
```

OUTPUTS:

```console

28657

______________________________________________________
Total time: 1.6890ms (CPU freq 4299210776)
        fib[57313]: 7219959, (99.43%)
        ans block[1]: 6578, (0.09%, 99.52% w/children)
```
