use crate::config::Context;
use provider::Provider;
use rocket::config::TlsConfig;
use rocket::futures::future;
use rocket::{Build, Rocket};
use std::net::{IpAddr, Ipv4Addr};

mod auth;
mod provider;
mod routes;
use routes::*;

pub async fn launch(conf_path: Option<std::path::PathBuf>, ctx: Context) {
    let cfg_tx = provider::Provider::new(ctx.clone(), move |ctx| {
        if let Some(path) = &conf_path {
            confy::store_path(&path, &ctx).expect("Failed to save to configuration file")
        } else {
            confy::store("pet-monitor-app", &ctx).expect("Failed to save to configuration file")
        };
    });

    let http_rocket = if ctx.tls.is_some() {
        Some(http_rocket(&ctx, cfg_tx.clone()).launch())
    } else {
        None
    };

    let rocket = rocket(&ctx, cfg_tx).launch();

    if let Some(http_rocket) = http_rocket {
        let result = future::join(http_rocket, rocket).await;
        let _ = result.0.unwrap();
        let _ = result.1.unwrap();
    } else {
        let _ = rocket.await.unwrap();
    }
}

fn http_rocket(ctx: &Context, ctx_provider: Provider<Context>) -> Rocket<Build> {
    let rocket_cfg = rocket::Config {
        port: ctx.port,
        address: ctx
            .host
            .clone()
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
        ..rocket::Config::figment()
            .extract::<rocket::Config>()
            .unwrap()
    };

    rocket::custom(&rocket_cfg)
        .mount("/", rocket::routes![redirect])
        .manage(ctx_provider)
}

fn rocket(ctx: &Context, ctx_provider: Provider<Context>) -> Rocket<Build> {
    let rocket_cfg = if let Some(tls) = &ctx.tls {
        rocket::Config {
            tls: Some(TlsConfig::from_paths(&tls.cert, &tls.key)),
            port: tls.port,
            address: ctx.host.unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    } else {
        rocket::Config {
            port: ctx.port,
            address: ctx.host.unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    };

    let rocket = rocket::custom(&rocket_cfg)
        .mount("/", rocket::routes![login, get_config, put_config])
        .manage(ctx_provider);

    #[cfg(not(debug_assertions))]
    let rocket = rocket.mount("/", rocket::routes![files]);

    rocket
}
