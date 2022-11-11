use anyhow::anyhow;
use rocket::tokio::{fs, task::spawn_blocking};
use rscam::{FormatInfo, IntervalInfo, ResolutionInfo};
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities(HashMap<PathBuf, Formats>);

pub type Formats = HashMap<[u8; 4], Resolutions>;
pub type Resolutions = HashMap<(u32, u32), Intervals>;
pub type Intervals = Vec<(u32, u32)>;

impl Serialize for Capabilities {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map: HashMap<PathBuf, HashMap<String, Vec<Resolution>>> = self
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
                                std::str::from_utf8(&format).unwrap().to_owned(),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Resolution {
    resolution: (u32, u32),
    intervals: Vec<(u32, u32)>,
}

trait Camera {
    type Error: std::error::Error;
    fn formats(&self) -> Box<dyn Iterator<Item = Result<FormatInfo, Self::Error>>>;
    fn resolutions(&self, format: &[u8]) -> Result<ResolutionInfo, Self::Error>;
    fn intervals(&self, format: &[u8], resolution: (u32, u32))
        -> Result<IntervalInfo, Self::Error>;
}

impl Camera for rscam::Camera {
    type Error = rscam::Error;

    fn formats(&self) -> Box<dyn Iterator<Item = Result<FormatInfo, Self::Error>>> {
        Box::new(
            self.formats()
                .map(|x| x.map_err(rscam::Error::from))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    fn resolutions(&self, format: &[u8]) -> Result<ResolutionInfo, Self::Error> {
        self.resolutions(format)
    }

    fn intervals(
        &self,
        format: &[u8],
        resolution: (u32, u32),
    ) -> Result<IntervalInfo, Self::Error> {
        self.intervals(format, resolution)
    }
}

pub async fn get_capabilities_all() -> anyhow::Result<Capabilities> {
    let mut caps = HashMap::new();

    let mut dir_items = fs::read_dir(PathBuf::from("/dev")).await?;

    while let Some(f) = dir_items.next_entry().await? {
        let path = f.path();
        if path
            .file_name()
            .and_then(OsStr::to_str)
            .map_or(false, |name| name.starts_with("video"))
        {
            let path_clone = path.clone();
            let path_caps =
                spawn_blocking(move || get_capabilities_from_path(&path_clone)).await??;
            caps.insert(path.clone(), path_caps);
        }
    }

    Ok(Capabilities(caps))
}

pub fn get_capabilities_from_path(device: &Path) -> anyhow::Result<Formats> {
    let camera = rscam::Camera::new(
        device
            .to_str()
            .ok_or_else(|| anyhow::Error::msg("Failed to convert device path to string"))?,
    )?;
    get_capabilities(&camera)
}

fn get_capabilities<C>(camera: &C) -> anyhow::Result<Formats>
where
    C: Camera,
    C::Error: Send + Sync + 'static,
{
    camera
        .formats()
        .filter_map(Result::ok)
        .map(|fmt| {
            let resolutions: anyhow::Result<_> = get_resolutions(camera.resolutions(&fmt.format)?)
                .into_iter()
                .map(|resolution| {
                    Ok((
                        resolution,
                        get_intervals(camera.intervals(&fmt.format, resolution)?),
                    ))
                })
                .collect();
            Ok((fmt.format, resolutions?))
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

pub fn check_config(config: &Config, caps: &Capabilities) -> anyhow::Result<()> {
    caps.0
        .get(&config.device)
        .ok_or_else(|| anyhow!("Invalid device: {:?}", config.device))?
        .get(&config.format)
        .ok_or_else(|| anyhow!("Invalid format: {:?}", std::str::from_utf8(&config.format)))?
        .get(&config.resolution)
        .ok_or_else(|| anyhow!("Invalid resolution: {:?}", config.resolution))?
        .contains(&config.interval)
        .then_some(())
        .ok_or_else(|| anyhow!("Invalid interval: {:?}", config.interval))
}
