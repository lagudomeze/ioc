use proc_macro::TokenStream;

use darling::{ast::NestedMeta, Error, FromMeta, Result};

use ioc_scan::{InitScanner, TypesMethodBuilder};

#[derive(Default, FromMeta)]
#[darling(default)]
struct LoadTypesParam {
    root: Option<String>,
}

pub fn load_types(input: TokenStream) -> Result<TokenStream> {
    let metas = NestedMeta::parse_meta_list(input.into())?;
    let param = LoadTypesParam::from_list(&metas)?;

    let root = param.root
        .as_ref()
        .map(String::as_str)
        .unwrap_or("src/main.rs");

    let expanded = InitScanner::default()
        .build_types_with(root)
        .map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}