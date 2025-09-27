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

fn expand_main(mut function: ItemFn) -> TokenStream2 {
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        use peach_profiler::{read_cpu_timer, read_os_timer, get_os_time_freq};
        #[cfg(feature = "profile")]
        use peach_profiler::PROFILER;

        let time_start = read_os_timer();
        let cpu_start = read_cpu_timer();

        #(#stmts)*

        let cpu_end = read_cpu_timer();
        let time_end = read_os_timer();

        let total_cpu = cpu_end - cpu_start;
        let total_time = time_end - time_start;

        println!("\n______________________________________________________");
        println!(
            "Total time: {:.4}ms (CPU freq {:.0})",
            total_time as f64 / 1_000.0,
            get_os_time_freq() as f64 * total_cpu as f64 / total_time as f64
        );


        #[cfg(feature = "profile")]
        unsafe {
            let mut i = 0;
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
        #function
    )
}

#[cfg(feature = "profile")]
fn expand_timing(mut function: ItemFn) -> TokenStream2 {
    let name = function.sig.ident.clone().to_string();
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        use peach_profiler::{read_cpu_timer, time_block};

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
        use peach_profiler::{PROFILER, GLOBAL_PROFILER_PARENT, Timer, compile_time_hash};

        const LOCATION: &str = concat!(file!(), ":", line!());
        const HASH: u32 = compile_time_hash(LOCATION);
        const ID: usize = (HASH & 0xFFF) as usize; // Mask to 12 bits (0-4095)

        assert!(ID < 4096);

        let timer = unsafe {
            Timer::new(#block_name, ID)
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
