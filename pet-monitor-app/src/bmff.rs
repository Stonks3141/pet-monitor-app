use bitflags::bitflags;
use chrono::{DateTime, Duration, Utc};
use fixed::types::{U16F16, I8F8, I16F16};
use rocket::async_trait;
use rocket::futures::io::{self, AsyncWrite, AsyncWriteExt};

pub const MATRIX_0: [[I16F16; 3]; 3] = [
    [I16F16::from_bits(0x00010000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(0x00000000), I16F16::from_bits(0x00010000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x40000000)],
];

pub const MATRIX_90: [[I16F16; 3]; 3] = [
    [I16F16::from_bits(0x00000000), I16F16::from_bits(0x00010000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(-0x40000000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x40000000)],
];

pub const MATRIX_180: [[I16F16; 3]; 3] = [
    [I16F16::from_bits(-0x40000000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(0x00000000), I16F16::from_bits(-0x40000000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x40000000)],
];

pub const MATRIX_270: [[I16F16; 3]; 3] = [
    [I16F16::from_bits(0x00000000), I16F16::from_bits(-0x40000000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(0x00010000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000)],
    [I16F16::from_bits(0x00000000), I16F16::from_bits(0x00000000), I16F16::from_bits(0x40000000)],
];

pub const fn matrix(degrees: u32) -> [[I16F16; 3]; 3] {
    match degrees {
        0 => MATRIX_0,
        90 => MATRIX_90,
        180 => MATRIX_180,
        270 => MATRIX_270,
        _ => panic!("`degrees` must be 0, 90, 180, or, 270"),
    }
}

#[async_trait]
pub trait BmffBox {
    const TYPE: [u8; 4];
    const EXTENDED_TYPE: Option<[u8; 16]> = None;
    fn size(&self) -> u64;
    async fn write_box<W>(&self, writer: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send;
}

pub trait FullBox: BmffBox {
    fn version(&self) -> u8;
    #[inline]
    fn flags(&self) -> [u8; 3] {
        [0; 3]
    }
}

#[async_trait]
pub trait WriteTo {
    async fn write_to<W>(&self, writer: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send;
}

async fn write_to<T, W>(bmff_box: &T, mut w: W) -> io::Result<()>
where
    T: BmffBox,
    W: AsyncWrite + Unpin + Send,
{
    let size = bmff_box.size();
    if size <= u32::MAX as u64 {
        w.write_all(&(size as u32).to_be_bytes()).await?;
        w.write_all(&T::TYPE).await?;
    } else {
        w.write_all(&1u32.to_be_bytes()).await?;
        w.write_all(&T::TYPE).await?;
        w.write_all(&(size + 8).to_be_bytes()).await?;
    }
    if let Some(ext_type) = T::EXTENDED_TYPE {
        w.write_all(&ext_type).await?;
    }
    bmff_box.write_box(&mut w).await?;
    Ok(())
}

async fn write_to_full<T, W>(bmff_box: &T, mut w: W) -> io::Result<()>
where
    T: FullBox,
    W: AsyncWrite + Unpin + Send,
{
    let size = bmff_box.size();
    if size <= u32::MAX as u64 {
        w.write_all(&(size as u32).to_be_bytes()).await?;
        w.write_all(&T::TYPE).await?;
    } else {
        w.write_all(&1u32.to_be_bytes()).await?;
        w.write_all(&T::TYPE).await?;
        w.write_all(&(size + 8).to_be_bytes()).await?;
    }
    if let Some(ext_type) = T::EXTENDED_TYPE {
        w.write_all(&ext_type).await?;
    }
    w.write_all(&[bmff_box.version()]).await?;
    w.write_all(&bmff_box.flags()).await?;
    bmff_box.write_box(&mut w).await?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct FileTypeBox {
    pub major_brand: [u8; 4],
    pub minor_version: u32,
    pub compatible_brands: Vec<[u8; 4]>,
}

#[async_trait]
impl BmffBox for FileTypeBox {
    const TYPE: [u8; 4] = *b"ftyp";

    #[inline]
    fn size(&self) -> u64 {
        8 + 4 + 4 + self.compatible_brands.len() as u64 * 4
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&self.major_brand).await?;
        w.write_all(&self.minor_version.to_be_bytes()).await?;
        for i in self.compatible_brands.iter() {
            w.write_all(i).await?;
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

#[async_trait]
impl BmffBox for MovieBox {
    const TYPE: [u8; 4] = *b"moov";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.mvhd.size()
            + self.trak.iter().map(|x| x.size()).sum::<u64>()
            + self.mvex.as_ref().map(|x| x.size()).unwrap_or(0)
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        write_to_full(&self.mvhd, &mut w).await?;
        for trak in self.trak.iter() {
            write_to(trak, &mut w).await?;
        }
        if let Some(mvex) = &self.mvex {
            write_to(mvex, &mut w).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MovieHeaderBox {
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
    pub timescale: u32,
    pub duration: Duration,
    pub rate: I16F16,
    pub volume: I8F8,
    pub matrix: [[I16F16; 3]; 3],
    pub next_track_id: u32,
}

#[async_trait]
impl BmffBox for MovieHeaderBox {
    const TYPE: [u8; 4] = *b"mvhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + (if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self.duration.num_seconds() as u64 > u32::MAX as u64
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

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let creation_timestamp = self.creation_time.timestamp();
        let modification_timestamp = self.modification_time.timestamp();
        let duration_secs = self.duration.num_seconds();
        if creation_timestamp as u64 > u32::MAX as u64
            || modification_timestamp as u64 > u32::MAX as u64
            || duration_secs as u64 > u32::MAX as u64
        {
            w.write_all(&(creation_timestamp as u64).to_be_bytes())
                .await?;
            w.write_all(&(modification_timestamp as u64).to_be_bytes())
                .await?;
            w.write_all(&self.timescale.to_be_bytes()).await?;
            w.write_all(&(duration_secs as u64).to_be_bytes()).await?;
        } else {
            w.write_all(&(creation_timestamp as u32).to_be_bytes())
                .await?;
            w.write_all(&(modification_timestamp as u32).to_be_bytes())
                .await?;
            w.write_all(&self.timescale.to_be_bytes()).await?;
            w.write_all(&(duration_secs as u32).to_be_bytes()).await?;
        }
        w.write_all(&self.rate.to_be_bytes()).await?;
        w.write_all(&self.volume.to_be_bytes()).await?;
        w.write_all(&0u16.to_be_bytes()).await?;
        w.write_all(&[0u32.to_be_bytes(); 2].concat())
            .await?;
        for i in self.matrix {
            for j in i {
                w.write_all(&j.to_be_bytes()).await?;
            }
        }
        w.write_all(&[0u32.to_be_bytes(); 6].concat())
        .await?;
        w.write_all(&self.next_track_id.to_be_bytes()).await?;
        Ok(())
    }
}

impl FullBox for MovieHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self.duration.num_seconds() as u64 > u32::MAX as u64
        {
            1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackBox {
    pub tkhd: TrackHeaderBox,
    pub tref: Option<TrackReferenceBox>,
    pub edts: Option<EditListBox>,
    pub mdia: MediaBox,
}

#[async_trait]
impl BmffBox for TrackBox {
    const TYPE: [u8; 4] = *b"trak";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.tkhd.size()
            + self.tref.as_ref().map(|x| x.size()).unwrap_or(0)
            + self.edts.as_ref().map(|x| x.size()).unwrap_or(0)
            + self.mdia.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        write_to_full(&self.tkhd, &mut w).await?;
        if let Some(tref) = &self.tref {
            write_to(tref, &mut w).await?;
        }
        if let Some(edts) = &self.edts {
            write_to_full(edts, &mut w).await?;
        }
        write_to(&self.mdia, &mut w).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TrackHeaderBox {
    pub flags: TrackHeaderFlags,
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
    pub track_id: u32,
    pub duration: Duration,
    pub layer: i16,
    pub alternate_group: i16,
    pub volume: I8F8,
    pub matrix: [[I16F16; 3]; 3],
    pub width: u32,
    pub height: u32,
}

bitflags! {
    pub struct TrackHeaderFlags: u32 {
        const TRACK_ENABLED = 0x000001;
        const TRACK_IN_MOVIE = 0x000002;
        const TRACK_IN_PREVIEW = 0x000004;
    }
}

#[async_trait]
impl BmffBox for TrackHeaderBox {
    const TYPE: [u8; 4] = *b"tkhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + (if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self.duration.num_seconds() as u64 > u32::MAX as u64
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

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let creation_timestamp = self.creation_time.timestamp();
        let modification_timestamp = self.modification_time.timestamp();
        let duration_secs = self.duration.num_seconds();
        if creation_timestamp as u64 > u32::MAX as u64
            || modification_timestamp as u64 > u32::MAX as u64
            || duration_secs as u64 > u32::MAX as u64
        {
            w.write_all(&(creation_timestamp as u64).to_be_bytes())
                .await?;
            w.write_all(&(modification_timestamp as u64).to_be_bytes())
                .await?;
            w.write_all(&self.track_id.to_be_bytes()).await?;
            w.write_all(&0u32.to_be_bytes()).await?;
            w.write_all(&(duration_secs as u64).to_be_bytes()).await?;
        } else {
            w.write_all(&(creation_timestamp as u32).to_be_bytes())
                .await?;
            w.write_all(&(modification_timestamp as u32).to_be_bytes())
                .await?;
            w.write_all(&self.track_id.to_be_bytes()).await?;
            w.write_all(&0u32.to_be_bytes()).await?;
            w.write_all(&(duration_secs as u32).to_be_bytes()).await?;
        }
        w.write_all(&[0u32.to_be_bytes(); 2].concat())
            .await?;
        w.write_all(&self.layer.to_be_bytes()).await?;
        w.write_all(&self.alternate_group.to_be_bytes()).await?;
        w.write_all(&self.volume.to_be_bytes()).await?;
        w.write_all(&0u16.to_be_bytes()).await?;
        for i in self.matrix {
            for j in i {
                w.write_all(&j.to_be_bytes()).await?;
            }
        }
        w.write_all(&self.width.to_be_bytes()).await?;
        w.write_all(&self.height.to_be_bytes()).await?;
        Ok(())
    }
}

impl FullBox for TrackHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self.duration.num_seconds() as u64 > u32::MAX as u64
        {
            1
        } else {
            0
        }
    }

    fn flags(&self) -> [u8; 3] {
        let flags = self.flags.bits.to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct TrackReferenceBox;

#[async_trait]
impl BmffBox for TrackReferenceBox {
    const TYPE: [u8; 4] = *b"tref";

    #[inline]
    fn size(&self) -> u64 {
        8
    }

    async fn write_box<W>(&self, _w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MediaBox {
    pub mdhd: MediaHeaderBox,
    pub hdlr: HandlerBox,
    pub minf: MediaInformationBox,
}

#[async_trait]
impl BmffBox for MediaBox {
    const TYPE: [u8; 4] = *b"mdia";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.mdhd.size() + self.hdlr.size() + self.minf.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        write_to_full(&self.mdhd, &mut w).await?;
        write_to_full(&self.hdlr, &mut w).await?;
        write_to(&self.minf, &mut w).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MediaHeaderBox {
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
    pub timescale: u32,
    pub duration: Duration,
    pub language: [u8; 3],
}

#[async_trait]
impl BmffBox for MediaHeaderBox {
    const TYPE: [u8; 4] = *b"mdhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + (if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self.duration.num_seconds() as u64 > u32::MAX as u64
        {
            8 + 8 + 4 + 8
        } else {
            4 * 4
        }) + 2 + 2
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let creation_timestamp = self.creation_time.timestamp();
        let modification_timestamp = self.modification_time.timestamp();
        let duration_secs = self.duration.num_seconds();
        if creation_timestamp as u64 > u32::MAX as u64
            || modification_timestamp as u64 > u32::MAX as u64
            || duration_secs as u64 > u32::MAX as u64
        {
            w.write_all(&creation_timestamp.to_be_bytes())
                .await?;
            w.write_all(&modification_timestamp.to_be_bytes())
                .await?;
            w.write_all(&self.timescale.to_be_bytes()).await?;
            w.write_all(&duration_secs.to_be_bytes()).await?;
        } else {
            w.write_all(&(creation_timestamp as u32).to_be_bytes())
                .await?;
            w.write_all(&(modification_timestamp as u32).to_be_bytes())
                .await?;
            w.write_all(&self.timescale.to_be_bytes()).await?;
            w.write_all(&(duration_secs as u32).to_be_bytes()).await?;
        }
        // 000aaaaa 000bbbbb 000ccccc
        //    |||||   //  \\\   |||||
        //    |||||  //    \\\  |||||
        //  0 xxxxx xx      xxx xxxxx
        let language = [
            (self.language[0] - 0x60) << 2 | (self.language[1] - 0x60) >> 3,
            (self.language[1] - 0x60) << 5 | (self.language[2] - 0x60),
        ];
        w.write_all(&language).await?;
        w.write_all(&0u16.to_be_bytes()).await?;
        Ok(())
    }
}

impl FullBox for MediaHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        if self.creation_time.timestamp() as u64 > u32::MAX as u64
            || self.modification_time.timestamp() as u64 > u32::MAX as u64
            || self.duration.num_seconds() as u64 > u32::MAX as u64
        {
            1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
pub struct HandlerBox {
    pub handler_type: HandlerType,
    // spec says a null-terminated UTF-8 string, so not a `CString`
    pub name: String,
}

#[derive(Debug, Clone)]
#[repr(u32)]
pub enum HandlerType {
    Video = u32::from_be_bytes(*b"vide"),
    Audio = u32::from_be_bytes(*b"soun"),
    Hint = u32::from_be_bytes(*b"hint"),
}

#[async_trait]
impl BmffBox for HandlerBox {
    const TYPE: [u8; 4] = *b"hdlr";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + 4 + 4 * 3 + self.name.len() as u64 + 1
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&0u32.to_be_bytes()).await?;
        w.write_all(&(self.handler_type as u32).to_be_bytes())
            .await?;
        w.write_all(&[0u32.to_be_bytes(); 3].concat())
            .await?;
        w.write_all(self.name.as_bytes()).await?;
        w.write_all(&[0u8]).await?; // Null terminator
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

#[async_trait]
impl BmffBox for MediaInformationBox {
    const TYPE: [u8; 4] = *b"minf";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.media_header.size() + self.dinf.size() + self.stbl.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        self.media_header.write_to(&mut w).await?;
        write_to(&self.dinf, &mut w).await?;
        write_to(&self.stbl, &mut w).await?;
        Ok(())
    }
}

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

#[async_trait]
impl WriteTo for MediaHeader {
    async fn write_to<W>(&self, w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        match self {
            Self::Video(vmhd) => write_to_full(vmhd, w).await,
            Self::Sound(smhd) => write_to_full(smhd, w).await,
            Self::Hint(hmhd) => write_to_full(hmhd, w).await,
            Self::Null(nmhd) => write_to_full(nmhd, w).await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoMediaHeaderBox {
    pub graphics_mode: GraphicsMode,
    pub opcolor: [u16; 3],
}

#[derive(Debug, Clone)]
#[repr(u16)]
pub enum GraphicsMode {
    Copy = 0,
}

#[async_trait]
impl BmffBox for VideoMediaHeaderBox {
    const TYPE: [u8; 4] = *b"vmhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 2 + 2 * 3
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&(self.graphics_mode as u16).to_be_bytes())
            .await?;
        for i in self.opcolor {
            w.write_all(&i.to_be_bytes()).await?;
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

#[async_trait]
impl BmffBox for SoundMediaHeaderBox {
    const TYPE: [u8; 4] = *b"smhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 2 + 2
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&self.balance.to_be_bytes()).await?;
        w.write_all(&0u16.to_be_bytes()).await?;
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

#[async_trait]
impl BmffBox for HintMediaHeaderBox {
    const TYPE: [u8; 4] = *b"hmhd";

    #[inline]
    fn size(&self) -> u64 {
        12 + 2 + 2 + 4 + 4 + 4 // reserved u32
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&self.max_pdu_size.to_be_bytes()).await?;
        w.write_all(&self.avg_pdu_size.to_be_bytes()).await?;
        w.write_all(&self.max_bitrate.to_be_bytes()).await?;
        w.write_all(&self.avg_bitrate.to_be_bytes()).await?;
        w.write_all(&0u32.to_be_bytes()).await?;
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

#[async_trait]
impl BmffBox for NullMediaHeaderBox {
    const TYPE: [u8; 4] = *b"nmhd";

    #[inline]
    fn size(&self) -> u64 {
        12
    }

    async fn write_box<W>(&self, _w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
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
        let flags = self.flags.bits.to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct DataInformationBox {
    pub dref: DataReferenceBox,
}

#[async_trait]
impl BmffBox for DataInformationBox {
    const TYPE: [u8; 4] = *b"dinf";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.dref.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        write_to_full(&self.dref, &mut w).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DataReferenceBox {
    pub data_entries: Vec<DataEntry>,
}

#[async_trait]
impl BmffBox for DataReferenceBox {
    const TYPE: [u8; 4] = *b"dref";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.data_entries.iter().map(|x| x.size()).sum::<u64>()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&(self.data_entries.len() as u32).to_be_bytes())
            .await?;
        for entry in self.data_entries.iter() {
            entry.write_to(&mut w).await?;
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

#[async_trait]
impl WriteTo for DataEntry {
    async fn write_to<W>(&self, w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        match self {
            Self::Url(url) => write_to_full(url, w).await,
            Self::Urn(urn) => write_to_full(urn, w).await,
        }
    }
}

bitflags! {
    pub struct DataEntryFlags: u32 {
        /// media data in same file as containing MovieBox
        const SELF_CONTAINED = 0x000001;
    }
}

#[derive(Debug, Clone)]
pub struct DataEntryUrlBox {
    pub flags: DataEntryFlags,
    pub location: String,
}

#[async_trait]
impl BmffBox for DataEntryUrlBox {
    const TYPE: [u8; 4] = *b"url ";

    #[inline]
    fn size(&self) -> u64 {
        12 + self.location.len() as u64 + 1
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        if self.flags.contains(DataEntryFlags::SELF_CONTAINED) {
            w.write_all(self.location.as_bytes()).await?;
            w.write_all(&[0u8]).await?; // Null terminator
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
        let flags = self.flags.bits.to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct DataEntryUrnBox {
    pub flags: DataEntryFlags,
    pub name: String,
    pub location: String,
}

#[async_trait]
impl BmffBox for DataEntryUrnBox {
    const TYPE: [u8; 4] = *b"urn ";

    #[inline]
    fn size(&self) -> u64 {
        12 + self.name.len() as u64 + 1 + self.location.len() as u64 + 1
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(self.name.as_bytes()).await?;
        w.write_all(&[0u8]).await?; // Null terminator
        w.write_all(self.location.as_bytes()).await?;
        w.write_all(&[0u8]).await?; // Null terminator
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
        let flags = self.flags.bits.to_be_bytes();
        [flags[1], flags[2], flags[3]]
    }
}

#[derive(Debug, Clone)]
pub struct SampleTableBox {
    pub stsd: SampleDescriptionBox,
    pub stts: TimeToSampleBox,
    pub stsc: SampleToChunkBox,
    pub stco: ChunkOffsetBox,
}

#[async_trait]
impl BmffBox for SampleTableBox {
    const TYPE: [u8; 4] = *b"stbl";

    #[inline]
    fn size(&self) -> u64 {
        12 + self.stsd.size() + self.stts.size() + self.stsc.size() + self.stco.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        write_to(&self.stsd, &mut w).await?;
        write_to_full(&self.stts, &mut w).await?;
        write_to_full(&self.stsc, &mut w).await?;
        write_to_full(&self.stco, &mut w).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TimeToSampleBox {
    /// `(sample_count, sample_delta)`
    pub samples: Vec<(u32, u32)>,
}

#[async_trait]
impl BmffBox for TimeToSampleBox {
    const TYPE: [u8; 4] = *b"stts";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.samples.len() as u64 * 8
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&(self.samples.len() as u32).to_be_bytes())
            .await?;
        for (sample_count, sample_delta) in self.samples.iter() {
            w.write_all(&sample_count.to_be_bytes()).await?;
            w.write_all(&sample_delta.to_be_bytes()).await?;
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

#[async_trait]
pub trait SampleEntry: std::fmt::Debug {
    fn size(&self) -> u64;
    async fn write_to(&self) -> Vec<u8>;
    fn clone_box(&self) -> Box<dyn SampleEntry + Send + Sync>;
}

#[derive(Debug)]
pub struct SampleDescriptionBox {
    pub entries: Vec<Box<dyn SampleEntry + Send + Sync>>,
}

impl Clone for SampleDescriptionBox {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.iter().map(|x| x.clone_box()).collect(),
        }
    }
}

#[async_trait]
impl BmffBox for SampleDescriptionBox {
    const TYPE: [u8; 4] = *b"stsd";

    #[inline]
    fn size(&self) -> u64 {
        8 + 4 + self.entries.iter().map(|x| x.size()).sum::<u64>()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&(self.entries.len() as u32).to_be_bytes())
            .await?;
        for entry in self.entries.iter() {
            w.write_all(&entry.write_to().await).await?;
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

#[async_trait]
impl BmffBox for VisualSampleEntry {
    const TYPE: [u8; 4] = *b"vide";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + 2 + 2 + 4 * 3 + 2 + 2 + 4 + 4 + 4 + 2 + 32 + 2 + 2
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&[0u8; 6]).await?;
        w.write_all(&self.data_reference_index.to_be_bytes())
            .await?;
        w.write_all(&0u16.to_be_bytes()).await?;
        w.write_all(&0u16.to_be_bytes()).await?;
        w.write_all(&[0u32.to_be_bytes(); 3].concat()).await?;
        w.write_all(&self.width.to_be_bytes()).await?;
        w.write_all(&self.height.to_be_bytes()).await?;
        w.write_all(&self.horiz_resolution.to_be_bytes()).await?;
        w.write_all(&self.vert_resolution.to_be_bytes()).await?;
        w.write_all(&0u32.to_be_bytes()).await?;
        w.write_all(&self.frame_count.to_be_bytes()).await?;
        assert!(self.compressor_name.len() <= 32);
        w.write_all(&[self.compressor_name.len() as u8]).await?;
        for _ in 0..(32 - 1 - self.compressor_name.len()) {
            w.write_all(&[0u8]).await?;
        }
        w.write_all(self.compressor_name.as_bytes()).await?;
        w.write_all(&self.depth.to_be_bytes()).await?;
        w.write_all(&(-1i16).to_be_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl SampleEntry for VisualSampleEntry {
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }

    async fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).await.unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry + Send + Sync> {
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

#[async_trait]
impl BmffBox for AudioSampleEntry {
    const TYPE: [u8; 4] = *b"soun";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + 4 * 2 + 2 + 2 + 2 + 2 + 4
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&[0u8; 6]).await?;
        w.write_all(&self.data_reference_index.to_be_bytes())
            .await?;
        w.write_all(&[0u32.to_be_bytes(); 2].concat()).await?;
        w.write_all(&self.channel_count.to_be_bytes()).await?;
        w.write_all(&self.sample_size.to_be_bytes()).await?;
        w.write_all(&0u16.to_be_bytes()).await?;
        w.write_all(&0u16.to_be_bytes()).await?;
        w.write_all(&self.sample_rate.to_be_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl SampleEntry for AudioSampleEntry {
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }
    async fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).await.unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry + Send + Sync> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct HintSampleEntry {
    pub data_reference_index: u16,
    pub data: Vec<u8>,
}

#[async_trait]
impl BmffBox for HintSampleEntry {
    const TYPE: [u8; 4] = *b"hint";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + self.data.len() as u64
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&[0u8; 6]).await?;
        w.write_all(&self.data_reference_index.to_be_bytes())
            .await?;
        w.write_all(&self.data).await?;
        Ok(())
    }
}

#[async_trait]
impl SampleEntry for HintSampleEntry {
    #[inline]
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }

    async fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).await.unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry + Send + Sync> {
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

#[async_trait]
impl BmffBox for AvcSampleEntry {
    const TYPE: [u8; 4] = *b"avc1";

    #[inline]
    fn size(&self) -> u64 {
        8 + 6 + 2 + 2 + 2 + 4 + 4 + 4 + 2 + 32 + 2 + 2 + self.avcc.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&[0u8; 6]).await?;
        w.write_all(&self.data_reference_index.to_be_bytes())
            .await?;
        w.write_all(&self.width.to_be_bytes()).await?;
        w.write_all(&self.height.to_be_bytes()).await?;
        w.write_all(&self.horiz_resolution.to_be_bytes()).await?;
        w.write_all(&self.vert_resolution.to_be_bytes()).await?;
        w.write_all(&0u32.to_be_bytes()).await?;
        w.write_all(&self.frame_count.to_be_bytes()).await?;
        w.write_all(&[0u8; 32]).await?;
        w.write_all(&self.depth.to_be_bytes()).await?;
        w.write_all(&u16::MAX.to_be_bytes()).await?; // pre_defined
        write_to(&self.avcc, &mut w).await?;
        Ok(())
    }
}

#[async_trait]
impl SampleEntry for AvcSampleEntry {
    #[inline]
    fn size(&self) -> u64 {
        <Self as BmffBox>::size(self)
    }

    async fn write_to(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(<Self as BmffBox>::size(self) as usize);
        #[allow(clippy::unwrap_used)] // writing into a `Vec` is infallible
        write_to(self, &mut buf).await.unwrap();
        buf
    }

    fn clone_box(&self) -> Box<dyn SampleEntry + Send + Sync> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AvcConfigurationBox {
    pub configuration: AvcDecoderConfigurationRecord,
}

#[async_trait]
impl BmffBox for AvcConfigurationBox {
    const TYPE: [u8; 4] = *b"avcC";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.configuration.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        self.configuration.write_to(&mut w).await?;
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
        1 + 1 + 1 + 1 + 1 + 1
            + 2 + self.sequence_parameter_set.len() as u64
            + 1 + 2 + self.picture_parameter_set.len() as u64
    }
}

#[async_trait]
impl WriteTo for AvcDecoderConfigurationRecord {
    async fn write_to<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&1u8.to_be_bytes()).await?;
        w.write_all(&self.profile_idc.to_be_bytes()).await?;
        w.write_all(&self.constraint_set_flag.to_be_bytes()).await?;
        w.write_all(&self.level_idc.to_be_bytes()).await?;
        w.write_all(&(0xfau8 | 0x03u8).to_be_bytes()).await?;
        w.write_all(&(0xe0u8 | 0x01u8).to_be_bytes()).await?;
        w.write_all(&(self.sequence_parameter_set.len() as u16).to_be_bytes())
            .await?;
        w.write_all(&self.sequence_parameter_set).await?;
        w.write_all(&0x01u8.to_be_bytes()).await?;
        w.write_all(&(self.picture_parameter_set.len() as u16).to_be_bytes())
            .await?;
        w.write_all(&self.picture_parameter_set).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SampleToChunkBox {
    /// `(first_chunk, samples_per_chunk, sample_description_index)`
    pub entries: Vec<(u32, u32, u32)>,
}

#[async_trait]
impl BmffBox for SampleToChunkBox {
    const TYPE: [u8; 4] = *b"stsc";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.entries.len() as u64 * 4 * 3
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&(self.entries.len() as u32).to_be_bytes())
            .await?;
        for (first_chunk, samples_per_chunk, sample_description_index) in self.entries.iter() {
            w.write_all(&first_chunk.to_be_bytes()).await?;
            w.write_all(&samples_per_chunk.to_be_bytes()).await?;
            w.write_all(&sample_description_index.to_be_bytes()).await?;
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

#[async_trait]
impl BmffBox for ChunkOffsetBox {
    const TYPE: [u8; 4] = *b"stco";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.chunk_offsets.len() as u64 * 4
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&(self.chunk_offsets.len() as u32).to_be_bytes())
            .await?;
        for chunk_offset in self.chunk_offsets.iter() {
            w.write_all(&chunk_offset.to_be_bytes()).await?;
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

#[async_trait]
impl BmffBox for EditBox {
    const TYPE: [u8; 4] = *b"edts";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.elst.size()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        write_to_full(&self.elst, &mut w).await?;
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

#[async_trait]
impl BmffBox for EditListBox {
    const TYPE: [u8; 4] = *b"elst";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + self.entries.len() as u64 * 8 * 2 + 2 + 2
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&(self.entries.len() as u32).to_be_bytes())
            .await?;
        for (segment_duration, media_time) in self.entries.iter() {
            w.write_all(&segment_duration.to_be_bytes()).await?;
            w.write_all(&media_time.to_be_bytes()).await?;
        }
        w.write_all(&self.media_rate_integer.to_be_bytes()).await?;
        w.write_all(&self.media_rate_fraction.to_be_bytes()).await?;
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

#[async_trait]
impl BmffBox for MovieExtendsBox {
    const TYPE: [u8; 4] = *b"mvex";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.mehd.as_ref().map(|x| x.size()).unwrap_or(0)
        + self.trex.iter().map(|x| x.size()).sum::<u64>()
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        if let Some(mehd) = &self.mehd {
            write_to_full(mehd, &mut w).await?;
        }
        for trex in self.trex.iter() {
            write_to_full(trex, &mut w).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MovieExtendsHeaderBox {
    pub fragment_duration: Duration,
}

#[async_trait]
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

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let fragment_duration = self.fragment_duration.num_seconds();
        if fragment_duration as u64 > u32::MAX as u64 {
            w.write_all(&fragment_duration.to_be_bytes()).await?;
        } else {
            w.write_all(&(fragment_duration as u32).to_be_bytes())
                .await?;
        }
        Ok(())
    }
}

impl FullBox for MovieExtendsHeaderBox {
    #[inline]
    fn version(&self) -> u8 {
        if self.fragment_duration.num_seconds() as u64 > u32::MAX as u64 {
            1
        } else {
            0
        }
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
        const SAMPLE_DEPENDS_ON = 0x03000000;
        const SAMPLE_IS_DEPENDED_ON = 0x00C00000;
        const SAMPLE_HAS_REDUNDANCY = 0x00300000;
        const SAMPLE_PADDING_VALUE = 0x000D0000;
    }
}

#[async_trait]
impl BmffBox for TrackExtendsBox {
    const TYPE: [u8; 4] = *b"trex";

    #[inline]
    fn size(&self) -> u64 {
        12 + 4 + 4 + 4 + 4 + 4
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&self.track_id.to_be_bytes()).await?;
        w.write_all(&self.default_sample_description_index.to_be_bytes())
            .await?;
        w.write_all(&self.default_sample_duration.to_be_bytes())
            .await?;
        w.write_all(&self.default_sample_size.to_be_bytes()).await?;
        w.write_all(&self.default_sample_flags.bits.to_be_bytes())
            .await?;
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

#[async_trait]
impl BmffBox for MediaDataBox {
    const TYPE: [u8; 4] = *b"mdat";

    #[inline]
    fn size(&self) -> u64 {
        8 + self.data.len() as u64
    }

    async fn write_box<W>(&self, mut w: W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        w.write_all(&self.data).await?;
        Ok(())
    }
}
