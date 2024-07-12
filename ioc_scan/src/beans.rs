use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, ItemStruct, Path};

use crate::{
    scan::Module,
    Scanner,
    transport::Transport,
};

#[derive(Debug, Default)]
pub struct Beans {
    deps: Vec<Path>,
    types: Vec<Path>,
}

impl Beans {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn deps(self, deps: &[Path]) -> Self {
        Self {
            deps: deps.to_vec(),
            ..self
        }
    }
}

impl Scanner for Beans {
    fn item_struct(&mut self, module_info: &Module, i: &ItemStruct) -> crate::Result<()> {
        for attr in i.attrs.iter() {
            if attr.path().is_ident("derive") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("Bean") {
                        let find_type = module_info.build_path(&i.ident);
                        self.types.push(find_type);
                    }
                    Ok(())
                })?;
            }
        }
        Ok(())
    }

    fn item_impl(&mut self, module_info: &Module, i: &ItemImpl) -> crate::Result<()> {
        for attr in i.attrs.iter() {
            if attr.path().is_ident("bean") {
                let find_type = module_info.build_path(&i.self_ty);
                self.types.push(find_type);
            }
        }
        Ok(())
    }
}

impl Transport for Beans {
    fn export(self) -> crate::Result<TokenStream> {
        let types = &self.types;
        let deps = &self.deps;

        Ok(quote! {
            pub fn all_beans_with<F: ioc::BeanFamily>(ctx: F::Ctx) -> ioc::Result<F::Ctx> {
                use ioc::Method;
                // init all beans in self crate
                #(let ctx = F::Method::<crate::#types>::run(ctx)?; )*
                // init deps crate
                #(#deps::all_beans_with::<F>(ctx)?; )*
                Ok(ctx)
            }
        })
    }

    fn import(self, crates: &[Path]) -> crate::Result<TokenStream> {
        Ok(quote! {
            #(#crates::all_beans_with::<ioc::Init>(&mut ctx)?; )*
        })
    }
}

#[cfg(test)]
mod test {
    use quote::quote;
    use syn::{parse_quote, Path, Type};

    #[test]
    fn test() {
        let path: Path = parse_quote!(crate);

        let self_ty: Type = parse_quote!(BeanA);
        {
            let path = &path;
            let self_ty = &self_ty;

            let full_path_ty: Type = parse_quote!(#path :: #self_ty);
            println!("{}", quote!(#full_path_ty).to_string());
        }
    }
}