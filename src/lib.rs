use cfg_rs::{Configuration, init_cargo_env};
use ioc_core::{Result, Context};
use linkme::distributed_slice;


#[distributed_slice]
pub static BEAN_COLLECTOR: [fn(&mut Context) -> Result<()>] = [..];

pub fn run_app() -> Result<()> {
    env_logger::init();

    init_cargo_env!();

    let config = Configuration::with_predefined_builder()
        .init()?
        .into();

    let mut ctx = Context::new(config);
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx)?;
    }

    Ok(())
}

pub use ioc_core::Bean;

pub use ioc_derive::{run, Bean};
