use crate::bean::Bean;
use crate::Context;

trait Init {
    fn init(ctx: &mut Context);
}

trait InitFamily {
    type Method<B>: Init

    where
        B: Bean;
}


#[cfg(test)]
mod tests {
    use std::any::type_name;

    use cfg_rs::Configuration;

    use crate::{Bean, Config, Context};
    use crate::types::{Init, InitFamily};
    use crate::types::tests::bean::init;

    mod bean {
        use crate::Context;
        use crate::types::InitFamily;
        use crate::types::tests::bean::a::A;
        use crate::types::tests::bean::b::B;

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

        pub fn init<F: InitFamily>(ctx: &mut Context) {
            use crate::types::Init;
            F::Method::<A>::init(ctx);
            F::Method::<B>::init(ctx);
        }
    }

    struct Wrapper<T>(T);

    struct XxxxInit;

    impl InitFamily for XxxxInit {
        type Method<B> = Wrapper<B>
        where
            B: Bean;
    }

    impl<B> Init for Wrapper<B>
    where
        B: Bean,
    {
        fn init(ctx: &mut Context) {
            let x = B::holder().get_or_init(|| B::build(ctx).unwrap());
            println!("Init bean {:p} of {}", x, type_name::<B>());
        }
    }

    #[test]
    fn test() {
        init::<XxxxInit>(&mut Context::new(Config {
            source: Configuration::with_predefined_builder().init().unwrap()
        }));
    }
}