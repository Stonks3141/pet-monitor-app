// Pet Montitor App
// Copyright (C) 2022  Samuel Nystrom
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
#[post("/api/auth", data = "<password>")]
pub fn login(
    password: String,
    cookies: &CookieJar<'_>,
    hash: &State<secrets::Password>,
    secret: &State<secrets::Secret>,
) -> Result<(), Status> {
    if let Ok(b) = auth::validate(&password, &**hash) {
        if b {
            let token = match auth::Token::new().to_string(&secret) {
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

/// A test route that validates the client's token.
///
/// It accepts GETs to `/api/stream` and returns the word "stream" as plain
/// text if the request has a `token` cookie that is a valid JWT. If JWT
/// decoding fails, it returns a
/// [`Err(Status::InternalServerError)`](rocket::http::Status::InternalServerError).
///
/// # Example
/// ```rust
/// use pet_monitor_app::routes::*;
/// use rocket::routes;
///
/// let rocket = rocket::build()
///     .mount("/", routes![stream]);
/// ```
#[get("/api/stream")]
pub fn stream(cookies: &CookieJar<'_>, secret: &State<secrets::Secret>) -> Result<String, Status> {
    match cookies.get("token") {
        Some(cookie) => match auth::Token::from_str(cookie.value(), secret) {
            Ok(t) => {
                if t.verify() {
                    Ok("stream".to_string())
                } else {
                    Err(Status::Unauthorized)
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::Base64(_)
                | ErrorKind::Crypto(_)
                | ErrorKind::Json(_)
                | ErrorKind::Utf8(_) => Err(Status::InternalServerError),
                _ => Err(Status::Unauthorized),
            },
        },
        None => Err(Status::Unauthorized),
    }
}
