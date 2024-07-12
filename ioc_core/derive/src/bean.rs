use darling::{
    ast::Data,
    Error,
    FromDeriveInput,
    FromField,
    FromMeta,
    Result,
};
use darling::ast::Style;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Path, Type};

pub(crate) fn resolve_ioc_crate(ioc_crate: &Option<Path>) -> Result<TokenStream> {
    if let Some(ioc_crate) = ioc_crate {
        return Ok(quote! { #ioc_crate });
    } else {
        use proc_macro_crate::{crate_name, FoundCrate};
        match crate_name("ioc") {
            Ok(FoundCrate::Itself) => {
                Ok(quote! { crate })
            }
            Ok(FoundCrate::Name(name)) => {
                let ident = format_ident!("{}", name);
                Ok(quote! { #ident })
            }
            Err(err) => {
                Err(Error::custom(err))
            }
        }
    }
}

#[derive(Debug, FromMeta)]
#[darling(default, rename_all = "snake_case")]
pub enum Inject {
    Bean,
    BeanWith(Path),
    Config(String),
    Default,
}

impl Default for Inject {
    fn default() -> Self {
        Inject::Default
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(inject))]
pub struct BeanField {
    ty: Type,
    ident: Option<Ident>,
    #[darling(flatten)]
    inject: Inject,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(bean))]
pub(crate) struct BeanSpecStruct {
    /// The struct ident.
    ident: Ident,

    /// Receives the body of the struct or enum. We don't care about
    /// struct fields because we previously told darling we only accept structs.
    data: Data<(), BeanField>,

    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    ioc_crate: Option<Path>,
}

struct FieldInitializer<'a>(&'a BeanField);

impl<'a> From<&'a BeanField> for FieldInitializer<'a> {
    fn from(value: &'a BeanField) -> Self {
        Self(value)
    }
}

impl ToTokens for FieldInitializer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let BeanField {
            ref ty,
            ref ident,
            ref inject,
        } = self.0;

        let initializer = match inject {
            Inject::Bean => {
                if let Type::Reference(type_ref)= ty {
                    let ty = type_ref.elem.as_ref();
                    quote! { ctx.get_or_init::<#ty>()? }
                } else {
                    quote! { ctx.get_or_init::<#ty>()? }
                }
            }
            Inject::BeanWith(ty) => {
                quote! { ctx.get_or_init::<#ty>()? }
            }
            Inject::Config(key) => {
                quote! { ctx.get_config::<_>(#key)?}
            }
            Inject::Default => {
                quote! { Default::default() }
            }
        };

        if let Some(field_name) = ident {
            tokens.extend(quote! { #field_name : #initializer })
        } else {
            tokens.extend(initializer)
        }
    }
}

struct BuildMethod<'a> {
    ident: &'a Ident,
    fields: &'a Data<(), BeanField>,
    ioc: &'a TokenStream,
}

impl BuildMethod<'_> {
    fn generate(&self) -> Result<TokenStream> {
        let Self { ident, fields, ioc } = *self;

        if !fields.is_struct() {
            return Err(Error::unsupported_shape("only struct is supported")
                .with_span(ident));
        } else {
            let struct_fields = fields
                .as_ref()
                .take_struct()
                .expect("not here!");

            let field_initializers = struct_fields
                .iter()
                .cloned()
                .map(FieldInitializer::from);

            let initializers = quote! {
                #(#field_initializers),*
            };

            let initializers = match struct_fields.style {
                Style::Tuple => {
                    quote! { Self(
                        #initializers
                    ) }
                }
                Style::Struct => {
                    quote! { Self{
                        #initializers
                    } }
                }
                Style::Unit => {
                    quote! { Self }
                }
            };
            Ok(quote! {
                fn build<I>(ctx: &mut I) -> #ioc::Result<Self::Bean>
                where
                    I: #ioc::InitContext {
                    Ok(#initializers)
                }
            })
        }
    }
}

impl BeanSpecStruct {
    pub(crate) fn generate(&self) -> Result<TokenStream> {
        let Self {
            ref ident,
            ref data,
            ref name,
            ref ioc_crate,
        } = *self;

        let ioc = resolve_ioc_crate(ioc_crate)?;

        let build_method = BuildMethod {
            ident,
            fields: data,
            ioc: &ioc,
        };

        let build_method = build_method.generate()?;

        let name = if let Some(name) = name {
            quote! { #name }
        } else {
            quote! { stringify!(#ident) }
        };


        Ok(quote! {
            impl #ioc::BeanSpec for #ident {
                type Bean = Self;

                fn name() -> &'static str {
                    #name
                }

                #build_method

                fn drop(bean: &Self::Bean) {
                    // drop
                }

                fn holder<'a>() -> &'a std::sync::OnceLock<Self::Bean> {
                    use std::sync::OnceLock;
                    static HOLDER: OnceLock<#ident> = OnceLock::new();
                    &HOLDER
                }
            }
        })
    }
}

impl ToTokens for BeanSpecStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.generate() {
            Ok(tt) => {
                tokens.extend(tt);
            }
            Err(err) => {
                tokens.extend(err.write_errors());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use syn::{parse_quote, parse_str};

    use super::*;

    #[test]
    fn it_works() {
        let input = r#"
            #[derive(Bean)]
            #[bean(ioc_crate = "ioc")]
            pub struct LogPatcher(
                #[inject(default)]
                Handle<EnvFilter, Formatter>
            );
        "#;

        let parsed = parse_str(input).unwrap();
        let result = BeanSpecStruct::from_derive_input(&parsed);
        if let Err(err) = result {
            println!("err 0:{}", err.write_errors().to_string());
            return;
        }
        let bean_struct = result.unwrap();

        if let Err(err) = bean_struct.generate() {
            println!("err 1:{}", err.write_errors().to_string());
            return;
        }

        let file : syn::File = parse_quote!( #bean_struct);

        println!("{}", prettyplease::unparse(&file));

    }

    #[test]
    fn construct() {
        let input = r#"
            #[derive(Bean)]
            #[bean(ioc_crate = "ioc", construct = "Init")]
            pub struct LogPatcher(
                #[inject(default)]
                Handle<EnvFilter, Formatter>
            );
        "#;

        let parsed = parse_str(input).unwrap();
        let result = BeanSpecStruct::from_derive_input(&parsed);
        if let Err(err) = result {
            println!("err 0:{}", err.write_errors().to_string());
            return;
        }
        let bean_struct = result.unwrap();

        if let Err(err) = bean_struct.generate() {
            println!("err 1:{}", err.write_errors().to_string());
            return;
        }

        let file : syn::File = parse_quote!( #bean_struct);

        println!("{}", prettyplease::unparse(&file));

    }

    #[test]
    fn test_inject_config() {
        let input = r#"
            #[derive(Bean)]
            #[bean(ioc_crate = "ioc")]
            pub struct WebConfig {
                #[inject(config = "web.addr")]
                addr: String,
                #[inject(config = "web.graceful_shutdown_timeout")]
                shutdown_timeout: Duration,
                #[inject(config = "web.tracing")]
                tracing: bool,
            }
        "#;

        let parsed = parse_str(input).unwrap();
        let result = BeanSpecStruct::from_derive_input(&parsed);
        if let Err(err) = result {
            println!("err 0:{}", err.write_errors().to_string());
            return;
        }
        let bean_struct = result.unwrap();

        if let Err(err) = bean_struct.generate() {
            println!("err 1:{}", err.write_errors().to_string());
            return;
        }

        let file : syn::File = parse_quote!( #bean_struct);

        println!("{}", prettyplease::unparse(&file));

    }
}