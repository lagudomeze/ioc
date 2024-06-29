use std::{fs::read_to_string, io, path::Path, path::PathBuf, result::Result as StdResult};

pub use syn;
use syn::{
    fold::{self, Fold},
    punctuated::Punctuated,
    spanned::Spanned,
    File, Ident, ItemMod, ItemStruct, Path as PathType, PathSegment,
};
use thiserror::Error;
use thiserror::__private::AsDisplay;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: `{0}`")]
    IoError(#[from] io::Error),
    #[error("syn error: `{0}`")]
    SynError(#[from] syn::Error),
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub struct Scanner {
    root: PathBuf,
}

impl Scanner {
    pub fn new<P>(root: P) -> Self
    where
        P: AsRef<str>,
    {
        Self {
            root: PathBuf::from(root.as_ref()),
        }
    }
}

struct ScanFold<'a> {
    types: Vec<PathType>,
    module_path: PathType,
    derive_trait: &'a Ident,
    file_path: &'a Path,
}

impl<'a> ScanFold<'a> {
    fn new(derive_trait: &'a Ident, file_path: &'a Path) -> Self {
        Self {
            types: Vec::new(),
            module_path: PathType {
                leading_colon: None,
                segments: Punctuated::new(),
            },
            derive_trait,
            file_path,
        }
    }
}

fn sub_module<'a, 'b>(
    source: &'a ScanFold,
    sub_module: &'b Ident,
    file_path: &'b Path,
) -> ScanFold<'b>
where
    'a: 'b,
{
    let mut module_path = source.module_path.clone();
    module_path
        .segments
        .push(PathSegment::from(sub_module.clone()));
    ScanFold {
        types: Vec::new(),
        module_path,
        derive_trait: source.derive_trait,
        file_path,
    }
}
fn sub_module_path(parent: &Path, sub_module: &Ident) -> PathBuf {
    let mod_dir_path = parent.join(format!("{}/mod.rs", sub_module));
    let mod_file_path = parent.join(format!("{}.rs", sub_module));
    if mod_dir_path.exists() && mod_dir_path.is_file() {
        mod_dir_path
    } else if mod_file_path.exists() && mod_file_path.is_dir() {
        mod_file_path
    } else {
        let segment = sub_module.to_string();
        panic!(
            "there is nether {}/mod.rs nor {}.rs under {} ",
            segment,
            segment,
            parent.as_display()
        )
    }
}

impl<'a> Fold for ScanFold<'a> {
    fn fold_file(&mut self, i: File) -> File {
        fold::fold_file(self, i)
    }

    fn fold_item_mod(&mut self, i: ItemMod) -> ItemMod {
        let sub_module_ident = &i.ident.clone();
        if i.content.is_none() {
            let path = sub_module_path(self.file_path, sub_module_ident);
            let mut fold = sub_module(self, sub_module_ident, path.as_path());
            let file =
                read_to_string(&path).expect(&format!("{} is not existed", path.as_display()));
            let file =
                syn::parse_file(&file).expect(&format!("{} parsed failed", path.as_display()));
            fold.fold_file(file);
            self.types.append(&mut fold.types);
            fold::fold_item_mod(self, i)
        } else {
            let mut fold = sub_module(self, sub_module_ident, self.file_path);
            let item_mod = fold::fold_item_mod(&mut fold, i);
            self.types.append(&mut fold.types);
            item_mod
        }
    }

    fn fold_item_struct(&mut self, i: ItemStruct) -> ItemStruct {
        for attr in i.attrs.iter() {
            if attr.path().is_ident("derive") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident(self.derive_trait) {
                        let mut find_type = self.module_path.clone();
                        find_type.segments.push(PathSegment::from(i.ident.clone()));
                        self.types.push(find_type);
                    }
                    Ok(())
                })
                .expect("");
            }
        }
        fold::fold_item_struct(self, i)
    }
}

impl Scanner {
    pub fn types_with_derive(&self, derive_trait: &str) -> Result<Vec<PathType>> {
        let file = read_to_string(&self.root)?;

        let file = syn::parse_file(&file)?;

        let derive_trait = Ident::new(derive_trait, file.span());

        let mut fold = ScanFold::new(&derive_trait, &self.root);

        fold.fold_file(file);

        Ok(fold.types)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use quote::quote;

    use super::*;

    #[test]
    fn it_works() {
        let scanner = Scanner::new("../examples/success/src/main.rs");

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
