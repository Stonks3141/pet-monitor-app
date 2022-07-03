use futures::{Stream, stream};
use rscam::{Camera, Config, Frame};

pub fn frame_stream() -> impl Stream<Item = Frame> {
    let mut camera = Camera::new("/dev/video0").unwrap();

    camera.start(&Config {
        interval: (1, 30),
        resolution: (1280, 720),
        format: b"H264",
        ..Default::default()
    }).unwrap();

    stream::unfold(camera, |c| async move {
        Some(
            blocking::unblock(|| {
                (c.capture().unwrap(), c)
            }).await
        )
    })
}
