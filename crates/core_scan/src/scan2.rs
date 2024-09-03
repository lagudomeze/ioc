use crate::{export::Exporter, Error, Result};
use quote::ToTokens;
use std::{
    env::current_dir,
    fmt::{self, Display, Formatter},
    fs::read_to_string,
    path::{Path as FsPath, PathBuf},
};
use syn::{
    parse_quote,
    visit::{visit_item_impl, visit_item_mod, visit_item_struct, Visit},
    File as SynFile, ItemImpl, ItemMod, ItemStruct, Meta, Path as SynPath,
};

pub(crate) struct Scanner<'a, C> {
    pub(crate) root: &'a FsPath,
    pub(crate) src_path: &'a FsPath,
    pub(crate) mod_path: &'a SynPath,
    pub(crate) collector: &'a mut C,
}

impl<C> Display for Scanner<'_, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let display = current_dir()
            .expect("fetch current dir failed!")
            .to_string_lossy()
            .to_string();
        f.debug_struct("Module")
            .field("root", &self.root)
            .field("src_path", &self.src_path)
            .field("mod_path", &self.mod_path)
            .field("current_dir", &display)
            .finish()
    }
}

impl<C> Scanner<'_, C> {
    fn sub_src_file(&self, segment: impl ToString) -> Result<(PathBuf, SynFile)> {
        let mut path = self.root.to_path_buf();
        for segment in self.mod_path.segments.iter() {
            path.push(format!("{}", segment.ident));
        }
        path.push(segment.to_string());
        path.push("mod.rs");
        if !path.exists() || path.is_file() {
            // pop mod.rs
            path.pop();
            // set xxx.rs
            path.set_extension("rs");
            if !path.exists() || !path.is_file() {
                return Err(Error::FileNotFound(format!("{self}")));
            }
        }

        let string = read_to_string(&path)?;

        Ok((path, syn::parse_file(&string)?))
    }

    fn sub_mod_path(&self, segment: impl ToTokens) -> SynPath {
        let parent = self.mod_path;
        parse_quote!(#parent::#segment)
    }
}

fn is_test_mod(i: &ItemMod) -> bool {
    for attr in &i.attrs {
        let cfg_test: Meta = parse_quote!(cfg(test));
        let meta = &attr.meta;
        if meta.eq(&cfg_test) {
            return true;
        }
    }
    false
}

pub struct ItemExt<'a, T> {
    mod_path: &'a SynPath,
    data: &'a T,
}

impl ItemExt<'_, ItemStruct> {
    pub fn build_path(&self, segment: &impl ToTokens) -> SynPath {
        let parent = self.mod_path;
        parse_quote!(#parent::#segment)
    }
}

pub type ItemStructExt<'a> = ItemExt<'a, ItemStruct>;

pub type ItemImplExt<'a> = ItemExt<'a, ItemImpl>;

impl<'ast, C> Visit<'ast> for Scanner<'_, C>
where
    C: Exporter,
{
    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        let item_impl = ItemExt {
            mod_path: self.mod_path,
            data: i,
        };
        self.collector
            .item_impl(&item_impl)
            .expect("item_impl failed!");
        visit_item_impl(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if is_test_mod(i) {
            eprintln!("skip test mod: {}", i.ident);
        } else {
            if i.content.is_none() {
                let (src_path, syn_file) = self
                    .sub_src_file(&i.ident)
                    .expect("sub mod file not found!");

                let mod_path = self.sub_mod_path(&i.ident);

                eprintln!("mod file: {:?}", src_path);

                let mut scanner = Scanner {
                    root: self.root,
                    src_path: &src_path,
                    mod_path: &mod_path,
                    collector: self.collector,
                };

                scanner.visit_file(&syn_file);
            } else {
                let mod_path = self.sub_mod_path(&i.ident);

                let mut scanner = Scanner {
                    root: self.root,
                    src_path: &self.src_path,
                    mod_path: &mod_path,
                    collector: self.collector,
                };

                visit_item_mod(&mut scanner, i);
            }
        }
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        let struct_ext = ItemExt {
            mod_path: self.mod_path,
            data: i,
        };
        self.collector
            .item_struct(&struct_ext)
            .expect("item_struct failed!");
        visit_item_struct(self, i);
    }
}
