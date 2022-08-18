//! This module provides Rocket routes for the server.

use crate::auth::{self, Token};
use crate::config::{Config, Context};
use include_dir::{include_dir, Dir};
use rocket::http::{ContentType, Cookie, CookieJar, SameSite, Status};
//use rocket::response::stream::ByteStream;
use rocket::serde::json::Json;
use rocket::tokio::sync::{mpsc, oneshot};
use rocket::{get, post, put, State};
use std::path::PathBuf;

type Manager<T> = mpsc::Sender<(Option<T>, oneshot::Sender<T>)>;

const STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../client/dist");

#[get("/<path..>", rank = 2)]
pub fn files(path: PathBuf) -> (ContentType, String) {
    if let Some(s) = STATIC_FILES
        .get_file(&path)
        .map(|f| f.contents_utf8().unwrap())
    {
        (
            if let Some(ext) = path.extension() {
                ContentType::from_extension(&ext.to_string_lossy()).unwrap_or(ContentType::Plain)
            } else {
                ContentType::Plain
            },
            s.to_string(),
        )
    } else {
        (
            ContentType::HTML,
            STATIC_FILES
                .get_file("index.html")
                .unwrap()
                .contents_utf8()
                .unwrap()
                .to_string(),
        )
    }
}

/// Validates a password and issues tokens.
///
/// It accepts POSTs to the `/api/auth` path with the password as plain text.
/// If the password is correct, it adds a `token` cookie containing a JWT.
#[post("/api/login", data = "<password>")]
pub async fn login(
    password: String,
    cookies: &CookieJar<'_>,
    ctx: &State<Manager<Context>>,
) -> Status {
    let (tx, rx) = oneshot::channel();
    ctx.send((None, tx)).await.unwrap();
    let ctx = rx.await.unwrap();

    if let Ok(b) = auth::validate(&password, &ctx.password_hash) {
        if b {
            match Token::new(ctx.jwt_timeout).to_string(&ctx.jwt_secret) {
                Ok(token) => {
                    let cookie = Cookie::build("token", token)
                        .http_only(true)
                        .max_age(rocket::time::Duration::seconds(
                            ctx.jwt_timeout.num_seconds(),
                        ))
                        .same_site(SameSite::Strict)
                        .finish();

                    cookies.add(cookie);

                    Status::Ok
                }
                Err(_) => Status::InternalServerError,
            }
        } else {
            Status::Unauthorized
        }
    } else {
        Status::InternalServerError
    }
}

#[get("/api/logout")]
pub fn logout(cookies: &CookieJar<'_>) {
    cookies.remove(Cookie::named("token"));
}

#[get("/api/config")]
pub async fn get_config(
    _token: Token,
    ctx: &State<Manager<Context>>,
) -> Result<Json<Config>, Status> {
    let (tx, rx) = oneshot::channel();
    ctx.send((None, tx)).await.unwrap();
    let ctx = rx.await.unwrap();
    Ok(Json(ctx.config))
}

#[put("/api/config", format = "json", data = "<new_config>")]
pub async fn put_config(
    _token: Token,
    ctx: &State<Manager<Context>>,
    new_config: Json<Config>,
) -> Result<(), Status> {
    let (tx, rx) = oneshot::channel();
    ctx.send((None, tx)).await.unwrap();
    let ctx_read = rx.await.unwrap();

    let new_ctx = Context {
        config: new_config.into_inner(),
        ..ctx_read
    };

    let (tx, rx) = oneshot::channel();
    ctx.send((Some(new_ctx.clone()), tx)).await.unwrap();
    rx.await.unwrap();

    Ok(())
}
/*
#[get("/stream.mp4")]
pub fn stream_mp4(
    cookies: &CookieJar<'_>,
    ctx: &State<RwLock<Context>>,
) -> Result<ByteStream![Vec<u8>], Status> {
    // can't match directly for some reason even though `Status` impls `Eq` and `PartialEq`.
    match verify(&cookies, &*ctx).code {
        200 => Ok(ByteStream(video_stream())),
        c => Err(Status::new(c)),
    }
}
*/
