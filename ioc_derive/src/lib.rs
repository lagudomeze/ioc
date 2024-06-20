use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{DeriveInput, Error, FieldValue, parse_macro_input, spanned::Spanned, Type, TypeReference};
use syn::Member::Named;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;

use bean::{FieldAttribute, TypeAttribute};

mod bean;
mod scan;

#[proc_macro]
pub fn preload_mods(_: TokenStream) -> TokenStream {
    use scan::CargoToml;
    let toml = CargoToml::current();

    let mut mod_names = vec![];
    let mut mods = vec![];

    for name in toml.mod_names() {
        // mod name may contain "-", in `use` statement need replace it to
        let mod_name = name.replace("-", "_");
        mods.push(format_ident!("{}", mod_name));
        mod_names.push(mod_name);
    }

    let test = format!("{mod_names:?}");

    let expanded = quote! {
        ioc::log::info!("preload mods: {}", #test);
        #( use #mods; )*
    };

    TokenStream::from(expanded)
}

/// See module level documentation for more information.
#[proc_macro_derive(Bean, attributes(inject, value, name, custom_factory))]
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
            impl ::ioc::BeanFactory for #name {
                type Bean = Self;

                fn build(ctx: &mut ::ioc::Context) -> ioc::Result<Self::Bean> {
                    Ok(Self::Bean {
                        #(#field_initializers),*
                    })
                }
            }
        }
    };

    let bean_name = type_attr.name.clone().unwrap_or_else(|| name.to_string());

    let bean_type = if custom_factory {
        quote! { <#name as BeanFactory>::Bean }
    } else {
        quote! { #name }
    };

    let bean_impl = quote! {
        impl ::ioc::Bean for #name {

            fn name() -> &'static str {
                #bean_name
            }

            fn holder<'a>() -> &'a std::sync::OnceLock<Self::Bean> {
                static HOLDER: std::sync::OnceLock<#bean_type> = std::sync::OnceLock::new();
                &HOLDER
            }
        }
    };

    let register_method = Ident::new(&format!("__register_bean_{}", bean_name), Span::call_site());

    let bean_register = quote! {
        #[allow(non_snake_case)]
        #[::ioc::distributed_slice(::ioc::BEAN_COLLECTOR)]
        #[linkme(crate = ::ioc::linkme)]
        fn #register_method(ctx: &mut ::ioc::Context) -> ::ioc::Result<()>  {
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


struct LoadConfigParam {
    fields: Punctuated<FieldValue, Comma>,
}

impl Parse for LoadConfigParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            fields: Punctuated::parse_terminated(input)?
        })
    }
}

#[proc_macro]
pub fn load_config(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as LoadConfigParam);

    let mut fields = vec![];
    let mut name = Some(quote! { env!("CARGO_PKG_NAME") });
    let mut dir = Some(quote! {"."});
    let mut profile = Some(quote! {"prod"});

    for field in params.fields {
        let value = field.expr;
        if let Named(key) = field.member {
            fields.push(quote! { #key : #value });
            if key.eq("name") {
                name = None;
            } else if key.eq("dir") {
                dir = None;
            } else if key.eq("profile") {
                profile = None;
            }
        }
    }

    if let Some(value) = name {
        fields.push(quote! { name : #value });
    }

    if let Some(value) = dir {
        fields.push(quote! { dir : #value });
    }

    if let Some(value) = profile {
        fields.push(quote! { profile : #value });
    }

    let expanded = quote! {
        {
            use ioc::AppConfigLoader;
            AppConfigLoader {
                #(#fields,)*
            }
        }
    };

    expanded.into()
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
