use std::path::PathBuf;

use proc_macro2::TokenStream;
use syn::Path;

pub use crate::{
    beans::Beans,
    error::{Error, Result},
    scan::{Module, Scanner},
    transport::Transport,
};
use crate::scan::ScanVisit;

mod error;
mod scan;
mod transport;
mod beans;

pub fn export<T>(transport: T, file: PathBuf) -> Result<TokenStream>
where
    T: Transport,
{
    let module = Module::new(file)?;
    let visit = ScanVisit::new(module, transport);
    visit.scan()?.export()
}

pub fn import<T>(transport: T, crates: &[Path]) -> Result<TokenStream>
where
    T: Transport,
{
    transport.import(crates)
}


#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::beans::Beans;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let path = PathBuf::from("../examples/success/src/lib.rs");
        let code = export(Beans::default(), path)?;

        let func = parse_quote!( #code );

        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![func],
        };

        println!("{}", prettyplease::unparse(&file));

        Ok(())
    }
}
