use crate::{
    scan2::{
        Scanner,
        ItemImplExt,
        ItemStructExt
    },
    Error,
    Result
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::path::PathBuf;
use syn::{
    visit::Visit,
    Path
};

pub trait Exporter {
    type R: ToTokens;

    fn item_struct(&mut self, _i: &ItemStructExt<'_>) -> Result<()> {
        Ok(())
    }

    fn item_impl(&mut self, _i: &ItemImplExt<'_>) -> Result<()> {
        Ok(())
    }

    fn finish(self) -> Result<Self::R>;
}

pub struct Exports<T, U>
where
    T: Exporter,
    U: Exporter,
{
    pub(crate) lft: T,
    pub(crate) rht: U,
}

pub struct EmptyExporter;

pub struct EmptyResult;

impl ToTokens for EmptyResult {
    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {}
}

impl Exporter for EmptyExporter {
    type R = EmptyResult;

    fn finish(self) -> Result<Self::R> {
        Ok(EmptyResult)
    }
}

impl<T> Exports<EmptyExporter, T>
where
    T: Exporter,
{
    pub fn new(t: T) -> Self {
        Exports {
            lft: EmptyExporter,
            rht: t,
        }
    }
}

impl<T, U> Exports<T, U>
where
    T: Exporter,
    U: Exporter,
{
    pub fn join<V>(self, v: V) -> Exports<V, Exports<T, U>>
    where
        V: Exporter,
    {
        Exports { lft: v, rht: self }
    }
}

pub struct CollectResult<R1, R2>(R1, R2);

impl<R1, R2> ToTokens for CollectResult<R1, R2>
where
    R1: ToTokens,
    R2: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
        self.1.to_tokens(tokens);
    }
}

impl<T, U> Exporter for Exports<T, U>
where
    T: Exporter,
    U: Exporter,
{
    type R = CollectResult<T::R, U::R>;

    fn item_struct(&mut self, i: &ItemStructExt<'_>) -> Result<()> {
        self.lft.item_struct(i)?;
        self.rht.item_struct(i)
    }

    fn item_impl(&mut self, i: &ItemImplExt<'_>) -> Result<()> {
        self.lft.item_impl(i)?;
        self.rht.item_impl(i)
    }

    fn finish(self) -> Result<Self::R> {
        let r1 = self.lft.finish()?;
        let r2 = self.rht.finish()?;
        Ok(CollectResult(r1, r2))
    }
}

impl<T, U> Exports<T, U>
where
    T: Exporter,
    U: Exporter,
{
    pub fn export(mut self, file: PathBuf) -> Result<TokenStream> {
        let root = file
            .parent()
            .ok_or(Error::NoParent(file.to_string_lossy().to_string()))?
            .to_path_buf();

        let mod_path = Path {
            leading_colon: None,
            segments: Default::default(),
        };

        let mut scanner = Scanner {
            root: &root,
            src_path: &file,
            mod_path: &mod_path,
            collector: &mut self,
        };
        let result = std::fs::read_to_string(&file)?;
        let file = syn::parse_file(&result)?;
        scanner.visit_file(&file);

        let tt = self.finish()?.to_token_stream();
        Ok(tt)
    }
}
