use std::{
    path::PathBuf,
    time::Duration,
    collections::HashMap
};
use cfg_rs::*;
use poem::{EndpointExt, listener::TcpListener, middleware::Tracing, Route, Server};
use poem_openapi::{OpenApi, OpenApiService};
use tracing::info;

use ioc_core as ioc;
use ioc_core_derive::Bean;

#[derive(FromConfig, Debug)]
pub struct StaticFilesMapping {
    path: String,
    dir: PathBuf,
    #[config(default = false)]
    listing: bool,
}

#[derive(Bean)]
#[bean(ioc_crate = ioc)]
pub struct WebConfig {
    #[inject(config = "web.addr")]
    addr: String,
    #[inject(config = "web.graceful_shutdown_timeout")]
    shutdown_timeout: Duration,
    #[inject(config = "web.tracing")]
    tracing: bool,
    #[cfg(feature = "static-files")]
    #[inject(config(name = "web.static.enable", default = false))]
    static_enable: bool,
    #[cfg(feature = "static-files")]
    #[inject(config(name = "web.static.mapping", default = Default::default()))]
    static_mappings: HashMap<String, StaticFilesMapping>,
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

    let mut route = Route::new()
        .nest("/", api_service)
        .nest("/swagger-ui", ui)
        .nest("/swagger-ui/spec.yaml", spec);

    #[cfg(feature = "static-files")]
    if config.static_enable {

        use poem::endpoint::StaticFilesEndpoint;

        for (name, mapping)  in config.static_mappings.iter() {
            info!("add static {mapping:?} for {name}");
            let mut endpoint = StaticFilesEndpoint::new(&mapping.dir);
            if mapping.listing {
                endpoint = endpoint.show_files_listing()
                    .redirect_to_slash_directory();
            }
            route = route.nest(&mapping.path, endpoint);

        }
    }

    let app = route.with_if(config.tracing, Tracing::default());

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