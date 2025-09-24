use proc_macro::TokenStream;
#[cfg(feature = "profile")]
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
#[cfg(feature = "profile")]
use syn::parse::{Nothing, Result};
#[cfg(feature = "profile")]
use syn::{parse_macro_input, parse_quote, ItemFn, Lit};

/// Example use of `#[time_main]`
///
/// ```
/// use peach_pit::{time_block, time_function, time_main};
///
/// #[time_main]
/// fn main() {
///     let ans = {
///         time_block!("ans block");
///
///         fib(6)
///     };
///
///     assert_eq!(ans, 13);
///
///     // inside baseball - shows the fib function was timed as a single function
///     // and was executed 25 times and the block that contained was named with the
///     // input and executed only once.
///     unsafe {
///         assert_eq!(
///             PROFILER
///                 .into_iter()
///                 .find(|&profile| profile.label[0..3] == *"fib".as_bytes())
///                 .unwrap()
///                 .hit_count,
///             25
///         );
///         assert_eq!(
///             PROFILER
///                 .into_iter()
///                 .find(|&profile| profile.label[0..9] == *"ans block".as_bytes())
///                 .unwrap()
///                 .hit_count,
///             1
///         );
///     }
///}
///
///
/// #[time_function]
/// fn fib(x: usize) -> usize {
///     if x == 0 || x == 1 {
///         return 1;
///     }
///
///     fib(x - 1) + fib(x - 2)
/// }
#[cfg(feature = "profile")]
#[proc_macro_attribute]
pub fn time_main(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = TokenStream2::from(input);
    TokenStream::from(match parse(args, input.clone()) {
        Ok(function) => {
            let expanded = expand_main(function);
            quote! {
                #expanded
            }
        }
        Err(parse_error) => {
            let compile_error = parse_error.to_compile_error();
            quote! {
                #compile_error
                #input
            }
        }
    })
}

#[cfg(feature = "profile")]
#[proc_macro_attribute]
pub fn time_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = TokenStream2::from(input);
    TokenStream::from(match parse(args, input.clone()) {
        Ok(function) => {
            let expanded = expand_timing(function);
            quote! {
                #expanded
            }
        }
        Err(parse_error) => {
            let compile_error = parse_error.to_compile_error();
            quote! {
                #compile_error
                #input
            }
        }
    })
}

#[cfg(feature = "profile")]
fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemFn> {
    let function: ItemFn = syn::parse2(input)?;
    let _: Nothing = syn::parse2::<Nothing>(args)?;

    Ok(function)
}

#[cfg(feature = "profile")]
fn expand_main(mut function: ItemFn) -> TokenStream2 {
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        use peach_metrics::{read_cpu_timer, read_os_timer, get_os_time_freq};

        let time_start = read_os_timer();
        let cpu_start = read_cpu_timer();

        #(#stmts)*

        let cpu_end = read_cpu_timer();
        let time_end = read_os_timer();

        let total_cpu = cpu_end - cpu_start;
        let total_time = time_end - time_start;

        println!(
            "Total time: {:.4}ms (CPU freq {:.0})",
            total_time as f64 / 1_000.0,
            get_os_time_freq() as f64 * total_cpu as f64 / total_time as f64
        );

        let mut i = 0;
        unsafe {
            while(i < PROFILER.len()) {
                let anchor = PROFILER[i];
                if anchor.elapsed_inclusive > 0 {
                    print!("\t{}[{}]: {}, ({:.2}%",
                        String::from_utf8_lossy(&anchor.label),
                        anchor.hit_count,
                        anchor.elapsed_exclusive,
                       (anchor.elapsed_exclusive as f64 / total_cpu as f64) * 100.0,
                    );
                    if anchor.elapsed_exclusive != anchor.elapsed_inclusive {
                        print!(", {:.2}% w/children",
                            (anchor.elapsed_inclusive as f64 / total_cpu as f64) * 100.0,
                        );
                    }
                    print!(")\n");
                }

                i += 1;
            }
        }
    }));

    quote!(
        use std::sync::{LazyLock, Mutex};
        use peach_metrics::read_cpu_timer;
        use std::collections::HashMap;

        #[derive(Copy, Clone)]
        pub struct ProfileAnchor {
            pub elapsed_exclusive: u64, // cycles not including children
            pub elapsed_inclusive: u64, // cycles including children
            pub hit_count: u64,
            pub label: [u8; 16],
        }

        impl ProfileAnchor {
            pub const fn new() -> Self {
                Self {
                    elapsed_exclusive: 0,
                    elapsed_inclusive: 0,
                    hit_count: 0,
                    label: [0; 16],
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Timer {
            pub start: u64,
            pub index: usize,
            pub parent_anchor: usize,
            pub old_elapsed_inclusive: u64,
        }

        impl Timer {
            pub unsafe fn new(name: &str, index: usize) -> Self {
                debug_assert!(GLOBAL_PROFILER_PARENT >= 0);
                debug_assert!(GLOBAL_PROFILER_PARENT < 4096);
                debug_assert!(index < 4096);

                let timer = Self {
                    start: read_cpu_timer(),
                    index,
                    parent_anchor: GLOBAL_PROFILER_PARENT,
                    old_elapsed_inclusive: PROFILER[index].elapsed_inclusive,
                };

                let label = name.as_bytes();
                let len = label.len().min(PROFILER[index].label.len());

                // SAFETY:  Assumes single threaded runtime! Label is an reserved 16 bytes.
                // Converting the name to a [u8] slice and then filling the reserved space
                // shouldn't fail. Updating the GLOBAL_PROFILER_PARENT with an asserted value.
                unsafe {
                    // write the name to the anchor
                    PROFILER[index].label[..len].copy_from_slice(&label[..len]);

                    GLOBAL_PROFILER_PARENT = index;
                }

                timer
            }
        }

        impl Drop for Timer {
            fn drop(&mut self) {
                debug_assert!(self.index < 4096);
                debug_assert!(self.parent_anchor < 4096);

                let elapsed = read_cpu_timer() - self.start;

                unsafe {
                    // set the global parent back to the popped anchor's parent
                    GLOBAL_PROFILER_PARENT = self.parent_anchor;

                    PROFILER[self.parent_anchor].elapsed_exclusive = PROFILER[self.parent_anchor].elapsed_exclusive.wrapping_sub(elapsed);
                    PROFILER[self.index].elapsed_exclusive = PROFILER[self.index].elapsed_exclusive.wrapping_add(elapsed);
                    PROFILER[self.index].elapsed_inclusive = self.old_elapsed_inclusive + elapsed;
                    PROFILER[self.index].hit_count += 1;
                    //PROFILER[self.anchor].label = self.name.;
                }
            }
        }

        // Helper function used to hash a timer to reference the anchors
        pub const fn compile_time_hash(s: &str) -> u32 {
            let bytes = s.as_bytes();
            let mut hash = 5381u32; // DJB2 hash initial value
            let mut i = 0;
            while i < bytes.len() {
                hash = hash.wrapping_mul(33).wrapping_add(bytes[i] as u32);
                i += 1;
            }
            hash
        }

        // initialize the global variables
        pub static mut PROFILER: [ProfileAnchor; 4096] = [ProfileAnchor::new(); 4096];
        pub static mut GLOBAL_PROFILER_PARENT: usize = 0;

        #function
    )
}

#[cfg(feature = "profile")]
fn expand_timing(mut function: ItemFn) -> TokenStream2 {
    let name = function.sig.ident.clone().to_string();
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        use peach_metrics::read_cpu_timer;
        use peach_pit::time_block;

        time_block!(#name);

        #(#stmts)*
    }));

    quote!(#function)
}

/// Macro to instrumentally time a block of code.
/// Requires that main is marked with `#[time_main]`
///
/// ```ignore
/// let output = {
///     time_block!("block_name");
///
///     // expressions
/// }
/// ```
#[cfg(feature = "profile")]
#[proc_macro]
pub fn time_block(input: TokenStream) -> TokenStream {
    let block_name: Lit = parse_macro_input!(input as Lit);
    quote!(
        use crate::{PROFILER, GLOBAL_PROFILER_PARENT, Timer, compile_time_hash};

        const LOCATION: &str = concat!(file!(), ":", line!());
        const HASH: u32 = compile_time_hash(LOCATION);
        const ID: usize = (HASH & 0xFFF) as usize; // Mask to 12 bits (0-4095)

        debug_assert!(ID < 4096);

        let timer = unsafe {
            Timer::new(#block_name, ID)
        };
    )
    .into()
}

#[doc(hidden)]
#[cfg(not(feature = "profile"))]
#[proc_macro_attribute]
pub fn time_main(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[doc(hidden)]
#[cfg(not(feature = "profile"))]
#[proc_macro_attribute]
pub fn time_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[doc(hidden)]
#[cfg(not(feature = "profile"))]
#[proc_macro]
pub fn time_block(_input: TokenStream) -> TokenStream {
    quote! {
        // profiling disabled
    }
    .into()
}
