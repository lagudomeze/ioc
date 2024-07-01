use crate::{Bean, Result};

pub trait MethodType {
    type Ctx;
    fn run(ctx: Self::Ctx) -> Result<Self::Ctx>;
}

pub trait BeanFamily {
    type Ctx;

    type Method<B>: MethodType<Ctx=Self::Ctx>
    where
        B: Bean;
}

#[cfg(test)]
mod tests {
    use cfg_rs::Configuration;

    use crate::{Config, Context, Init, types::tests::bean::all_types_with};

    mod bean {
        use crate::{Result, types::tests::bean::a::A, types::tests::bean::b::B};
        use crate::types::BeanFamily;

        mod a {
            use std::sync::OnceLock;

            use crate::{Bean, BeanFactory, Context};

            pub struct A;

            impl BeanFactory for A {
                type Bean = A;

                fn build(_ctx: &mut Context) -> crate::Result<Self::Bean> {
                    Ok(A)
                }
            }

            impl Bean for A {
                fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                    static HOLDER: OnceLock<A> = OnceLock::new();
                    &HOLDER
                }
            }
        }
        mod b {
            use std::sync::OnceLock;

            use crate::{Bean, BeanFactory, Context};

            pub struct B;

            impl BeanFactory for B {
                type Bean = B;

                fn build(_ctx: &mut Context) -> crate::Result<Self::Bean> {
                    Ok(B)
                }
            }

            impl Bean for B {
                fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                    static HOLDER: OnceLock<B> = OnceLock::new();
                    &HOLDER
                }
            }
        }

        pub fn all_types_with<F: BeanFamily>(ctx: F::Ctx) -> Result<F::Ctx>
        {
            use crate::types::MethodType;
            let ctx = F::Method::<A>::run(ctx)?;
            let ctx = F::Method::<B>::run(ctx)?;
            Ok(ctx)
        }
    }


    #[test]
    fn test() {
        let source = Configuration::with_predefined_builder().init().unwrap();
        let config = Config { source };
        let mut ctx = Context::new(config);
        all_types_with::<Init>(&mut ctx).unwrap();
    }
}