//! This module provides Rocket routes for the server.

use crate::auth;
use crate::config::{Config, Context};
use include_dir::{include_dir, Dir};
use jsonwebtoken::errors::ErrorKind;
use rocket::http::{ContentType, Cookie, CookieJar, Status};
//use rocket::response::stream::ByteStream;
use rocket::serde::json::Json;
use rocket::tokio::sync::{mpsc, oneshot};
use rocket::{get, post, put, State};
use std::path::PathBuf;

const STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../client/build");

#[get("/<path..>", rank = 2)]
pub fn files(path: PathBuf) -> (ContentType, &'static str) {
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
            s,
        )
    } else {
        (
            ContentType::HTML,
            STATIC_FILES
                .get_file("index.html")
                .unwrap()
                .contents_utf8()
                .unwrap(),
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
    ctx: &State<mpsc::Sender<(Option<Context>, oneshot::Sender<Context>)>>,
) -> Result<(), Status> {
    let (tx, rx) = oneshot::channel();
    ctx.send((None, tx)).await.unwrap();
    let ctx = rx.await.unwrap();

    if let Ok(b) = auth::validate(&password, &ctx.password_hash) {
        if b {
            let token = match auth::Token::with_expiration(ctx.config.jwt_timeout)
                .to_string(&ctx.jwt_secret)
            {
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

/// A utility function that returns [`Status::Ok`](rocket::http::Status::Ok)
/// if the request has a valid token.
///
/// It returns status code 200 if the request has a `token` cookie that is a
/// valid JWT. If JWT decoding fails, it returns a
/// [`Status::InternalServerError`](rocket::http::Status::InternalServerError).
/// If the token is expired or has an invalid signature, it returns a
/// [`Status::Unauthorized`](rocket::http::Status::Unauthorized).
pub fn verify(cookies: &CookieJar<'_>, ctx: &Context) -> Status {
    match cookies.get("token") {
        Some(cookie) => match auth::Token::from_str(cookie.value(), &ctx.jwt_secret) {
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

#[get("/api/config")]
pub async fn get_config(
    cookies: &CookieJar<'_>,
    ctx: &State<mpsc::Sender<(Option<Context>, oneshot::Sender<Context>)>>,
) -> Result<Json<Config>, Status> {
    let (tx, rx) = oneshot::channel();
    ctx.send((None, tx)).await.unwrap();
    let ctx = rx.await.unwrap();

    match verify(&cookies, &ctx).code {
        200 => Ok(Json(ctx.config)),
        c => Err(Status::new(c)),
    }
}

#[put("/api/config", format = "json", data = "<new_config>")]
pub async fn put_config(
    cookies: &CookieJar<'_>,
    ctx: &State<mpsc::Sender<(Option<Context>, oneshot::Sender<Context>)>>,
    new_config: Json<Config>,
) -> Result<(), Status> {
    let (tx, rx) = oneshot::channel();
    ctx.send((None, tx)).await.unwrap();
    let ctx_read = rx.await.unwrap();

    match verify(&cookies, &ctx_read).code {
        200 => {
            let new_config = new_config.into_inner();

            let new_ctx = Context {
                config: new_config,
                ..ctx_read
            };

            let (tx, rx) = oneshot::channel();
            ctx.send((Some(new_ctx.clone()), tx)).await.unwrap();
            rx.await.unwrap();

            Ok(())
        }
        c => Err(Status::new(c)),
    }
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
