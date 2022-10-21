//! This module provides Rocket routes for the server.

use super::auth::Token;
use super::provider::Provider;
use crate::config::{Config, Context};
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};
use log::warn;
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
pub async fn redirect(path: PathBuf, ctx: &State<Provider<Context>>) -> Result<Redirect, Status> {
    let path = path.to_str().ok_or_else(|| {
        warn!("Failed to convert path {:?} to string", path);
        Status::InternalServerError
    })?;
    let ctx = ctx.get();

    Ok(Redirect::permanent(format!(
        "https://{}/{}",
        ctx.domain, path
    )))
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
            f.contents_utf8().map_err(|e| {
                warn!(
                    "Converting included file {:?} to UTF-8 failed with error {:?}",
                    path, e
                );
                Err(Status::InternalServerError)
            })?
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
                    .ok_or_else(|e| {
                        warn!(
                            "Getting index.html from included bundle failed with error {:?}",
                            e
                        );
                        Status::InternalServerError
                    })?
                    .contents_utf8()
                    .ok_or_else(|e| {
                        warn!("Converting index.html to UTF-8 failed with error {:?}", e);
                        Status::InternalServerError
                    })?
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
    let ctx = ctx.get();

    match crate::secrets::validate(&password, &ctx.password_hash).await {
        Ok(b) => {
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
                    Err(e) => {
                        warn!("Stringifying token failed with error {:?}", e);
                        Status::InternalServerError
                    }
                }
            } else {
                Status::Unauthorized
            }
        }
        Err(e) => {
            warn!("Validating login attempt failed with error {:?}", e);
            Status::InternalServerError
        }
    }
}

/// Retrieves the current configuration. The request must have a valid JWT.
#[get("/api/config")]
pub async fn get_config(
    _token: Token,
    ctx: &State<Provider<Context>>,
) -> Result<Json<Config>, Status> {
    let ctx = ctx.get();
    Ok(Json(ctx.config))
}

/// Updates the current configuration. The request must have a valid JWT.
#[put("/api/config", format = "json", data = "<new_config>")]
pub async fn put_config(
    _token: Token,
    ctx: &State<Provider<Context>>,
    new_config: Json<Config>,
) -> Result<(), Status> {
    let ctx_read = ctx.get();

    let new_ctx = Context {
        config: new_config.into_inner(),
        ..ctx_read
    };

    ctx.set(new_ctx);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};
    use ring::rand::SystemRandom;
    use rocket::http::uri::{Origin, Reference};
    use rocket::local::asynchronous::Client;
    use rocket::tokio;

    quickcheck! {
        fn qc_redirect(domain: String, path: Vec<String>) -> TestResult {
            use rocket::local::blocking::Client;

            let path = "/".to_string() + &path.join("/");

            if Reference::parse(&domain).is_err() || Origin::parse(&path).is_err() {
                return TestResult::discard();
            }

            let ctx = Context {
                domain: domain.clone(),
                ..Default::default()
            };

            let rocket = rocket::build()
                .mount("/", rocket::routes![redirect])
                .manage(Provider::new(ctx));

            let client = Client::tracked(rocket).unwrap();
            let res = client.get(path.clone()).dispatch();

            TestResult::from_bool(
                res.status() == Status::PermanentRedirect
                && res.headers().get_one("Location").unwrap() == format!("https://{}/{}/", domain, path)
            )
        }
    }

    #[tokio::test]
    async fn redirect() {
        let ctx = Context {
            domain: "localhost".to_string(),
            ..Default::default()
        };

        let rocket = rocket::build()
            .mount("/", rocket::routes![redirect])
            .manage(Provider::new(ctx));

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
            .manage(Provider::new(ctx));

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
            .manage(Provider::new(ctx));

        let client = Client::tracked(rocket).await.unwrap();

        let res = client.post("/api/login").body("bar").dispatch().await;
        assert_eq!(res.status(), Status::Unauthorized);
    }
}
