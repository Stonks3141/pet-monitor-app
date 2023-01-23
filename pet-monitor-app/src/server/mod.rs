use crate::config::{Context, ContextManager};
use axum::routing::{get, post, put};
use axum_macros::FromRef;
use mp4_stream::{
    capabilities::{check_config, get_capabilities_all, Capabilities},
    stream_media_segments, StreamSubscriber,
};
use std::{net::SocketAddr, path::PathBuf};

mod auth;
mod routes;

#[derive(Debug, Clone, FromRef)]
struct AppState {
    ctx: ContextManager,
    caps: Capabilities,
    stream_sub_tx: Option<flume::Sender<StreamSubscriber>>,
}

pub async fn start(conf_path: Option<PathBuf>, ctx: Context, stream: bool) -> anyhow::Result<()> {
    let (ctx_manager, cfg_rx) = ContextManager::new(ctx.clone(), conf_path.clone());

    let caps = get_capabilities_all()?;
    check_config(&ctx.config, &caps)?;

    let mut state = AppState {
        ctx: ctx_manager,
        caps,
        stream_sub_tx: None,
    };

    let mut app = axum::Router::new()
        .route("/api/login", post(routes::login))
        .route("/api/config", get(routes::get_config))
        .route("/api/config", put(routes::put_config))
        .route("/api/capabilities", get(routes::capabilities));

    if stream {
        let (tx, rx) = flume::unbounded();
        let config = ctx.config.clone();
        tokio::task::spawn_blocking(move || {
            // not much we can do about an error at this point, the server is already started
            #[allow(clippy::unwrap_used)]
            stream_media_segments(rx, config, Some(cfg_rx)).unwrap();
        });
        app = app.route("/stream.mp4", get(routes::stream));
        state.stream_sub_tx = Some(tx);
    }

    #[cfg(not(debug_assertions))]
    let app = app.fallback(routes::files);

    let app = app.with_state(state);

    axum::Server::bind(&SocketAddr::new(ctx.host, ctx.port))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
