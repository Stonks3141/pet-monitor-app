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

#[debug_handler]
pub async fn redirect(uri: hyper::Uri, State(ctx): State<ContextManager>) -> Redirect {
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

    let body = Full::new(Bytes::from_static(
        STATIC_FILES.get_file(path).unwrap().contents(),
    ));

    let mut res = Response::builder();
    if let Some(content_type) = mime_guess::from_path(path).first_raw() {
        res = res.header(header::CONTENT_TYPE, content_type)
    }
    Ok(res.body(body).unwrap())
}

/// Validates a password and issues tokens.
#[debug_handler]
pub async fn login(
    State(ctx): State<ContextManager>,
    password: String,
) -> Result<Response<String>, StatusCode> {
    let ctx = ctx.get();

    let correct =
        spawn_blocking(move || argon2::verify_encoded(&ctx.password_hash, password.as_bytes()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map_err(|e| {
                tracing::error!("Validating login attempt failed: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    if correct {
        let token = Token::new(ctx.jwt_timeout)
            .encode(&ctx.jwt_secret)
            .map_err(|e| {
                tracing::error!("Token creation failed: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let max_age = ctx.jwt_timeout.as_secs();

        #[allow(clippy::unwrap_used)]
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(
                header::SET_COOKIE,
                format!("token={token}; Path=/; SameSite=Strict; Max-Age={max_age}; Secure"),
            )
            .body(String::new())
            .unwrap())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Retrieves the current configuration. The request must have a valid JWT.
#[debug_handler(state = AppState)]
pub async fn get_config(_token: Token, State(ctx): State<ContextManager>) -> Json<Config> {
    let ctx = ctx.get();
    Json(ctx.config)
}

/// Updates the current configuration. The request must have a valid JWT.
#[debug_handler(state = AppState)]
pub async fn put_config(
    _token: Token,
    State(ctx): State<ContextManager>,
    State(caps): State<Capabilities>,
    Json(config): Json<Config>,
) -> Result<(), StatusCode> {
    let ctx_read = ctx.get();

    let config_clone = config.clone();
    if let Err(e) = tokio::task::spawn_blocking(move || check_config(&config_clone, &caps)).await {
        tracing::warn!("Config validation failed with error {:?}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    let new_ctx = Context { config, ..ctx_read };

    ctx.set(new_ctx).await.map_err(|e| {
        tracing::error!("Writing to configuration file failed with error {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(())
}

#[debug_handler(state = AppState)]
pub async fn stream(
    _token: Token,
    State(ctx): State<ContextManager>,
    State(stream_sub_tx): State<Option<flume::Sender<StreamSubscriber>>>,
) -> Result<Response<StreamBody<impl Stream<Item = std::io::Result<Vec<u8>>>>>, StatusCode> {
    #[allow(clippy::unwrap_used)] // stream_sub_tx will always be `Some` if this route is mounted
    let stream = VideoStream::new(&ctx.get().config, stream_sub_tx.unwrap())
        .await
        .map_err(|e| {
            tracing::error!("Error constructing VideoStream: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let stream = StreamExt::inspect(stream, |it| match it {
        Err(e) => tracing::warn!("Error streaming segment: {:?}", e),
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
pub async fn capabilities(_token: Token, State(caps): State<Capabilities>) -> Json<Capabilities> {
    Json(caps)
}
