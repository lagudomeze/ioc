use std::time::Duration;

use poem::{EndpointExt, listener::TcpListener, middleware::Tracing, Route, Server};
use poem_openapi::{OpenApi, OpenApiService};
use tracing::info;

use ioc_core as ioc;
use ioc_core_derive::Bean;

#[derive(Bean)]
pub struct WebConfig {
    #[value("web.addr")]
    addr: String,
    #[value("web.graceful_shutdown_timeout")]
    shutdown_timeout: Duration,
    #[value("web.tracing")]
    tracing: bool,
}

async fn run_server<T>(api: T, title: &str, version: &str) -> ioc_core::Result<()>
where
    T: 'static + OpenApi,
{
    use ioc_core::Bean;
    let config = WebConfig::try_get()?;

    let api_service = OpenApiService::new(api, title, version);

    let ui = api_service.swagger_ui();

    let app = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
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