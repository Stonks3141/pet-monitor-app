use rscam::{Camera, Frame};

#[cfg(test)]
mod tests;

/// Stream of h.264 frames
pub struct Frames(Camera);

impl Frames {
    pub fn new(interval: (u32, u32), resolution: (u32, u32), format: [u8; 4]) -> Self {
        let mut camera = Camera::new("/dev/video0").unwrap();

        camera
            .start(&rscam::Config {
                interval,
                resolution,
                format: &format,
                ..Default::default()
            })
            .unwrap();

        Self(camera)
    }
}

impl std::iter::Iterator for Frames {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.capture().unwrap())
    }
}
