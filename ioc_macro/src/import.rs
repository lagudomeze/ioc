use proc_macro::TokenStream;

use darling::{
    ast::NestedMeta,
    Error,
    FromMeta,
    Result,
};
use syn::LitStr;
#[cfg(feature = "mvc")]
use ioc_mvc_scan::Mvcs;
use ioc_scan::{Beans, Transport};

#[derive(Default, FromMeta)]
#[darling(default)]
struct ImportParam {
    crates: Vec<LitStr>,
}

pub(crate) fn generate(input: TokenStream) -> Result<TokenStream> {
    let metas = NestedMeta::parse_meta_list(input.into())?;
    let param = ImportParam::from_list(&metas)?;

    let transport = Beans::default();

    #[cfg(feature = "mvc")]
    let transport = transport.join(Mvcs::default());

    let vec = param.crates.iter().map(|dep| dep.value()).collect::<Vec<_>>();
    let expanded = transport.import(&vec)
        .map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}