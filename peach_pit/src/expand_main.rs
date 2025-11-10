use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemFn, parse_quote};

pub(crate) fn expand_main(mut function: ItemFn) -> TokenStream2 {
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        // __MainTimer needs to be created with a variable otherwise it will just be dropped right
        // away.
        let __MAIN_TIMER = peach_profiler::__MainTimer::new();

        #(#stmts)*
    }));

    quote!(
        #function
    )
}
