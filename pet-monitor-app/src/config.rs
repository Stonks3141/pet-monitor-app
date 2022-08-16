use serde::{Deserialize, Serialize};
use chrono::Duration;

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub password_hash: String,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub jwt_secret: [u8; 32],
    /// width, height
    pub resolution: (u32, u32),
    pub rotation: u32,
    pub framerate: u32,
    pub device: String,
    /// days
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub jwt_timeout: Duration,
    #[serde(rename = "TLS")]
    pub tls: Option<Tls>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            password_hash: String::new(),
            jwt_secret: [0; 32],
            resolution: (1280, 720),
            rotation: 0,
            framerate: 30,
            device: "/dev/video0".to_string(),
            jwt_timeout: Duration::days(1),
            tls: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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