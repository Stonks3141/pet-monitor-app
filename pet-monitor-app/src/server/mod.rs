use crate::config::{Context, ContextManager};
use anyhow::anyhow;
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
use std::{fs::File, io::BufReader, net::SocketAddr, path::PathBuf, pin::Pin, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};
use tower::MakeService;

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
        ctx: ctx_manager.clone(),
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

    if ctx.tls.is_some() {
        let http_app = axum::Router::new()
            .fallback(routes::redirect)
            .with_state(ctx_manager);
        let http_server = axum::Server::bind(&SocketAddr::new(ctx.host, ctx.port))
            .serve(http_app.into_make_service());

        let https_server = start_https(ctx, app);

        let (r1, r2) = tokio::join!(http_server, https_server);
        r1?;
        r2?;
    } else {
        axum::Server::bind(&SocketAddr::new(ctx.host, ctx.port))
            .serve(app.into_make_service())
            .await?;
    }

    Ok(())
}

async fn start_https(ctx: Context, app: axum::Router) -> anyhow::Result<()> {
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
        let stream = std::future::poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .ok_or_else(|| anyhow!("hi"))??;

        let acceptor = acceptor.clone();
        let protocol = protocol.clone();
        let service = app.make_service(&stream);

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                let _ = protocol
                    .serve_connection(stream, service.await.unwrap())
                    .await;
            }
        });
    }
}
