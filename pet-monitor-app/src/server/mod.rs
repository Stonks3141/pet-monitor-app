//! This module contains all server-related logic.

use crate::config::Context;
use fmp4::stream_media_segments;
use provider::Provider;
use rocket::config::LogLevel;
use rocket::config::TlsConfig;
use rocket::futures::future;
use rocket::{Build, Rocket};
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

mod auth;
mod fmp4;
mod provider;
mod routes;
use routes::*;

/// Launches the server
pub async fn launch(
    conf_path: Option<PathBuf>,
    ctx: Context,
    log_level: LogLevel,
) -> anyhow::Result<()> {
    let ctx_prov = Provider::new(ctx.clone());
    let mut sub = ctx_prov.subscribe();
    rocket::tokio::spawn(async move {
        while let Ok(ctx) = sub.recv().await {
            crate::config::store(&conf_path, &ctx).await.unwrap_or(()); // do nothing if `store` fails
        }
    });

    let http_rocket = if ctx.tls.is_some() {
        Some(http_rocket(&ctx, ctx_prov.clone(), log_level).launch())
    } else {
        None
    };

    let rocket = rocket(&ctx, ctx_prov, log_level).launch();

    if let Some(http_rocket) = http_rocket {
        let result = future::join(http_rocket, rocket).await;
        let _ = result.0?;
        let _ = result.1?;
    } else {
        let _ = rocket.await?;
    }
    Ok(())
}

/// Returns a rocket that redirects to HTTPS
fn http_rocket(
    ctx: &Context,
    ctx_provider: Provider<Context>,
    log_level: LogLevel,
) -> Rocket<Build> {
    #[allow(clippy::unwrap_used)] // Deserializing into a `Config` will always succeed
    let rocket_cfg = rocket::Config {
        port: ctx.port,
        address: ctx.host.unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
        log_level,
        ..rocket::Config::figment()
            .extract::<rocket::Config>()
            .unwrap()
    };

    rocket::custom(&rocket_cfg)
        .mount("/", rocket::routes![redirect])
        .manage(ctx_provider)
}

/// Returns the main server rocket
fn rocket(ctx: &Context, ctx_provider: Provider<Context>, log_level: LogLevel) -> Rocket<Build> {
    #[allow(clippy::unwrap_used)] // Deserializing into a `Config` will always succeed
    let rocket_cfg = match &ctx.tls {
        Some(tls) => rocket::Config {
            tls: Some(TlsConfig::from_paths(&tls.cert, &tls.key)),
            port: tls.port,
            address: ctx.host.unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            log_level,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        },
        None => rocket::Config {
            port: ctx.port,
            address: ctx.host.unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            log_level,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        },
    };

    #[cfg(debug_assertions)]
    let routes = rocket::routes![login, get_config, put_config, stream];
    #[cfg(not(debug_assertions))]
    let routes = rocket::routes![files, login, get_config, put_config, stream];

    let media_seg_rx = stream_media_segments(ctx_provider.clone());

    rocket::custom(&rocket_cfg)
        .mount("/", routes)
        .manage(ctx_provider)
        .manage(media_seg_rx)
}
