//! This module provides Rocket routes for the server.

use super::auth::Token;
use crate::config::{Context, ContextManager};
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};
use log::{error, warn};
use mp4_stream::{
    capabilities::{check_config, Capabilities},
    config::Config,
    StreamSubscriber, VideoStream,
};
use rocket::{
    futures::{Stream, StreamExt},
    http::{ContentType, Cookie, CookieJar, Header, SameSite, Status},
    response::{stream::ByteStream, Redirect},
    serde::json::Json,
    tokio::task::spawn_blocking,
    {get, post, put, Responder, State},
};
use std::path::PathBuf;

/// Redirects any request to HTTPS. It preserves the original path and uses
/// Context.domain to construct the new URL.
#[get("/<path..>")]
pub fn redirect(path: PathBuf, ctx: &State<ContextManager>) -> Result<Redirect, Status> {
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
const STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/build");

/// A file server route that uses the static files compiled into the binary.
#[cfg(not(debug_assertions))]
#[get("/<path..>", rank = 2)]
pub fn files(path: PathBuf) -> Result<(ContentType, String), Status> {
    Ok(
        if let Some(s) = STATIC_FILES.get_file(&path).map(|f| {
            f.contents_utf8().ok_or_else(|| {
                error!("Failed to convert included file {:?} to UTF-8", path);
                Status::InternalServerError
            })
        }) {
            (
                match path.extension() {
                    Some(ext) => ContentType::from_extension(&ext.to_string_lossy())
                        .unwrap_or(ContentType::Plain),
                    None => ContentType::Plain,
                },
                s?.to_string(),
            )
        } else {
            (
                ContentType::HTML,
                STATIC_FILES
                    .get_file("index.html")
                    .ok_or_else(|| {
                        error!("Failed to get index.html from included bundle");
                        Status::InternalServerError
                    })?
                    .contents_utf8()
                    .ok_or_else(|| {
                        error!("Failed to convert index.html to UTF-8");
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
    ctx: &State<ContextManager>,
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
                        warn!("Stringifying token failed with error '{:?}'", e);
                        Status::InternalServerError
                    }
                }
            } else {
                Status::Unauthorized
            }
        }
        Err(e) => {
            warn!("Validating login attempt failed with error '{:?}'", e);
            Status::InternalServerError
        }
    }
}

/// Retrieves the current configuration. The request must have a valid JWT.
#[get("/api/config")]
pub fn get_config(_token: Token, ctx: &State<ContextManager>) -> Json<Config> {
    let ctx = ctx.get();
    Json(ctx.config)
}

/// Updates the current configuration. The request must have a valid JWT.
#[put("/api/config", format = "json", data = "<new_config>")]
pub async fn put_config(
    _token: Token,
    ctx: &State<ContextManager>,
    caps: &State<Capabilities>,
    new_config: Json<Config>,
) -> Result<(), Status> {
    let ctx_read = ctx.get();

    let config = new_config.into_inner();
    let config_clone = config.clone();
    let caps = caps.inner().clone();

    if let Err(e) = spawn_blocking(move || check_config(&config_clone, &caps)).await {
        warn!("Config validation failed with error {:?}", e);
        return Err(Status::BadRequest);
    }

    let new_ctx = Context { config, ..ctx_read };

    ctx.set(new_ctx).await.map_err(|e| {
        error!("Writing to configuration file failed with error {:?}", e);
        Status::InternalServerError
    })?;
    Ok(())
}

#[derive(Debug, Responder)]
pub struct StreamResponse<S: Stream<Item = Vec<u8>>> {
    stream: ByteStream<S>,
    content_type: ContentType,
    cache_control: CacheControl,
}

#[derive(Debug)]
struct CacheControl {
    max_age: Option<u32>,
    s_maxage: Option<u32>,
    no_store: bool,
}

impl From<CacheControl> for Header<'_> {
    fn from(cache_control: CacheControl) -> Self {
        let mut header = String::new();
        if let Some(max_age) = cache_control.max_age {
            header.push_str("max-age=");
            header.push_str(&max_age.to_string());
        }
        if let Some(s_maxage) = cache_control.s_maxage {
            if cache_control.max_age.is_some() {
                header.push_str(", ");
            }
            header.push_str("s-maxage=");
            header.push_str(&s_maxage.to_string());
        }
        if cache_control.no_store {
            if cache_control.s_maxage.is_some() {
                header.push_str(", ");
            }
            header.push_str("no-store");
        }
        Header::new("cache-control", header)
    }
}

#[get("/stream.mp4")]
pub async fn stream(
    _token: Token,
    ctx: &State<ContextManager>,
    stream_sub_tx: &State<flume::Sender<StreamSubscriber>>,
) -> Result<StreamResponse<impl Stream<Item = Vec<u8>>>, Status> {
    let ctx = ctx.get();
    let stream = VideoStream::new(&ctx.config, (**stream_sub_tx).clone())
        .await
        .map_err(|e| {
            warn!("Error constructing VideoStream: {:?}", e);
            Status::InternalServerError
        })?;
    Ok(StreamResponse {
        stream: ByteStream(StreamExt::filter_map(stream, |x| async move {
            match x {
                Ok(x) => Some(x),
                Err(e) => {
                    warn!("Error streaming segment: {:?}", e);
                    None
                }
            }
        })),
        content_type: ContentType::MP4,
        cache_control: CacheControl {
            max_age: Some(0),
            s_maxage: Some(0),
            no_store: true,
        },
    })
}

#[get("/api/capabilities")]
pub fn capabilities(_token: Token, caps: &State<Capabilities>) -> Json<Capabilities> {
    Json((*caps).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use once_cell::sync::Lazy;
    use parking_lot::Mutex;
    use quickcheck::{quickcheck, TestResult};
    use ring::rand::SystemRandom;
    use rocket::local::{asynchronous::Client as AsyncClient, blocking::Client as BlockingClient};
    use rocket::tokio;

    quickcheck! {
        fn qc_redirect(domain: String, path: Vec<String>) -> TestResult {
            let mut domain = domain;
            let mut path = path;

            // remove non-alphanumeric chars
            domain.retain(|c| c.is_ascii_alphanumeric());
            path.iter_mut().for_each(|seg| seg.retain(|c| c.is_ascii_alphanumeric()));

            // limit the size for performance reasons
            if let Some((i, _)) = domain.char_indices().nth(100) {
                domain.truncate(i);
            }
            path.truncate(4);
            path.iter_mut().for_each(|seg| {
                if let Some((i, _)) = seg.char_indices().nth(100) {
                    seg.truncate(i);
                }
            });

            if domain.len() == 0 || path.iter().map(|s| s.len() == 0).any(|x| x) {
                return TestResult::discard();
            }

            let path = "/".to_string() + &path.join("/");

            static CLIENT: Lazy<Mutex<BlockingClient>> = Lazy::new(|| {
                let rocket = rocket::build()
                    .mount("/", rocket::routes![redirect])
                    .manage(ContextManager::new(Context::default(), None).0);
                Mutex::new(BlockingClient::tracked(rocket).unwrap())
            });

            let ctx = Context {
                domain: domain.clone(),
                ..Default::default()
            };

            let lock = (*CLIENT).lock();
            block_on(lock.rocket().state::<ContextManager>().unwrap().set(ctx)).unwrap();
            let res = lock.get(path.clone()).dispatch();

            TestResult::from_bool(
                res.status() == Status::PermanentRedirect
                && res.headers().get_one("Location").unwrap() == format!("https://{}{}", domain, path)
            )
        }
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
            .manage(ContextManager::new(ctx, None).0);

        let client = AsyncClient::tracked(rocket).await.unwrap();

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
            .manage(ContextManager::new(ctx, None).0);

        let client = AsyncClient::tracked(rocket).await.unwrap();

        let res = client.post("/api/login").body("bar").dispatch().await;
        assert_eq!(res.status(), Status::Unauthorized);
    }
}
