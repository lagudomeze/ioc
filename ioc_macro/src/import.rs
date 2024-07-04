use proc_macro::TokenStream;

use darling::{
    ast::NestedMeta,
    Error,
    FromMeta,
    Result,
    util::PathList
};
use syn::Path;
#[cfg(feature = "mvc")]
use ioc_mvc_scan::Mvcs;
use ioc_scan::{Beans, Transport};

#[derive(Default, FromMeta)]
#[darling(default)]
struct ImportParam {
    crates: PathList,
    self_crate: Option<Path>
}

pub(crate) fn generate(input: TokenStream) -> Result<TokenStream> {
    let metas = NestedMeta::parse_meta_list(input.into())?;
    let param = ImportParam::from_list(&metas)?;

    let transport = Beans::new()
        .self_crate(param.self_crate)
        .deps(&param.crates);

    #[cfg(feature = "mvc")]
    let transport = transport.join(Mvcs::default());

    let expanded = transport.import(&param.crates)
        .map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}