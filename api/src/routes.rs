use crate::auth;
use jsonwebtoken::errors::ErrorKind;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::{get, post};

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

#[get("/api/stream")]
pub fn stream(cookies: &CookieJar<'_>) -> Result<String, Status> {
    match cookies.get("token") {
        Some(cookie) => match cookie.value().parse::<auth::Token>() {
            Ok(_) => Ok("stream".to_string()),
            Err(e) => match e.kind() {
                ErrorKind::Base64(_) |
                ErrorKind::Crypto(_) |
                ErrorKind::Json(_) |
                ErrorKind::Utf8(_) => Err(Status::InternalServerError),
                _ => Err(Status::Unauthorized),
            },
        },
        None => Err(Status::Unauthorized),
    }
}
