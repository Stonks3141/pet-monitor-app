//! Types for stream and camera configuration.

#[cfg(feature = "serde")]
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};
#[cfg(feature = "serde")]
use std::{collections::HashMap, fmt, path::PathBuf};

/// The main configuration struct.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The v4l2 device to capture video with (eg. "/dev/video0").
    pub device: PathBuf,
    /// The fourCC code to capture in (eg. "YUYV").
    pub format: Format,
    /// Pixel resolution (width, height).
    pub resolution: (u32, u32),
    /// A fraction representing the framerate.
    pub interval: (u32, u32),
    /// The rotation for the MP4 matrix. This is not supported by some media players.
    pub rotation: Rotation,
    /// Additional controls to pass to V4L2.
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

/// The fourCC code to capture in.
///
/// Currently supported formats are H264 and those supported by libx264.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum Format {
    /// H264 format. Selecting this option will disable software encoding, which is
    /// generally much faster but can reduce quality or compression ratio.
    H264 = u32::from_be_bytes(*b"H264"),
    /// YUYV format. In this format, 4 bytes encode 2 pixels, with the first encoding the
    /// Y or luminance component for the first pixel, the second encoding the U or Cb component for
    /// both pixels, the third encoding the Y component for the second pixel, and the fourth
    /// encoding the V or Cr component for both pixels.
    YUYV = u32::from_be_bytes(*b"YUYV"),
    /// YV12 format. 6 bytes encode 4 pixels, with the first 4 bytes encoding the Y
    /// component for each pixel, the fifth byte encoding the V component for all 4 pixels,
    /// and the last byte encoding the U component for all 4 pixels. The "12" refers to the
    /// format's bit depth.
    YV12 = u32::from_be_bytes(*b"YV12"),
    /// RGB format. 3 bytes encode 1 pixel, with the first encoding the red component,
    /// the second encoding the green component, and the third encoding the blue component.
    RGB3 = u32::from_be_bytes(*b"RGB3"),
    /// BGR format. 3 bytes encode 1 pixel, with the first encoding the blue component,
    /// the second encoding the green component, and the third encoding the red component.
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
        write!(f, "{format}")
    }
}

#[cfg(feature = "serde")]
impl Serialize for Format {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Format {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let format = String::deserialize(d)?;
        let format = format.as_bytes();
        let format = [format[0], format[1], format[2], format[3]];
        Self::try_from(u32::from_be_bytes(format)).map_err(D::Error::custom)
    }
}

/// The rotation for the MP4 rotation matrix.
///
/// This is not supported by some media players.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Rotation {
    /// 0 degrees of rotation.
    R0 = 0,
    /// 90 degrees of rotation.
    R90 = 90,
    /// 180 degrees of rotation.
    R180 = 180,
    /// 270 degrees of rotation.
    R270 = 270,
}

#[cfg(feature = "serde")]
impl Serialize for Rotation {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (*self as u32).serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(match u32::deserialize(d)? {
            0 => Self::R0,
            90 => Self::R90,
            180 => Self::R180,
            270 => Self::R270,
            x => {
                return Err(D::Error::invalid_value(
                    Unexpected::Unsigned(x as u64),
                    &"one of 0, 90, 180, or 270",
                ));
            }
        })
    }
}
