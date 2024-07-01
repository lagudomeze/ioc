use std::{
    env,
    fs::{self},
    path::PathBuf,
};

use syn::{parse_quote, Item, ItemStruct, Path, PathSegment};

use crate::{
    error::{Error, Result},
    module::{ModuleInfo, Scanner},
};

mod error;
mod module;

#[derive(Debug)]
pub struct InitScanner {
    types: Vec<Path>,
    out_dir: PathBuf,
}

impl InitScanner {
    fn out_dir<T: Into<PathBuf>>(self, out_dir: T) -> Self {
        Self {
            out_dir: out_dir.into(),
            ..self
        }
    }
}

impl Default for InitScanner {
    fn default() -> Self {
        Self {
            types: Vec::new(),
            out_dir: PathBuf::new(),
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
    fn build_init_method(self, file: &str) -> Result<()> {
        let scanner = self.scan(file)?;

        let types = &scanner.types;

        let dest_path = scanner.out_dir.join("init.rs");

        let func: Item = parse_quote! {

            pub fn all_types_with<F: ioc::BeanFamily>(ctx: F::Ctx) -> ioc::Result<F::Ctx> {
                use ioc::MethodType;
                #(let ctx = F::Method::<crate::#types>::run(ctx)?; )*
                Ok(ctx)
            }
        };

        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![func],
        };

        fs::write(&dest_path, &prettyplease::unparse(&file))?;

        Ok(())
    }
}

pub fn build_init_method() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR is not set");
    InitScanner::default()
        .out_dir(out_dir)
        .build_init_method("src/main.rs")
        .expect("error for scan main");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        InitScanner::default()
            .out_dir("./")
            .build_init_method("../examples/success/src/main.rs")
            .expect("error for scan main");
    }
}
