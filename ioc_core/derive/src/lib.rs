use proc_macro::TokenStream;

use darling::FromDeriveInput;
use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

use bean::BeanStruct;

mod bean;
mod init;

/// See module level documentation for more information.
#[proc_macro_derive(Bean, attributes(inject, bean))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    return match BeanStruct::from_derive_input(&input) {
        Ok(bean_struct) => {
            bean_struct.into_token_stream().into()
        }
        Err(err) => {
            err.write_errors().into()
        }
    };
}
