use bean::{FieldAttribute, TypeAttribute};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Ident};

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

#[proc_macro_derive(Bean, attributes(bean_ref, value, name))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("Bean derive macro only works with structs"),
    };

    let mut field_initializers = vec![];
    let mut dependenices = vec![];
    for field in fields.iter() {
        let field_name = &field.ident;
        let attr = FieldAttribute::from_attributes(&field.attrs).expect("");
        let field_initializer = match attr {
            FieldAttribute::Ref(Some(name)) => {
                let ty: &syn::Type = &field.ty;
                dependenices.push(quote! {
                    ::ioc_core::BeanQuery::named_from_holder::<#ty>(#name)
                });
                quote! {
                    #field_name: ctx.make_ref::<_>(Some(#name)).unwrap()
                }
            }
            FieldAttribute::Ref(None) => {
                let ty: &syn::Type = &field.ty;
                dependenices.push(quote! {
                    ::ioc_core::BeanQuery::from_holder::<#ty>()
                });
                quote! {
                    #field_name: ctx.make_ref::<_>(None).unwrap()
                }
            }
            FieldAttribute::Config(key) => quote! {
                #field_name: ctx.make_value::<_>(#key).unwrap()
            },
            FieldAttribute::Default => quote! {
                #field_name: Default::default()
            },
        };
        field_initializers.push(field_initializer);
    }

    let type_attr = TypeAttribute::from_attributes(&input.attrs).expect("");

    let bean_impl = {
        if let Some(bean_name) = type_attr.name {
            quote! {
                impl ::ioc_core::Bean for #name {
                    fn name() -> &'static str {
                        #bean_name
                    }

                    fn dependencies() -> Vec<::ioc_core::BeanQuery> {
                        vec![
                            #(#dependenices),*
                        ]
                    }
                }
            }
        } else {
            quote! {
                impl ::ioc_core::Bean for #name {
                    
                    fn dependencies() -> Vec<::ioc_core::BeanQuery> {
                        vec![
                            #(#dependenices),*
                        ]
                    }
                }
            }
        }
    };

    let register_method = Ident::new(&format!("__register_bean_{}", name), Span::call_site());

    let bean_register = quote! {
        #[allow(non_snake_case)]
        #[::linkme::distributed_slice(::ioc::BEAN_COLLECTOR)]
        fn #register_method(ctx: &mut ::ioc::BeanRegistry) {
            ctx.register::<#name>(module_path!());
        }
    };

    let expanded = quote! {
        #bean_register

        #bean_impl

        impl ::ioc_core::BeanFactory for #name  {
            type T = Self;

            unsafe fn init_in_place<C>(ptr: std::ptr::NonNull<Self::T>, ctx: &C)
            where
                C: ::ioc_core::BeanRetriever,
            {
                ptr.as_ptr().write(#name {
                    #(#field_initializers),*
                })
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
