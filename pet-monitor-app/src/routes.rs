//! This module provides Rocket routes for the server.

use crate::{auth, secrets};
use jsonwebtoken::errors::ErrorKind;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::{get, post, State};

/// Validates a password and issues tokens.
///
/// It accepts POSTs to the `/api/auth` path with the password as plain text.
/// If the password is correct, it adds a `token` cookie containing a JWT.
///
/// # Example
/// ```rust
/// use pet_monitor_app::routes::*;
/// use rocket::routes;
///
/// let rocket = rocket::build()
///     .mount("/", routes![login]);
/// ```
#[post("/api/login", data = "<password>")]
pub fn login(
    password: String,
    cookies: &CookieJar<'_>,
    hash: &State<secrets::Password>,
    secret: &State<secrets::Secret>,
) -> Result<(), Status> {
    if let Ok(b) = auth::validate(&password, &**hash) {
        if b {
            let token = match auth::Token::new().to_string(secret) {
                Ok(t) => t,
                Err(_) => return Err(Status::InternalServerError),
            };
            cookies.add(Cookie::new("token", token));
            Ok(())
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::InternalServerError)
    }
}

/// An authorization route that returns [`Status::Ok`](rocket::http::Status::Ok)
/// if the request has a valid token.
///
/// It accepts GETs to `/api/auth` and returns status code 200 if the request
/// has a `token` cookie that is a valid JWT. If JWT decoding fails, it returns
/// a [`Status::InternalServerError`](rocket::http::Status::InternalServerError).
/// If the token is expired or has an invalid signature, it returns a
/// [`Status::Unauthorized`](rocket::http::Status::Unauthorized).
///
/// # Example
/// ```rust
/// use pet_monitor_app::routes::*;
/// use rocket::routes;
///
/// let rocket = rocket::build()
///     .mount("/", routes![verify]);
/// ```
#[get("/api/auth")]
pub fn verify(cookies: &CookieJar<'_>, secret: &State<secrets::Secret>) -> Status {
    match cookies.get("token") {
        Some(cookie) => match auth::Token::from_str(cookie.value(), secret) {
            Ok(t) => {
                if t.verify() {
                    Status::Ok
                } else {
                    Status::Unauthorized
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::Base64(_)
                | ErrorKind::Crypto(_)
                | ErrorKind::Json(_)
                | ErrorKind::Utf8(_) => Status::InternalServerError,
                _ => Status::Unauthorized,
            },
        },
        None => Status::Unauthorized,
    }
}
