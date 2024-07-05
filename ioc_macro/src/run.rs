use proc_macro::TokenStream;

use darling::{ast::NestedMeta, Error, FromMeta, util::PathList};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitBool, LitStr, parse_quote, Path, spanned::Spanned};

#[derive(Default, FromMeta)]
#[darling(default)]
struct RunParam {
    name: Option<LitStr>,
    dir: Option<LitStr>,
    profile: Option<LitStr>,
    debug: Option<LitBool>,
    crates: PathList,
}


pub(crate) fn generate(input: TokenStream) -> darling::Result<TokenStream> {
    let stream: TokenStream2 = input.into();

    let source_file = stream.span().source_file().path();

    let RunParam {
        debug,
        name,
        dir,
        profile,
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
        let mut crates = crates.iter().cloned().collect::<Vec<_>>();
        if source_file.ends_with("main.rs") {
            let lib = source_file
                .parent()
                .expect("`main.rs`'s parent should be in a directory")
                .join("lib.rs");
            if !lib.exists() {
                let path : Path = parse_quote! { crate };
                crates.push(path);
            } else {
                eprintln!("`lib.rs` exists, please use `import!` in `lib.rs` and import your crate with `crates` attribute in run!.");
            }
        } else {
            return Err(Error::custom("This macro can only be used in main.rs"));
        }

        crate::import::generate(&crates)?
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