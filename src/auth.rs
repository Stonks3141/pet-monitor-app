use jsonwebtoken as jwt;
use once_cell::sync::Lazy;
use rand::random;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use chrono::{prelude::*, Duration};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iat: u64,
    exp: u64,
}

impl Claims {
    pub fn new(exp_duration: Duration) -> Self {
        let utc = Utc::now();
        Self {
            iat: utc.timestamp() as u64,
            exp: (utc + exp_duration).timestamp() as u64,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    header: jwt::Header,
    claims: Claims,
}

impl Token {
    pub fn new() -> Self {
        Self {
            header: jwt::Header::new(jwt::Algorithm::ES256),
            claims: Claims::new(Duration::days(1)),
        }
    }
}

impl TryFrom<&str> for Token {
    type Error = jwt::errors::Error;

    fn try_from(token: &str) -> Result<Self, Self::Error> {
        let dec_key = jwt::DecodingKey::from_secret(&*SECRET);
    
        match jwt::decode::<Claims>(
            token,
            &dec_key,
            &jwt::Validation::new(jwt::Algorithm::ES256),
        ) {
            Ok(t) => Ok(Self {
                header: t.header,
                claims: t.claims,
            }),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<Token> for String {
    type Error = jwt::errors::Error;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let enc_key = jwt::EncodingKey::from_secret(&*SECRET);
            
        jwt::encode(&token.header, &token.claims, &enc_key)
    }
}

const SECRET_PATH: &str = "/var/local/lib/pet-monitor-app/jwt_secret.dat";

// This program expects to be run in a Docker container with access to /var/local/..
static SECRET: Lazy<[u8; 32]> = Lazy::new(|| get_secret()
    .expect("Failed to initialize JWT secret. Is the program running in a Docker container?"));

fn get_secret() -> io::Result<[u8; 32]> {
    match fs::read(SECRET_PATH) {
        Ok(s) => {
            if let Ok(s) = s.try_into() {
                Ok(s)
            } else {
                init_secret(SECRET_PATH)
            }
        },
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => init_secret(SECRET_PATH),
            e => Err(io::Error::from(e)),
        },
    }
}

fn init_secret<P: AsRef<Path>>(path: P) -> io::Result<[u8; 32]> {
    if !path.as_ref().exists() {
        if let Some(p) = path.as_ref().parent() {
            fs::create_dir_all(p)?;
        }
    }

    let rand = random::<[u8; 32]>(); // 256-bit secret
    fs::write(path, rand)?;
    Ok(rand)
}
