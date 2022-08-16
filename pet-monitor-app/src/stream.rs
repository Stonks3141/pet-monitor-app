//! This module provides utilities for creating async streams of video data.

use mse_fmp4::fmp4::{InitializationSegment as InitSegment, MediaSegment};
use rocket::futures::stream::{self, Stream};
use rocket::tokio::task::spawn_blocking;
use rscam::{Camera, Config, Frame};
use std::io;

/// This function returns a byte stream that contains fragmented MP4 data. The
/// initialization segment is included.
pub fn video_stream() -> impl Stream<Item = Vec<u8>> {
    stream::repeat(vec![0])
}

fn init_segment() -> InitSegment {
    InitSegment::default()
}

fn media_seg_stream() -> impl Stream<Item = MediaSegment> {
    // `MediaSegment isn't `clone`, so we can't use `stream::repeat`
    stream::unfold((), |_| async move { Some((MediaSegment::default(), ())) })
}

fn frame_stream(fps: u32, resolution: (u32, u32)) -> impl Stream<Item = io::Result<Frame>> {
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
        Some(spawn_blocking(|| (c.capture(), c)).await.unwrap())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::futures::stream::StreamExt;
    use rocket::tokio::{self, fs, io::AsyncWriteExt};

    #[ignore = "requires a camera device"]
    #[tokio::test]
    async fn frame_stream_test() {
        let mut stream = Box::pin(frame_stream(30, (1280, 720))).take(120);
        let mut file = fs::File::create("data.mjpg").await.unwrap();

        while let Some(frame) = stream.next().await {
            if frame.is_err() {
                continue;
            }
            let frame = frame.unwrap();
            file.write_all(&*frame).await.unwrap();
        }
    }
}
