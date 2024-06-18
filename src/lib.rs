use linkme::distributed_slice;

pub use ioc_core::{Bean, BeanFactory, Config, Context, IocError, Result};
pub use ioc_derive::{run, Bean};
pub use linkme;

#[distributed_slice]
pub static BEAN_COLLECTOR: [fn(&mut Context) -> Result<()>] = [..];

pub fn run_app(config: Config) -> Result<()> {
    env_logger::init();

    let mut ctx = Context::new(config);
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx)?;
    }
    let _ = ctx.complete();

    Ok(())
}
