use futures::stream;
use rscam::{Camera, Config};

fn frame_stream() -> impl stream::Stream {
    let mut camera = Camera::new("/dev/video0").unwrap();

    camera.start(&Config {
        interval: (1, 30),
        resolution: (1280, 720),
        format: b"H264",
        ..Default::default()
    }).unwrap();

    stream::unfold(camera, |c| async move {
        let frame = blocking::unblock(|| c.capture().unwrap()).await;
        
        Some((frame, c))
    })
}
