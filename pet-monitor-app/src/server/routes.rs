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

/// Redirects any request to HTTPS. It preserves the original path and uses
/// Context.domain to construct the new URL.
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

/// Static HTML/CSS/JS frontend files
#[cfg(not(debug_assertions))]
const STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/dist");

/// A file server route that uses the static files compiled into the binary.
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
/// It accepts POSTs to the `/api/login` path with the password as plain text.
/// If the password is correct, it adds a `token` cookie containing a JWT.
#[post("/api/login", data = "<password>")]
pub async fn login(
    password: String,
    cookies: &CookieJar<'_>,
    ctx: &State<Provider<Context>>,
) -> Status {
    let ctx = ctx.get().await;

    if let Ok(b) = auth::validate(&password, &ctx.password_hash).await {
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

/// Retrieves the current configuration. The request must have a valid JWT.
#[get("/api/config")]
pub async fn get_config(
    _token: Token,
    ctx: &State<Provider<Context>>,
) -> Result<Json<Config>, Status> {
    let ctx = ctx.get().await;
    Ok(Json(ctx.config))
}

/// Updates the current configuration. The request must have a valid JWT.
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

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;
    use rocket::local::asynchronous::Client;
    use rocket::tokio;

    #[tokio::test]
    async fn redirect() {
        let ctx = Context {
            domain: "localhost".to_string(),
            ..Default::default()
        };
        let rocket = rocket::build()
            .mount("/", rocket::routes![redirect])
            .manage(Provider::new(ctx, |_| async {}));

        let client = Client::tracked(rocket).await.unwrap();

        let res = client.get("/").dispatch().await;
        assert_eq!(res.status(), Status::PermanentRedirect);
        assert_eq!(
            res.headers().get_one("Location").unwrap(),
            "https://localhost/"
        );

        let res = client.get("/index.html").dispatch().await;
        assert_eq!(res.status(), Status::PermanentRedirect);
        assert_eq!(
            res.headers().get_one("Location").unwrap(),
            "https://localhost/index.html"
        );
    }

    #[tokio::test]
    async fn login_valid() {
        let password = "foo";
        let rng = SystemRandom::new();
        let ctx = Context {
            password_hash: crate::secrets::init_password(&rng, password).await.unwrap(),
            ..Default::default()
        };
        let rocket = rocket::build()
            .mount("/", rocket::routes![login])
            .manage(Provider::new(ctx, |_| async {}));

        let client = Client::tracked(rocket).await.unwrap();

        let res = client.post("/api/login").body(password).dispatch().await;
        assert_eq!(res.status(), Status::Ok);
    }

    #[tokio::test]
    async fn login_invalid() {
        let password = "foo";
        let rng = SystemRandom::new();
        let ctx = Context {
            password_hash: crate::secrets::init_password(&rng, password).await.unwrap(),
            ..Default::default()
        };
        let rocket = rocket::build()
            .mount("/", rocket::routes![login])
            .manage(Provider::new(ctx, |_| async {}));

        let client = Client::tracked(rocket).await.unwrap();

        let res = client.post("/api/login").body("bar").dispatch().await;
        assert_eq!(res.status(), Status::Unauthorized);
    }
}
