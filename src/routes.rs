use crate::auth;
use jsonwebtoken::errors::ErrorKind;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::{get, post};

#[post("/api/auth", data = "<hash>")]
pub fn login(hash: String, cookies: &CookieJar<'_>) -> Status {
    if let Ok(b) = auth::validate(&hash) {
        if b {
            let token = match String::try_from(auth::Token::new()) {
                Ok(t) => t,
                Err(e) => match e.kind() {
                    ErrorKind::Base64(_)
                    | ErrorKind::Crypto(_)
                    | ErrorKind::Json(_)
                    | ErrorKind::Utf8(_) => return Status::InternalServerError,
                    _ => return Status::Unauthorized,
                },
            };
            cookies.add(Cookie::new("token", token));
            Status::Ok
        } else {
            Status::Unauthorized
        }
    } else {
        Status::InternalServerError
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
