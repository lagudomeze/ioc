use proc_macro::TokenStream;

use darling::{ast::NestedMeta, FromMeta, util::PathList};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitBool, LitStr};

#[derive(Default, FromMeta)]
#[darling(default)]
struct RunParam {
    name: Option<LitStr>,
    dir: Option<LitStr>,
    profile: Option<LitStr>,
    debug: Option<LitBool>,
    use_crate: Option<LitBool>,
    crates: PathList,
}


pub(crate) fn generate(input: TokenStream) -> darling::Result<TokenStream> {
    let stream: TokenStream2 = input.into();

    let RunParam {
        debug,
        name,
        dir,
        profile,
        use_crate,
        crates
    } = {
        let metas = NestedMeta::parse_meta_list(stream)?;
        RunParam::from_list(&metas)?
    };

    let log_init = {
        if let Some(v) = debug {
            if v.value {
                quote! {
                    __private::LogOptions::new().debug(true).init()?;
                }
            } else {
                quote! {
                    __private::LogOptions::new().init()?;
                }
            }
        } else {
            quote! {
              __private::LogOptions::new().init()?;
            }
        }
    };

    let config_load = {
        let name = name.iter();
        let dir = dir.iter();
        let profile = profile.iter();

        quote! {
            let pkg_name = env!("CARGO_PKG_NAME");

            let config = __private::AppConfigLoader::new()
                .name(pkg_name)
                #(.name(#name))*
                #(.dir(#dir))*
                #(.profile(#profile))*
                .load()?;


            let mut ctx = __private::Context::new(config);
        }
    };

    let import_code = {
        let use_crate = use_crate
            .as_ref()
            .map(LitBool::value)
            .unwrap_or(true);

        crate::import::generate(&crates, use_crate)?
    };

    let expanded = quote! {
        {
            use ioc::__private;

            #log_init

            #config_load

            __private::pre_init(&mut ctx)?;

            #import_code

            ctx.complete()
        }
    };

    Ok(expanded.into())
}