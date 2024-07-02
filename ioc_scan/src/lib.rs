use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemStruct, Path, PathSegment};

use crate::{
    error::{Error, Result},
    module::{ModuleInfo, Scanner},
};

mod error;
mod module;

#[derive(Debug)]
pub struct InitScanner {
    types: Vec<Path>,
}

impl Default for InitScanner {
    fn default() -> Self {
        Self {
            types: Vec::new(),
        }
    }
}

impl Scanner for InitScanner {
    fn item_struct(&mut self, module_info: &ModuleInfo, i: &ItemStruct) -> Result<()> {
        for attr in i.attrs.iter() {
            if attr.path().is_ident("derive") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("Bean") {
                        let mut find_type = module_info.module_path.clone();
                        find_type.segments.push(PathSegment::from(i.ident.clone()));
                        self.types.push(find_type);
                    }
                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}

impl InitScanner {
    pub fn build_all_types_with(self, file: &str) -> Result<TokenStream> {
        let scanner = self.scan(file)?;

        let types = &scanner.types;

        Ok(quote! {
            pub fn all_types_with<F: ioc::BeanFamily>(ctx: F::Ctx) -> ioc::Result<F::Ctx> {
                use ioc::MethodType;
                #(let ctx = F::Method::<crate::#types>::run(ctx)?; )*
                Ok(ctx)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let code = InitScanner::default()
            .build_all_types_with("../examples/success/src/main.rs")?;

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
