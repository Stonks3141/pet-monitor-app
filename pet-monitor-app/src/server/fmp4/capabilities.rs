use rocket::tokio::fs;
use rocket::tokio::task::spawn_blocking;
use rscam::{FormatInfo, IntervalInfo, ResolutionInfo};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type Capabilities = HashMap<PathBuf, Vec<Format>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    fourcc: [u8; 4],
    resolutions: Vec<Resolution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    width: u32,
    height: u32,
    framerates: Vec<u32>,
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
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with("video"))
            .unwrap_or(false)
        {
            let path_clone = path.clone();
            let path_caps =
                spawn_blocking(move || get_capabilities_from_path(&path_clone)).await??;
            caps.insert(path.clone(), path_caps);
        }
    }

    Ok(caps)
}

pub fn get_capabilities_from_path(device: &Path) -> anyhow::Result<Vec<Format>> {
    let camera = rscam::Camera::new(
        device
            .to_str()
            .ok_or_else(|| anyhow::Error::msg("Failed to convert device path to string"))?,
    )?;
    get_capabilities(&camera)
}

fn get_capabilities<C>(camera: &C) -> anyhow::Result<Vec<Format>>
where
    C: Camera,
    C::Error: Send + Sync + 'static,
{
    camera
        .formats()
        .filter_map(|fmt| fmt.ok())
        .map(|fmt| {
            let resolutions: anyhow::Result<_> = get_resolutions(camera.resolutions(&fmt.format)?)
                .into_iter()
                .map(|(width, height)| {
                    Ok(Resolution {
                        width,
                        height,
                        framerates: get_framerates(camera.intervals(&fmt.format, (width, height))?),
                    })
                })
                .collect();
            Ok(Format {
                fourcc: fmt.format,
                resolutions: resolutions?,
            })
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

fn get_framerates(intervals: IntervalInfo) -> Vec<u32> {
    match intervals {
        IntervalInfo::Discretes(r) => r
            .into_iter()
            .filter(|(num, _)| *num == 1)
            .map(|(_, den)| den)
            .collect(),
        IntervalInfo::Stepwise { min, max, step } => (min.0..max.0)
            .filter(|x| (x - min.0) % step.0 == 0)
            .zip((min.1..max.1).filter(|x| (x - min.1) % step.1 == 0))
            .filter(|(num, _)| *num == 1)
            .map(|(_, den)| den)
            .collect(),
    }
}
