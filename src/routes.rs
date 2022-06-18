use crate::auth;
use rocket::http::{CookieJar, Status, Cookie};
use rocket::{get, post};
use once_cell::sync::Lazy;
use std::env;
use jsonwebtoken::errors::ErrorKind;

#[post("/api/auth", data = "<password>")]
pub fn login(password: String, cookies: &CookieJar<'_>) -> Status {
    static PASSWORD: Lazy<String> =
        Lazy::new(|| env::var("PASSWORD").expect("Please set the 'PASSWORD' env var."));

    if password == *PASSWORD {
        let token = match String::try_from(auth::Token::new()) {
            Ok(t) => t,
            Err(e) => match e.kind() {
                ErrorKind::Base64(_) |
                ErrorKind::Crypto(_) |
                ErrorKind::Json(_) |
                ErrorKind::Utf8(_) => return Status::InternalServerError,
                _ => return Status::Unauthorized,
            }
        };
        cookies.add(Cookie::new("token", token));
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[get("/api/stream")]
pub fn stream(cookies: &CookieJar<'_>) -> Result<String, Status> {
    match cookies.get("token") {
        Some(cookie) => match auth::Token::try_from(cookie.value()) {
            Ok(_) => Ok("stream".to_string()),
            Err(_) => Err(Status::Unauthorized),
        },
        None => Err(Status::Unauthorized),
    }
}
