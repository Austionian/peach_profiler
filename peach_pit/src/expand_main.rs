use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemFn, parse_quote};

// Adds code to print the total time the binary took to execute.
fn print_baseline() -> TokenStream2 {
    quote! {
        let timerFreq = peach_profiler::get_os_time_freq() as f64* __total_cpu as f64 / __total_time as f64;
        peach_profiler::println!("\n______________________________________________________");
        peach_profiler::println!(
            "Total time: {:.4}ms (CPU freq {:.0})",
            __total_time as f64 / 1_000.0,
             timerFreq
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
            let mut i = 0;
            while(i < peach_profiler::__BLOCKS.len()) {
                let block = peach_profiler::__BLOCKS[i];
                if block.elapsed_inclusive > 0 {
                    peach_profiler::print!("\t{}[{}]: {}, ({:.2}%",
                        core::str::from_utf8(&block.label).unwrap_or(&"invalid name"),
                        block.hit_count,
                        block.elapsed_exclusive,
                       (block.elapsed_exclusive as f64 / __total_cpu as f64) * 100.0,
                    );
                    if block.elapsed_exclusive != block.elapsed_inclusive {
                        peach_profiler::print!(", {:.2}% w/children",
                            (block.elapsed_inclusive as f64 / __total_cpu as f64) * 100.0,
                        );
                    }
                    peach_profiler::print!(")");
                    if block.processed_byte_count > 0 {
                        const MEGABYTE: f64 = 1024.0 * 1024.0;
                        const GIGABYTE: f64 = MEGABYTE * 1024.0;

                        let seconds = block.elapsed_inclusive as f64 / timerFreq;
                        let bytes_per_second = block.processed_byte_count as f64 / seconds;
                        let megabytes = block.processed_byte_count as f64 / MEGABYTE;
                        let gigabytes_per_second = bytes_per_second / GIGABYTE;

                        peach_profiler::print!(" {megabytes:.3}mb at {gigabytes_per_second:.2}gb/s");
                    }

                    peach_profiler::print!("\n");
                }

                i += 1;
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
