use super::*;

#[test]
fn fmp4() {
    let mut camera = Frames::new((1, 30), (1280, 720), *b"H264");

    let mut buf: Vec<u8> = Vec::new();
    camera.iter().take(120).for_each(|frame| {
        buf.append(*frame);
    });

    let initseg = bmff::create_init_seg(30, 1280, 720);
    initseg

    let mediaseg = bmff::create_media_seg(0, &buf);
}
