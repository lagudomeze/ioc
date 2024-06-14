use linkme::distributed_slice;

use ioc_core::{Config, Context, Result};
pub use ioc_core::Bean;
pub use ioc_derive::{Bean, run};

#[distributed_slice]
pub static BEAN_COLLECTOR: [fn(&mut Context) -> Result<()>] = [..];

pub fn run_app(config: Config) -> Result<()> {
    env_logger::init();

    let mut ctx = Context::new(config);
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx)?;
    }

    Ok(())
}

