use darling::{
    Error,
    FromMeta,
    Result,
    util::PathList
};
use proc_macro2::TokenStream as TokenStream2;
use syn::Path;

#[cfg(feature = "mvc")]
use ioc_mvc_scan::Mvcs;
use ioc_scan::{Beans, Transport};

#[derive(Default, FromMeta)]
#[darling(default)]
struct ImportParam {
    crates: PathList,
    self_crate: Option<Path>,
}

pub(crate) fn generate(crates: &[Path], use_crate: bool) -> Result<TokenStream2> {

    let transport = Beans::new()
        .deps(&crates);

    #[cfg(feature = "mvc")]
    let transport = transport.join(Mvcs::default());

    let expanded = transport.import(crates, use_crate)
        .map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}