use proc_macro::TokenStream;

use darling::{ast::NestedMeta, Error, FromMeta, util::PathList};
use proc_macro2::TokenStream as TokenStream2;
use syn::{LitBool, parse_quote};

#[cfg(feature = "mvc")]
use ioc_mvc_scan::Mvcs;
use ioc_scan::{Beans, Transport};

#[derive(Default, FromMeta)]
#[darling(default)]
struct RunParam {
    use_crate: Option<LitBool>,
    crates: PathList,
}

pub(crate) fn generate(input: TokenStream) -> darling::Result<TokenStream> {
    let stream: TokenStream2 = input.into();

    let RunParam {
        use_crate,
        crates
    } = {
        let metas = NestedMeta::parse_meta_list(stream)?;
        RunParam::from_list(&metas)?
    };

    let expanded = {
        let crates = {
            let mut crates = crates.to_vec();
            if let Some(lit) = use_crate {
                if lit.value {
                    crates.push(parse_quote!(crate));
                }
            } else {
                crates.push(parse_quote!(crate));
            }
            crates
        };

        let transport = Beans::new()
            .deps(&crates);

        #[cfg(feature = "mvc")]
        let transport = transport.join(Mvcs::default());

        transport
            .import(&crates)
            .map_err(|err| Error::custom(err))?
    };

    Ok(expanded.into())
}