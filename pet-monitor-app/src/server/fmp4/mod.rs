mod boxes;
pub mod capabilities;

use crate::config::{Config, Context, Format};
use crate::server::provider::Provider;
use anyhow::Context as _;
use boxes::*;
use chrono::{Duration, Utc};
use fixed::types::{I16F16, I8F8, U16F16};
use log::{trace, warn};
use rocket::futures::Stream;
use rscam::Camera;
use std::{
    collections::HashMap,
    io::{self, prelude::*},
    pin::Pin,
    task::Poll,
    time::Instant,
};

#[derive(Debug, Clone)]
pub struct InitSegment {
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

impl From<&Config> for InitSegment {
    fn from(config: &Config) -> Self {
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
            mdat: MediaDataBox { data },
        }
    }

    fn size(&self) -> u64 {
        self.moof.size() + self.mdat.size()
    }

    fn set_base_data_offset(&mut self, offset: u64) {
        self.moof.traf[0].tfhd.base_data_offset = Some(offset);
    }

    fn set_sequence_number(&mut self, sequence_number: u32) {
        self.moof.mfhd.sequence_number = sequence_number;
    }

    fn add_headers(&mut self, mut headers: Vec<u8>) {
        #[allow(clippy::unwrap_used)]
        // MediaSegments constructed with `new` should always have sample_sizes
        {
            self.moof.traf[0].trun[0].sample_sizes.as_mut().unwrap()[0] += headers.len() as u32;
        }
        headers.append(&mut self.mdat.data);
        self.mdat.data = headers;
    }
}

impl WriteTo for MediaSegment {
    fn write_to(&self, mut w: impl Write) -> io::Result<()> {
        write_to(&self.moof, &mut w)?;
        write_to(&self.mdat, &mut w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct VideoStream {
    init_segment: Option<InitSegment>,
    use_headers: bool,
    size: u64,
    sequence_number: u32,
    media_seg_recv: MediaSegReceiver,
    other_recv: Option<MediaSegReceiver>,
    headers: Vec<u8>,
}

impl VideoStream {
    pub async fn new(
        config: &Config,
        stream_sub_tx: flume::Sender<StreamSubscriber>,
    ) -> anyhow::Result<Self> {
        let init_segment = InitSegment::from(config);

        let (tx, rx) = flume::unbounded();
        stream_sub_tx
            .send_async(tx)
            .await
            .context("`VideoStream::new`: Failed to communicate with streaming task")?;
        // if the send succeeds, the other side will respond immediately
        #[allow(clippy::unwrap_used)]
        let (headers, media_seg_recv) = rx.recv_async().await.unwrap();

        Ok(Self {
            size: init_segment.size(),
            init_segment: Some(init_segment),
            use_headers: true,
            sequence_number: 1,
            media_seg_recv,
            other_recv: None,
            headers,
        })
    }

    fn serialize_segment(&mut self, x: Option<MediaSegment>) -> Option<io::Result<Vec<u8>>> {
        let Some(mut media_segment) = x else {
            trace!("VideoStream ended");
            return None;
        };
        if self.use_headers {
            media_segment.add_headers(self.headers.clone());
            self.use_headers = false;
        }
        media_segment.set_base_data_offset(self.size);
        media_segment.set_sequence_number(self.sequence_number);
        self.sequence_number += 1;
        self.size += media_segment.size();
        let mut buf = Vec::with_capacity(media_segment.size() as usize);
        if let Err(e) = media_segment.write_to(&mut buf) {
            return Some(Err(e));
        }
        trace!(
            "VideoStream sent media segment with sequence number {}",
            self.sequence_number - 1
        );
        Some(Ok(buf))
    }
}

impl Stream for VideoStream {
    type Item = io::Result<Vec<u8>>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.init_segment.take() {
            Some(init_segment) => {
                let mut buf = Vec::with_capacity(init_segment.size() as usize);
                if let Err(e) = init_segment.write_to(&mut buf) {
                    return Poll::Ready(Some(Err(e)));
                }
                trace!("VideoStream sent init segment");
                Poll::Ready(Some(Ok(buf)))
            }
            None => {
                if let Some(rx) = &self.other_recv {
                    match rx.try_recv() {
                        Ok(x) => {
                            self.other_recv = None;
                            Poll::Ready(self.serialize_segment(x))
                        }
                        Err(e) => match e {
                            flume::TryRecvError::Disconnected => panic!(),
                            flume::TryRecvError::Empty => Poll::Pending,
                        },
                    }
                } else {
                    match self.media_seg_recv.try_recv() {
                        Ok(x) => Poll::Ready(self.serialize_segment(x)),
                        Err(e) => match e {
                            flume::TryRecvError::Disconnected => Poll::Ready(None),
                            flume::TryRecvError::Empty => {
                                let waker = cx.waker().clone();
                                let (return_tx, rx) = flume::unbounded();
                                self.other_recv = Some(rx);
                                let rx = self.media_seg_recv.clone();
                                rocket::tokio::spawn(async move {
                                    let Ok(seg) = rx.recv_async().await else {
                                        // wake up the stream one more time so it can detect
                                        // the disconnected channel and return `None`.
                                        waker.wake();
                                        return;
                                    };
                                    // rx isn't dropped until a value is received
                                    #[allow(clippy::unwrap_used)]
                                    return_tx.send(seg).unwrap();
                                    waker.wake();
                                });
                                trace!("VideoStream is pending");
                                Poll::Pending
                            }
                        },
                    }
                }
            }
        }
    }
}

struct FrameIter {
    camera: Camera,
}

impl FrameIter {
    fn new(config: &Config) -> anyhow::Result<Self> {
        let mut camera = Camera::new(
            config
                .device
                .as_os_str()
                .to_str()
                .ok_or_else(|| anyhow::Error::msg("failed to convert device path to string"))?,
        )?;

        let controls: HashMap<String, u32> = camera
            .controls()
            .filter_map(Result::ok)
            .map(|ctl| (ctl.name, ctl.id))
            .collect();

        for (name, val) in &config.v4l2_controls {
            if let Some(id) = controls.get(name) {
                camera.set_control(*id, val).unwrap_or(()); // ignore failure
            } else {
                log::warn!("Couldn't find control {}", name);
                // TODO: handle errors by returning a 400 for PUT /api/config
                // or printing an error message if loaded from the config file
            }
        }

        camera.start(&rscam::Config {
            interval: config.interval,
            resolution: config.resolution,
            format: b"YUYV",
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
        frames: FrameIter,
    },
}

impl SegmentIter {
    fn new(config: Config, frames: FrameIter) -> anyhow::Result<Self> {
        Ok(match config.format {
            Format::H264 => Self::Hardware { frames },
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
                        )
                        .map_err(|e| anyhow::Error::msg(format!("{:?}", e)))?
                },
                timestamp: 0,
                config,
                frames,
            },
        })
    }

    fn get_headers(&mut self) -> anyhow::Result<Vec<u8>> {
        Ok(match self {
            Self::Software { encoder, .. } => encoder
                .headers()
                .map_err(|e| anyhow::Error::msg(format!("{:?}", e)))?
                .entirety()
                .to_vec(),
            Self::Hardware { .. } => panic!(),
        })
    }
}

impl Iterator for SegmentIter {
    type Item = MediaSegment;

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
                            warn!("Capturing frame failed with error {:?}", e);
                            panic!();
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
                            warn!("Encoding frame failed with error {:?}", e);
                            panic!();
                        }
                    };

                    sample_sizes.push(data.entirety().len() as u32);
                    buf.extend_from_slice(data.entirety());
                    *timestamp +=
                        *timescale as i64 * config.interval.0 as i64 / config.interval.1 as i64;
                }

                Some(MediaSegment::new(config, 0, sample_sizes, buf))
            }
            Self::Hardware { .. } => panic!(),
        }
    }
}

pub type MediaSegReceiver = flume::Receiver<Option<MediaSegment>>;
pub type StreamSubscriber = flume::Sender<(Vec<u8>, MediaSegReceiver)>;

pub fn stream_media_segments(
    rx: flume::Receiver<StreamSubscriber>,
    ctx: Provider<Context>,
) -> anyhow::Result<std::convert::Infallible> {
    let mut config = ctx.get().config;
    let mut ctx_recv = ctx.subscribe();

    'main: loop {
        trace!("Starting stream with config {:?}", config);
        let mut senders: Vec<flume::Sender<Option<MediaSegment>>> = Vec::new();

        let frames = FrameIter::new(&config)?;
        let mut segments = SegmentIter::new(config.clone(), frames)?;
        let headers = segments.get_headers()?;

        loop {
            if let Ok(ctx) = ctx_recv.try_recv() {
                config = ctx.config;
                senders.retain(|sender| sender.send(None).is_ok());
                trace!("Config updated to {:?}, restarting stream", config);
                continue 'main;
            }
            if let Ok(sender) = rx.try_recv() {
                let (tx, rx) = flume::unbounded();
                senders.push(tx);
                sender.send((headers.clone(), rx)).unwrap_or(());
            }

            let time = Instant::now();
            #[allow(clippy::unwrap_used)] // the iterator never returns `None`
            let media_segment = segments.next().unwrap();
            senders.retain(|sender| sender.send(Some(media_segment.clone())).is_ok());
            trace!("Sent media segment, took {:?} to capture", time.elapsed());
        }
    }
}
