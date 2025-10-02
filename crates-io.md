<!-- Peach Profiler readme rendered on crates.io -->

**Peach Profiler 🍑 is a performant, low-overhead profiler. It's just peachy.**

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

## To Use

The Peach Profiler will only add the instrumentation needed to profile and
output performance metrics if the `profile` feature is enabled.

Either add it with the dependancy in the `Cargo.toml` file, i.e.:

```toml
[dependencies]
peach_profiler = { version = "0.1", features = ["profile"]}
```

to always profile your code.

Or add a feature to your crate in the `Cargo.toml` file, i.e.:

```toml
[features]
profile = ["peach_profiler/profile"]
```

And then instrumentation will only be added when your program is run with the
feature specified, i.e. `cargo r --features=profile`

Run in a no_std env by disabling default features:

```toml
[dependencies]
peach_profiler = { version = "0.1", default_features = false }
```
