use rscam::{Camera, Frame};
//use futures::prelude::*;
//use futures::task::SpawnExt;
//use futures::executor::ThreadPool;

#[cfg(test)]
mod tests;
//mod bmff;

/// Stream of h.264 frames
pub struct Frames(Camera);

impl Frames {
    pub fn new(interval: (u32, u32), resolution: (u32, u32)) -> Self {
        let mut camera = Camera::new("/dev/video0").unwrap();

        camera
            .start(&rscam::Config {
                interval,
                resolution,
                format: b"H264",
                ..Default::default()
            })
            .unwrap();

        Self(camera)
    }

    //pub fn get_frame<'a>(&'a mut self) -> impl Future<Output = Frame> + 'a {
    //    let pool = ThreadPool::new().unwrap();

    //    pool.spawn_with_handle(async {
    //        self.0.capture().unwrap()
    //    }).unwrap()
    //}
}

impl Iterator for Frames {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.capture().unwrap())
    }
}
