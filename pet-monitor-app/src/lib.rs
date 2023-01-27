#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]

mod auth;
pub mod config;
mod handlers;

use crate::config::{Context, ContextManager};
use axum::routing::{get, post, put};
use axum_macros::FromRef;
use hyper::server::{
    accept::Accept,
    conn::{AddrIncoming, Http},
};
use mp4_stream::{
    capabilities::{check_config, get_capabilities_all, Capabilities},
    stream_media_segments, StreamSubscriber,
};
use std::{
    fs::File, future::poll_fn, io::BufReader, net::SocketAddr, path::PathBuf, pin::Pin, sync::Arc,
};
use tokio::{net::TcpListener, task::spawn_blocking};
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};
use tower::MakeService;
use tower_cookies::CookieManagerLayer;

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
        ctx: ctx_manager.clone(),
        caps,
        stream_sub_tx: None,
    };

    let mut app = axum::Router::new()
        .route("/api/login", post(handlers::login))
        .route("/api/config", get(handlers::get_config))
        .route("/api/config", put(handlers::put_config))
        .route("/api/capabilities", get(handlers::capabilities));

    if stream {
        let (tx, rx) = flume::unbounded();
        let config = ctx.config.clone();
        spawn_blocking(move || {
            if let Err(e) = stream_media_segments(rx, config, Some(cfg_rx)) {
                tracing::error!("{e}");
            }
        });
        app = app.route("/stream.mp4", get(handlers::stream));
        state.stream_sub_tx = Some(tx);
    }

    #[cfg(not(debug_assertions))]
    let app = app.fallback(handlers::files);

    let app = app.with_state(state).layer(CookieManagerLayer::new());

    let addr = SocketAddr::new(ctx.host, ctx.port);

    if ctx.tls.is_some() {
        let http_app = axum::Router::new()
            .fallback(handlers::redirect)
            .with_state(ctx_manager)
            .into_make_service();
        let http_server = axum::Server::bind(&addr).serve(http_app);

        let https_server = start_https(ctx, app);

        tracing::info!("Listening on {addr}");
        let (r1, r2) = tokio::join!(tokio::spawn(http_server), tokio::spawn(https_server));
        r1??;
        r2??;
    } else {
        tracing::info!("Listening on {addr}");
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
    }

    Ok(())
}

async fn start_https(ctx: Context, app: axum::Router) -> anyhow::Result<()> {
    #[allow(clippy::unwrap_used)]
    let tls = ctx.tls.unwrap();
    let acceptor = {
        let mut cert_reader = BufReader::new(File::open(tls.cert)?);
        let mut key_reader = BufReader::new(File::open(tls.key)?);

        let key = PrivateKey(rustls_pemfile::pkcs8_private_keys(&mut key_reader)?.remove(0));
        let certs = rustls_pemfile::certs(&mut cert_reader)?
            .into_iter()
            .map(Certificate)
            .collect();

        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        TlsAcceptor::from(Arc::new(config))
    };

    let protocol = Arc::new(Http::new());

    let listener = TcpListener::bind(SocketAddr::new(ctx.host, tls.port)).await?;
    let mut listener = AddrIncoming::from_listener(listener)?;

    let mut app = app.into_make_service();

    loop {
        let Some(Ok(stream)) = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx)).await else {
            continue;
        };

        let acceptor = acceptor.clone();
        let protocol = protocol.clone();
        let service = app.make_service(&stream);

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                #[allow(clippy::unwrap_used)] // Error type is `Infallible`
                let _ = protocol
                    .serve_connection(stream, service.await.unwrap())
                    .await;
            }
        });
    }
}
