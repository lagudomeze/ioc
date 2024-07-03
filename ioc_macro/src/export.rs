use proc_macro::TokenStream;

use darling::{ast::NestedMeta, Error, FromMeta, Result};
use syn::LitStr;

#[cfg(feature = "mvc")]
use ioc_mvc_scan::Mvcs;

#[cfg(feature = "mvc")]
use ioc_scan::Transport;

use ioc_scan::{Beans, export};

#[derive(Default, FromMeta)]
#[darling(default)]
struct ExportParam {
    root: Option<String>,
    deps: Vec<LitStr>,
}

pub fn generate(input: TokenStream) -> Result<TokenStream> {
    let metas = NestedMeta::parse_meta_list(input.into())?;
    let param = ExportParam::from_list(&metas)?;

    let root = param.root
        .as_ref()
        .map(String::as_str)
        .unwrap_or("src/main.rs");

    let vec = param.deps.iter().map(|dep| dep.value()).collect::<Vec<_>>();

    let transport = Beans::with_deps(&vec);

    #[cfg(feature = "mvc")]
    let transport = transport.join(Mvcs::default());

    let expanded = export(transport, root)
        .map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}