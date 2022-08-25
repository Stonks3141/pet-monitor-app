use config::Context;
use ring::rand::SystemRandom;
use rocket::config::TlsConfig;
use rocket::futures::future;
use rocket::response::stream::ByteStream;
use rocket::tokio::sync::{mpsc, oneshot};
use routes::*;

mod auth;
mod cli;
mod config;
mod routes;
mod secrets;
mod stream;
#[cfg(test)]
mod tests;

#[rocket::main]
async fn main() {
    let options = cli::parse_args();

    let mut ctx: Context = if let Some(path) = &options.conf_path {
        confy::load_path(&path).expect("Failed to load configuration file")
    } else {
        confy::load("pet-monitor-app").expect("Failed to load configuration file")
    };

    let rng = SystemRandom::new();

    if let Some(pwd) = &options.password {
        ctx.password_hash = secrets::init_password(&rng, pwd).unwrap();
    }

    if options.regen_secret {
        ctx.jwt_secret = secrets::new_secret(&rng).unwrap();
    }

    if let Some(path) = &options.conf_path {
        confy::store_path(&path, &ctx).expect("Failed to load configuration file")
    } else {
        confy::store("pet-monitor-app", &ctx).expect("Failed to load configuration file")
    };

    let stream = ByteStream(stream::video_stream(&ctx.config));

    let (cfg_tx, mut cfg_rx) = mpsc::channel::<(Option<Context>, oneshot::Sender<Context>)>(100);

    let ctx_clone = ctx.clone();
    let conf_path = options.conf_path.clone();
    rocket::tokio::spawn(async move {
        let mut ctx = ctx_clone;
        while let Some((new, response)) = cfg_rx.recv().await {
            if let Some(new) = new {
                let prev = ctx.clone();
                ctx = new;

                if let Some(path) = &conf_path {
                    confy::store_path(&path, &ctx).expect("Failed to load configuration file")
                } else {
                    confy::store("pet-monitor-app", &ctx)
                        .expect("Failed to load configuration file")
                };

                response.send(prev).unwrap();
            } else {
                response.send(ctx.clone()).unwrap();
            }
        }
    });

    #[cfg(debug_assertions)]
    const PORT: u16 = 8080;
    #[cfg(not(debug_assertions))]
    const PORT: u16 = 80;

    #[cfg(debug_assertions)]
    const TLS_PORT: u16 = 8443;
    #[cfg(not(debug_assertions))]
    const TLS_PORT: u16 = 443;

    let http_rocket = if ctx.tls.is_some() {
        let rocket_cfg = rocket::Config {
            port: PORT,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        };

        Some(
            rocket::custom(&rocket_cfg)
                .mount("/", rocket::routes![redirect])
                .manage(cfg_tx.clone())
                .launch(),
        )
    } else {
        None
    };

    let rocket_cfg = if let Some(tls) = &ctx.tls {
        rocket::Config {
            tls: Some(TlsConfig::from_paths(&tls.cert, &tls.key)),
            port: TLS_PORT,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    } else {
        rocket::Config {
            port: PORT,
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    };

    let rocket = rocket::custom(&rocket_cfg)
        .mount("/", rocket::routes![login, get_config, put_config])
        .manage(cfg_tx)
        .manage(options)
        .manage(stream);

    #[cfg(not(debug_assertions))]
    rocket.mount("/", rocket::routes![files]);

    rocket.launch();

    if let Some(http_rocket) = http_rocket {
        let result = future::join(http_rocket, rocket).await;
        let _ = result.0.unwrap();
        let _ = result.1.unwrap();
    } else {
        let _ = rocket.await.unwrap();
    }
}
