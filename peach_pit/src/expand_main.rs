use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_quote, ItemFn};

// Creates a __MainTimer in the `main` function that will be dropped when the program completes, at
// which point __MainTimer will print out profiling data collected.
pub(crate) fn expand_main(mut function: ItemFn) -> TokenStream2 {
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        // __MainTimer needs to be binded to a variable otherwise it will just be dropped right
        // away.
        let __MAIN_TIMER = peach_profiler::__MainTimer::new();

        #(#stmts)*
    }));

    quote!(
        #function
    )
}
