use chrono::{prelude::*, Duration};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::secrets;

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

impl Default for Claims {
    fn default() -> Self {
        Self::new(Duration::days(1))
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
            claims: Claims::default(),
        }
    }
}

impl FromStr for Token {
    type Err = jwt::errors::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dec_key = jwt::DecodingKey::from_secret(&*secrets::SECRET);

        match jwt::decode::<Claims>(
            s,
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
        let enc_key = jwt::EncodingKey::from_secret(&*secrets::SECRET);

        jwt::encode(&token.header, &token.claims, &enc_key)
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate(password: &str) -> argon2::Result<bool> {
    argon2::verify_encoded(&*secrets::PASSWORD_HASH, password.as_bytes())
}
