use rocket::futures::io::{self, AsyncWrite};

#[rocket::async_trait]
trait BmffBox {
    const TYPE: [u8; 4];
    const EXT_TYPE: Option<[u8; 16]> = None;
    fn size(&self) -> usize;
    async fn write(writer: impl AsyncWrite) -> io::Result<()>;
}

trait FullBox: BmffBox {
    fn version(&self) -> u8;
    fn flags(&self) -> [u8; 3];
}
