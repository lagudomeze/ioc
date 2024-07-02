use proc_macro::TokenStream;

use darling::{ast::NestedMeta, Error, FromMeta, Result};
use proc_macro2::Ident;
use quote::quote;
use syn::{ItemImpl, parse_quote, Path, PathSegment};

use ioc_scan::{InitScanner, ModuleInfo, Scanner, TypesMethodBuilder};

#[derive(Default, FromMeta)]
#[darling(default)]
struct LoadTypesParam {
    root: Option<String>,
}

#[derive(Debug, Default)]
pub struct MvcInitScanner {
    types: Vec<Path>,
}

impl Scanner for MvcInitScanner {
    fn item_impl(&mut self, module_info: &ModuleInfo, i: &ItemImpl) -> ioc_scan::Result<()> {
        for attr in i.attrs.iter() {
            if attr.path().is_ident("mvc") {
                let raw_type: Ident = {
                    let raw_type = &i.self_ty;
                    parse_quote!(#raw_type)
                };

                let mut find_type = module_info.module_path.clone();
                find_type.segments.push(PathSegment::from(raw_type));
                self.types.push(find_type);
            }
        }
        Ok(())
    }
}

impl TypesMethodBuilder for MvcInitScanner {
    fn build_types_with(self, file: &str) -> ioc_scan::Result<proc_macro2::TokenStream> {
        let scanner = self.scan(file)?;

        let types = &scanner.types;

        Ok(quote! {
            pub fn open_api_service(title: impl Into<String>, version: impl Into<String>) ->
            simply_poem::OpenApiService<(
                    (#(&'static crate::#types,)*)
            ),()> {
                simply_poem::OpenApiService::new(
                    (#(crate::#types::get(),)*),
                    title,
                    version
                )
            }
        })
    }
}

pub fn load_types(input: TokenStream) -> Result<TokenStream> {
    let metas = NestedMeta::parse_meta_list(input.into())?;
    let param = LoadTypesParam::from_list(&metas)?;

    let root = param.root
        .as_ref()
        .map(String::as_str)
        .unwrap_or("src/main.rs");

    let expanded = InitScanner::default()
        .join(MvcInitScanner::default())
        .build_types_with(root)
        .map_err(|err| Error::custom(err))?;

    Ok(expanded.into())
}