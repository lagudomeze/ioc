use std::{
    env::current_dir,
    fmt::{
        self,
        Display,
        Formatter,
    },
    fs::read_to_string,
    mem::swap,
    path::{
        Path as FsPath,
        PathBuf,
    },
};

use quote::ToTokens;
use syn::{Ident, ItemImpl, ItemMod, ItemStruct, parse_quote, Path, PathSegment, visit::{
    Visit,
    visit_item_impl,
    visit_item_mod,
    visit_item_struct,
}};

use crate::{
    Error,
    Result,
};

#[derive(Debug)]
pub struct Module {
    root: PathBuf,
    file: PathBuf,
    module_path: Path,
}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let display = current_dir()
            .expect("fetch current dir failed!")
            .to_string_lossy()
            .to_string();
        f.debug_struct("Module")
            .field("root", &self.root)
            .field("file", &self.file)
            .field("module_path", &self.module_path)
            .field("current_dir", &display)
            .finish()
    }
}

impl Module {
    pub(crate) fn new(file: PathBuf) -> Result<Self> {
        Ok(Self {
            root: file
                .parent()
                .ok_or(Error::NoParent(file.to_string_lossy().to_string()))?
                .to_path_buf(),
            file: file.to_path_buf(),
            module_path: Path {
                leading_colon: None,
                segments: Default::default(),
            },
        })
    }

    pub(crate) fn sub_module(&self, segment: &Ident) -> Result<Self> {
        let file = {
            let mut buf = self.root.to_path_buf();
            for segment in self.module_path.segments.iter() {
                buf.push(format!("{}", segment.ident))
            }
            buf.push(segment.to_string());
            buf.push("mod.rs");
            if buf.exists() && buf.is_file() {
                buf
            } else {
                // pop mod.rs
                buf.pop();
                // set xxx.rs
                buf.set_extension("rs");
                if buf.exists() && buf.is_file() {
                    buf
                } else {
                    return Err(Error::FileNotFound(format!("{self}")));
                }
            }
        };

        let module_path = {
            let mut module_path = self.module_path.clone();
            module_path.segments.push(PathSegment::from(segment.clone()));
            module_path
        };


        Ok(Self {
            root: self.root.clone(),
            file,
            module_path,
        })
    }

    pub(crate) fn file(&self) -> &FsPath {
        &self.file
    }

    pub fn build_path(&self, ty: &impl ToTokens) -> Path {
        let module_path = &self.module_path;
        if module_path.segments.is_empty() {
            if module_path.leading_colon.is_none() {
                parse_quote!(#ty)
            } else {
                parse_quote!(::#ty)
            }
        } else {
            parse_quote!(#module_path :: #ty)
        }
    }
}

pub(crate) struct ScanVisit<T> {
    module: Module,
    scanner: T,
}

impl<T> ScanVisit<T> {
    pub(crate) fn new(module: Module, scanner: T) -> Self {
        Self {
            module,
            scanner,
        }
    }
}

pub trait Scanner {
    fn item_struct(&mut self, _module_info: &Module, _i: &ItemStruct) -> Result<()> {
        Ok(())
    }

    fn item_impl(&mut self, _module_info: &Module, _i: &ItemImpl) -> Result<()> {
        Ok(())
    }
}

impl<'ast, T: Scanner> Visit<'ast> for ScanVisit<T>
where
    T: Scanner,
{
    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        self.scanner
            .item_impl(&self.module, i)
            .expect("item_impl failed!");
        visit_item_impl(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if i.content.is_none() {
            let mut module = self
                .module
                .sub_module(&i.ident)
                .expect("sub module not found!");

            let string = read_to_string(&module.file()).expect("read file failed!");
            let file = syn::parse_file(&string).expect("parse file failed!");

            swap(&mut self.module, &mut module);
            self.visit_file(&file);
            swap(&mut self.module, &mut module);
        } else {
            let segment = PathSegment::from(i.ident.clone());

            self.module.module_path.segments.push(segment);
            visit_item_mod(self, &i);

            let pair = self
                .module
                .module_path
                .segments
                .pop()
                .expect("module path is empty! it should not happen!");

            assert_eq!(pair.value().ident, i.ident);
        }
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        self.scanner
            .item_struct(&self.module, i)
            .expect("item_struct failed!");
        visit_item_struct(self, i);
    }
}

impl<'ast, T: Scanner> ScanVisit<T> {
    pub(crate) fn scan(mut self) -> Result<T> {
        let string = read_to_string(&self.module.file)?;
        let file = syn::parse_file(&string)?;
        self.visit_file(&file);
        Ok(self.scanner)
    }
}