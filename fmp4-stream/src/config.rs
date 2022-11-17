#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{collections::HashMap, fmt, path::PathBuf};

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
    H264 = u32::from_be_bytes(*b"H264"),
    YUYV = u32::from_be_bytes(*b"YUYV"),
    YV12 = u32::from_be_bytes(*b"YV12"),
    RGB3 = u32::from_be_bytes(*b"RGB3"),
    BGR3 = u32::from_be_bytes(*b"BGR3"),
}

impl From<Format> for [u8; 4] {
    fn from(format: Format) -> Self {
        (format as u32).to_be_bytes()
    }
}

impl TryFrom<u32> for Format {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match &value.to_be_bytes() {
            b"H264" => Ok(Self::H264),
            b"YUYV" => Ok(Self::YUYV),
            b"YV12" => Ok(Self::YV12),
            b"RGB3" => Ok(Self::RGB3),
            b"BGR3" => Ok(Self::BGR3),
            other => Err(format!("Invalid fourCC: {:?}", std::str::from_utf8(other))),
        }
    }
}

impl TryFrom<[u8; 4]> for Format {
    type Error = String;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        u32::from_be_bytes(value).try_into()
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

#[cfg(feature = "quickcheck")]
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

#[cfg(feature = "quickcheck")]
impl Arbitrary for Format {
    fn arbitrary(g: &mut Gen) -> Self {
        match u32::arbitrary(g) % 5 {
            0 => Self::YUYV,
            1 => Self::YV12,
            2 => Self::RGB3,
            3 => Self::BGR3,
            4 => Self::H264,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "quickcheck")]
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
