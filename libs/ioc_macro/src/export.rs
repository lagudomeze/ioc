use proc_macro::{Span, TokenStream};
use std::path::PathBuf;

use darling::{ast::NestedMeta, util::PathList, Error, FromMeta, Result};

#[cfg(feature = "mvc")]
use ioc_core_scan::Transport;
use ioc_core_scan::{export, Beans};
#[cfg(feature = "mvc")]
use ioc_mvc_scan::Mvcs;

#[derive(Default, FromMeta)]
#[darling(default)]
struct ExportParam {
    root: Option<PathBuf>,
    deps: PathList,
}

pub fn generate(input: TokenStream) -> Result<TokenStream> {
    let metas = NestedMeta::parse_meta_list(input.into())?;
    let param = ExportParam::from_list(&metas)?;

    let source_file = Span::call_site().source_file().path();
    let root = param.root.unwrap_or(source_file);
    let transport = Beans::new().deps(&param.deps);

    #[cfg(feature = "mvc")]
    let transport = transport.join(Mvcs::default());

    let expanded = export(transport, root).map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}
