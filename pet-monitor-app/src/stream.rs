use futures::stream::{self, Stream};
use rscam::{Camera, Config, Frame};

pub fn frame_stream(fps: u32, resolution: (u32, u32)) -> impl Stream<Item = Frame> {
    let mut camera = Camera::new("/dev/video0").unwrap();

    camera
        .start(&Config {
            interval: (1, fps),
            resolution,
            format: b"H264",
            ..Default::default()
        })
        .unwrap();

    stream::unfold(camera, |c| async move {
        Some(blocking::unblock(|| (c.capture().unwrap(), c)).await)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::{fs, io::prelude::*};
    use futures::stream::StreamExt;
    use rocket::tokio;

    #[tokio::test]
    async fn frame_stream_test() {
        let mut stream = Box::pin(frame_stream(30, (1280, 720))).take(120);
        let mut file = fs::File::create("data.mjpg").await.unwrap();

        while let Some(frame) = stream.next().await {
            file.write_all(&*frame).await.unwrap();
        }
    }
}
