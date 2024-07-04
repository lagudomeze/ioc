use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, Error, parse_macro_input, spanned::Spanned, Type, TypeReference};

use bean::{FieldAttribute, TypeAttribute};

mod bean;
mod init;

/// See module level documentation for more information.
#[proc_macro_derive(Bean, attributes(inject, value, name, custom_factory))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("Bean derive derive only works with structs"),
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
            impl ioc::BeanFactory for #name {
                type Bean = Self;

                fn build(ctx: &mut ioc::Context) -> ioc::Result<Self::Bean> {
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
        impl ioc::Bean for #name {

            fn name() -> &'static str {
                #bean_name
            }

            fn holder<'a>() -> &'a std::sync::OnceLock<Self::Bean> {
                static HOLDER: std::sync::OnceLock<#bean_type> = std::sync::OnceLock::new();
                &HOLDER
            }
        }
    };

    let expanded = quote! {

        #bean_factory_impl

        #bean_impl
    };

    TokenStream::from(expanded)
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
