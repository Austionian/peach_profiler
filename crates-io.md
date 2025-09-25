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
        time_block!("answer_block");

        fibonacci(22)
    };

    println!("{answer}");
}

// Time a function
#[time_function]
fn fibonacci(x: usize) -> usize {
    if x == 0 || x == 1 {
        return 1;
    }

    fibonacci(x - 1) + fibonacci(x - 2)
}
```

---------- Outputs ----------

```console
28657

______________________________________________________
Total time: 1.7490ms (CPU freq 4300860492)
        answer_block[1]: 6665, (0.09%, 99.63% w/children)
        fibonacci[57313]: 7487891, (99.54%)
```
