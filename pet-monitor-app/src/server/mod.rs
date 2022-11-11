//! This module contains all server-related logic.

use crate::config::Context;
use fmp4::stream_media_segments;
use provider::Provider;
use rocket::config::LogLevel;
use rocket::config::TlsConfig;
use rocket::futures::future;
use rocket::{Build, Rocket};
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
    stream: bool,
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

    let rocket = rocket(&ctx, ctx_prov, log_level, stream).await?.launch();

    if let Some(http_rocket) = http_rocket {
        let result = future::join(http_rocket, rocket).await;
        let _x = result.0?;
        let _x = result.1?;
    } else {
        let _x = rocket.await?;
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
        address: ctx.host,
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
async fn rocket(
    ctx: &Context,
    ctx_provider: Provider<Context>,
    log_level: LogLevel,
    stream: bool,
) -> anyhow::Result<Rocket<Build>> {
    #[allow(clippy::unwrap_used)] // Deserializing into a `Config` will always succeed
    let rocket_cfg = match &ctx.tls {
        Some(tls) => rocket::Config {
            tls: Some(TlsConfig::from_paths(&tls.cert, &tls.key)),
            port: tls.port,
            address: ctx.host,
            log_level,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        },
        None => rocket::Config {
            port: ctx.port,
            address: ctx.host,
            log_level,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        },
    };

    let mut routes = rocket::routes![login, get_config, put_config, capabilities];
    if stream {
        routes.append(&mut rocket::routes![stream]);
    }
    #[cfg(not(debug_assertions))]
    routes.append(&mut rocket::routes![files]);

    let caps = fmp4::capabilities::get_capabilities_all().await?;
    fmp4::capabilities::check_config(&ctx.config, &caps)?;

    let mut rocket = rocket::custom(&rocket_cfg)
        .mount("/", routes)
        .manage(ctx_provider.clone())
        .manage(caps);

    if stream {
        let media_seg_rx = stream_media_segments(ctx_provider);
        rocket = rocket.manage(media_seg_rx);
    }
    Ok(rocket)
}
