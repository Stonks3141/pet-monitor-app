//! This module provides Rocket routes for the server.

use super::auth::{self, Token};
use super::provider::Provider;
use crate::config::{Config, Context};
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};
#[cfg(not(debug_assertions))]
use rocket::http::ContentType;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, post, put, State};
use std::path::PathBuf;

#[get("/<path..>")]
pub async fn redirect(path: PathBuf, ctx: &State<Provider<Context>>) -> Redirect {
    println!("{}", path.as_path().to_string_lossy());
    let ctx = ctx.get().await;

    Redirect::permanent(format!(
        "https://{}/{}",
        ctx.domain,
        path.as_path().to_string_lossy()
    ))
}

#[cfg(not(debug_assertions))]
const STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/dist");

#[cfg(not(debug_assertions))]
#[get("/<path..>", rank = 2)]
pub fn files(path: PathBuf) -> Result<(ContentType, String), Status> {
    Ok(
        if let Some(s) = STATIC_FILES.get_file(&path).map(|f| {
            f.contents_utf8()
                .expect("All HTML/CSS/JS should be valid UTF-8")
        }) {
            (
                if let Some(ext) = path.extension() {
                    ContentType::from_extension(&ext.to_string_lossy())
                        .unwrap_or(ContentType::Plain)
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
                    .ok_or(Status::InternalServerError)?
                    .contents_utf8()
                    .ok_or(Status::InternalServerError)?
                    .to_string(),
            )
        },
    )
}

/// Validates a password and issues tokens.
///
/// It accepts POSTs to the `/api/auth` path with the password as plain text.
/// If the password is correct, it adds a `token` cookie containing a JWT.
#[post("/api/login", data = "<password>")]
pub async fn login(
    password: String,
    cookies: &CookieJar<'_>,
    ctx: &State<Provider<Context>>,
) -> Status {
    let ctx = ctx.get().await;

    if let Ok(b) = auth::validate(&password, &ctx.password_hash) {
        if b {
            match Token::new(ctx.jwt_timeout).to_string(&ctx.jwt_secret) {
                Ok(token) => {
                    let cookie = Cookie::build("token", token)
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

#[get("/api/config")]
pub async fn get_config(
    _token: Token,
    ctx: &State<Provider<Context>>,
) -> Result<Json<Config>, Status> {
    let ctx = ctx.get().await;
    Ok(Json(ctx.config))
}

#[put("/api/config", format = "json", data = "<new_config>")]
pub async fn put_config(
    _token: Token,
    ctx: &State<Provider<Context>>,
    new_config: Json<Config>,
) -> Result<(), Status> {
    let ctx_read = ctx.get().await;

    let new_ctx = Context {
        config: new_config.into_inner(),
        ..ctx_read
    };

    ctx.set(new_ctx).await;
    Ok(())
}
