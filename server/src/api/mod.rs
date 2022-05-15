pub mod gql;
#[cfg(test)]
mod tests;

pub mod routes {
    use rocket::get;
    use rocket::fs::{NamedFile, relative};

    // match all for React Router
    #[get("/<_f..>", rank=2)]
    pub async fn index(_f: std::path::PathBuf) -> Option<NamedFile> {
        NamedFile::open(relative!("../client/dist/index.html")).await.ok()
    }
}
