use chrono::Duration;
use rocket::tokio::task::spawn_blocking;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};

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
    pub host: Option<IpAddr>,
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
            host: Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            #[cfg(not(debug_assertions))]
            port: 80,
            #[cfg(debug_assertions)]
            port: 8080,
            config: Config::default(),
            tls: None,
        }
    }
}

/// The config accessible by the browser
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Pixel resolution (width, height)
    pub resolution: (u32, u32),
    /// Rotation in degrees (0, 90, 180, or 270)
    pub rotation: u32,
    /// Framerate in frames per second
    pub framerate: u32,
    /// The v4l2 device to capture video with (eg. "/dev/video0")
    pub device: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: (1280, 720),
            rotation: 0,
            framerate: 30,
            device: PathBuf::from("/dev/video0"),
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
            #[cfg(not(debug_assertions))]
            port: 443,
            #[cfg(debug_assertions)]
            port: 8443,
            cert: PathBuf::from("path/to/cert.pem"),
            key: PathBuf::from("path/to/key.key"),
        }
    }
}

/// Writes out a [`Context`] to the config file.
pub async fn store<P: AsRef<Path>>(path: &Option<P>, ctx: &Context) -> anyhow::Result<()> {
    use anyhow::Context;
    let ctx = ctx.to_owned();
    if let Some(path) = path {
        let path = path.as_ref().to_owned();
        spawn_blocking(move || {
            confy::store_path(path, ctx).context("Failed to store configuration file")
        })
        .await?
    } else {
        spawn_blocking(move || {
            confy::store("pet-monitor-app", Some("config"), ctx).context("Failed to store configuration file")
        })
        .await?
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
            confy::load("pet-monitor-app", Some("config")).context("Failed to load configuration file")
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
        let tmp = NamedTempFile::new("pet-monitor-app.toml").unwrap();

        let ctx = Context::default();

        store(&Some(tmp.path()), &ctx).await.unwrap();
        assert!(tmp.exists());

        let ctx_file = load(&Some(tmp.path())).await.unwrap();
        assert_eq!(ctx, ctx_file);

        tmp.close().unwrap();
    }
}
