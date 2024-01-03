use proc_macro::TokenStream;
use quote::{format_ident, quote};

mod scan;

fn preload_mods() -> proc_macro2::TokenStream {
    use scan::CargoToml;
    let toml = CargoToml::current();

    let mut mod_names = vec![];
    let mut mods = vec![];

    for name in toml.mod_names() {
        // mod name may contains "-", in `use` statment nead replace it to
        let mod_name = name.replace("-", "_");
        mods.push(format_ident!("{}", mod_name));
        mod_names.push(mod_name);
    }

    let test = format!("{mod_names:?}");

    quote! {
        println!("preload mods: {}", #test);
        #( use #mods; )*
    }
}

#[proc_macro]
pub fn run(_: TokenStream) -> TokenStream {
    let preload_mods = preload_mods();

    let expanded = quote! {

        #preload_mods

        ::ioc::run_app();
    };

    TokenStream::from(expanded)
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}