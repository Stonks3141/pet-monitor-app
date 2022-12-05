use anyhow::Context as _;
use chrono::Duration;
use mp4_stream::config::Config;
use parking_lot::RwLock;
use rocket::tokio::task::spawn_blocking;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A wrapper for `Context` that syncs it with the config file and provides
/// interior mutability.
#[derive(Debug, Clone)]
pub struct ContextManager {
    ctx: Arc<RwLock<Context>>,
    sender: flume::Sender<Config>,
    conf_path: Option<PathBuf>,
}

impl ContextManager {
    pub fn new(context: Context, conf_path: Option<PathBuf>) -> (Self, flume::Receiver<Config>) {
        let (sender, rx) = flume::unbounded();
        (
            Self {
                ctx: Arc::new(RwLock::new(context)),
                sender,
                conf_path,
            },
            rx,
        )
    }

    pub fn get(&self) -> Context {
        (*self.ctx.read()).clone()
    }

    pub async fn set(&self, context: Context) -> anyhow::Result<()> {
        *self.ctx.write() = context.clone();

        // Don't mess with the global config file if we don't have a specific path
        // for tests
        #[cfg(not(test))]
        store(&self.conf_path, &context).await?;
        #[cfg(test)]
        if self.conf_path.is_some() {
            store(&self.conf_path, &context).await?;
        }

        self.sender.send(context.config).unwrap_or(());
        Ok(())
    }
}

/// Application state and configuration
#[serde_with::serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Context {
    /// argon2 hash of the current password
    pub password_hash: String,
    /// The secret used for signing JWTs
    #[serde_as(as = "serde_with::base64::Base64")]
    pub jwt_secret: [u8; 32],
    /// The JWT timeout, serialized as seconds
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub jwt_timeout: Duration,
    /// The domain to serve from (used by HTTPS redirect route)
    pub domain: String,
    /// The IP address to listen on
    pub host: IpAddr,
    /// The port to listen on
    pub port: u16,
    /// Configuration accessed by the browser
    #[serde(flatten)]
    pub config: Config,
    /// TLS settings
    pub tls: Option<Tls>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            password_hash: String::new(),
            jwt_secret: [0; 32],
            jwt_timeout: Duration::days(4),
            domain: "localhost".to_string(),
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
            config: Config::default(),
            tls: None,
        }
    }
}

/// TLS config options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tls {
    /// The port to use for HTTPS
    pub port: u16,
    /// Path to the SSL certificate to use
    pub cert: PathBuf,
    /// Path to the SSL certificate key
    pub key: PathBuf,
}

impl Default for Tls {
    fn default() -> Self {
        Self {
            port: 8443,
            cert: PathBuf::from("path/to/cert.pem"),
            key: PathBuf::from("path/to/key.key"),
        }
    }
}

/// Writes out a [`Context`] to the config file.
pub async fn store<P: AsRef<Path>>(path: &Option<P>, ctx: &Context) -> anyhow::Result<()> {
    let ctx = ctx.clone();
    match path {
        Some(path) => {
            let path = path.as_ref().to_owned();
            spawn_blocking(move || {
                confy::store_path(path, ctx).context("Failed to store configuration file")
            })
            .await?
        }
        None => {
            spawn_blocking(move || {
                confy::store("pet-monitor-app", Some("config"), ctx)
                    .context("Failed to store configuration file")
            })
            .await?
        }
    }
}

/// Loads the config file.
pub async fn load<P: AsRef<Path>>(path: &Option<P>) -> anyhow::Result<Context> {
    use anyhow::Context;
    if let Some(path) = path {
        let path = path.as_ref().to_owned();
        spawn_blocking(move || confy::load_path(path).context("Failed to load configuration file"))
            .await?
    } else {
        spawn_blocking(move || {
            confy::load("pet-monitor-app", Some("config"))
                .context("Failed to load configuration file")
        })
        .await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::NamedTempFile;
    use rocket::tokio;

    #[tokio::test]
    async fn config_load_store() {
        let tmp = NamedTempFile::new("config.toml").unwrap();

        let ctx = Context::default();

        store(&Some(tmp.path()), &ctx).await.unwrap();
        assert!(tmp.exists());

        let ctx_file = load(&Some(tmp.path())).await.unwrap();
        assert_eq!(ctx, ctx_file);

        tmp.close().unwrap();
    }

    #[tokio::test]
    async fn test_context_manager() {
        let mut val = Context::default();
        let (ctx_manager, sub) = ContextManager::new(val.clone(), None);

        assert_eq!(val, ctx_manager.get());

        val.jwt_secret = [42; 32];
        ctx_manager.set(val.clone()).await.unwrap();

        assert_eq!(val, ctx_manager.get());
        assert_eq!(val.config, sub.recv_async().await.unwrap());

        val.domain = "ferris.crab".to_string();
        ctx_manager.set(val.clone()).await.unwrap();

        assert_eq!(val, ctx_manager.get());
        assert_eq!(val.config, sub.recv_async().await.unwrap());
    }
}

#[cfg(test)]
mod qc {
    use super::*;
    use chrono::Duration;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Context {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                password_hash: Arbitrary::arbitrary(g),
                jwt_secret: [0; 32].map(|_| Arbitrary::arbitrary(g)),
                jwt_timeout: Duration::milliseconds(Arbitrary::arbitrary(g)),
                domain: Arbitrary::arbitrary(g),
                host: Arbitrary::arbitrary(g),
                port: Arbitrary::arbitrary(g),
                config: Arbitrary::arbitrary(g),
                tls: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for Tls {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                port: Arbitrary::arbitrary(g),
                cert: Arbitrary::arbitrary(g),
                key: Arbitrary::arbitrary(g),
            }
        }
    }
}
