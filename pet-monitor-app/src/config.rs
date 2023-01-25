use anyhow::Context as _;
use mp4_stream::config::Config;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    sync::Arc,
    thread::sleep,
    time::Duration,
};
use tokio::task::spawn_blocking;

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

    pub async fn set(&self, ctx: Context) -> anyhow::Result<()> {
        *self.ctx.write() = ctx.clone();

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
    pub domain: String,
    pub host: IpAddr,
    pub port: u16,
    #[serde(flatten)]
    pub config: Config,
    pub tls: Option<Tls>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            password_hash: String::new(),
            jwt_secret: [0; 32],
            jwt_timeout: Duration::from_secs(4 * 24 * 60 * 60),
            domain: "localhost".to_string(),
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
            config: Config::default(),
            tls: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tls {
    pub port: u16,
    pub cert: PathBuf,
    pub key: PathBuf,
}

pub async fn store(path: Option<PathBuf>, ctx: Context) -> anyhow::Result<()> {
    spawn_blocking(move || match path {
        Some(path) => confy::store_path(&path, ctx.clone()).or_else(|e| {
            log::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::store_path(path, ctx).context("Failed to store configuration file")
        }),
        None => confy::store("pet-monitor-app", Some("config"), ctx.clone()).or_else(|e| {
            log::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::store("pet-monitor-app", Some("config"), ctx)
                .context("Failed to write config file")
        }),
    })
    .await?
}

pub async fn load(path: Option<PathBuf>) -> anyhow::Result<Context> {
    spawn_blocking(move || match path {
        Some(path) => confy::load_path(&path).or_else(|e| {
            log::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::load_path(path).context("Failed to store configuration file")
        }),
        None => confy::load("pet-monitor-app", Some("config")).or_else(|e| {
            log::warn!("Writing config failed: {e}, retrying in 10 ms");
            sleep(Duration::from_millis(10));
            confy::load("pet-monitor-app", Some("config")).context("Failed to write config file")
        }),
    })
    .await?
}
