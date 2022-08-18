//! This crate provides a web server for [pet-monitor-app](https://github.com/Stonks3141/pet-monitor-app).
//!
//! The release binary should be run in a Docker container or have access to `/var/local`.

#![deny(missing_docs)]

use config::Context;
use ring::rand::SystemRandom;
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

/// The main function for the program. This is a library function to make unit
/// and integration testing easier.
///
/// # Example
/// ```no_test
/// use rocket::local::blocking::Client;
/// use rocket::http::Status;
///
/// let client = Client::tracked(pet_monitor_app::rocket().await).unwrap();
/// let res = client.get("/api/auth").dispatch();
/// ```
pub async fn rocket() -> rocket::Rocket<rocket::Build> {
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

    let conf_path = options.conf_path.clone();
    rocket::tokio::spawn(async move {
        let mut ctx = ctx;
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
    /*
    let (stream_tx, mut stream_rx) = mpsc::channel::<(Option<ByteStream>, oneshot::Sender<ByteStream>)>(100);

    rocket::tokio::spawn(async move {
        let mut ctx = ctx;
        while let Some((new, response)) = stream_rx.recv().await {
            if let Some(new) = new {
                let prev = ctx.clone();
                ctx = new;
                response.send(prev).unwrap();
            } else {
                response.send(ctx.clone()).unwrap();
            }
        }
    });*/

    rocket::build()
        .mount(
            "/",
            rocket::routes![files, login, logout, get_config, put_config],
        )
        .manage(cfg_tx)
        .manage(options)
        .manage(stream)
}
