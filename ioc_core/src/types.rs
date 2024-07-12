use crate::{BeanSpec, Result};

pub trait Method<T> {
    fn run(ctx: T) -> Result<T>;
}

pub trait BeanFamily {
    type Ctx;

    type Method<B>: Method<Self::Ctx>
    where
        B: 'static + BeanSpec;
}

#[cfg(test)]
mod tests {
    use cfg_rs::Configuration;

    use crate::{Config, Init, InitCtx, types::tests::bean::all_types_with};

    mod bean {
        use crate::{Result, types::tests::bean::a::A, types::tests::bean::b::B};
        use crate::types::BeanFamily;

        mod a {
            use std::sync::OnceLock;

            use crate::{BeanSpec, InitContext};

            pub struct A;

            impl BeanSpec for A {
                type Bean = Self;

                fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                    static HOLDER: OnceLock<A> = OnceLock::new();
                    &HOLDER
                }

                fn build<I>(_: &mut I) -> crate::Result<Self::Bean>
                where
                    I: InitContext,
                {
                    Ok(A)
                }
            }
        }
        mod b {
            use std::sync::OnceLock;

            use crate::{BeanSpec, InitContext};

            pub struct B;

            impl BeanSpec for B {
                type Bean = Self;

                fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                    static HOLDER: OnceLock<B> = OnceLock::new();
                    &HOLDER
                }

                fn build<I>(_: &mut I) -> crate::Result<Self::Bean>
                where
                    I: InitContext,
                {
                    Ok(B)
                }
            }
        }

        pub fn all_types_with<F: BeanFamily>(ctx: F::Ctx) -> Result<F::Ctx>
        {
            use crate::types::Method;
            let ctx = F::Method::<A>::run(ctx)?;
            let ctx = F::Method::<B>::run(ctx)?;
            Ok(ctx)
        }
    }


    #[test]
    fn test() {
        let source = Configuration::with_predefined_builder().init().unwrap();
        let config = Config { source };
        let mut ctx = InitCtx::new(config);
        all_types_with::<Init>(&mut ctx).unwrap();
    }
}