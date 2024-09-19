use quote::ToTokens;
use std::ops::Deref;
use syn::{
    parse_quote,
    Path as SynPath,
};

pub struct ItemExt<'a, T> {
    mod_path: &'a SynPath,
    data: &'a T,
}

impl<'a,T> ItemExt<'a, T> {
    pub(crate) fn new(mod_path: &'a SynPath, data: &'a T) -> Self {
        Self { mod_path, data }
    }
}

impl<T> AsRef<T> for ItemExt<'_, T> {
    fn as_ref(&self) -> &T {
        self.data
    }
}

impl<T> Deref for ItemExt<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> ItemExt<'_, T> {
    pub fn build_path(&self, segment: &impl ToTokens) -> SynPath {
        let parent = self.mod_path;
        parse_quote!(#parent::#segment)
    }
}