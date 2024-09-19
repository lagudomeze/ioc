use crate::{BootBuilder, CompileBootstrap, CrateBuilder};
use quote::{quote, ToTokens};
use std::marker::PhantomData;
use visit::{ItemImplExt, ItemStructExt, SynPath, Visit};

pub struct Builders<T, U> {
    lft: T,
    rht: U,
}

impl<T, U> Visit for Builders<T, U>
where
    T: Visit,
    U: Visit,
{
    fn item_struct(&mut self, i: &ItemStructExt<'_>) {
        self.lft.item_struct(i);
        self.rht.item_struct(i);
    }

    fn item_impl(&mut self, i: &ItemImplExt<'_>) {
        self.lft.item_impl(i);
        self.rht.item_impl(i);
    }
}

impl<T, U> CrateBuilder for Builders<T, U>
where
    T: CrateBuilder,
    U: CrateBuilder,
{
    fn into_token_stream(self) -> impl ToTokens {
        let lft = self.lft.into_token_stream();
        let rht = self.rht.into_token_stream();
        quote! {
            #lft
            #rht
        }
    }
}

impl<T, U> BootBuilder for Builders<T, U>
where
    T: BootBuilder,
    U: BootBuilder,
{
    fn append_crate(&mut self, crate_name: &SynPath) {
        self.lft.append_crate(crate_name);
        self.rht.append_crate(crate_name);
    }

    fn into_token_stream(self) -> impl ToTokens {
        let lft = self.lft.into_token_stream();
        let rht = self.rht.into_token_stream();
        quote! {
            #lft
            #rht
        }
    }
}

pub struct Bootstraps<T, U> {
    lft: PhantomData<T>,
    rht: PhantomData<U>,
}

impl<T, U> CompileBootstrap for Bootstraps<T, U>
where
    T: CompileBootstrap,
    U: CompileBootstrap,
{
    type CrateBuilder = Builders<T::CrateBuilder, U::CrateBuilder>;
    type BootBuilder = Builders<T::BootBuilder, U::BootBuilder>;

    fn crate_build() -> Self::CrateBuilder {
        Self::CrateBuilder {
            lft: T::crate_build(),
            rht: U::crate_build(),
        }
    }

    fn boot_build() -> Self::BootBuilder {
        Self::BootBuilder {
            lft: T::boot_build(),
            rht: U::boot_build(),
        }
    }
}

impl<U> CompileBootstrap for Bootstraps<(), U>
where
    U: CompileBootstrap,
{
    type CrateBuilder = U::CrateBuilder;
    type BootBuilder = U::BootBuilder;

    fn crate_build() -> Self::CrateBuilder {
        U::crate_build()
    }

    fn boot_build() -> Self::BootBuilder {
        U::boot_build()
    }
}

impl<T, U> Bootstraps<T, U> {

    pub fn new() -> Bootstraps<(), ()> {
        Bootstraps {
            lft: PhantomData::<()>,
            rht: PhantomData::<()>,
        }
    }

    pub fn append<V>(self) -> Bootstraps<Self, V> {
        Bootstraps {
            lft: PhantomData::<Self>,
            rht: PhantomData::<V>,
        }
    }
}