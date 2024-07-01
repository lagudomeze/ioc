use std::{
    fs::read_to_string,
    path::PathBuf,
};
use std::mem::swap;

use syn::{Ident, ItemImpl, ItemMod, ItemStruct, Path, PathSegment, visit::{
    Visit,
    visit_item_mod,
}};
use syn::__private::Span;
use syn::visit::{visit_item_impl, visit_item_struct};
use thiserror::__private::AsDisplay;

use crate::{Error, Result};

fn sub_module_file(parent: &std::path::Path, sub_module: &Ident) -> PathBuf {
    let mod_dir_path = parent.join(format!("{}/mod.rs", sub_module));
    let mod_file_path = parent.join(format!("{}.rs", sub_module));
    if mod_dir_path.exists() && mod_dir_path.is_file() {
        mod_dir_path
    } else if mod_file_path.exists() && mod_file_path.is_file() {
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

pub struct ModuleInfo {
    pub module_path: Path,
    pub file: PathBuf,
}

impl ModuleInfo {
    fn new(module_path: Path, file: PathBuf) -> Self {
        Self {
            module_path,
            file,
        }
    }

    fn sub(&self, sub_module_name: &Ident) -> Result<Self> {
        let parent = self
            .file
            .parent()
            .ok_or(Error::NoParent(self.file.to_string_lossy().to_string()))?;

        let file = sub_module_file(parent, sub_module_name);
        let module_path = {
            let mut path = self.module_path.clone();
            path.segments.push(PathSegment::from(sub_module_name.clone()));
            path
        };
        Ok(Self::new(module_path, file))
    }
}

pub struct ScanVisit<T> {
    module_info: ModuleInfo,
    scanner: T,
}

pub trait Scanner {
    fn item_struct(&mut self, _module_info: &ModuleInfo, _i: &ItemStruct) -> Result<()> {
        Ok(())
    }

    fn item_impl(&mut self, _module_info: &ModuleInfo, _i: &ItemImpl) -> Result<()> {
        Ok(())
    }

    fn scan_main(self) -> Result<Self>
    where
        Self: Sized,
    {
        main_visit(self).scan()
    }

    fn scan_lib_local(self) -> Result<Self>
    where
        Self: Sized,
    {
        lib_visit_local(self).scan()
    }

    fn scan_lib(self, name: &str) -> Result<Self>
    where
        Self: Sized,
    {
        lib_visit(name, self).scan()
    }
}

impl<'ast, T: Scanner> Visit<'ast> for ScanVisit<T>
where
    T: Scanner,
{
    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        self.scanner
            .item_impl(&self.module_info, i)
            .expect("item_impl failed!");
        visit_item_impl(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if i.content.is_none() {
            let mut module_info = self.module_info
                .sub(&i.ident)
                .expect("sub module not found!");

            let string = read_to_string(&module_info.file)
                .expect("read file failed!");
            let file = syn::parse_file(&string)
                .expect("parse file failed!");

            swap(&mut self.module_info, &mut module_info);
            self.visit_file(&file);
            swap(&mut self.module_info, &mut module_info);
        } else {
            let segment = PathSegment::from(i.ident.clone());

            self.module_info.module_path.segments.push(segment);
            visit_item_mod(self, &i);

            let pair = self.module_info
                .module_path
                .segments.pop()
                .expect("module path is empty! it should not happen!");

            assert_eq!(pair.value().ident, i.ident);
        }
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        self.scanner
            .item_struct(&self.module_info, i)
            .expect("item_struct failed!");
        visit_item_struct(self, i);
    }
}

impl<'ast, T: Scanner> ScanVisit<T> {
    fn scan(mut self) -> Result<T> {
        let string = read_to_string(&self.module_info.file)?;
        let file = syn::parse_file(&string)?;
        self.visit_file(&file);
        Ok(self.scanner)
    }
}

fn crate_path() -> Path {
    crate_path_with_name("crate")
}

fn crate_path_with_name(name: &str) -> Path {
    Path::from(Ident::new(name, Span::call_site()))
}

pub fn main_visit<T>(scanner: T) -> ScanVisit<T> {
    ScanVisit {
        module_info: ModuleInfo::new(crate_path(), PathBuf::from("src/main.rs")),
        scanner,
    }
}

pub fn lib_visit_local<T>(scanner: T) -> ScanVisit<T> {
    lib_visit("crate", scanner)
}

pub fn lib_visit<T>(name: &str, scanner: T) -> ScanVisit<T> {
    ScanVisit {
        module_info: ModuleInfo::new(crate_path_with_name(name), PathBuf::from("src/lib.rs")),
        scanner,
    }
}