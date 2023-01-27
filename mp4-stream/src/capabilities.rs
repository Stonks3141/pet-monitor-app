//! Utilities to detect a camera's capabilities.
//!
//! The easiest way to get the capabilities is with the [`get_capabilities_all`]
//! function. It will look for all camera devices (paths that match `/dev/video*`) and get the
//! available formats, resolutions, and framerates for each. If you want to get capabilities
//! for device paths that don't match that pattern, you can use [`get_capabilities_from_path`].
//!
//! This module also provides a [`check_config`] function to check whether a [`Config`](crate::config::Config)
//! is supported by a set of capabilities. You should always validate a config with [`check_config`]
//! before giving it to [`stream_media_segments`](crate::stream_media_segments).
//!
//! # Example
//!
//! ```rust,no_run
//! use mp4_stream::{
//!     capabilities::{get_capabilities_all, check_config},
//!     config::Config,
//! };
//!
//! let config = Config::default();
//! let capabilities = get_capabilities_all()?;
//! if check_config(&config, &capabilities).is_ok() {
//!     println!("All good!");
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::config::{Config, Format};
use crate::Error;
use rscam::{IntervalInfo, ResolutionInfo};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

/// A map of device paths to available formats.
///
/// It serializes like this since JSON only supports string keys:
///
/// ```json
/// {
///   "/dev/video0": {
///     "YUYV": [
///       {
///         "resolution": [640, 480],
///         "intervals": [
///           [1, 30]
///         ]
///       }
///     ]
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities(pub HashMap<PathBuf, Formats>);

/// A map of format codes to available resolutions.
pub type Formats = HashMap<Format, Resolutions>;

/// A map of resolutions to available intervals.
///
/// The resolutions are in (width, height) format.
pub type Resolutions = HashMap<(u32, u32), Intervals>;

/// A list of available intervals.
///
/// The framerate for an interval is the first tuple field divided by the second.
pub type Intervals = Vec<(u32, u32)>;

#[cfg(feature = "serde")]
impl Serialize for Capabilities {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map: HashMap<PathBuf, HashMap<Format, Vec<Resolution>>> = self
            .0
            .clone()
            .into_iter()
            .map(|(path, formats)| {
                (
                    path,
                    formats
                        .into_iter()
                        .map(|(format, resolutions)| {
                            #[allow(clippy::unwrap_used)] // FourCC codes are always printable ASCII
                            (
                                format,
                                resolutions
                                    .into_iter()
                                    .map(|(resolution, intervals)| Resolution {
                                        resolution,
                                        intervals,
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                )
            })
            .collect();
        map.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Resolution {
    resolution: (u32, u32),
    intervals: Vec<(u32, u32)>,
}

/// Gets the camera devices and capabilities for each.
///
/// See the [module-level docs](self) for more information.
///
/// # Errors
///
/// This function may return a [`Error::Io`] if interaction with the filesystem
/// fails or a [`Error::Camera`] if the camera returns an error.
pub fn get_capabilities_all() -> crate::Result<Capabilities> {
    let mut caps = HashMap::new();

    for f in fs::read_dir(PathBuf::from("/dev"))? {
        let path = f?.path();
        if path
            .file_name()
            .and_then(OsStr::to_str)
            .map_or(false, |name| name.starts_with("video"))
        {
            let path_clone = path.clone();
            let path_caps = get_capabilities_from_path(&path_clone)?;
            caps.insert(path.clone(), path_caps);
        }
    }

    Ok(Capabilities(caps))
}

/// Gets the capabilities of a camera at a certain path.
///
/// See the [module-level docs](self) for more information.
///
/// # Errors
///
/// This function may return a [`Error::Io`] if interaction with the filesystem
/// fails or a [`Error::Camera`] if the camera returns an error.
pub fn get_capabilities_from_path(device: &Path) -> crate::Result<Formats> {
    let camera = rscam::Camera::new(
        device
            .to_str()
            .ok_or_else(|| "Failed to convert device path to string".to_string())?,
    )?;
    get_capabilities(&camera)
}

fn get_capabilities(camera: &rscam::Camera) -> crate::Result<Formats> {
    camera
        .formats()
        .filter_map(|x| x.ok())
        .filter_map(|fmt| {
            u32::from_be_bytes(fmt.format)
                .try_into()
                .ok()
                .map(|format| (fmt, format))
        })
        .map(|(fmt, format)| {
            let resolutions: Result<_, Error> = get_resolutions(camera.resolutions(&fmt.format)?)
                .into_iter()
                .map(|resolution| {
                    Ok((
                        resolution,
                        get_intervals(camera.intervals(&fmt.format, resolution)?),
                    ))
                })
                .collect();
            Ok((format, resolutions?))
        })
        .collect()
}

fn get_resolutions(resolutions: ResolutionInfo) -> Vec<(u32, u32)> {
    match resolutions {
        ResolutionInfo::Discretes(r) => r,
        ResolutionInfo::Stepwise { min, max, step } => (min.0..max.0)
            .filter(|x| (x - min.0) % step.0 == 0)
            .zip((min.1..max.1).filter(|x| (x - min.1) % step.1 == 0))
            .collect(),
    }
}

fn get_intervals(intervals: IntervalInfo) -> Vec<(u32, u32)> {
    match intervals {
        IntervalInfo::Discretes(r) => r,
        IntervalInfo::Stepwise { min, max, step } => (min.0..max.0)
            .filter(|x| (x - min.0) % step.0 == 0)
            .zip((min.1..max.1).filter(|x| (x - min.1) % step.1 == 0))
            .collect(),
    }
}

/// Verifies that a config is valid using a given set of capabilities.
///
/// See the [module-level docs](self) for more information.
///
/// # Errors
///
/// This function may return a [`Error::Other`] if any part of the config is invalid,
/// including the V4L2 controls.
pub fn check_config(config: &Config, caps: &Capabilities) -> crate::Result<()> {
    caps.0
        .get(&config.device)
        .ok_or_else(|| format!("Invalid device: {:?}", config.device))?
        .get(&config.format)
        .ok_or_else(|| format!("Invalid format: {}", config.format))?
        .get(&config.resolution)
        .ok_or_else(|| format!("Invalid resolution: {:?}", config.resolution))?
        .contains(&config.interval)
        .then_some(())
        .ok_or_else(|| format!("Invalid interval: {:?}", config.interval))?;

    let camera = rscam::Camera::new(
        config
            .device
            .as_os_str()
            .to_str()
            .ok_or_else(|| "failed to convert device path to string".to_string())?,
    )?;

    let controls: HashSet<String> = config.v4l2_controls.keys().cloned().collect();
    let valid_controls: HashSet<String> = camera
        .controls()
        .filter_map(|x| x.ok())
        .map(|ctl| ctl.name)
        .collect();

    // all controls that are in `controls` but not `valid_controls`.
    for name in controls.difference(&valid_controls) {
        if controls.get(name).is_none() {
            return Err(Error::Other(format!("Invalid V4L2 control: '{name}'")));
        }
    }

    Ok(())
}
