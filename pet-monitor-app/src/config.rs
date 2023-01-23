use anyhow::Context as _;
use chrono::Duration;
use mp4_stream::config::Config;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    sync::Arc,
    thread::sleep,
};
use tokio::task::spawn_blocking;

/// A wrapper for `Context` that syncs it with the config file and provides
/// interior mutability for Rocket state.
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
            spawn_blocking(move || match confy::store_path(&path, ctx.clone()) {
                Ok(x) => Ok(x),
                Err(e) => {
                    log::warn!(
                        "Writing config failed with error: {:?}, retrying in 10 ms",
                        e
                    );
                    sleep(std::time::Duration::from_millis(10));
                    confy::store_path(path, ctx).context("Failed to store configuration file")
                }
            })
            .await?
        }
        None => {
            spawn_blocking(move || {
                match confy::store("pet-monitor-app", Some("config"), ctx.clone()) {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        log::warn!(
                            "Writing config failed with error: {:?}, retrying in 10 ms",
                            e
                        );
                        sleep(std::time::Duration::from_millis(10));
                        confy::store("pet-monitor-app", Some("config"), ctx)
                            .context("Failed to store configuration file")
                    }
                }
            })
            .await?
        }
    }
}

/// Loads the config file.
pub async fn load<P: AsRef<Path>>(path: &Option<P>) -> anyhow::Result<Context> {
    match path {
        Some(path) => {
            let path = path.as_ref().to_owned();
            spawn_blocking(move || match confy::load_path(&path) {
                Ok(x) => Ok(x),
                Err(e) => {
                    log::warn!(
                        "Writing config failed with error: {:?}, retrying in 10 ms",
                        e
                    );
                    sleep(std::time::Duration::from_millis(10));
                    confy::load_path(path).context("Failed to store configuration file")
                }
            })
            .await?
        }
        None => {
            spawn_blocking(
                move || match confy::load("pet-monitor-app", Some("config")) {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        log::warn!(
                            "Writing config failed with error: {:?}, retrying in 10 ms",
                            e
                        );
                        sleep(std::time::Duration::from_millis(10));
                        confy::load("pet-monitor-app", Some("config"))
                            .context("Failed to store configuration file")
                    }
                },
            )
            .await?
        }
    }
}
