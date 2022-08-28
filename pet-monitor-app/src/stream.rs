//! This module provides utilities for creating async streams of video data.

use crate::config::Config;
use rocket::futures::prelude::*;
use rocket::futures::stream::{self, Stream};
use rocket::tokio::task::spawn_blocking;
use rscam::{Camera, Frame};
use gstreamer as gst;
use gst::prelude::*;
use gst::glib::prelude::*;
use std::io;

pub fn video_stream(config: &Config) -> impl Stream<Item = Vec<u8>> {
    let stream = Box::pin(frame_stream(&config));

    stream::repeat(Vec::new())
}

fn frame_stream(config: &Config) -> impl Stream<Item = io::Result<Frame>> {
    let mut camera = Camera::new(&config.device).unwrap();

    camera
        .start(&rscam::Config {
            interval: (1, config.framerate),
            resolution: config.resolution,
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
    use rocket::tokio::{self, fs};
    use rocket::tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn h264() {
        let config = confy::load_path("./pet-monitor-app.toml").unwrap();
        let mut file = fs::File::create("data.h264").await.unwrap();
        let mut stream = Box::pin(frame_stream(&config).take(240));
        while let Some(frame) = stream.next().await {
            if let Ok(frame) = frame {
                file.write_all(&*frame).await.unwrap();
            }
        }
    }

    #[test]
    fn mp4() {
        let config: Config = confy::load_path("./pet-monitor-app.toml").unwrap();

        gst::init().unwrap();

        let v4l2src = gst::ElementFactory::make("v4l2src", None).unwrap();
        let x264enc = gst::ElementFactory::make("x264enc", None).unwrap();
        let mp4mux = gst::ElementFactory::make("mp4mux", None).unwrap();
        let filesink = gst::ElementFactory::make("filesink", None).unwrap();

        v4l2src.set_property("device", "/dev/video0");
        x264enc.set_property("bitrate", 2500u32);
        mp4mux.set_property("faststart", true);
        filesink.set_property("location", "video.mp4");

        let pipeline = gst::Pipeline::new(None);
        pipeline.add_many(&[&v4l2src, &x264enc, &filesink]).unwrap();

        v4l2src.link(&x264enc).unwrap();
        // let src_pad = x264enc.static_pad("src").unwrap();
        // let sink_pad = mp4mux.request_pad_simple("video_%u").unwrap();
        // src_pad.link(&sink_pad).unwrap();
        x264enc.link(&filesink).unwrap();

        pipeline.set_state(gst::State::Playing).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}