use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemFn, parse_quote};

// Adds code to print the total time the binary took to execute.
// Is always added whether the profile feature is enabled or not.
fn print_baseline() -> TokenStream2 {
    quote! {
        let __cpu_freq =
            peach_profiler::get_os_time_freq() as f64 * __total_cpu as f64 / __total_time as f64;
        peach_profiler::println!("\n______________________________________________________");
        peach_profiler::println!(
            "Total time: {:.4}ms (CPU freq {:.0})",
            __total_time as f64 / 1_000.0,
            __cpu_freq
        );
    }
}

// With profile enabled, loop through the __BLOCKS array and print out information for any
// TimedBlock that recorded elapsed_time.
#[cfg(feature = "profile")]
fn print_profile() -> TokenStream2 {
    let baseline_print = print_baseline();
    quote! {
        #baseline_print;

        unsafe {
            let mut __print_i = 0;
            while(__print_i < peach_profiler::__BLOCKS.len()) {
                let __block = peach_profiler::__BLOCKS[__print_i];
                if __block.elapsed_inclusive > 0 {
                    peach_profiler::print!("\t{}[{}]: {}, ({:.2}%",
                        core::str::from_utf8(&__block.label).unwrap_or(&"invalid name"),
                        __block.hit_count,
                        __block.elapsed_exclusive,
                       (__block.elapsed_exclusive as f64 / __total_cpu as f64) * 100.0,
                    );
                    if __block.elapsed_exclusive != __block.elapsed_inclusive {
                        peach_profiler::print!(", {:.2}% w/children",
                            (__block.elapsed_inclusive as f64 / __total_cpu as f64) * 100.0,
                        );
                    }
                    peach_profiler::print!(")");
                    if __block.processed_byte_count > 0 {
                        const __MEGABYTE: f64 = 1024.0 * 1024.0;
                        const __GIGABYTE: f64 = __MEGABYTE * 1024.0;

                        let __seconds = __block.elapsed_inclusive as f64 / __cpu_freq;
                        let __bytes_per_second = __block.processed_byte_count as f64 / __seconds;
                        let __megabytes = __block.processed_byte_count as f64 / __MEGABYTE;
                        let __gigabytes_per_second = __bytes_per_second / __GIGABYTE;

                        peach_profiler::print!(
                            " {__megabytes:.3}mb at {__gigabytes_per_second:.2}gb/s"
                        );
                    }

                    peach_profiler::print!("\n");
                }

                __print_i += 1;
            }
        }
    }
}

// With the profile feature disabled, just print the total amount of time taken during the
// execution of the binary.
#[cfg(not(feature = "profile"))]
fn print_profile() -> TokenStream2 {
    print_baseline()
}

pub(crate) fn expand_main(mut function: ItemFn) -> TokenStream2 {
    let stmts = function.block.stmts;
    let print = print_profile();
    function.block = Box::new(parse_quote!({
        let __time_start = peach_profiler::read_os_timer();
        let __cpu_start = peach_profiler::read_cpu_timer();

        #(#stmts)*

        let __cpu_end = peach_profiler::read_cpu_timer();
        let __time_end = peach_profiler::read_os_timer();

        let __total_cpu = __cpu_end - __cpu_start;
        let __total_time = __time_end - __time_start;

        #print;
    }));

    quote!(
        #function
    )
}
