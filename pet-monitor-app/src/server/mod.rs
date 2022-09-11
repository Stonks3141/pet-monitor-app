use crate::config::Context;
use rocket::config::TlsConfig;
use rocket::futures::future;
use std::net::{IpAddr, Ipv4Addr};

mod auth;
mod routes;
use routes::*;

pub async fn rocket(conf_path: Option<std::path::PathBuf>, ctx: Context) {
    let cfg_tx = provider::Provider::new(ctx.clone(), move |ctx| {
        if let Some(path) = &conf_path {
            confy::store_path(&path, &ctx).expect("Failed to save to configuration file")
        } else {
            confy::store("pet-monitor-app", &ctx).expect("Failed to save to configuration file")
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

    let http_rocket = if ctx.tls.is_some() {
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
            address: ctx.host.unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            ..rocket::Config::figment()
                .extract::<rocket::Config>()
                .unwrap()
        }
    } else {
        rocket::Config {
            port: PORT,
            address: ctx.host.unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
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
        let _ = result.0.unwrap();
        let _ = result.1.unwrap();
    } else {
        let result = rocket.await;
        let _ = result.unwrap();
    }
}

mod provider {
    //! Async mutable globals with channels

    use rocket::tokio::sync::{mpsc, oneshot};
    use std::fmt::Debug;

    #[derive(Debug, Clone)]
    pub struct Provider<T>(mpsc::Sender<(Option<T>, oneshot::Sender<T>)>);

    impl<T: Debug> Provider<T> {
        pub fn new<F>(mut val: T, mut on_set: F) -> Self
        where
            T: Clone + Debug + Send + 'static,
            F: FnMut(&T) + Send + 'static,
        {
            let (tx, mut rx) = mpsc::channel::<(Option<T>, oneshot::Sender<T>)>(100);
            rocket::tokio::spawn(async move {
                while let Some((new, response)) = rx.recv().await {
                    if let Some(new) = new {
                        let prev = val.clone();
                        val = new;
                        on_set(&val);
                        response.send(prev).unwrap();
                    } else {
                        response.send(val.clone()).unwrap();
                    }
                }
            });
            Self(tx)
        }

        pub async fn get(&self) -> T {
            let (tx, rx) = oneshot::channel();
            self.0.send((None, tx)).await.unwrap();
            rx.await.unwrap()
        }

        pub async fn set(&self, new: T) {
            let (tx, rx) = oneshot::channel();
            self.0.send((Some(new), tx)).await.unwrap();
            rx.await.unwrap();
        }
    }
}
