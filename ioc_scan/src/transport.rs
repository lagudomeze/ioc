use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, ItemStruct, Path};

use crate::{error::Result, ModuleInfo, scan::Scanner};

pub trait Transport: Scanner {
    fn export(self) -> Result<TokenStream>;

    fn import(self, crates: &[Path]) -> Result<TokenStream>;

    fn join<U>(self, rht: U) -> Transports<Self, U>
    where
        Self: Sized,
    {
        Transports {
            lft: self,
            rht,
        }
    }
}

pub struct Transports<T, U> {
    pub(crate) lft: T,
    pub(crate) rht: U,
}

impl<T, U> Scanner for Transports<T, U>
where
    T: Transport,
    U: Transport,
{
    fn item_struct(&mut self, module_info: &ModuleInfo, i: &ItemStruct) -> Result<()> {
        self.lft.item_struct(module_info, i)?;
        self.rht.item_struct(module_info, i)
    }

    fn item_impl(&mut self, module_info: &ModuleInfo, i: &ItemImpl) -> Result<()> {
        self.lft.item_impl(module_info, i)?;
        self.rht.item_impl(module_info, i)
    }
}

impl<T, U> Transport for Transports<T, U>
where
    T: Transport,
    U: Transport,
{
    fn export(self) -> Result<TokenStream> {
        let lft = self.lft.export()?;
        let rht = self.rht.export()?;
        Ok(quote! {
            #lft
            #rht
        })
    }

    fn import(self, crates: &[Path]) -> Result<TokenStream> {
        let lft = self.lft.import(crates)?;
        let rht = self.rht.import(crates)?;
        Ok(quote! {
            #lft
            #rht
        })
    }
}