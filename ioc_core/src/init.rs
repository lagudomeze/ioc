use std::collections::{HashSet, VecDeque};

use cfg_rs::{FromConfig, FromConfigWithPrefix};
use log::debug;

use crate::{BeanId, BeanInfo, BeanSpec, Config, IocError, types::{BeanFamily, Method}};

pub struct Init<'a>(std::marker::PhantomData<&'a ()>);

impl<'a> BeanFamily for Init<'a> {
    type Ctx = &'a mut InitCtx;

    type Method<B> = Wrapper<B, Self::Ctx>
    where
        B: 'static + BeanSpec;
}

pub struct Wrapper<T, C>(T, std::marker::PhantomData<C>);

impl<'a, B> Method<&'a mut InitCtx> for Wrapper<B, &'a mut InitCtx>
where
    B: 'static + BeanSpec,
{
    fn run(ctx: &'a mut InitCtx) -> crate::Result<&'a mut InitCtx> {
        ctx.get_or_init::<B>()?;
        debug!("Init bean of {} with type {}", B::name(), B::bean_type_name());
        Ok(ctx)
    }
}

pub trait InitContext {
    fn get_config<T: FromConfig>(&self, key: impl AsRef<str>) -> crate::Result<T>;

    fn get_config_or<T: FromConfig>(&self, key: impl AsRef<str>, default: T) -> crate::Result<T>;

    fn get_predefined_config<T: FromConfigWithPrefix>(&self) -> crate::Result<T>;

    fn get_or_init<'a, B>(&mut self) -> crate::Result<&'a B::Bean>
    where
        B: 'static + BeanSpec;
}

/// The `Context` struct represents the IoC container's context, managing bean lifecycle, dependencies, and configuration.
/// It contains a list of ready beans, a stack of pending beans, and sets of identifiers for ready and pending beans.
#[derive(Debug)]
pub struct InitCtx {
    /// The configuration settings for the IoC container.
    pub(crate) config: Config,

    /// A list of beans that are ready to be injected into other beans.
    ready_beans: Vec<(BeanInfo, fn())>,

    /// A set of identifiers for beans that are ready to be injected into other beans.
    ready_bean_ids: HashSet<BeanId>,

    /// A stack of beans that are pending initialization.
    pending_chain: VecDeque<BeanInfo>,
}

impl InitContext for InitCtx {
    fn get_config<T: FromConfig>(&self, key: impl AsRef<str>) -> crate::Result<T> {
        Ok(self.config.source.get(key.as_ref())?)
    }

    fn get_config_or<T: FromConfig>(&self, key: impl AsRef<str>, default: T) -> crate::Result<T> {
        Ok(self.config.source.get_or(key.as_ref(), default)?)
    }

    fn get_predefined_config<T: FromConfigWithPrefix>(&self) -> crate::Result<T> {
        Ok(self.config.source.get_predefined()?)
    }

    fn get_or_init<'a, B>(&mut self) -> crate::Result<&'a B::Bean>
    where
        B: 'static + BeanSpec,
    {
        let info = B::bean_info();
        let id = B::bean_id();


        // Check if the bean is already initialized and return it if so.
        if self.ready_bean_ids.contains(&id) {
            return B::try_get();
        }

        // Use the cache to detect potential circular dependencies by checking if the bean
        // is currently in the process of being initialized.
        // Check if the bean is currently being initialized and return an error if so.
        for (i, pending_spec) in self.pending_chain.iter().enumerate() {
            if pending_spec.eq(&id) {
                return Err(IocError::CircularDependency(generate_cycle_visualization(&self.pending_chain, i)));
            }
        }
        self.pending_chain.push_back(info);
        debug!("bean {:?} is pending! ", info);

        // The holder's `get_or_try_init` method will attempt to build the bean if it's not already initialized.
        let result = B::holder()
            .get_or_try_init(|| B::build(self));

        let ready_bean = self.pending_chain
            .pop_back()
            .expect("Initialization stack is unexpectedly empty");

        if ready_bean != id {
            panic!("Initialization stack order corrupted");
        }

        if result.is_ok() {
            self.ready_beans.push((ready_bean, || B::drop(B::get())));
            self.ready_bean_ids.insert(id);
            debug!("bean {:?} is ready! ", ready_bean);
        }

        result
    }
}

fn generate_cycle_visualization(bean_infos: &VecDeque<BeanInfo>, beans_in_cycle: usize) -> String {
    let single_bean = bean_infos.len() == 1;
    let mut message = String::new();

    for (i, bean) in bean_infos.iter().enumerate() {
        if i == beans_in_cycle {
            if single_bean {
                message.push_str("┌──->──┐\n");
            } else {
                message.push_str("┌─────┐\n");
            }
            message.push_str(&format!("{}   {}\n", " ", bean));
        } else if i > 0 {
            let left_side = if i < beans_in_cycle { " " } else { "↑" };
            message.push_str(&format!("{}     ↓\n", left_side));
            message.push_str(&format!("{}   {}\n", " ", bean));
        } else {
            let left_side = if i < beans_in_cycle { " " } else { "|" };
            message.push_str(&format!("{}   {}\n", left_side, bean));
        }
    }

    if single_bean {
        message.push_str("└──<-──┘\n");
    } else {
        message.push_str("└─────┘\n");
    }

    message
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::sync::OnceLock;
    use crate::{BeanSpec, InitContext};
    use crate::init::generate_cycle_visualization;

    struct A;
    struct B;
    struct C;
    struct D;

    impl BeanSpec for A {
        type Bean = Self;

        fn build(_ctx: &mut impl InitContext) -> crate::Result<Self::Bean> {
            unimplemented!()
        }

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            unimplemented!()
        }
    }

    impl BeanSpec for B {
        type Bean = Self;

        fn build(_ctx: &mut impl InitContext) -> crate::Result<Self::Bean> {
            unimplemented!()
        }

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            unimplemented!()
        }
    }

    impl BeanSpec for C {
        type Bean = Self;

        fn build(_ctx: &mut impl InitContext) -> crate::Result<Self::Bean> {
            unimplemented!()
        }

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            unimplemented!()
        }
    }

    impl BeanSpec for D {
        type Bean = Self;

        fn build(_ctx: &mut impl InitContext) -> crate::Result<Self::Bean> {
            unimplemented!()
        }

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            unimplemented!()
        }
    }

    #[test]
    fn test() {
        // 示例用法
        let beans = VecDeque::from(vec![
            A::bean_info(),
            B::bean_info(),
            C::bean_info(),
            D::bean_info(),
        ]);

        let visualization = generate_cycle_visualization(&beans, 2);
        println!("{}", visualization);
    }
}

impl InitCtx {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ready_beans: Default::default(),
            ready_bean_ids: Default::default(),
            pending_chain: Default::default(),
        }
    }

    pub fn complete(self) -> DropGuard {
        DropGuard {
            ready_beans: self.ready_beans
        }
    }
}

// `DropGuard` is responsible for the cleanup logic of the IoC container.
pub struct DropGuard {
    ready_beans: Vec<(BeanInfo, fn())>,
}

impl Drop for DropGuard {
    /// Automatically performs the cleanup of all registered beans when the `DropGuard` instance is dropped.
    fn drop(&mut self) {
        debug!("Starting cleanup of beans.");
        // Iterate and clean up all beans to ensure resources are properly released.
        for bean_spec in self.ready_beans.iter().rev() {
            debug!("bean {:?} is cleaning", bean_spec);
            // Call the drop function for each bean to perform cleanup.
            (bean_spec.1)();
        }
        // Perform any other necessary cleanup here.
        debug!("Cleanup of beans completed.");
    }
}