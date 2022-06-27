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

use crate::auth;
use jsonwebtoken::errors::ErrorKind;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::{get, post};

/// Validates a password and issues tokens
#[post("/api/auth", data = "<password>")]
pub fn login(password: String, cookies: &CookieJar<'_>) -> Result<(), Status> {
    if let Ok(b) = auth::validate(&password) {
        if b {
            let token = match String::try_from(auth::Token::new()) {
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

/// A test route that validates the client's token
#[get("/api/stream")]
pub fn stream(cookies: &CookieJar<'_>) -> Result<String, Status> {
    match cookies.get("token") {
        Some(cookie) => match cookie.value().parse::<auth::Token>() {
            Ok(_) => Ok("stream".to_string()),
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
