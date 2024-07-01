use std::{env, fs::{self}, path::PathBuf};

use quote::quote;
pub use syn;
use syn::{ItemStruct, Path, PathSegment};

pub use error::{Error, Result};

use crate::module::{ModuleInfo, Scanner};

mod error;
mod module;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Debug, Default)]
pub struct InitScanner {
    types: Vec<Path>,
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
    fn build_init_method(self) -> Result<()> {
        let scanner = self.scan_main()?;

        let types = &scanner.types;

        let out_dir = env::var_os("OUT_DIR")
            .expect("OUT_DIR is not set");
        let dest_path = PathBuf::from(&out_dir).join("init.rs");

        let code = quote! {
            pub fn all_types_with<F: ioc::BeanFamily>(ctx: F::Ctx) -> ioc::Result<F::Ctx> {
                use ioc::MethodType;
                #(let ctx = F::Method::<#types>::run(ctx)?; )*
                Ok(ctx)
            }
        };

        fs::write(&dest_path, &code.to_string())?;

        Ok(())
    }
}

pub fn build_init_method() {
    let scanner = InitScanner::default()
        .scan_main()
        .expect("error for scan main");

    scanner
        .build_init_method()
        .expect("error for build init method");

}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use quote::quote;

    use super::*;

    #[test]
    fn it_works() {
        let scanner = InitScanner::new("../examples/success/src/main.rs");

        let vec = scanner.types_with_derive("Bean").expect("exty");

        let mut file = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open("test.txt")
            .expect("");
        for path in vec {
            let code = quote! {#path}.to_string();
            file.write(format!("{code}\n").as_bytes()).expect("");
        }
    }
}
