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

> [!IMPORTANT]
> The Peach Profiler will only add instrumentation needed to profile and
> output performance metrics if the `profile` feature is enabled. Without it
> just the `Total time` will be shown.

Either add it with the dependancy in the `Cargo.toml` file, i.e.:

```toml
[dependencies]
peach_profiler = { version = "0.1", features = ["profile"] }
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

### Debugging

If you are missing a timed function or block from the output there could be the potential for hash
collisions that aren't checked for in the default execution. Running Peach Profiler with the `debug`
feature will panic with collision information if one is found.
