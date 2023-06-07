use color_eyre::eyre::{self, WrapErr};
use mp4_stream::config::Config;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    sync::{Arc, RwLock},
    thread::sleep,
    time::Duration,
};
use tokio::task::spawn_blocking;

#[derive(Debug, Clone)]
pub(crate) struct ContextManager {
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
        (*self.ctx.read().unwrap()).clone()
    }

    pub async fn set(&self, ctx: Context) -> eyre::Result<()> {
        *self.ctx.write().unwrap() = ctx.clone();

        // Don't mess with the global config file if we don't have a specific path
        #[cfg(not(test))]
        store(self.conf_path.clone(), ctx.clone()).await?;
        #[cfg(test)]
        if self.conf_path.is_some() {
            store(self.conf_path.clone(), ctx.clone()).await?;
        }

        self.sender.send(ctx.config).unwrap_or(());
        Ok(())
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Context {
    pub password_hash: String,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub jwt_secret: [u8; 32],
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub jwt_timeout: Duration,
    pub host: IpAddr,
    pub port: u16,
    #[serde(flatten)]
    pub config: Config,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            password_hash: String::new(),
            jwt_secret: [0; 32],
            jwt_timeout: Duration::from_secs(4 * 24 * 60 * 60),
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
            config: Config::default(),
        }
    }
}

pub async fn store(path: Option<PathBuf>, ctx: Context) -> eyre::Result<()> {
    spawn_blocking(move || match path {
        Some(path) => confy::store_path(&path, ctx.clone()).or_else(|e| {
            tracing::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::store_path(path, ctx).wrap_err("Failed to store configuration file")
        }),
        None => confy::store("pet-monitor-app", Some("config"), ctx.clone()).or_else(|e| {
            tracing::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::store("pet-monitor-app", Some("config"), ctx)
                .wrap_err("Failed to write config file")
        }),
    })
    .await?
}

pub async fn load(path: Option<PathBuf>) -> eyre::Result<Context> {
    spawn_blocking(move || match path {
        Some(path) => confy::load_path(&path).or_else(|e| {
            tracing::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::load_path(path).wrap_err("Failed to store configuration file")
        }),
        None => confy::load("pet-monitor-app", Some("config")).or_else(|e| {
            tracing::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::load("pet-monitor-app", Some("config")).wrap_err("Failed to write config file")
        }),
    })
    .await?
}
