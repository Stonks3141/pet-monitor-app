//! This crate provides a web server for [pet-monitor-app](https://github.com/Stonks3141/pet-monitor-app).
//!
//! The release binary should be run in a Docker container or have access to `/var/local`.

#![deny(missing_docs)]

mod auth;
mod routes;
mod secrets;
mod stream;
#[cfg(test)]
mod tests;

use ring::rand::SystemRandom;
use routes::*;

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
    let rng = SystemRandom::new();
    let pwd = secrets::Password::new(&rng)
        .await
        .expect("Failed to initialize password.");
    let secret = secrets::Secret::new(&rng)
        .await
        .expect("Failed to initialize JWT secret.");

    rocket::build()
        .mount("/", rocket::routes![login, verify])
        .manage(pwd)
        .manage(secret)
        .manage(rng)
}
