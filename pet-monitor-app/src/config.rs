use chrono::Duration;
use serde::{Deserialize, Serialize};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub password_hash: String,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub jwt_secret: [u8; 32],
    #[serde(rename = "TLS")]
    pub tls: Option<Tls>,
    #[serde(flatten)]
    pub config: Config,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            password_hash: String::new(),
            jwt_secret: [0; 32],
            tls: None,
            config: Config::default(),
        }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// width, height
    pub resolution: (u32, u32),
    pub rotation: u32,
    pub framerate: u32,
    pub device: String,
    /// days
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub jwt_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: (1280, 720),
            rotation: 0,
            framerate: 30,
            device: "/dev/video0".to_string(),
            jwt_timeout: Duration::days(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tls {
    cert: String,
    key: String,
}

impl Default for Tls {
    fn default() -> Self {
        Self {
            cert: "path/to/cert.pem".to_string(),
            key: "path/to/key.key".to_string(),
        }
    }
}
