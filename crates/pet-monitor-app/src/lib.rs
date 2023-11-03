#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

pub mod auth;
pub mod config;
mod handlers;

use crate::config::{Context, ContextManager};
use axum::{
    error_handling::HandleErrorLayer,
    extract::FromRef,
    middleware,
    routing::{get, post},
};
use mp4_stream::{
    capabilities::{check_config, get_capabilities_all, Capabilities},
    stream_media_segments, StreamSubscriber,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::task::spawn_blocking;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;

#[derive(Debug, Clone)]
struct AppState {
    ctx: ContextManager,
    caps: Capabilities,
    stream_sub_tx: Option<flume::Sender<StreamSubscriber>>,
}

impl FromRef<AppState> for ContextManager {
    fn from_ref(state: &AppState) -> Self {
        state.ctx.clone()
    }
}

impl FromRef<AppState> for Capabilities {
    fn from_ref(state: &AppState) -> Self {
        state.caps.clone()
    }
}

impl FromRef<AppState> for Option<flume::Sender<StreamSubscriber>> {
    fn from_ref(state: &AppState) -> Self {
        state.stream_sub_tx.clone()
    }
}

pub async fn start(conf_path: Option<PathBuf>, ctx: Context, stream: bool) -> anyhow::Result<()> {
    let (ctx_manager, cfg_rx) = ContextManager::new(ctx.clone(), conf_path.clone());

    let caps = get_capabilities_all()?;
    if !std::env::var("DISABLE_VALIDATE_CONFIG").map_or(false, |it| it == "1") {
        check_config(&ctx.config, &caps)?;
    }

    let mut state = AppState {
        ctx: ctx_manager.clone(),
        caps,
        stream_sub_tx: None,
    };

    let mut app = axum::Router::new()
        .route("/", get(handlers::base))
        .route("/style.css", get(handlers::files))
        .route("/login.html", get(handlers::files))
        .route(
            "/login.html",
            post(handlers::login).layer(ServiceBuilder::new().layer(HandleErrorLayer::new(
                |_| async move { hyper::StatusCode::SERVICE_UNAVAILABLE },
            ))),
        )
        .route("/stream.html", get(handlers::files))
        .route("/config.html", get(handlers::config))
        .route("/config.html", post(handlers::set_config))
        .layer(CookieManagerLayer::new())
        .layer(middleware::from_fn(auth::auth_error_layer));

    if stream {
        let (tx, rx) = flume::unbounded();
        let config = ctx.config.clone();
        spawn_blocking(move || {
            if let Err(e) = stream_media_segments(rx, config, Some(cfg_rx)) {
                log::error!("Streaming error: {e}");
            }
        });
        log::info!("Stream started");
        app = app.route("/stream.mp4", get(handlers::stream));
        state.stream_sub_tx = Some(tx);
    }

    let app = app.with_state(state);

    let addr = SocketAddr::new(ctx.host, ctx.port);

    log::info!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
