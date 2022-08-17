//! This module provides utilities for creating async streams of video data.

use crate::config::Config;
use mse_fmp4::fmp4::{InitializationSegment as InitSegment, MediaSegment};
use mse_fmp4::io::WriteTo;
use rocket::futures::prelude::*;
use rocket::futures::stream::{self, Stream};
use rocket::tokio::task::spawn_blocking;
use rscam::{Camera, Frame};
use std::io;

pub fn video_stream(config: &Config) -> impl Stream<Item = Vec<u8>> {
    let stream = Box::pin(media_seg_stream(&config));
    stream::unfold(stream, |mut s| async move {
        let mut vec = Vec::new();
        if let Some(item) = s.next().await {
            item.write_to(&mut vec).unwrap();
            Some((vec, s))
        } else {
            None
        }
    })
}

pub fn init_segment(config: &Config) -> InitSegment {
    InitSegment::default()
}

fn media_seg_stream(config: &Config) -> impl Stream<Item = MediaSegment> {
    // `MediaSegment isn't `clone`, so we can't use `stream::repeat`
    stream::unfold((), |_| async move { Some((MediaSegment::default(), ())) })
}

fn frame_stream(config: &Config) -> impl Stream<Item = io::Result<Frame>> {
    let mut camera = Camera::new("/dev/video0").unwrap();

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
