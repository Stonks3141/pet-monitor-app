use std::num::NonZeroU32;

use crate::config::Rotation;
use bitflags::bitflags;
use chrono::{DateTime, Duration, Utc};
use fixed::types::{I16F16, I8F8, U16F16};
use std::io::{self, prelude::*};

macro_rules! matrix {
    ( $( [ $($val:literal),* $(,)? ] ),* $(,)? ) => {
        [
            $([
                $( I16F16::from_bits($val), )*
            ],)*
        ]
    }
}

pub const MATRIX_0: [[I16F16; 3]; 3] = matrix![
    [0x0001_0000, 0x0000_0000, 0x0000_0000],
    [0x0000_0000, 0x0001_0000, 0x0000_0000],
    [0x0000_0000, 0x0000_0000, 0x4000_0000],
];

pub const MATRIX_90: [[I16F16; 3]; 3] = matrix![
    [0x0000_0000, 0x0001_0000, 0x0000_0000],
    [-0x4000_0000, 0x0000_0000, 0x0000_0000],
    [0x0000_0000, 0x0000_0000, 0x4000_0000],
];

pub const MATRIX_180: [[I16F16; 3]; 3] = matrix![
    [-0x4000_0000, 0x0000_0000, 0x0000_0000],
    [0x0000_0000, 0x4000_0000, 0x0000_0000],
    [0x0000_0000, 0x0000_0000, 0x4000_0000],
];

pub const MATRIX_270: [[I16F16; 3]; 3] = matrix![
    [0x0000_0000, -0x4000_0000, 0x0000_0000],
    [0x0001_0000, 0x0000_0000, 0x0000_0000],
    [0x0000_0000, 0x0000_0000, 0x4000_0000],
];

pub const fn matrix(rotation: Rotation) -> [[I16F16; 3]; 3] {
    match rotation {
        Rotation::R0 => MATRIX_0,
        Rotation::R90 => MATRIX_90,
        Rotation::R180 => MATRIX_180,
        Rotation::R270 => MATRIX_270,
    }
}

fn duration(duration: &Duration, timescale: u32) -> u64 {
    (duration.num_milliseconds() as f64 / 1000.0 * timescale as f64) as u64
}

pub trait BmffBox {
    const TYPE: [u8; 4];
    const EXTENDED_TYPE: Option<[u8; 16]> = None;
    fn size(&self) -> u64;
    fn write_box(&self, writer: impl Write) -> io::Result<()>;
}

pub trait FullBox: BmffBox {
    fn version(&self) -> u8;
    #[inline]
    fn flags(&self) -> [u8; 3] {
        [0; 3]
    }
}

pub trait WriteTo {
    fn write_to(&self, writer: impl Write) -> io::Result<()>;
}

pub fn write_to<T: BmffBox>(bmff_box: &T, mut w: impl Write) -> io::Result<()> {
    let size = bmff_box.size();
    if u32::try_from(size).is_ok() {
        w.write_all(&(size as u32).to_be_bytes())?;
        w.write_all(&T::TYPE)?;
    } else {
        w.write_all(&1u32.to_be_bytes())?;
        w.write_all(&T::TYPE)?;
        w.write_all(&(size + 8).to_be_bytes())?;
    }
    if let Some(ext_type) = T::EXTENDED_TYPE {
        w.write_all(&ext_type)?;
    }
    bmff_box.write_box(&mut w)?;
    Ok(())
}

pub fn write_to_full<T: FullBox>(bmff_box: &T, mut w: impl Write) -> io::Result<()> {
    let size = bmff_box.size();
    if u32::try_from(size).is_ok() {
        w.write_all(&(size as u32).to_be_bytes())?;
        w.write_all(&T::TYPE)?;
    } else {
        w.write_all(&1u32.to_be_bytes())?;
        w.write_all(&T::TYPE)?;
        w.write_all(&(size + 8).to_be_bytes())?;
    }
    if let Some(ext_type) = T::EXTENDED_TYPE {
        w.write_all(&ext_type)?;
    }
    w.write_all(&[bmff_box.version()])?;
    w.write_all(&bmff_box.flags())?;
    bmff_box.write_box(&mut w)?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct FileTypeBox {
    pub major_brand: [u8; 4],
    pub minor_version: u32,
    pub compatible_brands: Vec<[u8; 4]>,
}

impl BmffBox for FileTypeBox {
    const TYPE: [u8; 4] = *b"ftyp";

    #[inline]
    fn size(&self) -> u64 {
        8 + 4 + 4 + self.compatible_brands.len() as u64 * 4
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&self.major_brand)?;
        w.write_all(&self.minor_version.to_be_bytes())?;
        for i in &self.compatible_brands {
            w.write_all(i)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MovieBox {
    pub mvhd: MovieHeaderBox,
    pub trak: Vec<TrackBox>,
    pub mvex: Option<MovieExtendsBox>,
}

impl BmffBox for MovieBox {
    const TYPE: [u8; 4] = *b"moov";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.mvhd.size()
            + self.trak.iter().map(BmffBox::size).sum::<u64>()
            + self.mvex.as_ref().map_or(0, BmffBox::size)
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.mvhd, &mut w)?;
        for trak in &self.trak {
            write_to(trak, &mut w)?;
        }
        if let Some(mvex) = &self.mvex {
            write_to(mvex, &mut w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MovieHeaderBox {
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
    pub timescale: u32,
    pub duration: Option<Duration>,
    pub rate: I16F16,
    pub volume: I8F8,
    pub matrix: [[I16F16; 3]; 3],
    pub next_track_id: u32,
}

impl BmffBox for MovieHeaderBox {
    const TYPE: [u8; 4] = *b"mvhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + (if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self
                .duration
                .as_ref()
                .map_or(u32::MAX as u64, |x| duration(x, self.timescale))
                > u32::MAX as u64
        {
            8 + 8 + 4 + 8
        } else {
            4 + 4 + 4 + 4
        }) + 4
            + 2
            + 2
            + 4 * 2
            + 4 * 9
            + 4 * 6
            + 4
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        let creation_timestamp = self.creation_time.timestamp();
        let modification_timestamp = self.modification_time.timestamp();
        let duration_secs = self
            .duration
            .as_ref()
            .map_or(u32::MAX as u64, |x| duration(x, self.timescale));
        if creation_timestamp as u64 > u32::MAX as u64
            || modification_timestamp as u64 > u32::MAX as u64
            || duration_secs > u32::MAX as u64
        {
            w.write_all(&creation_timestamp.to_be_bytes())?;
            w.write_all(&modification_timestamp.to_be_bytes())?;
            w.write_all(&self.timescale.to_be_bytes())?;
            w.write_all(&duration_secs.to_be_bytes())?;
        } else {
            w.write_all(&(creation_timestamp as u32).to_be_bytes())?;
            w.write_all(&(modification_timestamp as u32).to_be_bytes())?;
            w.write_all(&self.timescale.to_be_bytes())?;
            w.write_all(&(duration_secs as u32).to_be_bytes())?;
        }
        w.write_all(&self.rate.to_be_bytes())?;
        w.write_all(&self.volume.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        w.write_all(&[0u32.to_be_bytes(); 2].concat())?;
        for i in self.matrix {
            for j in i {
                w.write_all(&j.to_be_bytes())?;
            }
        }
        w.write_all(&[0u32.to_be_bytes(); 6].concat())?;
        w.write_all(&self.next_track_id.to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for MovieHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        u8::from(
            self.creation_time.timestamp() as u64 > u32::MAX as u64
                || self.modification_time.timestamp() as u64 > u32::MAX as u64
                || self
                    .duration
                    .as_ref()
                    .map_or(u32::MAX as u64, |x| duration(x, self.timescale))
                    > u32::MAX as u64,
        )
    }
}

#[derive(Debug, Clone)]
pub struct TrackBox {
    pub tkhd: TrackHeaderBox,
    pub tref: Option<TrackReferenceBox>,
    pub edts: Option<EditListBox>,
    pub mdia: MediaBox,
}

impl BmffBox for TrackBox {
    const TYPE: [u8; 4] = *b"trak";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.tkhd.size()
            + self.tref.as_ref().map_or(0, BmffBox::size)
            + self.edts.as_ref().map_or(0, BmffBox::size)
            + self.mdia.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.tkhd, &mut w)?;
        if let Some(tref) = &self.tref {
            write_to(tref, &mut w)?;
        }
        if let Some(edts) = &self.edts {
            write_to_full(edts, &mut w)?;
        }
        write_to(&self.mdia, &mut w)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TrackHeaderBox {
    pub flags: TrackHeaderFlags,
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
    pub track_id: u32,
    /// must be the same as in mvhd
    pub timescale: u32,
    pub duration: Option<Duration>,
    pub layer: i16,
    pub alternate_group: i16,
    pub volume: I8F8,
    pub matrix: [[I16F16; 3]; 3],
    pub width: U16F16,
    pub height: U16F16,
}

bitflags! {
    pub struct TrackHeaderFlags: u32 {
        const TRACK_ENABLED = 0x00_0001;
        const TRACK_IN_MOVIE = 0x00_0002;
        const TRACK_IN_PREVIEW = 0x00_0004;
    }
}

impl BmffBox for TrackHeaderBox {
    const TYPE: [u8; 4] = *b"tkhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + (if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self
                .duration
                .as_ref()
                .map_or(u32::MAX as u64, |x| duration(x, self.timescale))
                > u32::MAX as u64
        {
            8 + 8 + 4 + 4 + 8
        } else {
            4 + 4 + 4 + 4 + 4
        }) + 4 * 2
            + 2
            + 2
            + 2
            + 2
            + 4 * 9
            + 4
            + 4
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        let creation_timestamp = self.creation_time.timestamp();
        let modification_timestamp = self.modification_time.timestamp();
        let duration_secs = self
            .duration
            .as_ref()
            .map_or(u32::MAX as u64, |x| duration(x, self.timescale));
        if creation_timestamp as u64 > u32::MAX as u64
            || modification_timestamp as u64 > u32::MAX as u64
            || duration_secs as u64 > u32::MAX as u64
        {
            w.write_all(&creation_timestamp.to_be_bytes())?;
            w.write_all(&modification_timestamp.to_be_bytes())?;
            w.write_all(&self.track_id.to_be_bytes())?;
            w.write_all(&0u32.to_be_bytes())?;
            w.write_all(&duration_secs.to_be_bytes())?;
        } else {
            w.write_all(&(creation_timestamp as u32).to_be_bytes())?;
            w.write_all(&(modification_timestamp as u32).to_be_bytes())?;
            w.write_all(&self.track_id.to_be_bytes())?;
            w.write_all(&0u32.to_be_bytes())?;
            w.write_all(&(duration_secs as u32).to_be_bytes())?;
        }
        w.write_all(&[0u32.to_be_bytes(); 2].concat())?;
        w.write_all(&self.layer.to_be_bytes())?;
        w.write_all(&self.alternate_group.to_be_bytes())?;
        w.write_all(&self.volume.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        for i in self.matrix {
            for j in i {
                w.write_all(&j.to_be_bytes())?;
            }
        }
        w.write_all(&self.width.to_be_bytes())?;
        w.write_all(&self.height.to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for TrackHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        u8::from(
            self.creation_time.timestamp() as u64 > u32::MAX as u64
                || self.modification_time.timestamp() as u64 > u32::MAX as u64
                || self
                    .duration
                    .as_ref()
                    .map_or(u32::MAX as u64, |x| duration(x, self.timescale))
                    > u32::MAX as u64,
        )
    }

    fn flags(&self) -> [u8; 3] {
        let flags = self.flags.bits().to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct TrackReferenceBox;

impl BmffBox for TrackReferenceBox {
    const TYPE: [u8; 4] = *b"tref";

    #[inline]
    fn size(&self) -> u64 {
        8
    }

    fn write_box(&self, _w: impl Write) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MediaBox {
    pub mdhd: MediaHeaderBox,
    pub hdlr: HandlerBox,
    pub minf: MediaInformationBox,
}

impl BmffBox for MediaBox {
    const TYPE: [u8; 4] = *b"mdia";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.mdhd.size() + self.hdlr.size() + self.minf.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.mdhd, &mut w)?;
        write_to_full(&self.hdlr, &mut w)?;
        write_to(&self.minf, &mut w)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MediaHeaderBox {
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
    pub timescale: u32,
    pub duration: Option<Duration>,
    pub language: [u8; 3],
}

impl BmffBox for MediaHeaderBox {
    const TYPE: [u8; 4] = *b"mdhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + (if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self
                .duration
                .as_ref()
                .map_or(u32::MAX as u64, |x| duration(x, self.timescale))
                > u32::MAX as u64
        {
            8 + 8 + 4 + 8
        } else {
            4 * 4
        }) + 2
            + 2
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        let creation_timestamp = self.creation_time.timestamp();
        let modification_timestamp = self.modification_time.timestamp();
        let duration_secs = self
            .duration
            .as_ref()
            .map_or(u32::MAX as u64, |x| duration(x, self.timescale));
        if creation_timestamp as u64 > u32::MAX as u64
            || modification_timestamp as u64 > u32::MAX as u64
            || duration_secs as u64 > u32::MAX as u64
        {
            w.write_all(&creation_timestamp.to_be_bytes())?;
            w.write_all(&modification_timestamp.to_be_bytes())?;
            w.write_all(&self.timescale.to_be_bytes())?;
            w.write_all(&duration_secs.to_be_bytes())?;
        } else {
            w.write_all(&(creation_timestamp as u32).to_be_bytes())?;
            w.write_all(&(modification_timestamp as u32).to_be_bytes())?;
            w.write_all(&self.timescale.to_be_bytes())?;
            w.write_all(&(duration_secs as u32).to_be_bytes())?;
        }
        // 000aaaaa 000bbbbb 000ccccc
        //    |||||   //  \\\   |||||
        //    |||||  //    \\\  |||||
        //  0 xxxxx xx      xxx xxxxx
        let language = [
            (self.language[0] - 0x60) << 2 | (self.language[1] - 0x60) >> 3,
            (self.language[1] - 0x60) << 5 | (self.language[2] - 0x60),
        ];
        w.write_all(&language)?;
        w.write_all(&0u16.to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for MediaHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        u8::from(
            self.creation_time.timestamp() as u64 > u32::MAX as u64
                || self.modification_time.timestamp() as u64 > u32::MAX as u64
                || self
                    .duration
                    .as_ref()
                    .map_or(u32::MAX as u64, |x| duration(x, self.timescale))
                    > u32::MAX as u64,
        )
    }
}

#[derive(Debug, Clone)]
pub struct HandlerBox {
    pub handler_type: HandlerType,
    // spec says a null-terminated UTF-8 string, so not a `CString`
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum HandlerType {
    Video = u32::from_be_bytes(*b"vide"),
    Audio = u32::from_be_bytes(*b"soun"),
    Hint = u32::from_be_bytes(*b"hint"),
}

impl BmffBox for HandlerBox {
    const TYPE: [u8; 4] = *b"hdlr";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + 4 + 4 * 3 + self.name.len() as u64 + 1
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&0u32.to_be_bytes())?;
        w.write_all(&(self.handler_type as u32).to_be_bytes())?;
        w.write_all(&[0u32.to_be_bytes(); 3].concat())?;
        w.write_all(self.name.as_bytes())?;
        w.write_all(&[0u8])?; // Null terminator
        Ok(())
    }
}

impl FullBox for HandlerBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct MediaInformationBox {
    pub media_header: MediaHeader,
    pub dinf: DataInformationBox,
    pub stbl: SampleTableBox,
}

impl BmffBox for MediaInformationBox {
    const TYPE: [u8; 4] = *b"minf";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.media_header.size() + self.dinf.size() + self.stbl.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        self.media_header.write_to(&mut w)?;
        write_to(&self.dinf, &mut w)?;
        write_to(&self.stbl, &mut w)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MediaHeader {
    Video(VideoMediaHeaderBox),
    Sound(SoundMediaHeaderBox),
    Hint(HintMediaHeaderBox),
    Null(NullMediaHeaderBox),
}

impl MediaHeader {
    #[inline]
    pub fn size(&self) -> u64 {
        match self {
            Self::Video(vmhd) => vmhd.size(),
            Self::Sound(smhd) => smhd.size(),
            Self::Hint(hmhd) => hmhd.size(),
            Self::Null(nmhd) => nmhd.size(),
        }
    }
}

impl WriteTo for MediaHeader {
    fn write_to(&self, w: impl Write) -> io::Result<()> {
        match self {
            Self::Video(vmhd) => write_to_full(vmhd, w),
            Self::Sound(smhd) => write_to_full(smhd, w),
            Self::Hint(hmhd) => write_to_full(hmhd, w),
            Self::Null(nmhd) => write_to_full(nmhd, w),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoMediaHeaderBox {
    pub graphics_mode: GraphicsMode,
    pub opcolor: [u16; 3],
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum GraphicsMode {
    Copy = 0,
}

impl BmffBox for VideoMediaHeaderBox {
    const TYPE: [u8; 4] = *b"vmhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 2 + 2 * 3
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&(self.graphics_mode as u16).to_be_bytes())?;
        for i in self.opcolor {
            w.write_all(&i.to_be_bytes())?;
        }
        Ok(())
    }
}

impl FullBox for VideoMediaHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }

    #[inline]
    fn flags(&self) -> [u8; 3] {
        [0, 0, 1]
    }
}

#[derive(Debug, Clone)]
pub struct SoundMediaHeaderBox {
    pub balance: I8F8,
}

impl BmffBox for SoundMediaHeaderBox {
    const TYPE: [u8; 4] = *b"smhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 2 + 2
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&self.balance.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for SoundMediaHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct HintMediaHeaderBox {
    pub max_pdu_size: u16,
    pub avg_pdu_size: u16,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
}

impl BmffBox for HintMediaHeaderBox {
    const TYPE: [u8; 4] = *b"hmhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 2 + 2 + 4 + 4 + 4 // reserved u32
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&self.max_pdu_size.to_be_bytes())?;
        w.write_all(&self.avg_pdu_size.to_be_bytes())?;
        w.write_all(&self.max_bitrate.to_be_bytes())?;
        w.write_all(&self.avg_bitrate.to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for HintMediaHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct NullMediaHeaderBox {
    pub flags: NullMediaHeaderFlags,
}

bitflags! {
    pub struct NullMediaHeaderFlags: u32 {}
}

impl BmffBox for NullMediaHeaderBox {
    const TYPE: [u8; 4] = *b"nmhd";

    #[inline]
    fn size(&self) -> u64 {
        12
    }

    fn write_box(&self, _w: impl Write) -> io::Result<()> {
        Ok(())
    }
}

impl FullBox for NullMediaHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }

    #[inline]
    fn flags(&self) -> [u8; 3] {
        let flags = self.flags.bits().to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct DataInformationBox {
    pub dref: DataReferenceBox,
}

impl BmffBox for DataInformationBox {
    const TYPE: [u8; 4] = *b"dinf";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.dref.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.dref, &mut w)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DataReferenceBox {
    pub data_entries: Vec<DataEntry>,
}

impl BmffBox for DataReferenceBox {
    const TYPE: [u8; 4] = *b"dref";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.data_entries.iter().map(DataEntry::size).sum::<u64>()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&(self.data_entries.len() as u32).to_be_bytes())?;
        for entry in &self.data_entries {
            entry.write_to(&mut w)?;
        }
        Ok(())
    }
}

impl FullBox for DataReferenceBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DataEntry {
    Url(DataEntryUrlBox),
    Urn(DataEntryUrnBox),
}

impl DataEntry {
    #[inline]
    pub fn size(&self) -> u64 {
        match self {
            Self::Url(url) => url.size(),
            Self::Urn(urn) => urn.size(),
        }
    }
}

impl WriteTo for DataEntry {
    fn write_to(&self, w: impl Write) -> io::Result<()> {
        match self {
            Self::Url(url) => write_to_full(url, w),
            Self::Urn(urn) => write_to_full(urn, w),
        }
    }
}

bitflags! {
    pub struct DataEntryFlags: u32 {
        /// Indicates that the media data is in the same file as the containing MovieBox.
        const SELF_CONTAINED = 0x00_0001;
    }
}

#[derive(Debug, Clone)]
pub struct DataEntryUrlBox {
    pub flags: DataEntryFlags,
    pub location: String,
}

impl BmffBox for DataEntryUrlBox {
    const TYPE: [u8; 4] = *b"url ";

    #[inline]
    fn size(&self) -> u64 {
        12 + if self.flags.contains(DataEntryFlags::SELF_CONTAINED) {
            0
        } else {
            self.location.len() as u64 + 1
        }
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        if !self.flags.contains(DataEntryFlags::SELF_CONTAINED) {
            w.write_all(self.location.as_bytes())?;
            w.write_all(&[0u8])?; // Null terminator
        }
        Ok(())
    }
}

impl FullBox for DataEntryUrlBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }

    #[inline]
    fn flags(&self) -> [u8; 3] {
        let flags = self.flags.bits().to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct DataEntryUrnBox {
    pub flags: DataEntryFlags,
    pub name: String,
    pub location: Option<String>,
}

impl BmffBox for DataEntryUrnBox {
    const TYPE: [u8; 4] = *b"urn ";

    #[inline]
    fn size(&self) -> u64 {
        12 + self.name.len() as u64 + 1 + self.location.as_ref().map_or(0, |x| x.len() as u64 + 1)
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(self.name.as_bytes())?;
        w.write_all(&[0u8])?; // Null terminator
        if let Some(location) = &self.location {
            w.write_all(location.as_bytes())?;
            w.write_all(&[0u8])?; // Null terminator
        }
        Ok(())
    }
}

impl FullBox for DataEntryUrnBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }

    #[inline]
    fn flags(&self) -> [u8; 3] {
        let flags = self.flags.bits().to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct SampleTableBox {
    pub stsd: SampleDescriptionBox,
    pub stts: TimeToSampleBox,
    pub stsc: SampleToChunkBox,
    pub stsz: SampleSizeBox,
    pub stco: ChunkOffsetBox,
}

impl BmffBox for SampleTableBox {
    const TYPE: [u8; 4] = *b"stbl";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.stsd.size()
            + self.stts.size()
            + self.stsc.size()
            + self.stsz.size()
            + self.stco.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.stsd, &mut w)?;
        write_to_full(&self.stts, &mut w)?;
        write_to_full(&self.stsc, &mut w)?;
        write_to_full(&self.stsz, &mut w)?;
        write_to_full(&self.stco, &mut w)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TimeToSampleBox {
    /// `(sample_count, sample_delta)`
    pub samples: Vec<(u32, u32)>,
}

impl BmffBox for TimeToSampleBox {
    const TYPE: [u8; 4] = *b"stts";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.samples.len() as u64 * 8
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&(self.samples.len() as u32).to_be_bytes())?;
        for (sample_count, sample_delta) in &self.samples {
            w.write_all(&sample_count.to_be_bytes())?;
            w.write_all(&sample_delta.to_be_bytes())?;
        }
        Ok(())
    }
}

impl FullBox for TimeToSampleBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

pub trait SampleEntry: std::fmt::Debug + Send + Sync {
    fn size(&self) -> u64;
    fn write_to(&self) -> Vec<u8>;
    fn clone_box(&self) -> Box<dyn SampleEntry>;
}

#[derive(Debug)]
pub struct SampleDescriptionBox {
    pub entries: Vec<Box<dyn SampleEntry>>,
}

impl Clone for SampleDescriptionBox {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.iter().map(|x| x.clone_box()).collect(),
        }
    }
}

impl BmffBox for SampleDescriptionBox {
    const TYPE: [u8; 4] = *b"stsd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.entries.iter().map(|x| x.size()).sum::<u64>()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&(self.entries.len() as u32).to_be_bytes())?;
        for entry in &self.entries {
            w.write_all(&entry.write_to())?;
        }
        Ok(())
    }
}

impl FullBox for SampleDescriptionBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct SampleSizeBox {
    pub sample_size: SampleSize,
}

impl BmffBox for SampleSizeBox {
    const TYPE: [u8; 4] = *b"stsz";

    #[inline]
    fn size(&self) -> u64 {
        12 + self.sample_size.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        self.sample_size.write_to(&mut w)?;
        Ok(())
    }
}

impl FullBox for SampleSizeBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SampleSize {
    Same(NonZeroU32),
    Different(Vec<u32>),
}

impl SampleSize {
    fn size(&self) -> u64 {
        match self {
            Self::Same(_) => 8,
            Self::Different(x) => 8 + x.len() as u64 * 4,
        }
    }
}

impl WriteTo for SampleSize {
    fn write_to(&self, mut w: impl Write) -> io::Result<()> {
        match self {
            Self::Same(sample_size) => {
                w.write_all(&sample_size.get().to_be_bytes())?;
                w.write_all(&0u32.to_be_bytes())?;
            }
            Self::Different(entry_sizes) => {
                w.write_all(&0u32.to_be_bytes())?;
                w.write_all(&(entry_sizes.len() as u32).to_be_bytes())?;
                for entry_size in entry_sizes.iter() {
                    w.write_all(&entry_size.to_be_bytes())?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VisualSampleEntry {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    /// DPI
    pub horiz_resolution: U16F16,
    /// DPI
    pub vert_resolution: U16F16,
    pub frame_count: u16,
    // must be less than 32 bytes
    pub compressor_name: String,
    pub depth: u16,
}

impl BmffBox for VisualSampleEntry {
    const TYPE: [u8; 4] = *b"vide";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + 2 + 2 + 4 * 3 + 2 + 2 + 4 + 4 + 4 + 2 + 32 + 2 + 2
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&[0u8; 6])?;
        w.write_all(&self.data_reference_index.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        w.write_all(&[0u32.to_be_bytes(); 3].concat())?;
        w.write_all(&self.width.to_be_bytes())?;
        w.write_all(&self.height.to_be_bytes())?;
        w.write_all(&self.horiz_resolution.to_be_bytes())?;
        w.write_all(&self.vert_resolution.to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?;
        w.write_all(&self.frame_count.to_be_bytes())?;
        assert!(self.compressor_name.len() <= 32);
        w.write_all(&[self.compressor_name.len() as u8])?;
        for _ in 0..(32 - 1 - self.compressor_name.len()) {
            w.write_all(&[0u8])?;
        }
        w.write_all(self.compressor_name.as_bytes())?;
        w.write_all(&self.depth.to_be_bytes())?;
        w.write_all(&(-1i16).to_be_bytes())?;
        Ok(())
    }
}

impl SampleEntry for VisualSampleEntry {
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }

    fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AudioSampleEntry {
    pub data_reference_index: u16,
    pub channel_count: u16,
    /// bits
    pub sample_size: u16,
    pub sample_rate: u32,
}

impl BmffBox for AudioSampleEntry {
    const TYPE: [u8; 4] = *b"soun";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + 4 * 2 + 2 + 2 + 2 + 2 + 4
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&[0u8; 6])?;
        w.write_all(&self.data_reference_index.to_be_bytes())?;
        w.write_all(&[0u32.to_be_bytes(); 2].concat())?;
        w.write_all(&self.channel_count.to_be_bytes())?;
        w.write_all(&self.sample_size.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        w.write_all(&self.sample_rate.to_be_bytes())?;
        Ok(())
    }
}

impl SampleEntry for AudioSampleEntry {
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }
    fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct HintSampleEntry {
    pub data_reference_index: u16,
    pub data: Vec<u8>,
}

impl BmffBox for HintSampleEntry {
    const TYPE: [u8; 4] = *b"hint";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + self.data.len() as u64
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&[0u8; 6])?;
        w.write_all(&self.data_reference_index.to_be_bytes())?;
        w.write_all(&self.data)?;
        Ok(())
    }
}

impl SampleEntry for HintSampleEntry {
    #[inline]
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }

    fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AvcSampleEntry {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub horiz_resolution: U16F16,
    pub vert_resolution: U16F16,
    pub frame_count: u16,
    pub depth: u16,
    pub avcc: AvcConfigurationBox,
}

impl BmffBox for AvcSampleEntry {
    const TYPE: [u8; 4] = *b"avc1";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + 2 + 2 + 4 + 4 + 4 + 2 + 2 + 4 + 4 + 4 + 2 + 32 + 2 + 2 + self.avcc.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&[0u8; 6])?;
        w.write_all(&self.data_reference_index.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        w.write_all(&0u16.to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?;
        w.write_all(&self.width.to_be_bytes())?;
        w.write_all(&self.height.to_be_bytes())?;
        w.write_all(&self.horiz_resolution.to_be_bytes())?;
        w.write_all(&self.vert_resolution.to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?;
        w.write_all(&self.frame_count.to_be_bytes())?;
        w.write_all(&[0u8; 32])?;
        w.write_all(&self.depth.to_be_bytes())?;
        w.write_all(&(-1i16).to_be_bytes())?; // pre_defined
        write_to(&self.avcc, &mut w)?;
        Ok(())
    }
}

impl SampleEntry for AvcSampleEntry {
    #[inline]
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }

    fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AvcConfigurationBox {
    pub configuration: AvcDecoderConfigurationRecord,
}

impl BmffBox for AvcConfigurationBox {
    const TYPE: [u8; 4] = *b"avcC";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.configuration.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        self.configuration.write_to(&mut w)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AvcDecoderConfigurationRecord {
    pub profile_idc: u8,
    pub constraint_set_flag: u8,
    pub level_idc: u8,
    pub sequence_parameter_set: Vec<u8>,
    pub picture_parameter_set: Vec<u8>,
}

impl AvcDecoderConfigurationRecord {
    fn size(&self) -> u64 {
        1 + 1
            + 1
            + 1
            + 1
            + 1
            + 2
            + self.sequence_parameter_set.len() as u64
            + 1
            + 2
            + self.picture_parameter_set.len() as u64
    }
}

impl WriteTo for AvcDecoderConfigurationRecord {
    fn write_to(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&1u8.to_be_bytes())?;
        w.write_all(&self.profile_idc.to_be_bytes())?;
        w.write_all(&self.constraint_set_flag.to_be_bytes())?;
        w.write_all(&self.level_idc.to_be_bytes())?;
        w.write_all(&(0xfau8 | 0x03u8).to_be_bytes())?;
        w.write_all(&(0xe0u8 | 0x01u8).to_be_bytes())?;
        w.write_all(&(self.sequence_parameter_set.len() as u16).to_be_bytes())?;
        w.write_all(&self.sequence_parameter_set)?;
        w.write_all(&0x01u8.to_be_bytes())?;
        w.write_all(&(self.picture_parameter_set.len() as u16).to_be_bytes())?;
        w.write_all(&self.picture_parameter_set)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SampleToChunkBox {
    /// `(first_chunk, samples_per_chunk, sample_description_index)`
    pub entries: Vec<(u32, u32, u32)>,
}

impl BmffBox for SampleToChunkBox {
    const TYPE: [u8; 4] = *b"stsc";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.entries.len() as u64 * 4 * 3
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&(self.entries.len() as u32).to_be_bytes())?;
        for (first_chunk, samples_per_chunk, sample_description_index) in &self.entries {
            w.write_all(&first_chunk.to_be_bytes())?;
            w.write_all(&samples_per_chunk.to_be_bytes())?;
            w.write_all(&sample_description_index.to_be_bytes())?;
        }
        Ok(())
    }
}

impl FullBox for SampleToChunkBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct ChunkOffsetBox {
    pub chunk_offsets: Vec<u32>,
}

impl BmffBox for ChunkOffsetBox {
    const TYPE: [u8; 4] = *b"stco";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.chunk_offsets.len() as u64 * 4
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&(self.chunk_offsets.len() as u32).to_be_bytes())?;
        for chunk_offset in &self.chunk_offsets {
            w.write_all(&chunk_offset.to_be_bytes())?;
        }
        Ok(())
    }
}

impl FullBox for ChunkOffsetBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct EditBox {
    pub elst: EditListBox,
}

impl BmffBox for EditBox {
    const TYPE: [u8; 4] = *b"edts";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.elst.size()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.elst, &mut w)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EditListBox {
    /// `(segment_duration, media_time)`
    pub entries: Vec<(u64, i64)>,
    pub media_rate_integer: i16,
    pub media_rate_fraction: i16,
}

impl BmffBox for EditListBox {
    const TYPE: [u8; 4] = *b"elst";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.entries.len() as u64 * 8 * 2 + 2 + 2
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&(self.entries.len() as u32).to_be_bytes())?;
        for (segment_duration, media_time) in &self.entries {
            w.write_all(&segment_duration.to_be_bytes())?;
            w.write_all(&media_time.to_be_bytes())?;
        }
        w.write_all(&self.media_rate_integer.to_be_bytes())?;
        w.write_all(&self.media_rate_fraction.to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for EditListBox {
    #[inline]
    fn version(&self) -> u8 {
        1
    }
}

#[derive(Debug, Clone)]
pub struct MovieExtendsBox {
    pub mehd: Option<MovieExtendsHeaderBox>,
    pub trex: Vec<TrackExtendsBox>,
}

impl BmffBox for MovieExtendsBox {
    const TYPE: [u8; 4] = *b"mvex";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.mehd.as_ref().map_or(0, BmffBox::size)
            + self.trex.iter().map(BmffBox::size).sum::<u64>()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        if let Some(mehd) = &self.mehd {
            write_to_full(mehd, &mut w)?;
        }
        for trex in &self.trex {
            write_to_full(trex, &mut w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MovieExtendsHeaderBox {
    pub fragment_duration: Duration,
}

impl BmffBox for MovieExtendsHeaderBox {
    const TYPE: [u8; 4] = *b"mehd";

    #[inline]
    fn size(&self) -> u64 {
        12 + if self.fragment_duration.num_seconds() as u64 > u32::MAX as u64 {
            8
        } else {
            4
        }
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        let fragment_duration = self.fragment_duration.num_seconds();
        if fragment_duration as u64 > u32::MAX as u64 {
            w.write_all(&fragment_duration.to_be_bytes())?;
        } else {
            w.write_all(&(fragment_duration as u32).to_be_bytes())?;
        }
        Ok(())
    }
}

impl FullBox for MovieExtendsHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        (self.fragment_duration.num_seconds() as u64 > u32::MAX as u64) as u8
    }
}

#[derive(Debug, Clone)]
pub struct TrackExtendsBox {
    pub track_id: u32,
    pub default_sample_description_index: u32,
    pub default_sample_duration: u32,
    pub default_sample_size: u32,
    pub default_sample_flags: DefaultSampleFlags,
}

bitflags! {
    pub struct DefaultSampleFlags: u32 {
        const SAMPLE_DEPENDS_ON = 0x0300_0000;
        const SAMPLE_IS_DEPENDED_ON = 0x00C0_0000;
        const SAMPLE_HAS_REDUNDANCY = 0x0030_0000;
        const SAMPLE_PADDING_VALUE = 0x000D_0000;
    }
}

impl BmffBox for TrackExtendsBox {
    const TYPE: [u8; 4] = *b"trex";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + 4 + 4 + 4 + 4
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&self.track_id.to_be_bytes())?;
        w.write_all(&self.default_sample_description_index.to_be_bytes())?;
        w.write_all(&self.default_sample_duration.to_be_bytes())?;
        w.write_all(&self.default_sample_size.to_be_bytes())?;
        w.write_all(&self.default_sample_flags.bits().to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for TrackExtendsBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct MediaDataBox {
    pub data: Vec<u8>,
}

impl BmffBox for MediaDataBox {
    const TYPE: [u8; 4] = *b"mdat";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.data.len() as u64
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&self.data)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MovieFragmentBox {
    pub mfhd: MovieFragmentHeaderBox,
    pub traf: Vec<TrackFragmentBox>,
}

impl BmffBox for MovieFragmentBox {
    const TYPE: [u8; 4] = *b"moof";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.mfhd.size() + self.traf.iter().map(BmffBox::size).sum::<u64>()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.mfhd, &mut w)?;
        for traf in &self.traf {
            write_to(traf, &mut w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MovieFragmentHeaderBox {
    pub sequence_number: u32,
}

impl BmffBox for MovieFragmentHeaderBox {
    const TYPE: [u8; 4] = *b"mfhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&self.sequence_number.to_be_bytes())?;
        Ok(())
    }
}

impl FullBox for MovieFragmentHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }
}

#[derive(Debug, Clone)]
pub struct TrackFragmentBox {
    pub tfhd: TrackFragmentHeaderBox,
    pub trun: Vec<TrackFragmentRunBox>,
    // pub sdtp: (),
    // pub sbgp: (),
    // pub subs: (),
}

impl BmffBox for TrackFragmentBox {
    const TYPE: [u8; 4] = *b"traf";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.tfhd.size() + self.trun.iter().map(BmffBox::size).sum::<u64>()
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        write_to_full(&self.tfhd, &mut w)?;
        for trun in &self.trun {
            write_to_full(trun, &mut w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TrackFragmentHeaderBox {
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<DefaultSampleFlags>,
}

bitflags! {
    struct TrackFragmentHeaderFlags: u32 {
        const BASE_DATA_OFFSET_PRESENT = 0x00_0001;
        const SAMPLE_DESCRIPTION_INDEX_PRESENT = 0x00_0002;
        const DEFAULT_SAMPLE_DURATION_PRESENT = 0x00_0008;
        const DEFAULT_SAMPLE_SIZE_PRESENT = 0x00_0010;
        const DEFAULT_SAMPLE_FLAGS_PRESENT = 0x00_0020;
        const DURATION_IS_EMPTY = 0x01_0000;
    }
}

impl BmffBox for TrackFragmentHeaderBox {
    const TYPE: [u8; 4] = *b"tfhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4
            + self.base_data_offset.as_ref().map_or(0, |_| 8)
            + self.sample_description_index.as_ref().map_or(0, |_| 4)
            + self.default_sample_duration.as_ref().map_or(0, |_| 4)
            + self.default_sample_size.as_ref().map_or(0, |_| 4)
            + self.default_sample_flags.as_ref().map_or(0, |_| 4)
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(&self.track_id.to_be_bytes())?;
        if let Some(base_data_offset) = self.base_data_offset {
            w.write_all(&base_data_offset.to_be_bytes())?;
        }
        if let Some(sample_description_index) = self.sample_description_index {
            w.write_all(&sample_description_index.to_be_bytes())?;
        }
        if let Some(default_sample_duration) = self.default_sample_duration {
            w.write_all(&default_sample_duration.to_be_bytes())?;
        }
        if let Some(default_sample_size) = self.default_sample_size {
            w.write_all(&default_sample_size.to_be_bytes())?;
        }
        if let Some(default_sample_flags) = self.default_sample_flags {
            w.write_all(&default_sample_flags.bits().to_be_bytes())?;
        }
        Ok(())
    }
}

impl FullBox for TrackFragmentHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }

    #[inline]
    fn flags(&self) -> [u8; 3] {
        let mut flags = TrackFragmentHeaderFlags::empty();
        if self.base_data_offset.is_some() {
            flags |= TrackFragmentHeaderFlags::BASE_DATA_OFFSET_PRESENT;
        }
        if self.sample_description_index.is_some() {
            flags |= TrackFragmentHeaderFlags::SAMPLE_DESCRIPTION_INDEX_PRESENT;
        }
        if self.default_sample_duration.is_some() {
            flags |= TrackFragmentHeaderFlags::DEFAULT_SAMPLE_DURATION_PRESENT;
        }
        if self.default_sample_size.is_some() {
            flags |= TrackFragmentHeaderFlags::DEFAULT_SAMPLE_SIZE_PRESENT;
        }
        if self.default_sample_flags.is_some() {
            flags |= TrackFragmentHeaderFlags::DEFAULT_SAMPLE_FLAGS_PRESENT;
        }
        let flags = flags.bits().to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct TrackFragmentRunBox {
    pub data_offset: Option<i32>,
    pub first_sample_flags: Option<u32>,
    pub sample_durations: Option<Vec<u32>>,
    pub sample_sizes: Option<Vec<u32>>,
    pub sample_flags: Option<Vec<u32>>,
    pub sample_composition_time_offsets: Option<Vec<u32>>,
}

bitflags! {
    struct TrackFragmentRunFlags: u32 {
        const DATA_OFFSET_PRESENT = 0x00_0001;
        const FIRST_SAMPLE_FLAGS_PRESENT = 0x00_0004;
        const SAMPLE_DURATION_PRESENT = 0x00_0100;
        const SAMPLE_SIZE_PRESENT = 0x00_0200;
        const SAMPLE_FLAGS_PRESENT = 0x00_0400;
        const SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT = 0x00_0800;
    }
}

impl TrackFragmentRunBox {
    fn len(&self) -> Option<usize> {
        let mut len = None;
        if let Some(sample_durations) = &self.sample_durations {
            len = Some(sample_durations.len());
        }
        if let Some(sample_sizes) = &self.sample_sizes {
            let l = sample_sizes.len();
            if let Some(len) = len {
                if len != l {
                    panic!();
                }
            } else {
                len = Some(l);
            }
        }
        if let Some(sample_flags) = &self.sample_flags {
            let l = sample_flags.len();
            if let Some(len) = len {
                if len != l {
                    panic!();
                }
            } else {
                len = Some(l);
            }
        }
        if let Some(sample_composition_time_offsets) = &self.sample_composition_time_offsets {
            let l = sample_composition_time_offsets.len();
            if let Some(len) = len {
                if len != l {
                    panic!();
                }
            } else {
                len = Some(l);
            }
        }
        len
    }
}

impl BmffBox for TrackFragmentRunBox {
    const TYPE: [u8; 4] = *b"trun";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4
            + self.data_offset.as_ref().map_or(0, |_| 4)
            + self.first_sample_flags.as_ref().map_or(0, |_| 4)
            + self.len().unwrap_or(0) as u64
                * 4
                * (self.sample_durations.is_some() as u64
                    + self.sample_sizes.is_some() as u64
                    + self.sample_flags.is_some() as u64
                    + self.sample_composition_time_offsets.is_some() as u64)
    }

    fn write_box(&self, mut w: impl Write) -> io::Result<()> {
        let len = self.len().unwrap_or(0);
        w.write_all(&(len as u32).to_be_bytes())?;
        if let Some(data_offset) = self.data_offset {
            w.write_all(&data_offset.to_be_bytes())?;
        }
        if let Some(first_sample_flags) = self.first_sample_flags {
            w.write_all(&first_sample_flags.to_be_bytes())?;
        }
        for i in 0..len {
            if let Some(sample_durations) = &self.sample_durations {
                w.write_all(&sample_durations[i].to_be_bytes())?;
            }
            if let Some(sample_sizes) = &self.sample_sizes {
                w.write_all(&sample_sizes[i].to_be_bytes())?;
            }
            if let Some(sample_flags) = &self.sample_flags {
                w.write_all(&sample_flags[i].to_be_bytes())?;
            }
            if let Some(sample_composition_time_offsets) = &self.sample_composition_time_offsets {
                w.write_all(&sample_composition_time_offsets[i].to_be_bytes())?;
            }
        }
        Ok(())
    }
}

impl FullBox for TrackFragmentRunBox {
    #[inline]
    fn version(&self) -> u8 {
        0
    }

    #[inline]
    fn flags(&self) -> [u8; 3] {
        let mut flags = TrackFragmentRunFlags::empty();

        flags.set(
            TrackFragmentRunFlags::DATA_OFFSET_PRESENT,
            self.data_offset.is_some(),
        );
        flags.set(
            TrackFragmentRunFlags::FIRST_SAMPLE_FLAGS_PRESENT,
            self.first_sample_flags.is_some(),
        );
        flags.set(
            TrackFragmentRunFlags::SAMPLE_DURATION_PRESENT,
            self.sample_durations.is_some(),
        );
        flags.set(
            TrackFragmentRunFlags::SAMPLE_SIZE_PRESENT,
            self.sample_sizes.is_some(),
        );
        flags.set(
            TrackFragmentRunFlags::SAMPLE_FLAGS_PRESENT,
            self.sample_flags.is_some(),
        );
        flags.set(
            TrackFragmentRunFlags::SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT,
            self.sample_composition_time_offsets.is_some(),
        );

        let flags = flags.bits().to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}
