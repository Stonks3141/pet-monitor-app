use crate::auth;
use rocket::http::{CookieJar, Status};
use rocket::{get, post};

#[post("/api/auth", data = "<password>")]
pub fn login(password: String, cookies: &CookieJar<'_>) -> Status {
    match auth::authenticate(&password) {
        Ok(cookie) => {
            cookies.add(cookie);
            Status::Ok
        }
        Err(e) => e,
    }
}

#[get("/api/stream")]
pub fn stream(cookies: &CookieJar<'_>) -> Result<String, Status> {
    let token = match cookies.get("token") {
        Some(token) => token.value(),
        None => return Err(Status::Unauthorized),
    };

    // normal matching doesn't work for some reason
    match auth::validate_token(token).code {
        200 => Ok("stream".to_string()),
        e => Err(Status::new(e)),
    }
}
