use std::path::PathBuf;

use proc_macro2::TokenStream;
use syn::Path;

pub use crate::{
    error::{Error, Result},
    scan::{ModuleInfo, Scanner},
    transport::Transport,
    beans::Beans,
};
use crate::scan::ScanVisit;

mod error;
mod scan;
mod transport;
mod beans;

pub fn export<T>(transport: T, file: &str) -> Result<TokenStream>
where
    T: Transport,
{
    let module_path = Path {
        leading_colon: None,
        segments: Default::default(),
    };
    let file = PathBuf::from(file);
    let module_info = ModuleInfo::new(module_path, file);
    let visit = ScanVisit::new(module_info, transport);
    visit.scan()?.export()
}

pub fn import<T>(transport: T, crates: &[String]) -> Result<TokenStream>
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
        let code = export(Beans::default(), "../examples/success/src/main.rs")?;

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
