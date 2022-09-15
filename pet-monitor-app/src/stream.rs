//! This module provides utilities for creating async streams of video data.
/*
use crate::config::Config;
use gst::glib::prelude::*;
use gst::prelude::*;
use gstreamer as gst;
use rocket::futures::prelude::*;
use rocket::futures::stream::{self, Stream};
use rocket::tokio::task::spawn_blocking;
use std::io;

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::tokio::io::AsyncWriteExt;
    use rocket::tokio::{self, fs};

    #[test]
    #[ignore]
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
*/
