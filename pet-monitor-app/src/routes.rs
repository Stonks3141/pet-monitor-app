//! This module provides Rocket routes for the server.

use crate::stream::video_stream;
use crate::{auth, config::Config, secrets};
use jsonwebtoken::errors::ErrorKind;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::stream::ByteStream;
use rocket::{get, post, State};

/// Validates a password and issues tokens.
///
/// It accepts POSTs to the `/api/auth` path with the password as plain text.
/// If the password is correct, it adds a `token` cookie containing a JWT.
#[post("/api/login", data = "<password>")]
pub fn login(
    password: String,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
) -> Result<(), Status> {
    if let Ok(b) = auth::validate(&password, &config.password_hash) {
        if b {
            let token = match auth::Token::new().to_string(&config.jwt_secret) {
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
#[get("/api/auth")]
pub fn verify(cookies: &CookieJar<'_>, config: &State<Config>) -> Status {
    match cookies.get("token") {
        Some(cookie) => match auth::Token::from_str(cookie.value(), &config.jwt_secret) {
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

#[get("/stream.mp4")]
pub fn stream_mp4() -> ByteStream![Vec<u8>] {
    ByteStream(video_stream())
}
