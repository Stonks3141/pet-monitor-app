use chrono::Duration;
use serde::{Deserialize, Serialize};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub password_hash: String,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub jwt_secret: [u8; 32],
    /// days
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub jwt_timeout: Duration,
    pub domain: String,
    #[serde(flatten)]
    pub config: Config,
    pub tls: Option<Tls>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            password_hash: String::new(),
            jwt_secret: [0; 32],
            jwt_timeout: Duration::days(4),
            domain: "localhost".to_string(),
            config: Config::default(),
            tls: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// width, height
    pub resolution: (u32, u32),
    pub rotation: u32,
    pub framerate: u32,
    pub device: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: (1280, 720),
            rotation: 0,
            framerate: 30,
            device: "/dev/video0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tls {
    pub cert: String,
    pub key: String,
}

impl Default for Tls {
    fn default() -> Self {
        Self {
            cert: "path/to/cert.pem".to_string(),
            key: "path/to/key.key".to_string(),
        }
    }
}
