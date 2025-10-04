use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Nothing, Result};
#[cfg(feature = "profile")]
use syn::{parse_macro_input, Lit};
use syn::{parse_quote, ItemFn};

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

fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemFn> {
    let function: ItemFn = syn::parse2(input)?;
    let _: Nothing = syn::parse2::<Nothing>(args)?;

    Ok(function)
}

fn print_baseline() -> TokenStream2 {
    quote! {
        peach_profiler::println!("\n______________________________________________________");
        peach_profiler::println!(
            "Total time: {:.4}ms (CPU freq {:.0})",
            __total_time as f64 / 1_000.0,
            peach_profiler::get_os_time_freq() as f64 * __total_cpu as f64 / __total_time as f64
        );
    }
}

#[cfg(feature = "profile")]
fn print_profile() -> TokenStream2 {
    let baseline_print = print_baseline();
    quote! {
        #baseline_print;

        unsafe {
            let mut i = 0;
            while(i < peach_profiler::__PROFILER.len()) {
                let anchor = peach_profiler::__PROFILER[i];
                if anchor.elapsed_inclusive > 0 {
                    peach_profiler::print!("\t{}[{}]: {}, ({:.2}%",
                        core::str::from_utf8(&anchor.label).unwrap_or(&"invalid name"),
                        anchor.hit_count,
                        anchor.elapsed_exclusive,
                       (anchor.elapsed_exclusive as f64 / __total_cpu as f64) * 100.0,
                    );
                    if anchor.elapsed_exclusive != anchor.elapsed_inclusive {
                        peach_profiler::print!(", {:.2}% w/children",
                            (anchor.elapsed_inclusive as f64 / __total_cpu as f64) * 100.0,
                        );
                    }
                    peach_profiler::print!(")\n");
                }

                i += 1;
            }
        }
    }
}

#[cfg(not(feature = "profile"))]
fn print_profile() -> TokenStream2 {
    print_baseline()
}

fn expand_main(mut function: ItemFn) -> TokenStream2 {
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

#[cfg(feature = "profile")]
fn expand_timing(mut function: ItemFn) -> TokenStream2 {
    let name = function.sig.ident.clone().to_string();
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        peach_profiler::time_block!(#name);

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
        // compute the hash here rather than in __Timer::new so that they can be const.
        const __LOCATION: &str = concat!(file!(), ":", line!());
        const __HASH: usize = peach_profiler::__peach_profiler_hash(__LOCATION);

        let __peach_timer = unsafe {
            peach_profiler::__Timer::new(#block_name, __HASH)
        };
    )
    .into()
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
