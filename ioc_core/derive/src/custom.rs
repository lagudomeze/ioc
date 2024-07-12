use darling::{ast::NestedMeta, Error, FromMeta, Result};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ImplItem, ItemImpl, parse_quote, Path, Type};

use crate::bean::resolve_ioc_crate;

struct VerifyTraitIsBean<'a> {
    trait_: &'a Path,
    self_ty: &'a Type,
    ioc: &'a TokenStream,
}

impl ToTokens for VerifyTraitIsBean<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { trait_, ioc, self_ty } = *self;
        tokens.extend(quote!(
            const _: fn() = || {
                fn impl_trait_is_not_bean_spce<T: #ioc::BeanSpec, U: #trait_>(t: Option<T>, u: Option<U>, stop: bool)
                where
                {
                    if !stop {
                        impl_trait_is_not_bean_spce::<U, T>(u, t, true);
                    }
                }
                impl_trait_is_not_bean_spce::<#self_ty, #self_ty>(None, None, false);
            };
        ))
    }
}

#[derive(Debug, FromMeta)]
pub(crate) struct CustomBeanSpecImpl {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    ioc_crate: Option<Path>,
}

const CUSTOM_BEAN_IMPL_ERROR_INFO: &str =
    "BeanSpec custom impls only allow:
    0. `type Bean = ..` required
    1. `fn build`method required
    2. `fn name` method optional
    3. `fn drop` method optional
    Other items will be auto generated!";


impl CustomBeanSpecImpl {
    fn patch(&self, mut impl_block: ItemImpl) -> Result<TokenStream> {
        if let Some((_, trait_, _)) = &impl_block.trait_ {
            let fn_build: Ident = parse_quote!(build);
            let fn_name: Ident = parse_quote!(name);
            let fn_drop: Ident = parse_quote!(drop);
            let type_bean: Ident = parse_quote!(Bean);

            let ioc = &resolve_ioc_crate(&self.ioc_crate)?;
            let self_ty = &impl_block.self_ty;

            let mut impl_name = false;

            for item in impl_block.items.iter() {
                match item {
                    ImplItem::Fn(fun) => {
                        let ident = &fun.sig.ident;
                        if ident.eq(&fn_name) {
                            impl_name = true;
                        } else {
                            if !ident.eq(&fn_drop) && !ident.eq(&fn_build) {
                                return Err(Error::custom(CUSTOM_BEAN_IMPL_ERROR_INFO)
                                    .with_span(&ident));
                            }
                        }
                    }
                    ImplItem::Type(bean_type) => {
                        if !(bean_type.ident.eq(&type_bean)) {
                            return Err(Error::custom(CUSTOM_BEAN_IMPL_ERROR_INFO)
                                .with_span(&bean_type.ident));
                        } else {

                        }
                    }
                    other => {
                        return Err(Error::custom(CUSTOM_BEAN_IMPL_ERROR_INFO)
                            .with_span(&other));
                    }
                }
            }

            // add holder function
            impl_block.items.push(parse_quote! {
                fn holder<'a>() -> &'a std::sync::OnceLock<Self::Bean> {
                    static HOLDER: std::sync::OnceLock<<#self_ty as #ioc::BeanSpec>::Bean> = std::sync::OnceLock::new();
                    &HOLDER
                }
            });

            if !impl_name {
                if let Some(name) = &self.name {
                    impl_block.items.push(parse_quote! {
                        fn name() -> &'static str {
                            #name
                        }
                    });
                }
            }

            let verify = VerifyTraitIsBean {
                trait_,
                self_ty,
                ioc,
            };

            Ok(quote! {
                #verify

                #impl_block
            })
        } else {
            Err(Error::custom("Bean attribute can only be used on trait (ioc::BeanSpec) impls")
                .with_span(&impl_block))
        }
    }
}

pub(crate) fn expand(attr: TokenStream, impl_block: ItemImpl) -> Result<TokenStream> {
    let metas = NestedMeta::parse_meta_list(attr)?;

    let custom = CustomBeanSpecImpl::from_list(&metas)?;

    custom.patch(impl_block)
}