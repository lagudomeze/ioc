use std::{
    any::TypeId,
    collections::{HashSet, VecDeque},
    sync::OnceLock,
};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, IocError>;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum IocError {
    #[error("fetch config error: `{0}`")]
    ConfigError(String),
    #[error("required {type_name} is not init!")]
    DependNotReady {
        type_name: &'static str,
    },
    #[error("circular dependency")]
    CircularDependency
}

impl From<cfg_rs::ConfigError> for IocError {
    fn from(value: cfg_rs::ConfigError) -> Self {
        Self::ConfigError(
            format!("{value:?}")
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashSet, VecDeque};
    use std::sync::OnceLock;

    use cfg_rs::{Configuration, init_cargo_env};

    use crate::IocError;

    // pub struct A(String);
    //
    // pub struct B(&'static A, String);
    //
    // pub struct C(&'static A, &'static B, String);
    //
    // pub struct D(&'static C, &'static F, String);
    //
    // pub struct E(&'static D, String);
    //
    // pub struct F(&'static E, String);
    //
    // impl BeanHolder for A {
    //     type Bean = Self;
    //
    //     fn build(_ctx: &mut Context) -> crate::Result<Self::Bean> {
    //         Ok(A("this is A".to_string()))
    //     }
    //
    //     fn holder<'a>() -> &'a OnceLock<Self::Bean> {
    //         static HOLDER: OnceLock<A> = OnceLock::new();
    //         &HOLDER
    //     }
    // }
    //
    // impl BeanHolder for B {
    //     type Bean = Self;
    //
    //     fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
    //         let a = ctx.init::<A>()?;
    //         Ok(B(a, "this is B".to_string()))
    //     }
    //
    //     fn holder<'a>() -> &'a OnceLock<Self::Bean> {
    //         static HOLDER: OnceLock<B> = OnceLock::new();
    //         &HOLDER
    //     }
    // }
    //
    // impl BeanHolder for C {
    //     type Bean = Self;
    //
    //     fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
    //         let a = ctx.init::<A>()?;
    //         let b = ctx.init::<B>()?;
    //         Ok(C(a, b, "this is C".to_string()))
    //     }
    //
    //     fn holder<'a>() -> &'a OnceLock<Self::Bean> {
    //         static HOLDER: OnceLock<C> = OnceLock::new();
    //         &HOLDER
    //     }
    // }
    //
    // impl BeanHolder for D {
    //     type Bean = Self;
    //
    //     fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
    //         let c = ctx.init::<C>()?;
    //         let f = ctx.init::<F>()?;
    //         Ok(D(c, f, "this is D".to_string()))
    //     }
    //
    //     fn holder<'a>() -> &'a OnceLock<Self::Bean> {
    //         static HOLDER: OnceLock<D> = OnceLock::new();
    //         &HOLDER
    //     }
    // }
    //
    // impl BeanHolder for E {
    //     type Bean = Self;
    //
    //     fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
    //         let c = ctx.init::<D>()?;
    //         Ok(E(c, "this is E".to_string()))
    //     }
    //
    //     fn holder<'a>() -> &'a OnceLock<Self::Bean> {
    //         static HOLDER: OnceLock<E> = OnceLock::new();
    //         &HOLDER
    //     }
    // }
    //
    // impl BeanHolder for F {
    //     type Bean = Self;
    //
    //     fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
    //         let e = ctx.init::<E>()?;
    //         Ok(F(e, "this is E".to_string()))
    //     }
    //
    //     fn holder<'a>() -> &'a OnceLock<Self::Bean> {
    //         static HOLDER: OnceLock<F> = OnceLock::new();
    //         &HOLDER
    //     }
    // }
    //
    // #[test]
    // fn it_works() -> crate::Result<()> {
    //     init_cargo_env!();
    //     let config = Configuration::with_predefined_builder()
    //         .set("cfg_a.a", "data")
    //         .set("cfg_a.b", "babel")
    //         .init().unwrap();
    //
    //     let mut ctx = Context {
    //         ready_beans: HashSet::new(),
    //         pending_bean_stack: VecDeque::new(),
    //         pending_bean_ids: HashSet::new(),
    //
    //     };
    //
    //     let a = ctx.init::<A>()?;
    //     let b = ctx.init::<B>()?;
    //     let c = ctx.init::<C>()?;
    //     assert_eq!(a as *const A, b.0 as *const A);
    //     assert_eq!(a as *const A, c.0 as *const A);
    //     assert_eq!(b as *const B, c.1 as *const B);
    //
    //     assert_eq!(Some(IocError::CircularDependency), ctx.init::<E>().err());
    //     assert_eq!(Some(IocError::CircularDependency), ctx.init::<F>().err());
    //     assert_eq!(Some(IocError::CircularDependency), ctx.init::<D>().err());
    //
    //     Ok(())
    // }
}