//! This module contains all server-related logic.

use crate::config::{Context, ContextManager};
use mp4_stream::{
    capabilities::{check_config, get_capabilities_all},
    config::Config,
    stream_media_segments,
};
use rocket::{
    config::{LogLevel, TlsConfig},
    futures::future,
    Build, Rocket,
};
use routes::*;
use std::path::PathBuf;

mod auth;
mod routes;

/// Launches the server
pub async fn launch(
    conf_path: Option<PathBuf>,
    ctx: Context,
    log_level: LogLevel,
    stream: bool,
) -> anyhow::Result<()> {
    let (ctx_manager, cfg_rx) = ContextManager::new(ctx.clone(), conf_path.clone());

    let http_rocket = if ctx.tls.is_some() {
        Some(http_rocket(&ctx, ctx_manager.clone(), log_level).launch())
    } else {
        None
    };

    let rocket = rocket(&ctx, ctx_manager, cfg_rx, log_level, stream)?.launch();

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
fn http_rocket(ctx: &Context, ctx_manager: ContextManager, log_level: LogLevel) -> Rocket<Build> {
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
        .manage(ctx_manager)
}

/// Returns the main server rocket
fn rocket(
    ctx: &Context,
    ctx_manager: ContextManager,
    cfg_rx: flume::Receiver<Config>,
    log_level: LogLevel,
    stream: bool,
) -> anyhow::Result<Rocket<Build>> {
    #[allow(clippy::unwrap_used)] // Deserializing into a `Config` will always succeed
    let rocket_cfg = if let Some(tls) = &ctx.tls {
        rocket::Config {
            tls: Some(TlsConfig::from_paths(&tls.cert, &tls.key)),
            port: tls.port,
            address: ctx.host,
            log_level,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    } else {
        rocket::Config {
            port: ctx.port,
            address: ctx.host,
            log_level,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    };

    let mut routes = rocket::routes![login, get_config, put_config, capabilities];
    if stream {
        routes.append(&mut rocket::routes![stream]);
    }
    #[cfg(not(debug_assertions))]
    routes.append(&mut rocket::routes![files]);

    let caps = get_capabilities_all()?;
    check_config(&ctx.config, &caps)?;

    let mut rocket = rocket::custom(&rocket_cfg)
        .mount("/", routes)
        .manage(ctx_manager)
        .manage(caps);

    if stream {
        let (tx, rx) = flume::unbounded();
        let config = ctx.config.clone();
        rocket::tokio::task::spawn_blocking(move || {
            // not much we can do about an error at this point, the server is already started
            #[allow(clippy::unwrap_used)]
            stream_media_segments(rx, config, Some(cfg_rx)).unwrap();
        });
        rocket = rocket.manage(tx);
    }
    Ok(rocket)
}
