use ioc::{Bean, run};
use success::{A, B};

fn main() -> anyhow::Result<()> {
    let _ = run!(
        debug = true;
        profile = "dev";
        use_crate = false;
        crates(success);
    );
    println!("{:p}", A::get());
    println!("{:p}", B::get());
    println!("{:p}", B::get()._a);
    Ok(())
}
