use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{
    Attribute,
    Error,
    FnArg,
    ImplItem,
    ImplItemFn,
    ItemImpl,
    parse_macro_input,
    parse_quote,
    spanned::Spanned,
    TraitItemFn,
    Type,
    visit_mut,
    visit_mut::VisitMut,
};

mod load;

struct ApiTraitInfo {
    raw_type: Ident,
    api_trait: Ident,
}

struct ItemImplPatch {
    span: Span,
    api_trait_type: Option<ApiTraitInfo>,
    attrs_for_impl: Vec<Attribute>,
    methods: Vec<ImplItemFn>,
    methods_for_traits: Vec<TraitItemFn>,
}

impl VisitMut for ItemImplPatch {
    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        if !self.api_trait_type.is_some() {
            // get the raw type and api trait type
            self.api_trait_type = Some(Self::new_api_trait(&i.self_ty));

            // extract all no mvc attributes to self.attrs remove Openapi attributes
            i.attrs.retain(|attr| {
                if attr.path().is_ident("mvc") {
                    false
                } else if attr.path().is_ident("OpenApi") {
                    false
                } else {
                    self.attrs_for_impl.push(attr.clone());
                    true
                }
            });

            i.items.retain(|item| {
                if let ImplItem::Fn(item_fn) = item {
                    if Self::is_api_fn(item_fn) {
                        // remove this method for impl &'static Self
                        self.methods.push(item_fn.clone());
                        self.methods_for_traits.push(Self::build_trait_fn(item_fn));
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
        }


        visit_mut::visit_item_impl_mut(self, i);
    }
}

impl ItemImplPatch {
    fn new(span: Span) -> Self {
        Self {
            span,
            api_trait_type: None,
            attrs_for_impl: vec![],
            methods: vec![],
            methods_for_traits: vec![],
        }
    }

    fn new_api_trait(raw_type: &Type) -> ApiTraitInfo {
        let raw_type: Ident = {
            parse_quote!(#raw_type)
        };

        let api_trait = format_ident!("__{}Api__", raw_type);

        ApiTraitInfo {
            raw_type,
            api_trait,
        }
    }

    fn is_api_fn(item_fn: &ImplItemFn) -> bool {
        // async method && has oai attribute
        item_fn.sig.asyncness.is_some() && item_fn.attrs
            .iter()
            .any(|attr| attr.path().is_ident("oai"))
    }

    fn build_trait_fn(item_fn: &ImplItemFn) -> TraitItemFn {
        let mut sig = item_fn.sig.clone();

        for fn_args in sig.inputs.iter_mut() {
            if let FnArg::Typed(p) = fn_args {
                p.pat = parse_quote!(_);
            }
        }

        TraitItemFn {
            attrs: Default::default(),
            sig,
            default: None,
            semi_token: None,
        }
    }

    fn build(self) -> proc_macro2::TokenStream {
        let Self {
            api_trait_type,
            attrs_for_impl,
            methods,
            methods_for_traits,
            span,
        } = self;
        if let Some(ApiTraitInfo { raw_type, api_trait }) = api_trait_type {
            quote! {
                 trait #api_trait {
                    #(#methods_for_traits)*
                }

                #[simply_poem::OpenApi]
                #(#attrs_for_impl)*
                impl #api_trait for &'static #raw_type {
                    #(#methods)*
                }
            }
        } else {
            Error::new(span, "No api trait found").to_compile_error()
        }
    }
}

#[proc_macro_attribute]
pub fn mvc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    let mut patch = ItemImplPatch::new(input.span());

    patch.visit_item_impl_mut(&mut input);

    let impl_block = patch.build();


    let expanded = quote! {

        #impl_block

        #input
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn load_types(input: TokenStream) -> TokenStream {
    load::load_types(input)
        .unwrap_or_else(|err| err.write_errors().into())
}