use rocket::fs::{relative, NamedFile};
use rocket::http::{CookieJar, Status};
use rocket::{get, post};
use crate::auth;

// match all for React Router
#[get("/<_f..>", rank = 2)]
pub async fn index(_f: std::path::PathBuf) -> Option<NamedFile> {
    NamedFile::open(relative!("client/build/index.html"))
        .await
        .ok()
}

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
    match auth::authorize(token).code {
        200 => Ok("stream".to_string()),
        e => Err(Status::new(e)),
    }
}
