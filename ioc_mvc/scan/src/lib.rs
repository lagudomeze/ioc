use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemImpl, parse_quote, Path, PathSegment};

use ioc_scan::{ModuleInfo, Result, Scanner, Transport};

#[derive(Default, FromMeta)]
#[darling(default)]
struct LoadTypesParam {
    root: Option<String>,
}

#[derive(Debug, Default)]
pub struct Mvcs {
    types: Vec<Path>,
}

impl Scanner for Mvcs {
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

impl Transport for Mvcs {
    fn export(self) -> Result<TokenStream> {
        let types = &self.types;

        Ok(quote! {
            // here only support current crate mvc scan
            pub fn all_mvcs<T>(api:T) -> impl poem_openapi::OpenApi
                where T: poem_openapi::OpenApi {
                #(let api = api.join(crate::#types::get());)*
                api
            }
        })
    }

    fn import(self, crates: &[String]) -> Result<TokenStream> {
        Ok(quote! {
            let api = crate::all_mvcs(());
            #(let api = #crates::all_mvcs(api); )*

            let name = std::env!("CARGO_PKG_NAME");
            let version = std::env!("CARGO_PKG_VERSION");

            ioc::run_mvc(api, name, version)?;
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;
    use ioc_scan::export;
    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let code = export(Mvcs::default(), "../../examples/success/src/main.rs")?;


        let func = parse_quote!( #code );

        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![func],
        };

        println!("{}", prettyplease::unparse(&file));

        Ok(())
    }
}