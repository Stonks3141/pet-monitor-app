use mse_fmp4::fmp4::*;
use mse_fmp4::avc::AvcDecoderConfigurationRecord;

pub fn create_init_seg(timescale: u32, width: u32, height: u32) -> InitializationSegment {
    InitializationSegment {
        moov_box: MovieBox {
            mvhd_box: MovieHeaderBox {
                timescale,
                duration: todo!(),
            },
            trak_boxes: vec![
                TrackBox {
                    tkhd_box: TrackHeaderBox {
                        duration: todo!(),
                        width,
                        height,
                        ..Default::default()
                    },
                    edts_box: EditBox {
                        elst_box: EditListBox {
                            media_time: 0,
                        },
                    },
                    mdia_box: MediaBox {
                        mdhd_box: MediaHeaderBox {
                            timescale,
                            duration: todo!(),
                        },
                        hdlr_box: todo!(), // ?
                        minf_box: MediaInformationBox {
                            vmhd_box: Some(VideoMediaHeaderBox), // ?
                            smhd_box: None,
                            dinf_box: DataInformationBox {
                                dref_box: DataReferenceBox {
                                    ..Default::default()
                                },
                            },
                            stbl_box: SampleTableBox {
                                stsd_box: SampleDescriptionBox {
                                    sample_entries: vec![
                                        SampleEntry::Avc(AvcSampleEntry {
                                            width: width.try_into().unwrap(), // u32 to u16
                                            height: height.try_into().unwrap(),
                                            avcc_box: AvcConfigurationBox {
                                                configuration: AvcDecoderConfigurationRecord { // ?
                                                    profile_idc: todo!(),
                                                    constraint_set_flag: todo!(),
                                                    level_idc: todo!(),
                                                    sequence_parameter_set: todo!(),
                                                    picture_parameter_set: todo!(),
                                                },
                                            },
                                        }),
                                    ],
                                },
                                ..Default::default()
                            },
                        },
                    },
                },
            ],
            mvex_box: MovieExtendsBox {
                mehd_box: todo!(),
                trex_boxes: vec![
                    TrackExtendsBox::new(true),
                ],
            }
        },
        ..Default::default()
    }
}

pub fn create_media_seg(sequence_number: u32, data: &Vec<u8>) -> MediaSegment {
    let mut traf = TrackFragmentBox::new(true);
    traf.trun_box.samples.push(todo!());

    MediaSegment {
        moof_box: MovieFragmentBox {
            mfhd_box: MovieFragmentHeaderBox {
                sequence_number,
            },
            traf_boxes: vec![
                traf,
            ],
        },
        mdat_boxes: vec![
            MediaDataBox { data: data.to_vec(), },
        ],
    }
}
