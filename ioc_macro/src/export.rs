use proc_macro::TokenStream;
use std::path::PathBuf;
use proc_macro2::TokenStream as TokenStream2;

use darling::{
    ast::NestedMeta,
    Error,
    FromMeta,
    Result,
    util::PathList
};
use syn::spanned::Spanned;
#[cfg(feature = "mvc")]
use ioc_mvc_scan::Mvcs;
use ioc_scan::{Beans, export};
#[cfg(feature = "mvc")]
use ioc_scan::Transport;

#[derive(Default, FromMeta)]
#[darling(default)]
struct ExportParam {
    root: Option<PathBuf>,
    deps: PathList,
}

pub fn generate(input: TokenStream) -> Result<TokenStream> {
    let stream : TokenStream2 = input.into();

    let source_file = stream.span().source_file().path();

    let metas = NestedMeta::parse_meta_list(stream)?;
    let param = ExportParam::from_list(&metas)?;

    let root = param.root.unwrap_or(source_file);
    let transport = Beans::new()
        .deps(&param.deps);

    #[cfg(feature = "mvc")]
    let transport = transport.join(Mvcs::default());

    let expanded = export(transport, root)
        .map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}