#[cfg(debug_assertions)]
use super::AppState;
use crate::auth::Token;
use crate::config::{Context, ContextManager};
use axum::body::{Bytes, Full};
use axum::{
    body::StreamBody,
    extract::State,
    http::{header, Response, StatusCode},
    response::Redirect,
    Form,
};
use axum_macros::debug_handler;
use futures_lite::{Stream, StreamExt};
use mp4_stream::{
    capabilities::{check_config, Capabilities},
    config::Config,
    StreamSubscriber,
};
use serde::Deserialize;
use tokio::task::spawn_blocking;
use tower_cookies::{Cookie, Cookies};
use tracing::instrument;

macro_rules! error {
    ($($args:tt)*) => {{
        tracing::error!($($args)*);
        StatusCode::INTERNAL_SERVER_ERROR
    }};
}

#[debug_handler]
#[instrument(skip(ctx))]
pub(crate) async fn redirect(uri: hyper::Uri, State(ctx): State<ContextManager>) -> Redirect {
    #[allow(clippy::unwrap_used)]
    Redirect::permanent(&format!(
        "https://{}{}",
        ctx.get().domain,
        uri.path_and_query().unwrap().as_str()
    ))
}

#[debug_handler(state = AppState)]
#[instrument(skip_all)]
pub(crate) async fn base(token: Option<Token>) -> Redirect {
    if token.is_some() {
        tracing::debug!("Redirecting to '/stream.html'");
        Redirect::to("/stream.html")
    } else {
        tracing::debug!("Redirecting to '/login.html'");
        Redirect::to("/login.html")
    }
}

async fn get_file(path: &str) -> Option<Bytes> {
    const FILES: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/static");

    if std::env::var("STATIC_RELOAD").map_or(false, |it| it == "1") {
        tokio::fs::read(format!("{}/static/{path}", env!("CARGO_MANIFEST_DIR")))
            .await
            .ok()
            .map(Bytes::from)
    } else {
        FILES
            .get_file(path)
            .map(|it| Bytes::from_static(it.contents()))
    }
}

#[debug_handler]
#[instrument]
pub async fn files(uri: axum::http::Uri) -> Response<Full<Bytes>> {
    let path = uri.path().trim_start_matches('/');
    #[allow(clippy::unwrap_used)]
    let (body, status) = match get_file(path).await {
        Some(it) => (it, StatusCode::OK),
        None => (get_file("404.html").await.unwrap(), StatusCode::NOT_FOUND),
    };

    let mut res = Response::builder().status(status);
    if let Some(content_type) = mime_guess::from_path(path).first_raw() {
        res = res.header(header::CONTENT_TYPE, content_type)
    }
    #[allow(clippy::unwrap_used)]
    res.body(Full::new(body)).unwrap()
}

#[derive(Deserialize)]
pub(crate) struct Login {
    password: String,
}

#[debug_handler]
#[instrument(skip_all)]
pub(crate) async fn login(
    State(ctx): State<ContextManager>,
    cookies: Cookies,
    Form(Login { password }): Form<Login>,
) -> Result<Redirect, StatusCode> {
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

        Ok(Redirect::to("/stream.html"))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[debug_handler(state = AppState)]
#[instrument(skip_all)]
pub(crate) async fn config(
    _token: Token,
    State(ctx): State<ContextManager>,
    State(caps): State<Capabilities>,
) -> Result<Response<String>, StatusCode> {
    let config = serde_json::to_string(&ctx.get().config)
        .map_err(|e| error!("{e}"))?
        .replace('"', "'");
    let caps = serde_json::to_string(&caps)
        .map_err(|e| error!("{e}"))?
        .replace('"', "'");

    #[allow(clippy::unwrap_used)]
    let html = std::str::from_utf8(&get_file("config.html").await.unwrap())
        .map_err(|_| error!("config.html contains invalid UTF-8"))?
        .replacen("{{config}}", &config, 1)
        .replacen("{{caps}}", &caps, 1);

    #[allow(clippy::unwrap_used)]
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        .body(html)
        .unwrap())
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigForm {
    csrf: String,
    device: std::path::PathBuf,
    format: mp4_stream::config::Format,
    resolution: (u32, u32),
    interval: (u32, u32),
    rotation: mp4_stream::config::Rotation,
    v4l2_controls: Option<std::collections::HashMap<String, String>>,
}

#[debug_handler(state = AppState)]
#[instrument(skip(_token, ctx, caps))]
pub(crate) async fn set_config(
    _token: Token,
    State(ctx): State<ContextManager>,
    State(caps): State<Capabilities>,
    form: String,
) -> Result<Redirect, StatusCode> {
    let form = percent_encoding::percent_decode_str(&form)
        .decode_utf8()
        .map_err(|e| error!("Percent decoding error: {e}"))?;
    let form: ConfigForm = serde_qs::from_str(&form).map_err(|e| error!("{e}"))?;
    let config = Config {
        device: form.device,
        format: form.format,
        resolution: form.resolution,
        interval: form.interval,
        rotation: form.rotation,
        v4l2_controls: form.v4l2_controls.unwrap_or_default(),
    };
    let ctx_read = ctx.get();

    if !Token::decode(&form.csrf, &ctx_read.jwt_secret)
        .map_err(|e| error!("{e}"))?
        .verify()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let config_clone = config.clone();
    if let Err(e) = tokio::task::spawn_blocking(move || check_config(&config_clone, &caps)).await {
        tracing::warn!("Config validation error: {e}");
        return Err(StatusCode::BAD_REQUEST);
    }

    let new_ctx = Context { config, ..ctx_read };
    ctx.set(new_ctx)
        .await
        .map_err(|e| error!("Error writing to config file: {e}"))?;

    Ok(Redirect::to("/stream.html"))
}

#[debug_handler(state = AppState)]
#[instrument(skip_all)]
pub(crate) async fn stream(
    _token: Token,
    State(ctx): State<ContextManager>,
    State(stream_sub_tx): State<Option<flume::Sender<StreamSubscriber>>>,
) -> Result<Response<StreamBody<impl Stream<Item = std::io::Result<Vec<u8>>>>>, StatusCode> {
    #[allow(clippy::unwrap_used)] // stream_sub_tx will always be `Some` if this route is mounted
    let stream = mp4_stream::stream(&ctx.get().config, stream_sub_tx.unwrap())
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
