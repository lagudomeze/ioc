use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemStruct, Path, PathSegment};

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
                        let mut find_type = module_info.module_path().clone();
                        find_type.segments.push(PathSegment::from(i.ident.clone()));
                        self.types.push(find_type);
                    }
                    Ok(())
                })?;
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