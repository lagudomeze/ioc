#[cfg(all(feature = "env_logger", feature = "tracing_log"))]
compile_error!("feature \"env_logger\" and feature \"tracing_log\" cannot be enabled at the same time");

#[cfg(feature = "env_logger")]
pub(crate) fn log_init() -> crate::Result<()> {
    env_logger::init();
    Ok(())
}

#[cfg(feature = "tracing_log")]
pub(crate) fn log_init() -> crate::Result<()> {
    tracing_subscriber::fmt::init();
    Ok(())
}

#[cfg(not(any(feature = "env_logger", feature = "tracing_log")))]
pub(crate) fn log_init() -> crate::Result<()> {
    //do nothing
    Ok(())
}