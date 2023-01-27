#[cfg(debug_assertions)]
use super::AppState;
use crate::auth::Token;
use crate::config::{Context, ContextManager};
#[cfg(not(debug_assertions))]
use axum::body::{Bytes, Full};
use axum::{
    body::StreamBody,
    extract::State,
    http::{header, Response, StatusCode},
    response::Redirect,
    Json,
};
use axum_macros::debug_handler;
use futures_lite::{Stream, StreamExt};
use mp4_stream::{
    capabilities::{check_config, Capabilities},
    config::Config,
    StreamSubscriber, VideoStream,
};
use tokio::task::spawn_blocking;
use tower_cookies::{Cookie, Cookies};

macro_rules! error {
    ($($args:tt)*) => {{
        tracing::error!($($args)*);
        StatusCode::INTERNAL_SERVER_ERROR
    }};
}

#[debug_handler]
pub(crate) async fn redirect(uri: hyper::Uri, State(ctx): State<ContextManager>) -> Redirect {
    #[allow(clippy::unwrap_used)]
    Redirect::permanent(&format!(
        "https://{}{}",
        ctx.get().domain,
        uri.path_and_query().unwrap().as_str()
    ))
}

#[cfg(not(debug_assertions))]
#[debug_handler]
pub async fn files(uri: axum::http::Uri) -> Result<Response<Full<Bytes>>, StatusCode> {
    const STATIC_FILES: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/build");

    let mut path = uri.path().trim_start_matches('/');
    if !STATIC_FILES.contains(path) {
        path = "index.html";
    }

    #[allow(clippy::unwrap_used)] // index.html is guaranteed
    let body = Full::new(Bytes::from_static(
        STATIC_FILES.get_file(path).unwrap().contents(),
    ));

    let mut res = Response::builder();
    if let Some(content_type) = mime_guess::from_path(path).first_raw() {
        res = res.header(header::CONTENT_TYPE, content_type)
    }
    #[allow(clippy::unwrap_used)]
    Ok(res.body(body).unwrap())
}

/// Validates a password and issues tokens.
#[debug_handler]
pub(crate) async fn login(
    State(ctx): State<ContextManager>,
    cookies: Cookies,
    password: String,
) -> Result<(), StatusCode> {
    let ctx = ctx.get();

    let correct =
        spawn_blocking(move || argon2::verify_encoded(&ctx.password_hash, password.as_bytes()))
            .await
            .map_err(|e| error!("{e}"))?
            .map_err(|e| error!("Validating login attempt failed: {e}"))?;

    if correct {
        let token = Token::new(ctx.jwt_timeout)
            .encode(&ctx.jwt_secret)
            .map_err(|e| error!("Token creation failed: {e}"))?;

        #[allow(clippy::unwrap_used)] // conversion of u64 to i64
        let cookie = Cookie::build("token", token)
            .path("/")
            .max_age(ctx.jwt_timeout.try_into().unwrap())
            .same_site(cookie::SameSite::Strict)
            .finish();

        cookies.add(cookie);

        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Retrieves the current configuration. The request must have a valid JWT.
#[debug_handler(state = AppState)]
pub(crate) async fn get_config(_token: Token, State(ctx): State<ContextManager>) -> Json<Config> {
    let ctx = ctx.get();
    Json(ctx.config)
}

/// Updates the current configuration. The request must have a valid JWT.
#[debug_handler(state = AppState)]
pub(crate) async fn put_config(
    _token: Token,
    State(ctx): State<ContextManager>,
    State(caps): State<Capabilities>,
    Json(config): Json<Config>,
) -> Result<(), StatusCode> {
    let ctx_read = ctx.get();

    let config_clone = config.clone();
    if let Err(e) = tokio::task::spawn_blocking(move || check_config(&config_clone, &caps)).await {
        tracing::warn!("Config validation error: {e}");
        return Err(StatusCode::BAD_REQUEST);
    }

    let new_ctx = Context { config, ..ctx_read };

    ctx.set(new_ctx)
        .await
        .map_err(|e| error!("Error writing to config file: {e}"))?;
    Ok(())
}

#[debug_handler(state = AppState)]
pub(crate) async fn stream(
    _token: Token,
    State(ctx): State<ContextManager>,
    State(stream_sub_tx): State<Option<flume::Sender<StreamSubscriber>>>,
) -> Result<Response<StreamBody<impl Stream<Item = std::io::Result<Vec<u8>>>>>, StatusCode> {
    #[allow(clippy::unwrap_used)] // stream_sub_tx will always be `Some` if this route is mounted
    let stream = VideoStream::new(&ctx.get().config, stream_sub_tx.unwrap())
        .await
        .map_err(|e| error!("Error starting stream: {e}"))?;

    let stream = StreamExt::inspect(stream, |it| match it {
        Err(e) => tracing::warn!("Error streaming segment: {e}"),
        Ok(_) => (),
    });

    #[allow(clippy::unwrap_used)]
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CACHE_CONTROL, "max-age=0, s-maxage=0, no-store")
        .body(StreamBody::new(stream))
        .unwrap())
}

#[debug_handler(state = AppState)]
pub(crate) async fn capabilities(
    _token: Token,
    State(caps): State<Capabilities>,
) -> Json<Capabilities> {
    Json(caps)
}
