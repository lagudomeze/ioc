use proc_macro::TokenStream;

use darling::FromDeriveInput;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{DeriveInput, ItemImpl, parse_macro_input};

use bean::BeanSpecStruct;

mod bean;
mod custom;
mod init;

/// See module level documentation for more information.
#[proc_macro_derive(Bean, attributes(inject, bean))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    return match BeanSpecStruct::from_derive_input(&input) {
        Ok(bean_struct) => {
            bean_struct.into_token_stream().into()
        }
        Err(err) => {
            err.write_errors().into()
        }
    };
}

#[proc_macro_attribute]
pub fn bean(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    return match custom::expand(TokenStream2::from(attr), impl_block) {
        Ok(tt) => {
            tt.into()
        }
        Err(err) => {
            err.write_errors().into()
        }
    }
}