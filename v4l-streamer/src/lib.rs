//use rscam::Camera;
//use futures::prelude::*;

#[cfg(test)]
mod tests;

//pub fn stream_of_frames(framerate: u32, resolution: (u32, u32)) -> impl Stream {
//    let mut camera = Camera::new("/dev/video0").unwrap();
//
//    camera
//        .start(&rscam::Config {
//            interval: (framerate, 1),
//            resolution,
//            format: b"H264",
//            ..Default::default()
//        }).unwrap();
//
//    futures::stream::unfold(camera, |state: Camera| async move {
//        let frame = blocking::unblock(|| state.capture().unwrap()).await;
//
//        Some((frame, state))
//    })
//}
