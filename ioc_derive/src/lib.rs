use bean::FieldAttribute;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

mod scan;

mod bean;

fn preload_mods() -> proc_macro2::TokenStream {
    use scan::CargoToml;
    let toml = CargoToml::current();

    let mut mod_names = vec![];
    let mut mods = vec![];

    for name in toml.mod_names() {
        // mod name may contains "-", in `use` statment nead replace it to
        let mod_name = name.replace("-", "_");
        mods.push(format_ident!("{}", mod_name));
        mod_names.push(mod_name);
    }

    let test = format!("{mod_names:?}");

    quote! {
        println!("preload mods: {}", #test);
        #( use #mods; )*
    }
}

#[proc_macro]
pub fn run(_: TokenStream) -> TokenStream {
    let preload_mods = preload_mods();

    let expanded = quote! {

        #preload_mods

        ::ioc::run_app();
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Bean, attributes(r#ref, value))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("Bean derive macro only works with structs"),
    };

    let field_initializers = fields.iter().map(|field| {
        let field_name = &field.ident;
        let attr = FieldAttribute::from_attributes(&field.attrs).expect("");
        match attr {
            FieldAttribute::Ref(name) => quote! {
                #field_name: context.get_ref::<_>(#name)
            },
            FieldAttribute::Config(key) => quote! {
                #field_name: context.get_value::<_>(#key).unwrap()
            },
            FieldAttribute::Default => quote! {
                #field_name: Default::default()
            },
        }
    });

    let expanded = quote! {
        impl ::ioc_core::Factory for #name {
            fn create(context: &::ioc::Context) -> Self {
                #name {
                    #(#field_initializers),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
