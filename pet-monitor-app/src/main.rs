#![deny(unsafe_code)]

use config::Context;
use human_panic::setup_panic;
use ring::rand::SystemRandom;
use rocket::config::TlsConfig;
use rocket::futures::future;
use routes::*;
use std::net::{IpAddr, Ipv4Addr};

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
    setup_panic!();

    let cmd = cli::parse_args(std::env::args());

    let mut ctx: Context = if let Some(path) = &cmd.conf_path {
        confy::load_path(&path).expect("Failed to load configuration file")
    } else {
        confy::load("pet-monitor-app").expect("Failed to load configuration file")
    };

    match cmd.command {
        cli::SubCmd::Configure { password, regen_secret } => {
            let rng = SystemRandom::new();

            if let Some(pwd) = password {
                ctx.password_hash = secrets::init_password(&rng, &pwd).unwrap();
                println!("Hashed new password");
            }

            if regen_secret {
                ctx.jwt_secret = secrets::new_secret(&rng).unwrap();
                println!("Regenerated JWT signing secret");
            }

            if let Some(path) = &cmd.conf_path {
                confy::store_path(&path, &ctx).expect("Failed to load configuration file")
            } else {
                confy::store("pet-monitor-app", &ctx).expect("Failed to load configuration file")
            };
        }
        cli::SubCmd::Start => rocket(cmd.conf_path, ctx).await
    }
}

async fn rocket(conf_path: Option<std::path::PathBuf>, ctx: Context) {
    let conf_path = conf_path.clone();
    let cfg_tx = provider::new_provider(ctx.clone(), move |ctx| {
        if let Some(path) = &conf_path {
            confy::store_path(&path, &ctx).expect("Failed to save to configuration file")
        } else {
            confy::store("pet-monitor-app", &ctx)
                .expect("Failed to save to configuration file")
        };
    });

    #[cfg(debug_assertions)]
    const PORT: u16 = 8080;
    #[cfg(not(debug_assertions))]
    const PORT: u16 = 80;

    #[cfg(debug_assertions)]
    const TLS_PORT: u16 = 8443;
    #[cfg(not(debug_assertions))]
    const TLS_PORT: u16 = 443;

    let http_rocket = ctx.tls.clone().map(|_| {
        let rocket_cfg = rocket::Config {
            port: PORT,
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
            .manage(cfg_tx.clone())
            .launch()
    });

    let rocket_cfg = if let Some(tls) = &ctx.tls {
        rocket::Config {
            tls: Some(TlsConfig::from_paths(&tls.cert, &tls.key)),
            port: TLS_PORT,
            address: ctx
                .host
                .clone()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    } else {
        rocket::Config {
            port: PORT,
            address: ctx
                .host
                .clone()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    };

    let rocket = rocket::custom(&rocket_cfg)
        .mount("/", rocket::routes![login, get_config, put_config])
        .manage(cfg_tx);

    #[cfg(not(debug_assertions))]
    let rocket = rocket.mount("/", rocket::routes![files]);

    let rocket = rocket.launch();

    if let Some(http_rocket) = http_rocket {
        let result = future::join(http_rocket, rocket).await;
        // Rocket takes over the panic hook, so we have to reset it.
        setup_panic!();
        let _ = result.0.unwrap();
        let _ = result.1.unwrap();
    } else {
        let result = rocket.await;
        setup_panic!();
        let _ = result.unwrap();
    }
}

pub mod provider {
    //! Async mutable globals with channels

    use rocket::tokio::sync::{mpsc, oneshot};
    use std::fmt::Debug;

    pub type Provider<T> = mpsc::Sender<(Option<T>, oneshot::Sender<T>)>;

    pub async fn get_provider<T: Debug>(provider: &Provider<T>) -> T {
        let (tx, rx) = oneshot::channel();
        provider.send((None, tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn set_provider<T: Debug>(provider: &Provider<T>, new: T) {
        let (tx, rx) = oneshot::channel();
        provider.send((Some(new), tx)).await.unwrap();
        rx.await.unwrap();
    }

    pub fn new_provider<T, F>(initial: T, mut on_set: F) -> Provider<T>
    where
        T: Clone + Debug + Send + 'static,
        F: FnMut(T) + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<(Option<T>, oneshot::Sender<T>)>(100);
        rocket::tokio::spawn(async move {
            let mut val = initial.clone();
            while let Some((new, response)) = rx.recv().await {
                if let Some(new) = new {
                    let prev = val.clone();
                    val = new;
                    on_set(val.clone());
                    response.send(prev).unwrap();
                } else {
                    response.send(val.clone()).unwrap();
                }
            }
        });
        tx
    }
}
