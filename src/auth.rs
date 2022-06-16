use jsonwebtoken as jwt;
use jwt::errors::ErrorKind;
use once_cell::sync::Lazy;
use rocket::http::{Cookie, Status};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: u64,
    exp: u64,
}

pub fn authenticate(password: &str) -> Result<Cookie<'static>, Status> {
    static PASSWORD: Lazy<String> =
        Lazy::new(|| env::var("PASSWORD").expect("Please set the 'PASSWORD' env var."));

    if password != *PASSWORD {
        return Err(Status::Unauthorized);
    }

    let token = match make_token() {
        Ok(token) => token,
        Err(_) => return Err(Status::InternalServerError),
    };

    Ok(Cookie::build("token", token)
        .max_age(rocket::time::Duration::DAY)
        .finish())
}

pub fn authorize(token: &str) -> Status {
    static DEC_KEY: Lazy<jwt::DecodingKey> = Lazy::new(|| {
        jwt::DecodingKey::from_base64_secret(
            &env::var("SECRET").expect("Please set the 'SECRET' env var with a base64 secret."),
        )
        .unwrap() // TODO
    });

    match jwt::decode::<Claims>(
        token,
        &*DEC_KEY,
        &jwt::Validation::new(jwt::Algorithm::HS256),
    ) {
        Ok(_) => Status::Ok,
        Err(e) => match e.kind() {
            ErrorKind::Base64(_) => Status::InternalServerError,
            ErrorKind::Json(_) => Status::InternalServerError,
            ErrorKind::Utf8(_) => Status::InternalServerError,
            ErrorKind::Crypto(_) => Status::InternalServerError,
            _ => Status::Unauthorized,
        },
    }
}

fn make_token() -> jwt::errors::Result<String> {
    static ENC_KEY: Lazy<jwt::EncodingKey> = Lazy::new(|| {
        jwt::EncodingKey::from_base64_secret(
            &env::var("SECRET").expect("Please set the 'SECRET' env var with a base64 secret."),
        )
        .unwrap() // TODO
    });

    let time = jwt::get_current_timestamp();

    let claims = Claims {
        iat: time,
        exp: time + 60 * 60 * 24,
    };

    jwt::encode(&jwt::Header::default(), &claims, &*ENC_KEY)
}
