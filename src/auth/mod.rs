use jsonwebtoken as jwt;
use once_cell::sync::Lazy;
use rand::random;
use rocket::http::{Cookie, Status};
use serde::{Deserialize, Serialize};
use std::{env, fs, io};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: u64,
    exp: u64,
}

pub fn authenticate(password: &str) -> Result<Cookie<'static>, Status> {
    static PASSWORD: Lazy<String> =
        Lazy::new(|| env::var("PASSWORD").expect("Please set the 'PASSWORD' env var."));

    if password != *PASSWORD {
        Err(Status::Unauthorized)
    } else {
        let token = match make_token() {
            Ok(token) => token,
            Err(_) => return Err(Status::InternalServerError),
        };

        Ok(Cookie::build("token", token)
            .max_age(rocket::time::Duration::DAY)
            .finish())
    }
}

pub fn validate_token(token: &str) -> Status {
    let dec_key = jwt::DecodingKey::from_secret(&*SECRET);

    match jwt::decode::<Claims>(
        token,
        &dec_key,
        &jwt::Validation::new(jwt::Algorithm::HS256),
    ) {
        Ok(_) => Status::Ok,
        Err(e) => match e.kind() {
            jwt::errors::ErrorKind::Base64(_) => Status::InternalServerError,
            jwt::errors::ErrorKind::Json(_) => Status::InternalServerError,
            jwt::errors::ErrorKind::Utf8(_) => Status::InternalServerError,
            jwt::errors::ErrorKind::Crypto(_) => Status::InternalServerError,
            _ => Status::Unauthorized,
        },
    }
}

fn make_token() -> jwt::errors::Result<String> {
    let enc_key = jwt::EncodingKey::from_secret(&*SECRET);

    let time = jwt::get_current_timestamp();

    let claims = Claims {
        iat: time,
        exp: time + 60 * 60 * 24,
    };

    jwt::encode(&jwt::Header::default(), &claims, &enc_key)
}

const SECRET_PATH: &str = "/var/local/lib/pet-monitor-app/jwt_secret";

static SECRET: Lazy<[u8; 32]> = Lazy::new(|| {
    match fs::read(SECRET_PATH) {
        Ok(s) => read_secret(s, &SECRET_PATH),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                init_secret(SECRET_PATH).unwrap() // TODO
            }
            io::ErrorKind::PermissionDenied => {
                let path = format!("{}/.pet-monitor-app/jwt_secret", env::var("HOME").unwrap());

                match fs::read(&path) {
                    Ok(s) => read_secret(s, &path),
                    _ => panic!(""),
                }
            }
            _ => panic!(""),
        },
    }
});

fn init_secret<P: AsRef<std::path::Path>>(path: P) -> io::Result<[u8; 32]> {
    if !path.as_ref().exists() {
        if let Some(p) = path.as_ref().parent() {
            fs::create_dir_all(p)?;
        }
    }

    let rand = random::<[u8; 32]>(); // 256-bit secret
    fs::write(path, rand)?;
    Ok(rand)
}

fn read_secret<P: AsRef<std::path::Path>>(s: Vec<u8>, path: &P) -> [u8; 32] {
    if s.len() == 32 {
        s.try_into().unwrap() // infallible
    } else {
        init_secret(path).unwrap() // TODO
    }
}
