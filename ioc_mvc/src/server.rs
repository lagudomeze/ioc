use std::time::Duration;

use poem::{EndpointExt, listener::TcpListener, middleware::Tracing, Route, Server};
use poem_openapi::{OpenApi, OpenApiService};
use tracing::info;

use ioc_core as ioc;
use ioc_core_derive::Bean;

#[derive(Bean)]
#[bean(ioc_crate = ioc)]
pub struct WebConfig {
    #[inject(config = "web.addr")]
    addr: String,
    #[inject(config = "web.graceful_shutdown_timeout")]
    shutdown_timeout: Duration,
    #[inject(config = "web.tracing")]
    tracing: bool,
    #[inject(config(name = "web.static.dir", default = "."))]
    static_dir: String,
    #[inject(config(name = "web.static.path", default = "static"))]
    static_path: String,
}

async fn run_server<T>(api: T, title: &str, version: &str) -> ioc_core::Result<()>
where
    T: 'static + OpenApi,
{
    use ioc_core::BeanSpec;
    let config = WebConfig::try_get()?;

    let api_service = OpenApiService::new(api, title, version);

    let ui = api_service.swagger_ui();

    let spec = api_service.spec_endpoint_yaml();

    let app = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .nest("/ui/spec/yaml", spec)
        .with_if(config.tracing, Tracing::default());

    let listener = TcpListener::bind(config.addr.as_str());

    Server::new(listener)
        .run_with_graceful_shutdown(
            app,
            gracefully_shutdown(),
            Some(config.shutdown_timeout),
        ).await?;


    Ok(())
}

async fn gracefully_shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}

pub fn run_mvc<T>(api: T, title: &str, version: &str) -> ioc_core::Result<()>
where
    T: 'static + OpenApi,
{
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let metrics = runtime.metrics();
    info!("workers: {}", metrics.num_workers());
    runtime.block_on(async {
        run_server(api, title, version).await
    })?;
    Ok(())
}