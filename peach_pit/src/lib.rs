//! The pit of peach_profiler--where the macros are made. Not really useful on its own as
//! attribute macros expand assuming the presence of `peach_profiler`.

mod expand_main;
#[cfg(feature = "profile")]
mod expand_timing;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    ItemFn,
    parse::{Nothing, Result},
};

#[cfg(feature = "profile")]
use syn::{
    Expr, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Attribtue macro to instrumentally time a binary.
///
/// ```ignore
/// #[time_main]
/// fn main() {
///     // ..
/// }
///
/// // Will print out total time taken:
/// ______________________________________________________
/// Total time: 1.7120ms (CPU freq 4300627921)
///
/// // Will print out total time and functions or blocks marked with #[time_function] and
/// // time_block!(), respectively with `--feature=profile`
/// ______________________________________________________
/// Total time: 1.7120ms (CPU freq 4300627921)
///     answer_block[1]: 7396, (0.10%, 99.71% w/children)
///     fibonacci[57313]: 7334252, (99.61%)
/// ```
#[proc_macro_attribute]
pub fn time_main(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = TokenStream2::from(input);
    TokenStream::from(match parse(args, input.clone()) {
        Ok(function) => {
            let expanded = expand_main::expand_main(function);
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

/// Attribtue macro to instrumentally time a function and how many times it was entered.
///
/// ```ignore
/// #[time_function]
/// fn some_function() {
///     // ..
/// }
///
/// // Will produce something like this with the profile feature enabled:
///     some_function[57313]: 7334252, (99.61%)
///
/// // ^ Output:
/// //  - function name - _limited to 16 bytes_
/// //  - [hit count] - number of times this block was executed
/// //  - number of cycles spent executing this block
/// //  - (percent of time spent in this block relative to the total time of the binary's run)
/// ```
#[cfg(feature = "profile")]
#[proc_macro_attribute]
pub fn time_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = TokenStream2::from(input);
    TokenStream::from(match parse(args, input.clone()) {
        Ok(function) => {
            let expanded = expand_timing::expand_timing(function);
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
struct TimeBandwidthArgs {
    name: LitStr,
    _comma: Token![,],
    bytes: Expr,
}

#[cfg(feature = "profile")]
impl Parse for TimeBandwidthArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TimeBandwidthArgs {
            name: input.parse()?,
            _comma: input.parse()?,
            bytes: input.parse()?,
        })
    }
}

#[cfg(feature = "profile")]
#[proc_macro]
/// Proc macro to instrumentally time a block of code.
///
/// Creates the hash based on its location and then the timer. Quasi-private as it should be used
/// through `peach_profiler::time_block!()` rather than directly.
pub fn __time_bandwidth(input: TokenStream) -> TokenStream {
    let TimeBandwidthArgs { name, bytes, .. } = parse_macro_input!(input as TimeBandwidthArgs);

    let block_name = name.value();

    quote!(
        // compute the hash here rather than in __Timer::new so that they can be const.
        const __LOCATION: &str = concat!(file!(), ":", line!());
        const __HASH: usize = peach_profiler::__peach_hash(__LOCATION);

        let __peach_timer = unsafe {
            peach_profiler::__Timer::new(#block_name, #bytes, __HASH)
        };
    )
    .into()
}

// Function to exact function AST for time_main and time_function
fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemFn> {
    let function: ItemFn = syn::parse2(input)?;
    let _: Nothing = syn::parse2::<Nothing>(args)?;

    Ok(function)
}

#[doc(hidden)]
#[cfg(not(feature = "profile"))]
#[proc_macro_attribute]
// With profiling disabled, return the function tokens as is.
pub fn time_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[doc(hidden)]
#[cfg(not(feature = "profile"))]
#[proc_macro]
// With profiling disabled, add a comment in the place of the proc macro.
pub fn __time_bandwidth(_input: TokenStream) -> TokenStream {
    quote! {
        // profiling disabled
    }
    .into()
}
