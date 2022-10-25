
pub mod boxes;

use boxes::*;
use fixed::types::{U16F16, I8F8, I16F16};
use chrono::{Duration, Utc};
use rocket::tokio::{sync::broadcast, task::{spawn_blocking, JoinHandle}};
use rscam::Camera;

use crate::config::Config;

pub async fn init_segment(config: &Config) -> (FileTypeBox, MovieBox) {
    let sps = vec![
        0x67,0x64,0x00,0x1f,0xac,0xd9,0x80,0x50,0x05,0xbb,0x01,0x6a,0x02,0x02,0x02,0x80,0x00,0x00,0x03,0x00,0x80,0x00,0x00,0x1e,0x07,0x8c,0x18,0xcd,
    ]; // TODO
    let pps = vec![0x68,0xe9,0x7b,0x2c,0x8b]; // TODO

    let ftyp = FileTypeBox {
        major_brand: *b"isom",
        minor_version: 0,
        compatible_brands: vec![*b"isom", *b"iso6", *b"iso2", *b"avc1", *b"mp41"],
    };

    let moov = moov(config.resolution.0, config.resolution.1, config.rotation, config.framerate, sps, pps);

    (ftyp, moov)
}

pub async fn stream_media_segments(config: Config, init_segment_size: u64, sender: broadcast::Sender<(MovieFragmentBox, MediaDataBox)>) -> JoinHandle<()> {    
    spawn_blocking(move || {
        let mut size = init_segment_size;
        let mut timestamp = 0;
        let timescale = config.framerate;
        let bitrate = 896_000;

        let mut camera = Camera::new("/dev/video0").unwrap();
        camera.start(&rscam::Config {
            interval: (1, config.framerate),
            resolution: config.resolution,
            format: b"YUYV",
            ..Default::default()
        }).unwrap();

        let encoding = x264::Encoding::from(x264::Colorspace::YUYV);

        let mut encoder = x264::Setup::preset(x264::Preset::Fast, x264::Tune::None, false, true)
            .fps(1, config.framerate)
            .timebase(1, timescale)
            .bitrate(bitrate)
            .high()
            .annexb(false)
            .max_keyframe_interval(60)
            .scenecut_threshold(0)
            .build(encoding, config.resolution.0 as i32, config.resolution.1 as i32)
            .unwrap();

        for sequence_number in 1.. {
            let mut sample_sizes = vec![];
            let mut buf = vec![];

            if sequence_number == 0 {
                buf.extend_from_slice(encoder.headers().unwrap().entirety());
            }

            for _ in 0..60 {
                let frame = &*camera.capture().unwrap();

                let image = x264::Image::new(x264::Colorspace::YUYV, config.resolution.0 as i32, config.resolution.1 as i32, &[
                    x264::Plane {
                        stride: config.resolution.0 as i32 * 2,
                        data: &frame,
                    },
                ]);

                let (data, _) = encoder.encode(timestamp, image).unwrap();

                sample_sizes.push(data.entirety().len() as u32);
                buf.extend_from_slice(data.entirety());
                timestamp += timescale as i64 / config.framerate as i64;
            }

            let mut moof = MovieFragmentBox {
                mfhd: MovieFragmentHeaderBox { sequence_number },
                traf: vec![TrackFragmentBox {
                    tfhd: TrackFragmentHeaderBox {
                        track_id: 1,
                        base_data_offset: Some(size),
                        sample_description_index: None,
                        default_sample_duration: Some(timescale / config.framerate),
                        default_sample_size: None,
                        default_sample_flags: Some(DefaultSampleFlags::from_bits(0x01010000).unwrap()), // not I-frame
                    },
                    trun: vec![TrackFragmentRunBox {
                        data_offset: Some(0),
                        first_sample_flags: Some(0x02000000),// I-frame
                        sample_durations: None,
                        sample_sizes: Some(sample_sizes),
                        sample_flags: None,
                        sample_composition_time_offsets: None,
                    }],
                }],
            };

            moof.traf[0].trun[0].data_offset = Some(moof.size() as i32 + 8);

            let mdat = MediaDataBox { data: buf };

            size += moof.size();
            size += mdat.size();

            sender.send((moof, mdat)).unwrap_or(0);
        }
    })
}

fn moov(
    width: u32,
    height: u32,
    rotation: u32,
    timescale: u32,
    sps: Vec<u8>,
    pps: Vec<u8>,
) -> MovieBox {
    let time = Utc::now();
    let duration = Some(Duration::zero());

    MovieBox {
        mvhd: MovieHeaderBox {
            creation_time: time,
            modification_time: time,
            timescale,
            duration,
            rate: I16F16::from_num(1),
            volume: I8F8::from_num(1),
            matrix: matrix(rotation).unwrap(),
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
                matrix: matrix(rotation).unwrap(),
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
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rocket::tokio::{self, fs};
    use super::*;

    //#[ignore]
    #[tokio::test]
    async fn test_mp4() {
        let config = crate::config::Config {
            resolution: (640, 480),
            rotation: 0,
            framerate: 30,
            device: PathBuf::from("/dev/video0"),
        };

        let mut file = fs::File::create("video.mp4").await.unwrap();

        let (ftyp, moov) = init_segment(&config).await;
        write_to(&ftyp, &mut file).await.unwrap();
        write_to(&moov, &mut file).await.unwrap();

        let (tx, mut rx) = broadcast::channel(2);
        let handle = stream_media_segments(config, ftyp.size() + moov.size(), tx).await;
        for _ in 0..5 {
            let (moof, mdat) = rx.recv().await.unwrap();
            write_to(&moof, &mut file).await.unwrap();
            write_to(&mdat, &mut file).await.unwrap();
        }
        handle.abort();
    }
}
