//! This crate provides a web server for [pet-monitor-app](https://github.com/Stonks3141/pet-monitor-app).
//!
//! The release binary should be run in a Docker container or have access to `/var/local`.

#![deny(missing_docs)]

use log::{debug, error, info, trace, warn};
use ring::rand::SystemRandom;
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

    info!("Reading configuration");
    let mut conf: config::Config = if let Some(path) = &options.conf_path {
        confy::load_path(&path).expect("Failed to load configuration file")
    } else {
        confy::load("pet-monitor-app").expect("Failed to load configuration file")
    };

    info!("Initializing PRNG");
    let rng = SystemRandom::new();

    if let Some(pwd) = options.password {
        info!("Initializing password hash");
        conf.password_hash = secrets::init_password(&rng, &pwd).unwrap();
    }

    if options.regen_secret {
        info!("Initializing JWT secret");
        conf.jwt_secret = secrets::new_secret(&rng).unwrap();
    }

    if let Some(path) = &options.conf_path {
        confy::store_path(&path, &conf).expect("Failed to load configuration file")
    } else {
        confy::store("pet-monitor-app", &conf).expect("Failed to load configuration file")
    };

    rocket::build()
        .mount("/", rocket::routes![files, login, verify, stream_mp4])
        .manage(conf)
        .manage(rng)
}
