use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{DeriveInput, Error, parse_macro_input, Type, TypeReference};
use syn::spanned::Spanned;

use bean::{FieldAttribute, TypeAttribute};

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

        ::ioc::run_app()?;
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Bean, attributes(bean, value, name))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("Bean derive macro only works with structs"),
    };

    let mut field_initializers = vec![];

    for field in fields.iter() {
        let span = field.ty.span();

        let attr = match FieldAttribute::from_attributes(&field.attrs) {
            Ok(attr) => attr,
            Err(err) => return err.to_compile_error().into(),
        };
        let initializer = match attr {
            FieldAttribute::Ref(Some(name)) => quote! { ctx.get_or_init::<#name>()? },
            FieldAttribute::Ref(None) => {
                if let Type::Reference(TypeReference { ref elem, .. }) = field.ty {
                    quote! { ctx.get_or_init::<#elem>()? }
                } else {
                    return Error::new(span, "Only &'s BeanType is support! Here s could be static!")
                        .to_compile_error()
                        .into()
                }
            },
            FieldAttribute::Config(key) => quote! { ctx.get_config::<_>(#key)? },
            FieldAttribute::Default => quote! { Default::default() },
        };

        let field_initializer = if let Some(field_name) = &field.ident {
            quote! { #field_name : #initializer }
        } else {
            initializer
        };
        field_initializers.push(field_initializer);
    }

    let type_attr = match TypeAttribute::from_attributes(&input.attrs) {
        Ok(attr) => attr,
        Err(err) => return err.to_compile_error().into(),
    };

    let bean_factory_impl = quote! {
        impl ::ioc_core::BeanFactory for #name {
            type Bean = #name;

            fn build(ctx: &mut ::ioc_core::Context) -> ioc_core::Result<Self::Bean> {
                Ok(Self::Bean {
                    #(#field_initializers),*
                })
            }
        }
    };

    let bean_name = type_attr.name.clone().unwrap_or_else(|| name.to_string());

    let bean_impl = quote! {
        impl ::ioc_core::Bean for #name {
            type Type = Self;
            type Factory = Self;

            fn name() -> &'static str {
                #bean_name
            }

            fn holder<'a>() -> &'a std::sync::OnceLock<Self::Type> {
                static HOLDER: std::sync::OnceLock<#name> = std::sync::OnceLock::new();
                &HOLDER
            }
        }
    };

    let register_method = Ident::new(&format!("__register_bean_{}", bean_name), Span::call_site());

    let bean_register = quote! {
        #[allow(non_snake_case)]
        #[::linkme::distributed_slice(::ioc::BEAN_COLLECTOR)]
        fn #register_method(ctx: &mut ::ioc_core::Context) -> ::ioc_core::Result<()>  {
            ctx.get_or_init::<#name>()?;
            Ok(())
        }
    };

    let expanded = quote! {

        #bean_factory_impl

        #bean_impl

        #bean_register
    };

    TokenStream::from(expanded)
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}