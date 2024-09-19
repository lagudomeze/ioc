pub use quote::{quote, ToTokens};
pub use visit::{ItemImplExt, ItemStructExt, SynPath, Visit};

mod bootstraps;


pub trait CrateBuilder: Visit {
    fn into_token_stream(self) -> impl ToTokens;
}

pub trait BootBuilder {
    fn append_crate(&mut self, crate_name: &SynPath);

    fn into_token_stream(self) -> impl ToTokens;
}

pub trait CompileBootstrap {
    type CrateBuilder: CrateBuilder;
    type BootBuilder: BootBuilder;

    fn crate_build() -> Self::CrateBuilder;

    fn boot_build() -> Self::BootBuilder;
}

pub use bootstraps::{Bootstraps, Builders};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
