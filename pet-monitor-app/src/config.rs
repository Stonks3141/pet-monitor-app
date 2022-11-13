use anyhow::Context as _;
use chrono::Duration;
use rocket::tokio::task::spawn_blocking;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::fmt;
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
    /// The v4l2 device to capture video with (eg. "/dev/video0")
    pub device: PathBuf,
    /// The fourCC code to capture in
    pub format: Format,
    /// Pixel resolution (width, height)
    pub resolution: (u32, u32),
    /// A fraction representing the length of time for a frame
    pub interval: (u32, u32),
    /// Rotation in degrees (0, 90, 180, or 270)
    pub rotation: Rotation,
    /// Additional options to pass to v4l2
    #[serde(rename = "v4l2Controls")]
    pub v4l2_controls: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: PathBuf::from("/dev/video0"),
            format: Format::YUYV,
            resolution: (640, 480),
            interval: (1, 30),
            rotation: Rotation::R0,
            v4l2_controls: HashMap::new(),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum Format {
    YUYV = u32::from_be_bytes(*b"YUYV"),
    YV12 = u32::from_be_bytes(*b"YV12"),
    RGB3 = u32::from_be_bytes(*b"RGB3"),
    BGR3 = u32::from_be_bytes(*b"BGR3"),
}

impl TryFrom<u32> for Format {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match &value.to_be_bytes() {
            b"YUYV" => Ok(Self::YUYV),
            b"YV12" => Ok(Self::YV12),
            b"RGB3" => Ok(Self::RGB3),
            b"BGR3" => Ok(Self::BGR3),
            other => Err(format!("Invalid fourCC: {:?}", std::str::from_utf8(other))),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = &(*self as u32).to_be_bytes();
        #[allow(clippy::unwrap_used)] // all enum variants are valid UTF-8
        let format = std::str::from_utf8(bytes).unwrap();
        write!(f, "{}", format)
    }
}

impl Serialize for Format {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(s)
    }
}

impl<'de> Deserialize<'de> for Format {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let format = String::deserialize(d)?;
        let format = format.as_bytes();
        let format = [format[0], format[1], format[2], format[3]];
        Self::try_from(u32::from_be_bytes(format)).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum Rotation {
    R0 = 0,
    R90 = 90,
    R180 = 180,
    R270 = 270,
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
        let tmp = NamedTempFile::new("pet-monitor-app.toml").unwrap();

        let ctx = Context::default();

        store(&Some(tmp.path()), &ctx).await.unwrap();
        assert!(tmp.exists());

        let ctx_file = load(&Some(tmp.path())).await.unwrap();
        assert_eq!(ctx, ctx_file);

        tmp.close().unwrap();
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

    impl Arbitrary for Config {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                device: Arbitrary::arbitrary(g),
                format: Arbitrary::arbitrary(g),
                resolution: Arbitrary::arbitrary(g),
                interval: Arbitrary::arbitrary(g),
                rotation: Arbitrary::arbitrary(g),
                v4l2_controls: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for Format {
        fn arbitrary(g: &mut Gen) -> Self {
            match u32::arbitrary(g) % 4 {
                0 => Self::YUYV,
                1 => Self::YV12,
                2 => Self::RGB3,
                3 => Self::BGR3,
                _ => unreachable!(),
            }
        }
    }

    impl Arbitrary for Rotation {
        fn arbitrary(g: &mut Gen) -> Self {
            match u32::arbitrary(g) % 4 {
                0 => Self::R0,
                1 => Self::R90,
                2 => Self::R180,
                3 => Self::R270,
                _ => unreachable!(),
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
