//! A fast and easy to use fMP4 streaming implementation.
//!
//! mp4-stream is an efficient and scalable implementation of fragmented MP4
//! video streaming. It uses channels to separate video capture and encoding from
//! MP4 muxing, making it possible to stream live video over multiple
//! connections. It can also handle live configuration updates, which require
//! restarting the individual streams, but the video capture worker does not
//! have to be restarted.
//!
//! # Example
//!
//! ```rust,ignore
//! use mp4_stream::{config::Config, VideoStream, stream_media_segments};
//! use std::{fs, thread, io::Write};
//!
//! // Create a configuration
//! let config = Config::default();
//! let config_clone = config.clone();
//! // Create a channel to send requests for video data on
//! let (tx, rx) = flume::unbounded();
//! // Start another thread to capture video and send it on the channel
//! thread::spawn(move || {
//!     stream_media_segments(rx, config_clone, None).unwrap();
//! });
//!
//! let mut file = fs::File::create("video.mp4")?;
//! // Create a stream from the channel
//! let stream = VideoStream::new(&config, tx)?;
//! // Take the first 10 segments and write them to a file
//! for segment in stream.take(10) {
//!     file.write_all(&segment?)?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Cargo Features
//!
//! - `serde`: Add implementations of [`Serialize`](serde::Serialize) and [`Deserialize`](serde::Deserialize)
//!   for types in the [`config`] and [`capabilities`] modules.

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::dbg_macro)]
#![warn(missing_docs)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_errors_doc)]

pub mod capabilities;
pub mod config;

use config::{Config, Format, Rotation};

use bmff::*;
use chrono::{Duration, Utc};
use fixed::types::{I16F16, I8F8, U16F16};
use flume::r#async::RecvStream;
use futures_lite::stream::{self, Stream, StreamExt};
use quick_error::quick_error;
use rscam::Camera;
use std::{
    collections::HashMap,
    io::{self, prelude::*},
    sync::Arc,
};

quick_error! {
    /// The error type for `mp4-stream`.
    #[derive(Debug)]
    #[non_exhaustive]
    pub enum Error {
        /// I/O error. This wraps an [`std::io::Error`].
        Io(err: std::io::Error) {
            source(err)
            display("{}", err)
            from()
        }
        /// Software encoding error. This wraps an [`x264::Error`], which carries
        /// no additional information.
        Encoding(err: x264::Error) {
            display("Encoding error: {:?}", err)
            from()
        }
        /// Camera or video capture error. This wraps an [`rscam::Error`].
        Camera(err: rscam::Error) {
            source(err)
            display("{}", err)
            from()
        }
        /// Another unspecified error.
        Other(err: String) {
            display("{}", err)
            from()
        }
    }
}

/// A `Result` type alias for `mp4-stream`'s [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;

fn matrix(rotation: Rotation) -> [[fixed::types::I16F16; 3]; 3] {
    match rotation {
        Rotation::R0 => MATRIX_0,
        Rotation::R90 => MATRIX_90,
        Rotation::R180 => MATRIX_180,
        Rotation::R270 => MATRIX_270,
    }
}

#[derive(Debug, Clone)]
struct InitSegment {
    ftyp: FileTypeBox,
    moov: MovieBox,
}

impl InitSegment {
    fn size(&self) -> u64 {
        self.ftyp.size() + self.moov.size()
    }
}

impl WriteTo for InitSegment {
    fn write_to(&self, mut w: impl Write) -> io::Result<()> {
        write_to(&self.ftyp, &mut w)?;
        write_to(&self.moov, &mut w)?;
        Ok(())
    }
}

impl InitSegment {
    fn new(config: &Config) -> Self {
        let sps = vec![
            0x67, 0x64, 0x00, 0x1f, 0xac, 0xd9, 0x80, 0x50, 0x05, 0xbb, 0x01, 0x6a, 0x02, 0x02,
            0x02, 0x80, 0x00, 0x00, 0x03, 0x00, 0x80, 0x00, 0x00, 0x1e, 0x07, 0x8c, 0x18, 0xcd,
        ]; // TODO
        let pps = vec![0x68, 0xe9, 0x7b, 0x2c, 0x8b]; // TODO
        let timescale = config.interval.1;
        let (width, height) = config.resolution;

        let ftyp = FileTypeBox {
            major_brand: *b"isom",
            minor_version: 0,
            compatible_brands: vec![*b"isom", *b"iso6", *b"iso2", *b"avc1", *b"mp41"],
        };

        let time = Utc::now();
        let duration = Some(Duration::zero());

        let moov = MovieBox {
            mvhd: MovieHeaderBox {
                creation_time: time,
                modification_time: time,
                timescale,
                duration,
                rate: I16F16::from_num(1),
                volume: I8F8::from_num(1),
                matrix: matrix(config.rotation),
                next_track_id: 0,
            },
            trak: vec![TrackBox {
                tkhd: TrackHeaderBox {
                    flags: TrackHeaderFlags::TRACK_ENABLED
                        | TrackHeaderFlags::TRACK_IN_MOVIE
                        | TrackHeaderFlags::TRACK_IN_PREVIEW,
                    creation_time: time,
                    modification_time: time,
                    track_id: 1,
                    timescale,
                    duration,
                    layer: 0,
                    alternate_group: 0,
                    volume: I8F8::from_num(1),
                    matrix: matrix(config.rotation),
                    width: U16F16::from_num(width),
                    height: U16F16::from_num(height),
                },
                tref: None,
                edts: None,
                mdia: MediaBox {
                    mdhd: MediaHeaderBox {
                        creation_time: time,
                        modification_time: time,
                        timescale,
                        duration,
                        language: *b"und",
                    },
                    hdlr: HandlerBox {
                        handler_type: HandlerType::Video,
                        name: "foo".to_string(), // TODO
                    },
                    minf: MediaInformationBox {
                        media_header: MediaHeader::Video(VideoMediaHeaderBox {
                            graphics_mode: GraphicsMode::Copy,
                            opcolor: [0, 0, 0],
                        }),
                        dinf: DataInformationBox {
                            dref: DataReferenceBox {
                                data_entries: vec![DataEntry::Url(DataEntryUrlBox {
                                    flags: DataEntryFlags::SELF_CONTAINED,
                                    location: String::new(),
                                })],
                            },
                        },
                        stbl: SampleTableBox {
                            stsd: SampleDescriptionBox {
                                entries: vec![Box::new(AvcSampleEntry {
                                    data_reference_index: 1,
                                    width: width as u16,
                                    height: height as u16,
                                    horiz_resolution: U16F16::from_num(72),
                                    vert_resolution: U16F16::from_num(72),
                                    frame_count: 1,
                                    depth: 0x0018,
                                    avcc: AvcConfigurationBox {
                                        configuration: AvcDecoderConfigurationRecord {
                                            profile_idc: 0x64, // high
                                            constraint_set_flag: 0x00,
                                            level_idc: 0x1f, // 0x2a: 4.2 0b0010_1100
                                            sequence_parameter_set: sps,
                                            picture_parameter_set: pps,
                                        },
                                    },
                                })],
                            },
                            stts: TimeToSampleBox { samples: vec![] },
                            stsc: SampleToChunkBox { entries: vec![] },
                            stsz: SampleSizeBox {
                                sample_size: SampleSize::Different(vec![]),
                            },
                            stco: ChunkOffsetBox {
                                chunk_offsets: vec![],
                            },
                        },
                    },
                },
            }],
            mvex: Some(MovieExtendsBox {
                mehd: None,
                trex: vec![TrackExtendsBox {
                    track_id: 1,
                    default_sample_description_index: 1,
                    default_sample_duration: 0,
                    default_sample_size: 0,
                    default_sample_flags: DefaultSampleFlags::empty(),
                }],
            }),
        };

        Self { ftyp, moov }
    }
}

/// An opaque type representing an fMP4 media segment.
///
/// It is passed between the streaming thread and [`VideoStream`]s.
#[derive(Debug, Clone)]
pub struct MediaSegment {
    moof: MovieFragmentBox,
    mdat: MediaDataBox,
}

impl MediaSegment {
    fn new(config: &Config, sequence_number: u32, sample_sizes: Vec<u32>, data: Vec<u8>) -> Self {
        let timescale = config.interval.1;
        let mut moof = MovieFragmentBox {
            mfhd: MovieFragmentHeaderBox { sequence_number },
            traf: vec![TrackFragmentBox {
                tfhd: TrackFragmentHeaderBox {
                    track_id: 1,
                    base_data_offset: Some(0),
                    sample_description_index: None,
                    default_sample_duration: Some(
                        timescale * config.interval.0 / config.interval.1,
                    ),
                    default_sample_size: None,
                    default_sample_flags: {
                        #[allow(clippy::unwrap_used)] // infallible
                        Some(DefaultSampleFlags::from_bits(0x0101_0000).unwrap())
                    }, // not I-frame
                    default_base_is_moof: false,
                },
                trun: vec![TrackFragmentRunBox {
                    data_offset: Some(0),
                    first_sample_flags: Some(0x0200_0000), // I-frame
                    sample_durations: None,
                    sample_sizes: Some(sample_sizes),
                    sample_flags: None,
                    sample_composition_time_offsets: None,
                }],
            }],
        };

        moof.traf[0].trun[0].data_offset = Some(moof.size() as i32 + 8);

        Self {
            moof,
            mdat: MediaDataBox {
                headers: None,
                data: Arc::new(data),
            },
        }
    }

    fn size(&self) -> u64 {
        self.moof.size() + self.mdat.size()
    }

    fn base_data_offset(&mut self) -> &mut Option<u64> {
        &mut self.moof.traf[0].tfhd.base_data_offset
    }

    fn sequence_number(&mut self) -> &mut u32 {
        &mut self.moof.mfhd.sequence_number
    }

    fn add_headers(&mut self, headers: Vec<u8>) {
        // MediaSegments constructed with `new` should always have sample_sizes
        #[allow(clippy::unwrap_used)]
        {
            self.moof.traf[0].trun[0].sample_sizes.as_mut().unwrap()[0] += headers.len() as u32;
        }
        self.mdat.headers = Some(headers);
    }
}

impl WriteTo for MediaSegment {
    fn write_to(&self, mut w: impl Write) -> io::Result<()> {
        write_to(&self.moof, &mut w)?;
        write_to(&self.mdat, &mut w)?;
        Ok(())
    }
}

/// Creates a new video stream.
///
/// # Errors
///
/// This function may return an [`Error::Other`] if all receivers on
/// `stream_sub_tx` have ben dropped.
#[allow(clippy::missing_panics_doc)]
pub async fn stream(
    config: &Config,
    stream_sub_tx: flume::Sender<StreamSubscriber>,
) -> Result<impl Stream<Item = io::Result<Vec<u8>>>> {
    struct StreamState {
        init_segment: Option<InitSegment>,
        size: u64,
        sequence_number: u32,
        segment_stream: RecvStream<'static, MediaSegment>,
        headers: Option<Vec<u8>>,
    }

    let (tx, rx) = flume::unbounded();
    stream_sub_tx
        .send_async(tx)
        .await
        .map_err(|_| "Failed to communicate with streaming task".to_string())?;
    // if the send succeeds, the other side will respond immediately
    #[allow(clippy::unwrap_used)]
    let (headers, segment_rx) = rx.recv_async().await.unwrap();

    let init_segment = InitSegment::new(config);
    let state = StreamState {
        size: init_segment.size(),
        init_segment: Some(init_segment),
        sequence_number: 1,
        segment_stream: segment_rx.into_stream(),
        headers: Some(headers),
    };

    Ok(stream::try_unfold(state, |mut state| async move {
        if let Some(init_segment) = state.init_segment.take() {
            let mut buf = Vec::with_capacity(init_segment.size() as usize);
            init_segment.write_to(&mut buf)?;
            return Ok(Some((buf, state)));
        }

        let Some(mut segment) = state.segment_stream.next().await else {
            #[cfg(feature = "log")]
            log::trace!("VideoStream ended");
            return Ok(None);
        };

        if let Some(headers) = state.headers.take() {
            segment.add_headers(headers);
        }
        *segment.base_data_offset() = Some(state.size);
        *segment.sequence_number() = state.sequence_number;
        state.sequence_number += 1;
        let size = segment.size();
        state.size += size;

        let mut buf = Vec::with_capacity(size as usize);
        segment.write_to(&mut buf)?;

        #[cfg(feature = "log")]
        log::trace!(
            "VideoStream sent media segment with sequence number {}",
            state.sequence_number - 1
        );

        Ok(Some((buf, state)))
    }))
}

struct FrameIter {
    camera: Camera,
}

impl FrameIter {
    fn new(config: &Config) -> Result<Self> {
        let mut camera = Camera::new(
            config
                .device
                .as_os_str()
                .to_str()
                .ok_or_else(|| "failed to convert device path to string".to_string())?,
        )?;

        let controls: HashMap<String, u32> = camera
            .controls()
            .filter_map(|x| x.ok())
            .map(|ctl| (ctl.name, ctl.id))
            .collect();

        for (name, val) in &config.v4l2_controls {
            if let Some(id) = controls.get(name) {
                camera.set_control(*id, val).unwrap_or(()); // ignore failure
            } else {
                #[cfg(feature = "log")]
                log::warn!("Couldn't find control {}", name);
            }
        }

        camera.start(&rscam::Config {
            interval: config.interval,
            resolution: config.resolution,
            format: &<[u8; 4]>::from(config.format),
            ..Default::default()
        })?;

        Ok(Self { camera })
    }
}

impl Iterator for FrameIter {
    type Item = std::io::Result<rscam::Frame>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.camera.capture())
    }
}

enum SegmentIter {
    Software {
        config: Config,
        encoder: x264::Encoder,
        timestamp: i64,
        timescale: u32,
        frames: FrameIter,
    },
    Hardware {
        config: Config,
        frames: FrameIter,
    },
}

impl SegmentIter {
    fn new(config: Config, frames: FrameIter) -> x264::Result<Self> {
        Ok(match config.format {
            Format::H264 => Self::Hardware { frames, config },
            format => Self::Software {
                timescale: config.interval.1,
                encoder: {
                    let timescale = config.interval.1;
                    let bitrate = 896_000;
                    let colorspace = match format {
                        Format::H264 => unreachable!(),
                        Format::BGR3 => x264::Colorspace::BGR,
                        Format::RGB3 => x264::Colorspace::RGB,
                        Format::YUYV => x264::Colorspace::YUYV,
                        Format::YV12 => x264::Colorspace::YV12,
                    };
                    let encoding = x264::Encoding::from(colorspace);

                    x264::Setup::preset(x264::Preset::Superfast, x264::Tune::None, false, true)
                        .fps(config.interval.0, config.interval.1)
                        .timebase(1, timescale)
                        .bitrate(bitrate)
                        .high()
                        .annexb(false)
                        .max_keyframe_interval(60)
                        .scenecut_threshold(0)
                        .build(
                            encoding,
                            config.resolution.0 as i32,
                            config.resolution.1 as i32,
                        )?
                },
                timestamp: 0,
                config,
                frames,
            },
        })
    }

    fn get_headers(&mut self) -> x264::Result<Vec<u8>> {
        Ok(match self {
            Self::Software { encoder, .. } => encoder.headers()?.entirety().to_vec(),
            Self::Hardware { .. } => Vec::new(),
        })
    }
}

impl Iterator for SegmentIter {
    type Item = Result<MediaSegment>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Software {
                config,
                encoder,
                timestamp,
                timescale,
                frames,
            } => {
                let mut sample_sizes = vec![];
                let mut buf = vec![];

                for _ in 0..60 {
                    let frame = match frames.next() {
                        Some(Ok(f)) => f,
                        Some(Err(e)) => {
                            #[cfg(feature = "log")]
                            log::warn!("Capturing frame failed with error {:?}", e);
                            return Some(Err(e.into()));
                        }
                        None => unreachable!(),
                    };

                    let image = x264::Image::new(
                        x264::Colorspace::YUYV,
                        config.resolution.0 as i32,
                        config.resolution.1 as i32,
                        &[x264::Plane {
                            stride: config.resolution.0 as i32 * 2,
                            data: &frame,
                        }],
                    );

                    let (data, _) = match encoder.encode(*timestamp, image) {
                        Ok(x) => x,
                        Err(e) => {
                            #[cfg(feature = "log")]
                            log::warn!("Encoding frame failed with error {:?}", e);
                            return Some(Err(e.into()));
                        }
                    };

                    sample_sizes.push(data.entirety().len() as u32);
                    buf.extend_from_slice(data.entirety());
                    *timestamp +=
                        *timescale as i64 * config.interval.0 as i64 / config.interval.1 as i64;
                }

                Some(Ok(MediaSegment::new(config, 0, sample_sizes, buf)))
            }
            Self::Hardware { frames, config } => {
                let mut sample_sizes = Vec::new();
                let mut buf = Vec::new();
                for _ in 0..60 {
                    let frame = match frames.next() {
                        Some(Ok(f)) => f,
                        Some(Err(e)) => {
                            #[cfg(feature = "log")]
                            log::warn!("Capturing frame failed with error {:?}", e);
                            return Some(Err(e.into()));
                        }
                        None => unreachable!(),
                    };
                    sample_sizes.push(frame.len() as u32);
                    buf.extend_from_slice(&frame);
                }
                Some(Ok(MediaSegment::new(config, 0, sample_sizes, buf)))
            }
        }
    }
}

/// A channel receiver for [`MediaSegment`]s.
///
/// `None` is a marker indicating that the config has changed and the stream
/// has restarted.
pub type MediaSegReceiver = flume::Receiver<MediaSegment>;

/// A channel for adding a subscriber to the stream.
///
/// The main capture and encoding thread will receive these and respond with a
/// tuple of a [`MediaSegReceiver`] and the H264 headers for the stream.
pub type StreamSubscriber = flume::Sender<(Vec<u8>, MediaSegReceiver)>;

/// Start capturing video.
///
/// The optional `config_rx` parameter can be used to send configuration updates. The
/// function will send `None` to all subscribed channels to indicate that the config has
/// changed and then restart the stream with the new config.
///
/// This function may block indefinitely, and should be called in its own thread
/// or with Tokio's [`spawn_blocking`](tokio::task::spawn_blocking) function or similar.
///
/// # Errors
///
/// This function may return an [`Error::Camera`] if interacting with the provided camera
/// device fails, an [`Error::Other`] if the device path is invalid UTF-8, or an
/// [`Error::Encoding`] if constructing an encoder fails.
#[allow(clippy::missing_panics_doc)]
pub fn stream_media_segments(
    rx: flume::Receiver<StreamSubscriber>,
    mut config: Config,
    config_rx: Option<flume::Receiver<Config>>,
) -> Result<std::convert::Infallible> {
    'main: loop {
        #[cfg(feature = "log")]
        log::trace!("Starting stream with config {:?}", config);
        let mut senders: Vec<flume::Sender<MediaSegment>> = Vec::new();

        let frames = FrameIter::new(&config)?;
        let mut segments = SegmentIter::new(config.clone(), frames)?;
        let headers = segments.get_headers()?;

        loop {
            if let Some(Ok(new_config)) = config_rx.as_ref().map(flume::Receiver::try_recv) {
                config = new_config;
                senders.clear();
                #[cfg(feature = "log")]
                log::trace!("Config updated to {:?}, restarting stream", config);
                continue 'main;
            }
            if let Ok(sender) = rx.try_recv() {
                let (tx, rx) = flume::unbounded();
                senders.push(tx);
                sender.send((headers.clone(), rx)).unwrap_or(());
            }

            #[cfg(feature = "log")]
            let time = std::time::Instant::now();
            #[allow(clippy::unwrap_used)] // the iterator never returns `None`
            let Ok(media_segment) = segments.next().unwrap() else {
                break;
            };
            senders.retain(|sender| sender.send(media_segment.clone()).is_ok());
            #[cfg(feature = "log")]
            log::trace!("Sent media segment, took {:?} to capture", time.elapsed());
        }
    }
}
