use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{DeriveInput, Error, LitStr, parse_macro_input, Token, Type, TypeReference};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

use bean::{FieldAttribute, TypeAttribute};

mod bean;
mod scan;

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

struct AppConfig {
    name: proc_macro2::TokenStream,
    dir: proc_macro2::TokenStream,
    profile: proc_macro2::TokenStream,
}

impl AppConfig {
    fn build(self) -> proc_macro2::TokenStream {
        let Self { name, dir, profile } = self;

        quote! {
            {
                use cfg_rs::{Configuration, init_cargo_env};
                init_cargo_env!();

                Configuration::with_predefined_builder()
                    .set_cargo_env(init_cargo_env())
                    .set_name(#name)
                    .set_dir(#dir)
                    .set_profile(#profile)
                    .init()
                    .map_err(::ioc_core::IocError::from)?
                    .into()
            }
        }
    }
}

struct KeyValue {
    key: Ident,
    _eq_token: Token![=],
    value: LitStr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse::<_>()?,
            _eq_token: input.parse::<_>()?,
            value: input.parse::<_>()?,
        })
    }
}

impl Parse for AppConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key_values = input.parse_terminated(KeyValue::parse, Token![,])?;

        let mut name = quote! { env!("CARGO_PKG_NAME") };
        let mut dir = quote! { "." };
        let mut profile = quote! { "prod"};
        for kv in key_values.iter() {
            if kv.key == "name" {
                let value = kv.value.value();
                name = quote! { #value };
            } else if kv.key == "dir" {
                let value = kv.value.value();
                dir = quote! { #value };
            } else if kv.key == "profile" {
                let value = kv.value.value();
                profile = quote! { #value };
            } else {
                eprintln!("key:{} value:{} is ignored ", kv.key, kv.value.value());
            }
        }

        Ok(Self { name, dir, profile })
    }
}

#[proc_macro]
pub fn run(input: TokenStream) -> TokenStream {
    let preload_mods = preload_mods();

    let config = parse_macro_input!(input as AppConfig).build();

    let expanded = quote! {

        #preload_mods

        let config = #config;

        ::ioc::run_app(config)?;
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Bean, attributes(inject, value, bean, name, custom_factory))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("Bean derive macro only works with structs"),
    };

    let mut custom_factory = false;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("custom_factory") {
            custom_factory = true;
            break;
        }
    }

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
                    return Error::new(
                        span,
                        "Only &'s BeanType is support! Here s could be static!",
                    )
                    .to_compile_error()
                    .into();
                }
            }
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

    let bean_factory_impl = if custom_factory {
        quote! {}
    } else {
        quote! {
            impl ::ioc_core::BeanFactory for #name {
                type Bean = #name;

                fn build(ctx: &mut ::ioc_core::Context) -> ioc_core::Result<Self::Bean> {
                    Ok(Self::Bean {
                        #(#field_initializers),*
                    })
                }
            }
        }
    };

    let bean_name = type_attr.name.clone().unwrap_or_else(|| name.to_string());

    let bean_impl = quote! {
        impl ::ioc_core::Bean for #name {
            type Type = <#name as BeanFactory>::Bean;
            type Factory = Self;

            fn name() -> &'static str {
                #bean_name
            }

            fn holder<'a>() -> &'a std::sync::OnceLock<Self::Type> {
                static HOLDER: std::sync::OnceLock<<#name as BeanFactory>::Bean> = std::sync::OnceLock::new();
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
