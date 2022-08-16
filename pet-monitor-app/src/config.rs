use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub password_hash: String,
    #[serde(with = "base64")]
    pub jwt_secret: [u8; 32],
    /// width, height
    pub resolution: (u32, u32),
    pub rotation: u32,
    pub framerate: u32,
    pub device: String,
    /// days
    pub jwt_timeout: u32,
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
            jwt_timeout: 3,
        }
    }
}

mod base64 {
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let base64 = String::deserialize(d)?;
        base64::decode(base64.as_bytes())
            .map_err(|e| serde::de::Error::custom(e))
            .map(|v| v.try_into().expect("Expected 32 bytes"))
    }
}
