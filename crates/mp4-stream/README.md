# mp4-stream

[![Repository][repo]](https://github.com/Stonks3141/pet-monitor-app)
[![crates.io][cratesio]](https://crates.io/crates/mp4-stream)
[![docs.rs][docsrs]](https://docs.rs/mp4-stream)
[![CI][ci]](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)

A fast and easy to use fMP4 streaming implementation.

mp4-stream is an efficient and scalable implementation of fragmented MP4
video streaming. It uses channels to separate video capture and encoding from
MP4 muxing, making it possible to stream live video over multiple
connections. It can also handle live configuration updates, which require
restarting the individual streams, but the video capture worker does not
have to be restarted.

# Usage

```rust
use mp4_stream::{config::Config, VideoStream, stream_media_segments};
use std::{fs, thread, io::Write};

fn main() -> Result<(), Box<dyn std::error::Error> {
    // Create a configuration
    let config = Config::default();
    let config_clone = config.clone();
    // Create a channel to send requests for video data on
    let (tx, rx) = flume::unbounded();
    // Start another thread to capture video and send it on the channel
    thread::spawn(move || {
        stream_media_segments(rx, config_clone, None).unwrap();
    });

    let mut file = fs::File::create("video.mp4")?;
    // Create a stream from the channel
    let stream = VideoStream::new(&config, tx)?;
    // Take the first 10 segments and write them to a file
    for segment in stream.take(10) {
        file.write_all(&segment?)?;
    }
    Ok(())
}
```

[repo]: https://img.shields.io/badge/Github-Stonks3141/pet--monitor--app-orange?style=for-the-badge&logo=github&color=red
[cratesio]: https://img.shields.io/crates/v/mp4-stream?style=for-the-badge
[docsrs]: https://img.shields.io/docsrs/mp4-stream?style=for-the-badge&color=blue
[ci]: https://img.shields.io/github/actions/workflow/status/Stonks3141/pet-monitor-app/ci.yml?style=for-the-badge
