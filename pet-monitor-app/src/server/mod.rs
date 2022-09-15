use crate::config::Context;
use provider::Provider;
use rocket::config::TlsConfig;
use rocket::futures::future;
use rocket::{Build, Rocket};
use std::net::{IpAddr, Ipv4Addr};

mod auth;
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

mod provider {
    //! Async interior mutability with channels

    use rocket::tokio::sync::{mpsc, oneshot};
    use std::fmt::Debug;

    /// The `Provider` type uses async channels to implement thread-safe interior
    /// mutability.
    #[derive(Debug, Clone)]
    pub struct Provider<T>(mpsc::Sender<(Option<T>, oneshot::Sender<T>)>);

    impl<T: Debug> Provider<T> {
        /// Creates a new `Provider`.
        /// 
        /// The `on_set` callback will be run with the new value whenever
        /// `Provider::set` is called.
        pub fn new<F>(val: T, mut on_set: F) -> Self
        where
            T: Clone + Send + 'static,
            F: FnMut(&T) + Send + 'static,
        {
            let (tx, mut rx) = mpsc::channel::<(Option<T>, oneshot::Sender<T>)>(100);
            let mut val = val.clone();
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

        /// Gets the value stored in the `Provider`.
        #[inline]
        pub async fn get(&self) -> T {
            let (tx, rx) = oneshot::channel();
            self.0.send((None, tx)).await.unwrap();
            rx.await.unwrap()
        }

        /// Replaces the value in the `Provider` with a new value.
        #[inline]
        pub async fn set(&self, new: T) {
            let (tx, rx) = oneshot::channel();
            self.0.send((Some(new), tx)).await.unwrap();
            rx.await.unwrap();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rocket::tokio;
        use std::sync::{Arc, Mutex};

        #[tokio::test]
        async fn test_provider() {
            let val = "foo".to_string();
            let mutex = Arc::new(Mutex::new(false));
            let mutex_clone = mutex.clone();

            let prov = Provider::new(val.clone(), move |_| {
                *mutex_clone.lock().unwrap() = true;
            });

            assert_eq!(val, prov.get().await);
            assert_eq!(false, *mutex.lock().unwrap());

            let val = "bar".to_string();
            prov.set(val.clone()).await;

            assert_eq!(val, prov.get().await);
            assert_eq!(true, *mutex.lock().unwrap());
        }
    }
}
