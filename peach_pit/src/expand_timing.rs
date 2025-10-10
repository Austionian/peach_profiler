use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_quote, ItemFn};

// Creates a time_block! with function's name that will be dropped and recorded in the profile
// when the function is completed.
pub(crate) fn expand_timing(mut function: ItemFn) -> TokenStream2 {
    let name = function.sig.ident.clone().to_string();
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        peach_profiler::time_block!(#name);

        #(#stmts)*
    }));

    quote!(#function)
}
