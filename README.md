# Peach Profiler 🍑

_The juiciest profiler_

---

## Peach Profiler in action

<details>
<summary>
Click to show Cargo.toml.
</summary>

```toml
[dependencies]
peach_profiler = "0.1"
# Alternatively list peach_profiler like so to always enable profiling.
# peach_profiler = { version = "0.1", features=["profile"] }

[features]
# Point your profile feature are the peach_profilers profile feature. Running
# with `cargo r --features=profile` will display profile information from run.
# Running without the feature removes all macro generated code.
profile = ["peach_profiler/profile"]

```

</details>
<p></p>

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
