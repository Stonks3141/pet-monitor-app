use super::auth::Token;
#[cfg(debug_assertions)]
use super::AppState;
use crate::config::{Context, ContextManager};
use axum::{
    body::StreamBody,
    extract::{Path, State},
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
pub async fn files(uri: axum::http::Uri) -> Result<Response<String>, StatusCode> {
    use include_dir::{include_dir, Dir};
    const STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/build");

    let mut path = uri.path().trim_start_matches('/');
    if !STATIC_FILES.contains(path) {
        path = "index.html";
    }

    let body = STATIC_FILES
        .get_file(path)
        .map(|f| {
            f.contents_utf8().ok_or_else(|| {
                log::error!("Failed to convert included file {path:?} to UTF-8");
                StatusCode::INTERNAL_SERVER_ERROR
            })
        })
        .unwrap()?
        .to_string();

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

    match crate::secrets::validate(&password, &ctx.password_hash).await {
        Ok(true) => match Token::new(ctx.jwt_timeout).encode(&ctx.jwt_secret) {
            Ok(token) => {
                let max_age = ctx.jwt_timeout.num_seconds();
                #[allow(clippy::unwrap_used)]
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        header::SET_COOKIE,
                        format!(
                            "token={token}; Path=/; SameSite=Strict; Max-Age={max_age}; Secure"
                        ),
                    )
                    .body("".to_string())
                    .unwrap())
            }
            Err(e) => {
                log::warn!("Stringifying token failed: {e:?}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Ok(false) => Err(StatusCode::UNAUTHORIZED),
        Err(e) => {
            log::warn!("Validating login attempt failed: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
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
        log::warn!("Config validation failed with error {:?}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    let new_ctx = Context { config, ..ctx_read };

    ctx.set(new_ctx).await.map_err(|e| {
        log::error!("Writing to configuration file failed with error {:?}", e);
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
            log::error!("Error constructing VideoStream: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let stream = StreamExt::inspect(stream, |x| {
        if let Err(e) = x {
            log::warn!("Error streaming segment: {:?}", e);
        }
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
