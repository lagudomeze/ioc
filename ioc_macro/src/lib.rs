#![feature(proc_macro_span)]

use proc_macro::TokenStream;

mod export;

mod import;
mod run;

#[proc_macro]
pub fn export(input: TokenStream) -> TokenStream {
    export::generate(input)
        .unwrap_or_else(|err| err.write_errors().into())
}

#[proc_macro]
pub fn run(input: TokenStream) -> TokenStream {
    run::generate(input)
        .unwrap_or_else(|err| err.write_errors().into())
}
