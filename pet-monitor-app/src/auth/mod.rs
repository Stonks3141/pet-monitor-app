//! This module provides utilities for authentication and verification.
//!
//! It contains the [`Token`] struct, which can be parsed to and from a string
//! and represents a JWT, and the [`validate()`] function, which verifies a
//! password against a hash.

use chrono::{prelude::*, Duration};
use jsonwebtoken as jwt;
use rocket::request::{Request, Outcome, FromRequest};
use rocket::http::Status;
use rocket::tokio::sync::{mpsc, oneshot};
use serde::{Deserialize, Serialize};
use crate::config::Context;

#[cfg(test)]
mod tests;

/// The claims in a JWT issued by this server.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Claims {
    /// Issued-at time (Unix timestamp)
    iat: u64,
    /// Expiration time (Unix timestamp)
    exp: u64,
}

impl Claims {
    /// Creates new JWT claims that expire in `expires_in` time.
    fn new(expires_in: Duration) -> Self {
        let utc = Utc::now();
        Self {
            iat: utc.timestamp() as u64,
            exp: (utc + expires_in).timestamp() as u64,
        }
    }
}

impl Default for Claims {
    /// Creates new JWT claims that expire in 1 day.
    fn default() -> Self {
        Self::new(Duration::days(1))
    }
}

/// The algorithm used for signing JWTs.
const ALG: jwt::Algorithm = jwt::Algorithm::HS256;

/// A struct representing a JWT.
#[derive(Debug, PartialEq)]
pub struct Token {
    header: jwt::Header,
    claims: Claims,
}

impl Token {
    /// Creates a new token that expires in `expires_in` time.
    pub fn new(expires_in: Duration) -> Self {
        Self {
            header: jwt::Header::new(ALG),
            claims: Claims::new(expires_in),
        }
    }

    /// Verifies the validity of a `Token`.
    pub fn verify(&self) -> bool {
        let utc = Utc::now();
        let exp = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(self.claims.exp as i64, 0),
            Utc,
        );

        utc < exp
    }

    /// Parses a JWT into a `Token`.
    pub fn from_str(s: &str, secret: &[u8; 32]) -> jwt::errors::Result<Self> {
        let dec_key = jwt::DecodingKey::from_secret(secret);
        let val = jwt::Validation::new(ALG);

        jwt::decode::<Claims>(s, &dec_key, &val).map(|t| Self {
            header: t.header,
            claims: t.claims,
        })
    }

    /// Creates a JWT from a `Token`.
    pub fn to_string(&self, secret: &[u8; 32]) -> jwt::errors::Result<String> {
        let enc_key = jwt::EncodingKey::from_secret(secret);

        jwt::encode(&self.header, &self.claims, &enc_key)
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::new(Duration::days(1))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = jwt::errors::Error;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use jwt::errors::{Error, ErrorKind};
        if let Some(token) = req
            .headers()
            .get_one("Authorization")
            .map(|t| t.split_once("Bearer "))
            .flatten()
            .map(|t| t.1)
        {
            let ctx = req.rocket().state::<mpsc::Sender<(Option<Context>, oneshot::Sender<Context>)>>().unwrap();
            let (tx, rx) = oneshot::channel();
            ctx.send((None, tx)).await.unwrap();
            let ctx = rx.await.unwrap();

            match Self::from_str(token, &ctx.jwt_secret) {
                Ok(token) => {
                    if token.verify() {
                        Outcome::Success(token)
                    } else {
                        Outcome::Failure((Status::Unauthorized, Error::from(ErrorKind::InvalidToken)))
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::Base64(_)
                    | ErrorKind::Crypto(_)
                    | ErrorKind::Json(_)
                    | ErrorKind::Utf8(_) => Outcome::Failure((Status::InternalServerError, e)),
                    _ => Outcome::Failure((Status::Unauthorized, e)),
                },
            }
        } else {
            Outcome::Failure((Status::Unauthorized, Error::from(ErrorKind::InvalidToken)))
        }
    }
}

/// Validates a password against a hash.
pub fn validate(password: &str, hash: &str) -> argon2::Result<bool> {
    argon2::verify_encoded(hash, password.as_bytes())
}
