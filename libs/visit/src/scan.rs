use crate::{
    ext::ItemExt,
    Visit,
};
use quote::ToTokens;
use std::{
    env::current_dir,
    fmt::{
        self,
        Display,
        Formatter,
    },
    fs::read_to_string,
    path::{
        Path as FsPath,
        PathBuf,
    },
};
use syn::{
    parse_quote,
    visit::{
        visit_item_impl,
        visit_item_mod,
        visit_item_struct,
        Visit as SynVisit,
    },
    File as SynFile,
    ItemImpl,
    ItemMod,
    ItemStruct,
    Meta,
    Path as SynPath,
    Visibility,
};

pub(crate) struct Scanner<'a, V> {
    root: &'a FsPath,
    src_path: &'a FsPath,
    mod_path: &'a SynPath,
    visit: &'a mut V,
}

impl<V> Display for Scanner<'_, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let display = current_dir()
            .expect("fetch current dir failed!")
            .to_string_lossy()
            .to_string();
        f.debug_struct("Module")
            .field("root", &self.root)
            .field("src_path", &self.src_path)
            .field("current_dir", &display)
            .field("mod_path", &self.mod_path)
            .finish()
    }
}

static ROOT_MOD_PATH: SynPath = SynPath {
    leading_colon: None,
    segments: Default::default(),
};

impl<V> Scanner<'_, V> {
    fn sub_src_file(&self, segment: impl ToString) -> (PathBuf, SynFile) {
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
                panic!("{path:?} does not exist!", );
            }
        }

        let string = read_to_string(&path)
            .expect("failed to read src file!");

        let file = syn::parse_file(&string)
            .expect("failed to parse src file!");

        (path, file)
    }

    fn sub_mod_path(&self, segment: impl ToTokens) -> SynPath {
        let parent = self.mod_path;
        parse_quote!(#parent::#segment)
    }

    pub(crate) fn root<'a>(src_path: &'a FsPath) -> Scanner<'a, ()> {
        Self::new(src_path, &ROOT_MOD_PATH)
    }

    pub(crate) fn new<'a>(src_path: &'a FsPath, mod_path: &'a SynPath) -> Scanner<'a, ()> {
        let root = src_path
            .parent()
            .expect("file has no parent");

        Self {
            root,
            src_path,
            mod_path,
            visit: &mut (),
        }
    }

    pub(crate) fn with<'a, T>(self, visit: &'a mut T) -> Scanner<'a, T> {
        Self {
            root: self.root,
            src_path: self.src_path,
            mod_path: self.mod_path,
            visit,
        }
    }
}

fn is_test_mod(i: &ItemMod) -> bool {
    for attr in i.attrs.iter() {
        let cfg_test: Meta = parse_quote!(cfg(test));
        let meta = &attr.meta;
        if meta.eq(&cfg_test) {
            return true;
        }
    }
    false
}

impl<'ast, C> SynVisit<'ast> for Scanner<'_, C>
where
    C: Visit,
{
    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        let item_impl = ItemExt::new(self.mod_path, i);

        self.visit
            .item_impl(&item_impl);
        visit_item_impl(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if is_test_mod(i) {
            eprintln!("skip test mod: {}", i.ident);
        } else {
            if i.content.is_none() {
                match i.vis {
                    Visibility::Public(_) => {}
                    Visibility::Restricted(_) => {}
                    Visibility::Inherited => {}
                }

                let (src_path, syn_file) = self
                    .sub_src_file(&i.ident);

                let mod_path = self.sub_mod_path(&i.ident);

                eprintln!("mod file: {:?}", src_path);

                let mut scanner = Scanner {
                    root: self.root,
                    src_path: &src_path,
                    mod_path: &mod_path,
                    visit: self.visit,
                };

                scanner.visit_file(&syn_file);
            } else {
                let mod_path = self.sub_mod_path(&i.ident);

                let mut scanner = Scanner {
                    root: self.root,
                    src_path: &self.src_path,
                    mod_path: &mod_path,
                    visit: self.visit,
                };

                visit_item_mod(&mut scanner, i);
            }
        }
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        let item_struct = ItemExt::new(self.mod_path, i);
        self.visit
            .item_struct(&item_struct);
        visit_item_struct(self, i);
    }
}
