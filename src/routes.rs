use rocket::fs::{relative, NamedFile};
use rocket::get;

// match all for React Router
#[get("/<_f..>", rank = 2)]
pub async fn index(_f: std::path::PathBuf) -> Option<NamedFile> {
    NamedFile::open(relative!("client/build/index.html"))
        .await
        .ok()
}
